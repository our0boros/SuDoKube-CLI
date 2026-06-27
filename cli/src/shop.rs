//! 局外金币与商店系统
//!
//! 提供：
//! - 道具类型 [`ItemType`] 与道具目录 [`shop_catalog`]
//! - 金币结算 [`calculate_gold_reward`]
//! - 提示价（用于 3 提示上限计算）
//!
//! 商店与背包的运行时状态保存在 [`crate::App`] 中。

use sudokube_core::cube::Difficulty;

/// 道具类型。商店中可购买的所有道具枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    /// 🎲 随机立方体提示 — 在整个立方体上随机提示一格
    Cube,
    /// 🐍 贪吃蛇(3果实) — 启动贪吃蛇小游戏,3 个果实
    Snake3,
    /// 🔀 随机当前面提示 — 在当前可见面随机提示一格
    Face,
    /// 🐍 贪吃蛇(5果实) — 启动贪吃蛇小游戏,5 个果实
    Snake5,
    /// ❗ 当前选择目标提示 — 提示当前光标所在格的答案
    Target,
}

impl ItemType {
    pub fn all() -> [ItemType; 5] {
        [
            ItemType::Cube,
            ItemType::Snake3,
            ItemType::Face,
            ItemType::Snake5,
            ItemType::Target,
        ]
    }

    /// 默认价格(金币): 按强度从弱到强递增
    pub fn price(&self) -> i32 {
        match self {
            ItemType::Cube => 10,
            ItemType::Snake3 => 15,
            ItemType::Face => 20,
            ItemType::Snake5 => 30,
            ItemType::Target => 40,
        }
    }

    /// 图标字符
    pub fn icon(&self) -> &'static str {
        match self {
            ItemType::Cube => "🎲",
            ItemType::Snake3 => "🐍",
            ItemType::Face => "🔀",
            ItemType::Snake5 => "🐍",
            ItemType::Target => "❗",
        }
    }

    /// 短标识(用于按钮栏): 1-2 字符
    pub fn tag(&self) -> &'static str {
        match self {
            ItemType::Cube => "Cb",
            ItemType::Snake3 => "S3",
            ItemType::Face => "Fc",
            ItemType::Snake5 => "S5",
            ItemType::Target => "Tg",
        }
    }

    /// 贪吃蛇类型对应的果实数量
    pub fn fruit_count(&self) -> u8 {
        match self {
            ItemType::Snake3 => 3,
            ItemType::Snake5 => 5,
            _ => 0,
        }
    }

    /// 是否为贪吃蛇类型
    pub fn is_snake(&self) -> bool {
        matches!(self, ItemType::Snake3 | ItemType::Snake5)
    }

    /// i18n key: 名称
    pub fn name_key(&self) -> &'static str {
        match self {
            ItemType::Cube => "shop.cube",
            ItemType::Snake3 => "shop.snake3",
            ItemType::Face => "shop.face",
            ItemType::Snake5 => "shop.snake5",
            ItemType::Target => "shop.target",
        }
    }

    /// i18n key: 描述
    pub fn desc_key(&self) -> &'static str {
        match self {
            ItemType::Cube => "shop.cube_desc",
            ItemType::Snake3 => "shop.snake3_desc",
            ItemType::Face => "shop.face_desc",
            ItemType::Snake5 => "shop.snake5_desc",
            ItemType::Target => "shop.target_desc",
        }
    }
}

/// 商店条目
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub item_type: ItemType,
    pub price: i32,
}

/// 商店目录
pub fn shop_catalog() -> Vec<ShopItem> {
    ItemType::all()
        .iter()
        .map(|&t| ShopItem {
            item_type: t,
            price: t.price(),
        })
        .collect()
}

/// 根据难度与完成时间计算金币奖励。
///
/// - 简单: 3 min 达到最高收益
/// - 中等: 8 min 达到最高收益
/// - 高难: 25 min 达到最高收益
///
/// 实际收益随时间反比衰减;快于目标时间时给最多 1.5× 加成;
/// 慢于目标时间时按比例降低,但不低于 1 金币。
///
/// 最高收益 = 3 × 10 = 30 金币(可购买 3 次立方体提示)
pub fn calculate_gold_reward(difficulty: Difficulty, elapsed_seconds: u64) -> i32 {
    let target_time = match difficulty {
        Difficulty::Easy => 180,   // 3 min
        Difficulty::Medium => 480,  // 8 min
        Difficulty::Hard => 1500,   // 25 min
    };
    // 最高收益: 3 次提示 (以立方体提示为基准)
    let max_reward = 3 * ItemType::Cube.price();

    let elapsed = elapsed_seconds.max(1) as f64;
    let ratio = target_time as f64 / elapsed;
    let bonus = ratio.clamp(0.1, 1.5);
    (max_reward as f64 * bonus).round().max(1.0) as i32
}

/// 应用道具效果。返回 Some(消息) 显示在状态栏。
///
/// 在 App 上执行实际的提示/小游戏效果。
pub fn apply_tool(app: &mut crate::App, item: ItemType) -> Option<String> {
    let lang = crate::i18n::Lang::from_code(&app.settings.language);
    match item {
        ItemType::Cube => Some(apply_cube_hint(app, lang)),
        ItemType::Face => Some(apply_face_hint(app, lang)),
        ItemType::Target => Some(apply_target_hint(app, lang)),
        ItemType::Snake3 | ItemType::Snake5 => {
            start_snake_game(app, item.fruit_count());
            None
        }
    }
}

// ── 贪吃蛇小游戏 ──

use std::collections::HashSet;
use std::time::{Duration, Instant};
use sudokube_core::cube::Face;

/// 贪吃蛇状态(基于面坐标系,与光标共享跨面逻辑)
pub struct SnakeState {
    /// 蛇身(头在前): 每节存储 (Face, u, v)
    pub body: Vec<(Face, u8, u8)>,
    /// 移动方向: (dx, dy) — 面局部坐标系下的增量
    pub dir: (i8, i8),
    /// 果实位置集合: (Face, u, v)
    pub fruits: HashSet<(Face, u8, u8)>,
    /// 墙壁位置集合: (Face, u8, u8)
    pub walls: HashSet<(Face, u8, u8)>,
    /// 倒计时截止时间
    pub deadline: Instant,
    /// 上次前进时间(用于控制速度)
    pub last_step: Instant,
    /// 单步间隔(蛇移速)
    pub step_interval: Duration,
    /// 状态: 进行中 / 胜利 / 失败
    pub outcome: SnakeOutcome,
    /// 分数(吃到的果实数)
    pub score: u32,
    /// 果实总数
    pub total_fruits: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeOutcome {
    Running,
    Won,
    Lost,
}

/// 在 App 上启动贪吃蛇小游戏。设置 `app.snake = Some(...)`。
/// `fruit_count`: 果实数量(由道具类型决定)
pub fn start_snake_game(app: &mut crate::App, fruit_count: u8) {
    // 收集未填的空格(可生成果实/墙),转为 (Face, u, v) 形式
    let empties: Vec<(Face, u8, u8)> = app
        .game
        .grid
        .cells
        .keys()
        .copied()
        .filter(|c| {
            app.game
                .grid
                .get(c)
                .map(|cell| cell.user_value.is_none() && !cell.given)
                .unwrap_or(false)
        })
        .filter_map(|c| {
            // 取第一个面坐标(角/边会属于多个面,取第一个即可)
            c.to_face_coords().first().map(|fc| (fc.face, fc.u, fc.v))
        })
        .collect();
    if empties.is_empty() {
        let lang = crate::i18n::Lang::from_code(&app.settings.language);
        app.set_message(
            crate::i18n::t("tool.no_empty", lang).to_string(),
            Duration::from_secs(2),
        );
        return;
    }
    // 蛇起始: 在当前面中央,6 节身体向左铺开(头在右)
    let start_face = app.current_face;
    let mut body = Vec::with_capacity(6);
    for i in 0..6u8 {
        let u = 4u8.saturating_sub(i); // 头 u=4, 依次 u=3,2,1,0,...
        body.push((start_face, u, 4));
    }
    // 随机器
    let mut rng_state = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut next_rand = || {
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        rng_state
    };
    // 移除蛇身位置
    let body_set: HashSet<_> = body.iter().copied().collect();
    let pool: Vec<_> = empties.into_iter().filter(|c| !body_set.contains(c)).collect();
    // 果实
    let fruit_count_usize = fruit_count as usize;
    let mut fruits = HashSet::new();
    for _ in 0..fruit_count_usize {
        if pool.is_empty() {
            break;
        }
        let idx = (next_rand() as usize) % pool.len();
        fruits.insert(pool[idx]);
    }
    // 墙: 少量随机
    let mut walls = HashSet::new();
    let pool2: Vec<_> = pool.iter().copied().filter(|c| !fruits.contains(c)).collect();
    for _ in 0..fruit_count_usize.min(pool2.len()) {
        let idx = (next_rand() as usize) % pool2.len();
        walls.insert(pool2[idx]);
    }
    let now = Instant::now();
    app.snake = Some(SnakeState {
        body,
        dir: (1, 0), // 向右
        fruits,
        walls,
        deadline: now + Duration::from_secs(30),
        last_step: now,
        step_interval: Duration::from_millis(200),
        outcome: SnakeOutcome::Running,
        score: 0,
        total_fruits: fruit_count as u32,
    });
}

/// 推进贪吃蛇一步(使用面坐标系,共享光标跨面逻辑)
pub fn snake_step(app: &mut crate::App) {
    // 阶段1: 在 snake 借用中计算,收集结果
    let result = {
        let Some(snake) = app.snake.as_mut() else {
            return;
        };
        if snake.outcome != SnakeOutcome::Running {
            return;
        }
        let now = Instant::now();
        if now < snake.last_step + snake.step_interval {
            return;
        }
        snake.last_step = now;

        // 计算新头: 使用 move_on_surface 实现跨面
        let (head_face, head_u, head_v) = snake.body[0];
        let (new_face, (new_u, new_v)) = crate::input::move_on_surface(
            head_face,
            (head_u, head_v),
            snake.dir.0,
            snake.dir.1,
        );

        let new_pos = (new_face, new_u, new_v);

        // 撞墙
        if snake.walls.contains(&new_pos) {
            snake.outcome = SnakeOutcome::Lost;
            return;
        }
        // 撞自己
        if snake.body.contains(&new_pos) {
            snake.outcome = SnakeOutcome::Lost;
            return;
        }
        // 是否吃到果实
        let ate = snake.fruits.remove(&new_pos);
        // 移动: 头插入,若吃果实则不删尾;否则删尾
        snake.body.insert(0, new_pos);
        if ate {
            snake.score += 1;
        } else {
            snake.body.pop();
        }
        let ate_at = if ate { Some(new_pos) } else { None };
        if ate && snake.fruits.is_empty() {
            snake.outcome = SnakeOutcome::Won;
        }
        // 超时检查
        if Instant::now() >= snake.deadline {
            snake.outcome = SnakeOutcome::Lost;
        }
        let head_face_now = snake.body[0].0;
        (head_face_now, new_u, new_v, ate_at)
    };
    // 阶段2: snake 借用已释放,可安全修改 app
    let (head_face_now, new_u, new_v, ate_at) = result;
    app.current_face = head_face_now;
    app.cursor = (new_u, new_v);
    // 写入答案
    if let Some((face, u, v)) = ate_at {
        let coord = face.to_cube(u, v);
        if let Some(&ans) = app.game.solution.get(&coord) {
            app.game.set_value(coord, Some(ans));
            app.adjust_digit_remaining(coord.x, coord.y, coord.z, None, Some(ans));
        }
    }
    // 胜利条件(整局已完成)
    if app.game.check_completion() {
        app.trigger_victory();
    }
}

/// 处理贪吃蛇游戏的退出/结算
pub fn end_snake_game(app: &mut crate::App) -> Option<String> {
    let lang = crate::i18n::Lang::from_code(&app.settings.language);
    let msg = if let Some(snake) = app.snake.as_ref() {
        match snake.outcome {
            SnakeOutcome::Won => Some(crate::i18n::t("snake.win", lang).to_string()),
            SnakeOutcome::Lost => Some(crate::i18n::t("snake.lose", lang).to_string()),
            SnakeOutcome::Running => Some(crate::i18n::t("snake.exit", lang).to_string()),
        }
    } else {
        None
    };
    app.snake = None;
    msg
}

/// 🎲 随机立方体提示:在整个立方体上随机一个未填的空格填上正确答案,并跳转光标
fn apply_cube_hint(app: &mut crate::App, lang: crate::i18n::Lang) -> String {
    // 收集所有空格
    let empties: Vec<sudokube_core::cube::CubeCoord> = app
        .game
        .grid
        .cells
        .keys()
        .copied()
        .filter(|coord| {
            app.game
                .grid
                .get(coord)
                .map(|c| c.user_value.is_none() && !c.given)
                .unwrap_or(false)
        })
        .collect();
    if empties.is_empty() {
        return crate::i18n::t("tool.no_empty", lang).to_string();
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let pick = empties[nanos as usize % empties.len()];
    if let Some(&answer) = app.game.solution.get(&pick) {
        app.game.set_value(pick, Some(answer));
        app.adjust_digit_remaining(pick.x, pick.y, pick.z, None, Some(answer));
        // 跳转到对应面并移动光标
        jump_cursor_to(app, pick);
        if app.game.check_completion() {
            app.trigger_victory();
        }
        return crate::i18n::t("tool.cube_done", lang).to_string();
    }
    crate::i18n::t("tool.no_empty", lang).to_string()
}

/// 🔀 随机当前面提示:在当前可见面上随机一个空格填上正确答案,并移动光标
fn apply_face_hint(app: &mut crate::App, lang: crate::i18n::Lang) -> String {
    let face = app.current_face;
    let empties: Vec<sudokube_core::cube::CubeCoord> = app
        .game
        .grid
        .cells
        .keys()
        .copied()
        .filter(|coord| {
            let on_face = coord
                .to_face_coords()
                .iter()
                .any(|fc| fc.face == face);
            if !on_face {
                return false;
            }
            app.game
                .grid
                .get(coord)
                .map(|c| c.user_value.is_none() && !c.given)
                .unwrap_or(false)
        })
        .collect();
    if empties.is_empty() {
        return crate::i18n::t("tool.no_empty_face", lang).to_string();
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let pick = empties[nanos as usize % empties.len()];
    if let Some(&answer) = app.game.solution.get(&pick) {
        app.game.set_value(pick, Some(answer));
        app.adjust_digit_remaining(pick.x, pick.y, pick.z, None, Some(answer));
        // 跳转光标(当前面内)
        jump_cursor_to(app, pick);
        if app.game.check_completion() {
            app.trigger_victory();
        }
        return crate::i18n::t("tool.face_done", lang).to_string();
    }
    crate::i18n::t("tool.no_empty_face", lang).to_string()
}

/// ❗ 当前选择目标提示:把光标所在格的答案填入
fn apply_target_hint(app: &mut crate::App, lang: crate::i18n::Lang) -> String {
    let coord = crate::current_coord(app);
    let cell = app.game.grid.get(&coord);
    if cell.is_none() {
        return crate::i18n::t("tool.no_empty", lang).to_string();
    }
    let cell = cell.unwrap();
    if cell.given || cell.user_value.is_some() {
        return crate::i18n::t("tool.not_empty", lang).to_string();
    }
    if let Some(&answer) = app.game.solution.get(&coord) {
        app.game.set_value(coord, Some(answer));
        app.adjust_digit_remaining(coord.x, coord.y, coord.z, None, Some(answer));
        if app.game.check_completion() {
            app.trigger_victory();
        }
        return crate::i18n::t("tool.target_done", lang).to_string();
    }
    crate::i18n::t("tool.no_empty", lang).to_string()
}

/// 跳转光标到指定立方体坐标:选择最佳面并设置 cursor (u, v)
fn jump_cursor_to(app: &mut crate::App, coord: sudokube_core::cube::CubeCoord) {
    // 优先当前面,否则选第一个匹配面
    let face_coords = coord.to_face_coords();
    // 先检查当前面
    if let Some(fc) = face_coords.iter().find(|fc| fc.face == app.current_face) {
        app.cursor = (fc.u, fc.v);
        return;
    }
    // 否则切换到第一个匹配面
    if let Some(fc) = face_coords.first() {
        app.current_face = fc.face;
        app.cursor = (fc.u, fc.v);
    }
}
