use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use sudokube_core::cube::Face;
use std::time::Instant;

use crate::{App, AppScreen, MenuItem, total_elapsed, AppSettings};
use crate::i18n::{self, Lang};

// ── 常量 ──

const LOGO: &[&str] = &[
    " ██████  █    ██ ▓█████▄  ▒█████   ██ ▄█▀ █    ██  ▄▄▄▄   ▓█████   ",
    " ▒██    ▒  ██  ▓██▒▒██▀ ██▌▒██▒  ██▒ ██▄█▒  ██  ▓██▒▓█████▄ ▓█   ▀ ",
    " ░ ▓██▄   ▓██  ▒██░░██   █▌▒██░  ██▒▓███▄░ ▓██  ▒██░▒██▒ ▄██▒███   ",
    "   ▒   ██▒▓▓█  ░██░░▓█▄   ▌▒██   ██░▓██ █▄ ▓▓█  ░██░▒██░█▀  ▒▓█  ▄ ",
    " ▒██████▒▒▒▒█████▓ ░▒████▓ ░ ████▓▒░▒██▒ █▄▒▒█████▓ ░▓█  ▀█▓░▒████▒",
    " ▒ ▒▓▒ ▒ ░░▒▓▒ ▒ ▒  ▒▒▓  ▒ ░ ▒░▒░▒░ ▒ ▒▒ ▓▒░▒▓▒ ▒ ▒ ░▒▓███▀▒░░ ▒░ ░",
    " ░ ░▒  ░ ░░░▒░ ░ ░  ░ ▒  ▒   ░ ▒ ▒░ ░ ░▒ ▒░░░▒░ ░ ░ ▒░▒   ░  ░ ░  ░",
    " ░  ░  ░   ░░░ ░ ░  ░ ░  ░ ░ ░ ░ ▒  ░ ░░ ░  ░░░ ░ ░  ░    ░    ░   ",
    "       ░     ░        ░        ░ ░  ░  ░      ░      ░         ░  ░",
    "                    ░                                     ░        ",
];

// ── 类型 ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Standard,
    Monospace,
}

impl RenderMode {
    pub fn toggle(self) -> Self {
        match self {
            RenderMode::Standard => RenderMode::Monospace,
            RenderMode::Monospace => RenderMode::Standard,
        }
    }

    pub fn cell_width(self, settings: &AppSettings) -> usize {
        match self {
            RenderMode::Standard => settings.standard_cell_width,
            RenderMode::Monospace => 3,
        }
    }

    pub fn cell_height(self) -> usize {
        3
    }
}

pub fn mode_label(mode: RenderMode, lang: Lang) -> &'static str {
    match mode {
        RenderMode::Standard => i18n::t("game.mode_standard", lang),
        RenderMode::Monospace => i18n::t("game.mode_mono", lang),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonId {
    Number(u8),
    Erase,
    Hint,
    Undo,
    ToggleGuidance,
    ToggleMode,
    Quit,
}

/// 按钮布局信息（用于鼠标点击检测）。
pub struct ButtonLayout {
    pub id: ButtonId,
    pub label: String,
    pub col: u16,
    pub row: u16,
    pub width: u16,
}

/// 游戏画面布局（三列式：左 = 控制面板，中间 = 数独网格，右 = 3D 立方体 / 预留）。
#[allow(dead_code)]
pub struct GameLayout {
    /// 整个游戏区域
    pub game_area: Rect,
    /// 左侧列外框
    pub left_column: Rect,
    /// 左侧第一个面板 (Status)
    pub status_panel: Rect,
    /// 左侧第二个面板 (Navigator)
    pub navigator_panel: Rect,
    /// 左侧第三个面板 (Logs)
    pub logs_panel: Rect,
    /// 中间数独网格 (内容区域)
    pub grid_area: Rect,
    /// 中间数独外框 (含双线边框)
    pub grid_frame: Rect,
    /// 网格顶方向指示条
    pub grid_top_dir: Rect,
    /// 网格底方向指示条
    pub grid_bot_dir: Rect,
    /// 网格左方向指示条
    pub grid_left_dir: Rect,
    /// 网格右方向指示条
    pub grid_right_dir: Rect,
    /// 消息行（位于网格下方）
    pub msg_area: Rect,
    /// 按钮栏
    pub btn_area: Rect,
    /// 按钮实际起始列
    pub btn_content_x: u16,
    /// 右侧列外框
    pub right_column: Rect,
    /// 3D 立方体区域
    pub cube_area: Rect,
    /// 立方体顶方向指示条
    pub cube_dir_top: Rect,
    /// 立方体底方向指示条
    pub cube_dir_bot: Rect,
    /// 立方体左方向指示条
    pub cube_dir_left: Rect,
    /// 立方体右方向指示条
    pub cube_dir_right: Rect,
    /// 商店预留区域
    pub shop_area: Rect,
    /// 按钮列表
    pub buttons: Vec<ButtonLayout>,
}

// ── 主入口 ──

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        AppScreen::Menu => draw_menu(f, app),
        AppScreen::Game => draw_game(f, app),
        AppScreen::Settings => draw_settings(f, app),
        AppScreen::Generating => draw_generating(f, app),
        AppScreen::Victory => draw_victory(f, app),
        AppScreen::ExportSelect => draw_export_select(f, app),
        AppScreen::ImportInput => draw_import_input(f, app),
    }
}

// ── 菜单画面 ──

fn draw_menu(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let logo_h = LOGO.len() as u16;
    let items_len = app.menu.items.len() as u16;
    let total_h = logo_h + 1 + items_len + 2 + 2 + 1; // logo + gap + box(1+items+1) + gap + hint

    let start_y = area.y + area.height.saturating_sub(total_h) / 2;

    // 绘制 LOGO
    let logo_width = LOGO.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    for (i, line) in LOGO.iter().enumerate() {
        let col = area.x + area.width.saturating_sub(logo_width) / 2;
        let row = start_y + i as u16;
        if row < area.bottom() {
            f.render_widget(
                Paragraph::new(*line).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Rect::new(col, row, logo_width, 1),
            );
        }
    }

    let box_y = start_y + logo_h + 1;

    // 计算菜单项文本
    let lang = Lang::from_code(&app.settings.language);
    let item_texts: Vec<String> = app.menu.items.iter().map(|item| match item {
        MenuItem::NewGame(d) => {
            let diff_key = match d {
                sudokube_core::cube::Difficulty::Easy => "game.diff_easy",
                sudokube_core::cube::Difficulty::Medium => "game.diff_medium",
                sudokube_core::cube::Difficulty::Hard => "game.diff_hard",
            };
            format!("{} - {}", i18n::t("menu.new_easy", lang).split(" - ").next().unwrap_or("New"), i18n::t(diff_key, lang))
        }
        MenuItem::Settings => i18n::t("menu.settings", lang).into(),
        MenuItem::Export => i18n::t("menu.export_all", lang).into(),
        MenuItem::Import => i18n::t("menu.import_all", lang).into(),
        MenuItem::Continue(r) => {
            let total = r.answer.len() + r.puzzle.values().filter(|&&v| v == 0).count();
            let filled = r.puzzle.values().filter(|&&v| v > 0).count();
            let remaining = total.saturating_sub(filled);
            let name = if app.settings.naming_mode == "vivid" {
                i18n::vivid_name(r.id, lang)
            } else {
                format!("#{}", r.id)
            };
            format!(
                "{} {} | {} | {} | {}/{} | {:02}:{:02} | {}",
                i18n::t("menu.continue", lang), name, r.difficulty,
                r.started_at.format("%m-%d").to_string(),
                remaining, total,
                r.elapsed_seconds / 60, r.elapsed_seconds % 60,
                if r.completed { i18n::t("menu.victory", lang) } else { i18n::t("menu.in_progress", lang) }
            )
        }
    }).collect();

    let max_text_w = item_texts.iter().map(|t| display_width(t) + 4).max().unwrap_or(20);
    let box_w = max_text_w + 4; // ╭ + " " + text + " " + ╮

    // 侧边栏宽度
    let sidebar_w: u16 = 28;
    let total_w = box_w as u16 + 2 + sidebar_w; // menu + gap + sidebar

    // If sidebar doesn't fit, skip it
    let has_sidebar = area.width >= total_w;
    let box_x = if has_sidebar {
        area.x + area.width.saturating_sub(total_w) / 2
    } else {
        area.x + area.width.saturating_sub(box_w as u16) / 2
    };

    // 画菜单框
    draw_menu_box(f, box_x, box_y, box_w as u16, &item_texts, app.menu.selected, area);

    // 侧边栏：胜利记录
    if has_sidebar {
        let sidebar_x = box_x + box_w as u16 + 2;
        let victories = &app.menu.victories;
        let total_v = victories.len();
        let easy_count = victories.iter().filter(|r| r.difficulty == "简单" || r.difficulty == "easy").count();
        let med_count = victories.iter().filter(|r| r.difficulty == "中等" || r.difficulty == "medium").count();
        let hard_count = victories.iter().filter(|r| r.difficulty == "困难" || r.difficulty == "hard").count();

        // 侧边栏高 = 顶 + 1(标题) + 1(分隔) + 1(统计) + 1(分隔) + N(列表) + 1(底)
        let list_rows = victories.len().min(area.bottom().saturating_sub(box_y + 5).saturating_sub(2) as usize);
        let sidebar_h = 5 + list_rows as u16 + 1;
        let sidebar_rect = Rect::new(sidebar_x, box_y, sidebar_w, sidebar_h);

        let block = Block::bordered()
            .title(Line::from(Span::styled(
                format!(" {} ", i18n::t("menu.sidebar_title", lang)),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        f.render_widget(block.clone(), sidebar_rect);
        let inner = block.inner(sidebar_rect);
        if inner.height == 0 || inner.width == 0 { return; }

        let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);
        // 统计行
        let stats = format!(
            " {}:{} {} {}:{} {} {}:{}",
            i18n::t("menu.sidebar_total", lang),
            total_v,
            i18n::t("game.diff_easy", lang).chars().next().unwrap_or('E'),
            easy_count,
            i18n::t("game.diff_medium", lang).chars().next().unwrap_or('M'),
            med_count,
            i18n::t("game.diff_hard", lang).chars().next().unwrap_or('H'),
            hard_count
        );
        lines.push(Line::from(Span::styled(
            stats,
            Style::default().fg(Color::Yellow),
        )));
        // 分隔线
        lines.push(Line::from(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::Yellow),
        )));
        // 胜利列表
        for r in victories.iter().take(list_rows) {
            let name = if app.settings.naming_mode == "vivid" {
                i18n::vivid_name(r.id, lang)
            } else {
                format!("#{}", r.id)
            };
            let diff_short = match r.difficulty.as_str() {
                "简单" | "easy" => i18n::t("game.diff_easy", lang),
                "困难" | "hard" => i18n::t("game.diff_hard", lang),
                _ => i18n::t("game.diff_medium", lang),
            };
            let text = format!(
                " {} {} {:02}:{:02}",
                name,
                diff_short,
                r.elapsed_seconds / 60,
                r.elapsed_seconds % 60
            );
            lines.push(Line::from(Span::styled(
                text,
                Style::default().fg(Color::DarkGray),
            )));
        }

        f.render_widget(Paragraph::new(lines), inner);
    }

    // 提示文字 - 固定在屏幕底部
    let hint_row = area.bottom().saturating_sub(1);
    if !app.message.is_empty() {
        // Show message if present (use display_width so CJK chars count as 2)
        let msg_dw = display_width(&app.message) as u16;
        let msg_w = msg_dw.min(area.width);
        let msg_col = area.x + area.width.saturating_sub(msg_dw) / 2;
        f.render_widget(
            Paragraph::new(app.message.as_str()).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Rect::new(msg_col, hint_row, msg_w, 1),
        );
    } else {
        let hint = i18n::t("menu.hint_nav", lang);
        let hint_dw = display_width(hint) as u16;
        let hint_w = hint_dw.min(area.width);
        let hint_col = area.x + area.width.saturating_sub(hint_dw) / 2;
        f.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::White)),
            Rect::new(hint_col, hint_row, hint_w, 1),
        );
    }
}

fn draw_menu_box(f: &mut Frame, x: u16, y: u16, w: u16, items: &[String], selected: usize, area: Rect) {
    if y >= area.bottom() { return; }
    let h = items.len() as u16 + 2; // 顶 + items + 底
    if y + h > area.bottom() { return; }

    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let block_area = Rect::new(x, y, w, h);
    f.render_widget(block.clone(), block_area);

    let inner = block.inner(block_area);
    if inner.height == 0 { return; }

    let mut lines: Vec<Line> = Vec::with_capacity(items.len());
    for (i, text) in items.iter().enumerate() {
        let prefix = if i == selected { "▸ " } else { "  " };
        let line_text = format!("{}{}", prefix, text);
        let style = if i == selected {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };
        lines.push(Line::from(Span::styled(line_text, style)));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

// ── 游戏画面 ──

fn draw_game(f: &mut Frame, app: &App) {
    let area = f.area();
    let bg = parse_color(&app.settings.bg_color);
    let border = parse_color(&app.settings.border_color);
    // 整体背景
    f.render_widget(
        Block::default().style(Style::default().bg(bg)),
        area,
    );

    let layout = compute_game_layout_from_rect(f.area(), app);

    // 左侧：Status / Navigator / Logs
    draw_status_panel(f, &layout, app, bg, border);
    draw_navigator_panel(f, &layout, app, bg, border);
    draw_logs_panel(f, &layout, app, bg, border);

    // 中间：数独网格 + 消息行
    draw_sudoku_grid(f, &layout, app, bg, border);
    draw_message(f, &layout, app, bg);

    // 按钮栏
    draw_button_bar(f, &layout, app, bg, border);

    // 右侧：3D 立方体 + 商店预留
    if app.settings.show_cube == "yes" {
        draw_3d_cube(f, &layout, app);
    }
    draw_shop_panel(f, &layout, app, bg, border);
}

pub fn compute_game_layout_from_rect(area: Rect, app: &App) -> GameLayout {
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let grid_inner_w = (1 + 9 * (cw + 1)) as u16;
    let grid_inner_h = (1 + 9 * (ch + 1)) as u16;
    let grid_h = grid_inner_h + 2; // 外边框上下各 1
    let grid_w = grid_inner_w + 2; // 外边框左右各 1

    // 各方向指示边占 1 列/行
    let dir_border: u16 = 1;
    let center_w = grid_w + dir_border * 2; // 含 4 个方向边

    // 最小/默认列宽
    let left_min_w: u16 = 22;
    let cube_w: u16 = app.settings.cube_width.parse().unwrap_or(20);
    let right_min_w: u16 = cube_w + 4; // direction borders + cube width + 2

    // 整体高：方向边(2) + 网格 + 消息 + 按钮 + 间隔
    let msg_h = 1u16;
    let btn_h = 3u16;
    let gap = 1u16;
    let total_h = dir_border * 2 + grid_h + gap + msg_h + gap + btn_h;

    // ── 纵向总布局：垂直居中，长度固定 ──
    let outer_v = Layout::vertical([Constraint::Length(total_h)])
        .flex(ratatui::layout::Flex::Center)
        .split(area);
    let v_chunk = outer_v[0];

    // ── 横向三列布局 ──
    // 策略：
    //   - 屏幕过窄时（< center_w + left_min + right_min）：左右列收缩到 0，中心保留
    //   - 屏幕足够时：根据可用空间按比例计算左右两列的实际宽度，
    //     然后用 Length 约束精确放置 —— 避免中间网格"挤在中间"。
    let min_w_sum = left_min_w.saturating_add(right_min_w).saturating_add(center_w);
    if area.width < min_w_sum {
        // 太窄：直接退化为纯中心列，等宽布局
        let h_layout = Layout::horizontal([Constraint::Length(center_w)])
            .flex(ratatui::layout::Flex::Center);
        let h_chunks = h_layout.split(v_chunk);
        let left_chunk = Rect::default();
        let center_chunk = h_chunks[0];
        let right_chunk = Rect::default();
        return build_layout(app, left_chunk, center_chunk, right_chunk, center_w, dir_border, grid_h, grid_w, msg_h, btn_h, gap, cube_w, area);
    }

    // 按比例分配左右列宽度（1:1）
    let side_total = area.width.saturating_sub(center_w);
    let half = side_total / 2;
    let left_actual = half.max(left_min_w);
    let right_actual = side_total.saturating_sub(left_actual).max(right_min_w);

    // 使用 Length 约束精确放置；Flex::Legacy 下整体填满整宽
    let h_layout = Layout::horizontal([
        Constraint::Length(left_actual),
        Constraint::Length(center_w),
        Constraint::Length(right_actual),
    ])
    .flex(ratatui::layout::Flex::Legacy);
    let h_chunks = h_layout.split(v_chunk);
    let left_chunk = h_chunks[0];
    let center_chunk = h_chunks[1];
    let right_chunk = h_chunks[2];

    build_layout(app, left_chunk, center_chunk, right_chunk, center_w, dir_border, grid_h, grid_w, msg_h, btn_h, gap, cube_w, area)
}

#[allow(clippy::too_many_arguments)]
fn build_layout(
    app: &App,
    left_chunk: Rect,
    center_chunk: Rect,
    right_chunk: Rect,
    _center_w: u16,
    dir_border: u16,
    grid_h: u16,
    grid_w: u16,
    msg_h: u16,
    btn_h: u16,
    gap: u16,
    _cube_w: u16,
    area: Rect,
) -> GameLayout {
    // ── 中间列：方向上 / 网格 / 方向下 / 消息 / 按钮 ──
    let center_layout = Layout::vertical([
        Constraint::Length(dir_border),                            // 顶方向边
        Constraint::Length(grid_h),                                // 网格(含外框)
        Constraint::Length(dir_border),                            // 底方向边
        Constraint::Length(gap),
        Constraint::Length(msg_h),
        Constraint::Length(gap),
        Constraint::Length(btn_h),
    ]);
    let center_parts = center_layout.split(center_chunk);
    let grid_top_dir = center_parts[0];
    let grid_outer = center_parts[1];
    let grid_bot_dir = center_parts[2];
    let _msg_gap = center_parts[3];
    let msg_area = center_parts[4];
    let _btn_gap = center_parts[5];
    let btn_area = center_parts[6];

    // 网格内容（去掉外框）
    let grid_inner_rect = grid_outer.inner(Margin { vertical: 1, horizontal: 1 });

    // 中间列水平布局：左方向边 / 网格外框 / 右方向边
    let center_h_layout = Layout::horizontal([
        Constraint::Length(dir_border),
        Constraint::Length(grid_w),
        Constraint::Length(dir_border),
    ]);
    let center_h_parts = center_h_layout.split(Rect::new(
        center_chunk.x,
        grid_outer.y,
        center_chunk.width,
        grid_outer.height,
    ));
    let grid_left_dir = center_h_parts[0];
    let _grid_center = center_h_parts[1];
    let grid_right_dir = center_h_parts[2];
    // 将方向边对齐到网格高度
    let grid_left_dir = Rect::new(grid_left_dir.x, grid_outer.y, dir_border, grid_outer.height);
    let grid_right_dir = Rect::new(grid_right_dir.x, grid_outer.y, dir_border, grid_outer.height);

    // 网格框 (含外框)
    let grid_frame = grid_outer;

    // ── 左列：Status / Navigator / Logs ──
    let left_layout = Layout::vertical([
        Constraint::Length(6),  // status
        Constraint::Length(9),  // navigator
        Constraint::Fill(1),    // logs
    ]);
    let left_parts = left_layout.split(left_chunk);
    let status_panel = left_parts[0];
    let navigator_panel = left_parts[1];
    let logs_panel = left_parts[2];

    // ── 右列：3D 立方体 / 商店 ──
    let cube_h: u16 = app.settings.cube_height.parse().unwrap_or(18);
    let right_layout = Layout::vertical([
        Constraint::Length(cube_h),
        Constraint::Length(1),  // 间隔
        Constraint::Fill(1),    // shop
    ]);
    let right_parts = right_layout.split(right_chunk);
    let cube_area = right_parts[0];
    let shop_area = right_parts[2];

    // 立方体外侧方向边（与 grid 类似：上下左右各 1）
    let cube_dir_top = Rect::new(cube_area.x, cube_area.y.saturating_sub(1), cube_area.width, 1);
    let cube_dir_bot = Rect::new(cube_area.x, cube_area.y + cube_area.height, cube_area.width, 1);
    let cube_dir_left = Rect::new(cube_area.x.saturating_sub(1), cube_area.y, 1, cube_area.height);
    let cube_dir_right = Rect::new(cube_area.x + cube_area.width, cube_area.y, 1, cube_area.height);

    // ── 按钮定义 ──
    let btn_defs: Vec<(String, ButtonId, u16)> = (1..=9u8)
        .map(|n| {
            let label = format!("[{}]", n);
            let w = label.chars().count() as u16;
            (label, ButtonId::Number(n), w)
        })
        .chain(
            [
                ("[X]Erase", ButtonId::Erase),
                ("[H]Hint", ButtonId::Hint),
                ("[Z]Undo", ButtonId::Undo),
                ("[G]Guide", ButtonId::ToggleGuidance),
                ("[M]Mode", ButtonId::ToggleMode),
                ("[Q]Menu", ButtonId::Quit),
            ]
            .iter()
            .map(|(label, id)| {
                let w = label.chars().count() as u16;
                (label.to_string(), *id, w)
            }),
        )
        .collect();

    // 按钮栏使用 Block 渲染，buttons 区域是 Block 内部
    // Block 高度 = btn_h = 3, 内部高度 = 1
    let btn_block = Block::bordered()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1));
    let btn_inner = btn_block.inner(btn_area);
    let btn_row = btn_inner.y;

    let total_btn_w: usize = btn_defs
        .iter()
        .map(|(_, _, w)| *w as usize + 1)
        .sum::<usize>()
        .saturating_sub(1);
    let bar_x = btn_inner.x + btn_inner.width.saturating_sub(total_btn_w as u16) / 2;

    let mut col = bar_x;
    let mut buttons = Vec::new();
    for (label, id, w) in btn_defs {
        buttons.push(ButtonLayout {
            id,
            label,
            col,
            row: btn_row,
            width: w,
        });
        col += w + 1;
    }

    GameLayout {
        game_area: area,
        left_column: left_chunk,
        status_panel,
        navigator_panel,
        logs_panel,
        grid_area: grid_inner_rect,
        grid_frame,
        msg_area,
        btn_area,
        btn_content_x: bar_x,
        right_column: right_chunk,
        cube_area,
        shop_area,
        buttons,
        // 方向边区域
        grid_top_dir,
        grid_bot_dir,
        grid_left_dir,
        grid_right_dir,
        cube_dir_top,
        cube_dir_bot,
        cube_dir_left,
        cube_dir_right,
    }
}

/// 绘制带标题的通用面板（使用 Block::bordered 渲染边框与标题）。
/// `lines` 会被渲染到面板内部，超出 (panel.height - 2) 的部分会被截断。
fn draw_framed_panel(
    f: &mut Frame,
    panel: Rect,
    title: &str,
    title_color: Color,
    border_color: Color,
    bg: Color,
    lines: Vec<Line>,
) {
    if panel.width < 4 || panel.height < 3 {
        return;
    }
    let block = Block::bordered()
        .title(Line::from(Span::styled(
            format!(" {} ", title),
            Style::default().fg(title_color).add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg));

    // 先渲染 Block 自身（边框 + 标题）
    f.render_widget(block.clone(), panel);

    // 再渲染内容到 inner area
    let inner = block.inner(panel);
    let content_h = inner.height as usize;
    for (i, line) in lines.into_iter().take(content_h).enumerate() {
        f.render_widget(
            Paragraph::new(line).style(Style::default().bg(bg).fg(Color::White)),
            Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
        );
    }
}

/// 顶部 Status 面板：游戏名 / 难度 / 时间 / 进度。
fn draw_status_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.status_panel;
    let inner_w = (panel.width - 4) as usize; // 可用文本宽度(去边框去前导空格)
    let total = app.game.grid.cells.len();
    let filled = app.game.grid.cells.values().filter(|c| c.user_value.is_some()).count();
    let remaining = total - filled;
    let game_name = app.game.id.map_or(i18n::t("game.unnamed", lang).to_string(), |id| {
        if app.settings.naming_mode == "vivid" {
            i18n::vivid_name(id, lang)
        } else {
            format!("#{}", id)
        }
    });
    let progress_pct = if total == 0 { 0 } else { (filled * 100) / total };
    let bar_w = inner_w.saturating_sub(2).max(4);
    let filled_w = (bar_w * filled) / total.max(1);
    let progress_bar = format!(
        "[{}{}]",
        "█".repeat(filled_w),
        "░".repeat(bar_w.saturating_sub(filled_w))
    );

    let lines = vec![
        Line::from(vec![
            Span::styled(game_name.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(format!(
            " {}: {}  {}: {}",
            i18n::t("info.face", lang),
            face_name(app.current_face, lang),
            i18n::t("info.diff", lang),
            app.game.difficulty.as_str(),
        )),
        Line::from(format!(
            " {}: {}",
            i18n::t("info.time", lang),
            format_timer(app),
        )),
        Line::from(format!(" {}/{} ({}%)", remaining, total, progress_pct)),
        Line::from(progress_bar),
    ];

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.status", lang),
        Color::Cyan,
        border,
        bg,
        lines,
    );
}

/// Navigator 面板：方向键 / WASD / FBLRTU 提示 + 当前面指示。
fn draw_navigator_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.navigator_panel;
    let face_color = face_to_color(app.current_face);
    let (up, down, left, right, _back) = arrow_neighbor_faces(app.current_face);

    let current_marker = format!("● {} ({})", app.current_face.short_code(), face_name(app.current_face, lang));

    let lines = vec![
        Line::from(format!(" ↑ {}    [W]上", face_name(up, lang))),
        Line::from(format!(" ↓ {}    [S]下", face_name(down, lang))),
        Line::from(format!(" ← {}    [A]左", face_name(left, lang))),
        Line::from(format!(" → {}    [D]右", face_name(right, lang))),
        Line::from(format!(" [F]前 [B]后 [L]左 [R]右")),
        Line::from(format!(" [T]上 [U]下")),
        Line::from(Span::styled(
            current_marker,
            Style::default().fg(face_color).add_modifier(Modifier::BOLD),
        )),
    ];

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.navigator", lang),
        Color::Green,
        border,
        bg,
        lines,
    );
}

/// Logs 面板：最近 N 条操作。
fn draw_logs_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.logs_panel;
    let inner_h = panel.height.saturating_sub(2) as usize;

    // 收集行：新 -> 旧
    let mut entries: Vec<String> = app.action_log.iter().rev().cloned().collect();
    // 如果为空，至少给一行提示
    if entries.is_empty() {
        entries.push(i18n::t("panel.logs_empty", lang).to_string());
    }
    entries.truncate(inner_h);

    let lines: Vec<Line> = entries
        .into_iter()
        .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::White))))
        .collect();

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.logs", lang),
        Color::Yellow,
        border,
        bg,
        lines,
    );
}

/// 商店预留面板。
fn draw_shop_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.shop_area;
    if panel.width < 4 || panel.height < 3 {
        return;
    }

    let lines = vec![
        Line::from(Span::styled(
            i18n::t("panel.shop_hint1", lang),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            i18n::t("panel.shop_hint2", lang),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ];

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.shop", lang),
        Color::Magenta,
        border,
        bg,
        lines,
    );
}

fn draw_sudoku_grid(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let ox = layout.grid_area.x;
    let oy = layout.grid_area.y;
    let grid_w = layout.grid_area.width;
    let grid_h = layout.grid_area.height;

    // 外边框 - 颜色对应当前面（用 Block 渲染）
    let face_color = face_to_color(app.current_face);
    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(face_color))
        .style(Style::default().bg(bg));
    let block_area = Rect::new(ox.saturating_sub(1), oy.saturating_sub(1), grid_w + 2, grid_h + 2);
    f.render_widget(block, block_area);

    // 逐行逐列绘制网格
    for v in 0..9usize {
        // 分隔行
        let sep_y = oy + v as u16 * (ch as u16 + 1);
        draw_separator(f, ox, sep_y, cw, v == 0, false, v % 3 == 0, layout.grid_area, border);

        // 单元格内容行
        for line in 0..ch {
            let row_y = oy + v as u16 * (ch as u16 + 1) + 1 + line as u16;
            draw_cell_row(f, ox, row_y, cw, ch, v, line, app, layout.grid_area, bg, border);
        }
    }

    // 底部分隔行
    let bot_y = oy + 9 * (ch as u16 + 1);
    draw_separator(f, ox, bot_y, cw, false, true, true, layout.grid_area, border);

    // ── Direction indicator borders ──
    // 4 个方向上的邻居面色（用 layout 中预计算好的 Rect）
    let (up_face, down_face, left_face, right_face, back_face) = wasd_neighbor_faces(app.current_face);
    let up_color = face_to_color(up_face);
    let down_color = face_to_color(down_face);
    let left_color = face_to_color(left_face);
    let right_color = face_to_color(right_face);
    let back_color = face_to_color(back_face);

    // 顶方向
    let top_line: String = "▔".repeat(layout.grid_top_dir.width as usize);
    if layout.grid_top_dir.width > 0 && layout.grid_top_dir.height > 0 {
        f.render_widget(
            Paragraph::new(top_line).style(Style::default().fg(up_color)),
            layout.grid_top_dir,
        );
    }
    // 底方向
    let bot_line: String = "▁".repeat(layout.grid_bot_dir.width as usize);
    if layout.grid_bot_dir.width > 0 && layout.grid_bot_dir.height > 0 {
        f.render_widget(
            Paragraph::new(bot_line).style(Style::default().fg(down_color)),
            layout.grid_bot_dir,
        );
    }
    // 左方向
    if layout.grid_left_dir.width > 0 && layout.grid_left_dir.height > 0 {
        let mut s = String::new();
        for _ in 0..layout.grid_left_dir.height {
            s.push('▏');
            s.push('\n');
        }
        f.render_widget(
            Paragraph::new(s).style(Style::default().fg(left_color)),
            layout.grid_left_dir,
        );
    }
    // 右方向
    if layout.grid_right_dir.width > 0 && layout.grid_right_dir.height > 0 {
        let mut s = String::new();
        for _ in 0..layout.grid_right_dir.height {
            s.push('▕');
            s.push('\n');
        }
        f.render_widget(
            Paragraph::new(s).style(Style::default().fg(right_color)),
            layout.grid_right_dir,
        );
    }

    // ── Back-face indicator: solid block at bottom-right corner ──
    let outer_x = block_area.x;
    let outer_y = block_area.y;
    let outer_w = block_area.width;
    let outer_h = block_area.height;
    let indicator_x = outer_x + outer_w;
    let indicator_y = outer_y + outer_h;
    if indicator_x < f.area().width && indicator_y < f.area().height {
        f.render_widget(
            Paragraph::new("■").style(Style::default().fg(back_color)),
            Rect::new(indicator_x, indicator_y, 1, 1),
        );
    }
}

fn draw_separator(
    f: &mut Frame, x: u16, y: u16, cw: usize,
    is_top: bool, is_bottom: bool, is_thick_h: bool,
    bounds: Rect, border: Color,
) {
    if y >= bounds.bottom() { return; }

    let mut buf = String::new();
    // 左角
    buf.push(if is_top { '╔' } else if is_bottom { '╚' } else { '╟' });

    for u in 0..9usize {
        let h_char = if is_thick_h { '═' } else { '─' };
        buf.push_str(&h_char.to_string().repeat(cw));

        if u < 8 {
            let is_major_v = (u + 1) % 3 == 0;
            buf.push(if is_top {
                if is_major_v { '╦' } else { '╤' }
            } else if is_bottom {
                if is_major_v { '╩' } else { '╧' }
            } else if is_thick_h {
                if is_major_v { '╬' } else { '╪' }
            } else if is_major_v {
                '╫'
            } else {
                '┼'
            });
        }
    }

    // 右角
    buf.push(if is_top { '╗' } else if is_bottom { '╝' } else { '╢' });

    let w = buf.chars().count() as u16;
    f.render_widget(
        Paragraph::new(buf).style(Style::default().fg(border)),
        Rect::new(x, y, w.min(bounds.width), 1),
    );
}

fn draw_cell_row(
    f: &mut Frame, x: u16, y: u16, cw: usize, ch: usize,
    v: usize, line: usize, app: &App, bounds: Rect, _bg: Color, border: Color,
) {
    if y >= bounds.bottom() { return; }

    let mut spans: Vec<Span> = Vec::new();

    for u in 0..9usize {
        // 竖线
        let v_char = if u % 3 == 0 { '║' } else { '│' };
        spans.push(Span::styled(
            v_char.to_string(),
            Style::default().fg(border),
        ));

        // 单元格内容
        let coord = app.current_face.to_cube(u as u8, v as u8);
        let cell = app.game.grid.get(&coord);
        let selected = app.cursor == (u as u8, v as u8);
        let is_given = cell.map(|c| c.given).unwrap_or(false);
        let value = cell.and_then(|c| c.user_value);
        let is_error = value.map_or(false, |n| is_wrong(app, coord, n));

        let (in_same_group, has_same_number) = if app.guidance && !selected {
            let sel_coord = app.current_face.to_cube(app.cursor.0, app.cursor.1);
            let same_row = app.cursor.1 == v as u8;
            let same_col = app.cursor.0 == u as u8;
            let same_box = app.cursor.0 / 3 == u as u8 / 3 && app.cursor.1 / 3 == v as u8 / 3;
            let in_group = same_row || same_col || same_box;
            let sel_value = app.game.grid.get(&sel_coord).and_then(|c| c.user_value);
            let same_num = value.is_some() && value == sel_value;
            (in_group, same_num)
        } else {
            (false, false)
        };

        let mid_line = ch / 2;
        let mut content = " ".repeat(cw);
        if line == mid_line {
            if let Some(n) = value {
                let s = ((b'0' + n) as char).to_string();
                let start = (cw - 1) / 2;
                content.replace_range(start..start + 1, &s);
            }
        }

        let guide_group = parse_color(&app.settings.guide_group_color);
        let guide_same = parse_color(&app.settings.guide_same_color);
        let style = if selected && is_error {
            Style::default().bg(Color::White).fg(Color::Red)
        } else if selected && app.blink_on {
            Style::default().bg(Color::White).fg(Color::Black)
        } else if selected {
            Style::default().bg(Color::Gray).fg(Color::White)
        } else if in_same_group && has_same_number {
            Style::default().bg(guide_same).fg(Color::White)
        } else if in_same_group {
            Style::default().bg(guide_group).fg(Color::White)
        } else if has_same_number {
            Style::default().bg(guide_same).fg(Color::White)
        } else if is_error {
            Style::default().fg(Color::Red)
        } else if is_given {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if value.is_some() {
            Style::default().fg(border)
        } else {
            Style::default().fg(Color::White)
        };

        spans.push(Span::styled(content, style));
    }

    // 右侧封口
    spans.push(Span::styled("║".to_string(), Style::default().fg(border)));

    let total_w: u16 = spans.iter().map(|s| s.content.chars().count() as u16).sum();
    f.render_widget(
        Paragraph::new(Line::from(spans)),
        Rect::new(x, y, total_w.min(bounds.width), 1),
    );
}

fn draw_message(f: &mut Frame, layout: &GameLayout, app: &App, _bg: Color) {
    if app.message.is_empty() { return; }
    let style = Style::default().fg(Color::Green);
    f.render_widget(
        Paragraph::new(app.message.as_str()).style(style),
        layout.msg_area,
    );
}

fn draw_button_bar(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    if layout.btn_area.width < 4 || layout.btn_area.height < 3 { return; }

    // Block 自带边框与内部 padding
    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(bg));
    f.render_widget(block.clone(), layout.btn_area);

    // 按钮渲染到 Block inner 区域
    let inner = block.inner(layout.btn_area);
    if inner.height == 0 || inner.width == 0 { return; }

    // 内容行：用一行 Line 包含全部按钮，hover 时反色
    let mut spans: Vec<Span> = Vec::with_capacity(layout.buttons.len() * 2);
    for (i, btn) in layout.buttons.iter().enumerate() {
        let is_hover = app.hover_button == Some(btn.id);
        let style = if is_hover {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(bg).fg(Color::White)
        };
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(&btn.label, style));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)),
        inner,
    );
}

// ── 3D 立方体 ──

fn draw_3d_cube(f: &mut Frame, layout: &GameLayout, app: &App) {
    let area = layout.cube_area;
    if area.width < 10 || area.height < 10 { return; }

    // ── Arrow-key direction indicator border (outer) ──
    let (up_face, down_face, left_face, right_face, _back_face) = arrow_neighbor_faces(app.current_face);
    let up_color = face_to_color(up_face);
    let down_color = face_to_color(down_face);
    let left_color = face_to_color(left_face);
    let right_color = face_to_color(right_face);

    // 顶方向
    if layout.cube_dir_top.width > 0 && layout.cube_dir_top.height > 0 {
        let line: String = "▔".repeat(layout.cube_dir_top.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(up_color)),
            layout.cube_dir_top,
        );
    }
    // 底方向
    if layout.cube_dir_bot.width > 0 && layout.cube_dir_bot.height > 0 {
        let line: String = "▁".repeat(layout.cube_dir_bot.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(down_color)),
            layout.cube_dir_bot,
        );
    }
    // 左方向
    if layout.cube_dir_left.width > 0 && layout.cube_dir_left.height > 0 {
        let mut s = String::new();
        for _ in 0..layout.cube_dir_left.height {
            s.push('▏');
            s.push('\n');
        }
        f.render_widget(
            Paragraph::new(s).style(Style::default().fg(left_color)),
            layout.cube_dir_left,
        );
    }
    // 右方向
    if layout.cube_dir_right.width > 0 && layout.cube_dir_right.height > 0 {
        let mut s = String::new();
        for _ in 0..layout.cube_dir_right.height {
            s.push('▕');
            s.push('\n');
        }
        f.render_widget(
            Paragraph::new(s).style(Style::default().fg(right_color)),
            layout.cube_dir_right,
        );
    }

    // 绘制外边框（用 Block）
    let border_color = face_to_color(app.current_face);
    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    f.render_widget(block.clone(), area);

    // 内容区域（去掉边框）
    let content_area = block.inner(area);
    if content_area.width < 2 || content_area.height < 2 { return; }

    let cx = content_area.x as f64 + content_area.width as f64 / 2.0;
    let cy = content_area.y as f64 + content_area.height as f64 / 2.0;
    let scale_factor: f64 = app.settings.cube_scale.parse().unwrap_or(0.38);
    // 终端字符高宽比约2:1，Y方向缩小以补偿
    let scale_x = content_area.width as f64 * scale_factor;
    let scale_y = content_area.height as f64 * scale_factor * 0.5;

    let cos_y = app.cube_angle_y.cos();
    let sin_y = app.cube_angle_y.sin();
    let cos_x = app.cube_angle_x.cos();
    let sin_x = app.cube_angle_x.sin();

    // 8 vertices of a unit cube
    let verts: [(f64, f64, f64); 8] = [
        (-1.0, -1.0, -1.0), ( 1.0, -1.0, -1.0),
        ( 1.0,  1.0, -1.0), (-1.0,  1.0, -1.0),
        (-1.0, -1.0,  1.0), ( 1.0, -1.0,  1.0),
        ( 1.0,  1.0,  1.0), (-1.0,  1.0,  1.0),
    ];

    // Project: rotate Y then X, with aspect ratio correction
    let project = |x: f64, y: f64, z: f64| {
        // Rotate around Y axis
        let x1 = x * cos_y + z * sin_y;
        let z1 = -x * sin_y + z * cos_y;
        // Rotate around X axis
        let y1 = y * cos_x - z1 * sin_x;
        let z2 = y * sin_x + z1 * cos_x;
        (cx + x1 * scale_x, cy - y1 * scale_y, z2)
    };

    let proj: Vec<(f64, f64, f64)> = verts.iter().map(|&(x, y, z)| project(x, y, z)).collect();

    // 6 faces: vertex indices + color
    let faces: [([usize; 4], Face); 6] = [
        ([4, 5, 6, 7], Face::Front),  // front (z=1)
        ([0, 3, 2, 1], Face::Back),   // back (z=-1)
        ([0, 4, 7, 3], Face::Left),   // left (x=-1)
        ([1, 2, 6, 5], Face::Right),  // right (x=1)
        ([3, 7, 6, 2], Face::Top),    // top (y=1)
        ([0, 1, 5, 4], Face::Bottom), // bottom (y=-1)
    ];

    // Sort faces by average z (painter's algorithm, far first)
    let mut sorted_faces: Vec<([usize; 4], Face)> = faces.to_vec();
    sorted_faces.sort_by(|a, b| {
        let za: f64 = a.0.iter().map(|&i| proj[i].2).sum::<f64>() / 4.0;
        let zb: f64 = b.0.iter().map(|&i| proj[i].2).sum::<f64>() / 4.0;
        za.partial_cmp(&zb).unwrap()
    });

    // Draw faces
    for (indices, face) in &sorted_faces {
        let color = face_to_color(*face);
        let pts: Vec<(f64, f64)> = indices.iter().map(|&i| (proj[i].0, proj[i].1)).collect();

        // Fill face with colored dots
        let min_x = pts.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let max_x = pts.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        let min_y = pts.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let max_y = pts.iter().map(|p| p.1).fold(f64::MIN, f64::max);

        for py in (min_y as u16)..=(max_y as u16) {
            for px in (min_x as u16)..=(max_x as u16) {
                if px < content_area.x || px >= content_area.x + content_area.width { continue; }
                if py < content_area.y || py >= content_area.y + content_area.height { continue; }
                if point_in_quad(px as f64, py as f64, &pts) {
                    let style = if *face == app.current_face {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };
                    let ch = if *face == app.current_face { '●' } else { '░' };
                    f.render_widget(
                        Paragraph::new(ch.to_string()).style(style),
                        Rect::new(px, py, 1, 1),
                    );
                }
            }
        }

        // Draw edges
        for i in 0..4 {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % 4];
            draw_line(f, x1, y1, x2, y2, color, content_area);
        }
    }

    // Draw orbiting sphere - position on the face's center, projected outward
    let face_center_3d = match app.current_face {
        Face::Front  => (0.0, 0.0, 1.8),
        Face::Back   => (0.0, 0.0, -1.8),
        Face::Left   => (-1.8, 0.0, 0.0),
        Face::Right  => (1.8, 0.0, 0.0),
        Face::Top    => (0.0, 1.8, 0.0),
        Face::Bottom => (0.0, -1.8, 0.0),
    };
    let (sx, sy, _) = project(face_center_3d.0, face_center_3d.1, face_center_3d.2);
    let sphere_color = face_to_color(app.current_face);

    let sx_u = sx as u16;
    let sy_u = sy as u16;
    if sx_u >= content_area.x && sx_u < content_area.x + content_area.width
        && sy_u >= content_area.y && sy_u < content_area.y + content_area.height {
        f.render_widget(
            Paragraph::new("◉").style(Style::default().fg(sphere_color).add_modifier(Modifier::BOLD)),
            Rect::new(sx_u, sy_u, 1, 1),
        );
    }

    // Face label - inside border at bottom
    let lang = Lang::from_code(&app.settings.language);
    let label = face_name(app.current_face, lang);
    let label_x = area.x + area.width.saturating_sub(label.len() as u16) / 2;
    let label_y = area.y + area.height.saturating_sub(1); // 底部边框上一行
    if label_y > area.y {
        f.render_widget(
            Paragraph::new(label).style(Style::default().fg(face_to_color(app.current_face))),
            Rect::new(label_x, label_y, label.len() as u16, 1),
        );
    }
}

fn point_in_quad(px: f64, py: f64, pts: &[(f64, f64)]) -> bool {
    if pts.len() != 4 { return false; }
    // Cross product method
    let mut inside = true;
    for i in 0..4 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % 4];
        let cross = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if cross > 0.0 { inside = false; break; }
    }
    if inside { return true; }
    inside = true;
    for i in 0..4 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % 4];
        let cross = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if cross < 0.0 { inside = false; break; }
    }
    inside
}

fn draw_line(f: &mut Frame, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, bounds: Rect) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let steps = dx.max(dy).ceil() as u16;
    if steps == 0 { return; }

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = (x1 + (x2 - x1) * t) as u16;
        let y = (y1 + (y2 - y1) * t) as u16;
        if x >= bounds.x && x < bounds.x + bounds.width && y >= bounds.y && y < bounds.y + bounds.height {
            f.render_widget(
                Paragraph::new("·").style(Style::default().fg(color)),
                Rect::new(x, y, 1, 1),
            );
        }
    }
}

// ── 公共工具函数 ──

pub fn find_button_at(layout: &GameLayout, col: u16, row: u16) -> Option<ButtonId> {
    for btn in &layout.buttons {
        if row == btn.row && col >= btn.col && col < btn.col + btn.width {
            return Some(btn.id);
        }
    }
    None
}

pub fn cell_at(layout: &GameLayout, cw: usize, ch: usize, col: u16, row: u16) -> Option<(u8, u8)> {
    let gx = col.saturating_sub(layout.grid_area.x);
    let gy = row.saturating_sub(layout.grid_area.y);

    if gy % (ch as u16 + 1) == 0 { return None; }
    let v = (gy / (ch as u16 + 1)) as u8;
    if v >= 9 { return None; }

    if gx % (cw as u16 + 1) == 0 { return None; }
    let u = (gx / (cw as u16 + 1)) as u8;
    if u >= 9 { return None; }

    Some((u, v))
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// Left-align `s` in a field of `width` display columns (pad on the right).
fn pad_right(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if dw >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - dw))
    }
}

/// Center `s` in a field of `width` display columns.
fn pad_center(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if dw >= width {
        return s.to_string();
    }
    let total = width - dw;
    let left = total / 2;
    let right = total - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// Build a bordered line `│{padded}│` where `padded` fills `inner_w` display
/// columns. `center` toggles center vs left alignment. Correctly handles
/// double-width CJK characters (unlike `format!("{:^width$}")`).
fn bordered_line(content: &str, inner_w: usize, center: bool) -> String {
    let padded = if center {
        pad_center(content, inner_w)
    } else {
        pad_right(content, inner_w)
    };
    format!("│{}│", padded)
}

fn format_timer(app: &App) -> String {
    let elapsed = total_elapsed(app);
    format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
}

fn is_wrong(app: &App, coord: sudokube_core::cube::CubeCoord, value: u8) -> bool {
    app.game.solution.get(&coord).map_or(true, |&sol| sol != value)
}

fn face_name(face: Face, lang: Lang) -> &'static str {
    match face {
        Face::Front => i18n::t("game.face_front", lang),
        Face::Back => i18n::t("game.face_back", lang),
        Face::Left => i18n::t("game.face_left", lang),
        Face::Right => i18n::t("game.face_right", lang),
        Face::Top => i18n::t("game.face_top", lang),
        Face::Bottom => i18n::t("game.face_bottom", lang),
    }
}

fn face_to_color(face: Face) -> Color {
    match face {
        Face::Front => Color::Red,
        Face::Back => Color::Blue,
        Face::Left => Color::Green,
        Face::Right => Color::Yellow,
        Face::Top => Color::Magenta,
        Face::Bottom => Color::Cyan,
    }
}

/// WASD edge-crossing neighbors: which face you reach when cursor moves off the edge.
/// Based on move_on_surface logic in input.rs.
fn wasd_neighbor_faces(face: Face) -> (Face, Face, Face, Face, Face) {
    match face {
        Face::Front  => (Face::Bottom, Face::Top, Face::Left, Face::Right, Face::Back),
        Face::Back   => (Face::Left, Face::Right, Face::Bottom, Face::Top, Face::Front),
        Face::Top    => (Face::Back, Face::Front, Face::Left, Face::Right, Face::Bottom),
        Face::Bottom => (Face::Left, Face::Right, Face::Back, Face::Front, Face::Top),
        Face::Left   => (Face::Bottom, Face::Top, Face::Back, Face::Front, Face::Right),
        Face::Right  => (Face::Back, Face::Front, Face::Bottom, Face::Top, Face::Left),
    }
}

/// Arrow-key face-switching neighbors: which face you switch to with ↑↓←→.
/// Based on switch_face logic in input.rs.
fn arrow_neighbor_faces(face: Face) -> (Face, Face, Face, Face, Face) {
    match face {
        Face::Front  => (Face::Top, Face::Bottom, Face::Left, Face::Right, Face::Back),
        Face::Back   => (Face::Top, Face::Bottom, Face::Right, Face::Left, Face::Front),
        Face::Top    => (Face::Back, Face::Front, Face::Left, Face::Right, Face::Bottom),
        Face::Bottom => (Face::Front, Face::Back, Face::Left, Face::Right, Face::Top),
        Face::Left   => (Face::Top, Face::Bottom, Face::Back, Face::Front, Face::Right),
        Face::Right  => (Face::Top, Face::Bottom, Face::Front, Face::Back, Face::Left),
    }
}

pub fn parse_color(name: &str) -> Color {
    match name {
        "black" => Color::Black,
        "darkgray" => Color::DarkGray,
        "white" => Color::White,
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "red" => Color::Red,
        "yellow" => Color::Yellow,
        "gray" => Color::Gray,
        _ => Color::Cyan,
    }
}

// ── 设置画面 ──

fn draw_settings(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let box_w = 42u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let fields = &app.settings_ui.fields;
    let content_h = fields.len() as u16;
    let box_h = content_h + 4; // title(2) + content + hint(1) + bottom(1)
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;

    // 标题
    let inner_w = box_w as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, box_y, box_w, 1),
    );

    let title_line = bordered_line(i18n::t("settings.title", lang), inner_w, true);
    f.render_widget(
        Paragraph::new(title_line).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, box_y + 1, box_w, 1),
    );

    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, box_y + 2, box_w, 1),
    );

    // 设置项
    for (i, field) in fields.iter().enumerate() {
        let row = box_y + 3 + i as u16;
        let is_selected = i == app.settings_ui.selected;
        // Translate special values
        let display_value = if field.label == "Naming Mode" {
            i18n::t(&format!("naming.{}", field.value), lang).to_string()
        } else {
            field.value.clone()
        };
        let label_part = format!(" {} ", field.label);
        let value_part = format!(" < {} >", display_value);
        let padding = inner_w.saturating_sub(display_width(&label_part) + display_width(&value_part));
        let line_text = format!("{}{}{}", label_part, " ".repeat(padding), value_part);

        let style = if is_selected {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };

        let line = Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Cyan)),
            Span::styled(line_text, style),
            Span::styled("│", Style::default().fg(Color::Cyan)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(box_x, row, box_w, 1));
    }

    // 提示
    let hint_row = box_y + 3 + content_h;
    let hint = bordered_line(i18n::t("settings.hint", lang), inner_w, true);
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        Rect::new(box_x, hint_row, box_w, 1),
    );

    // 底部
    let bot_row = hint_row + 1;
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, bot_row, box_w, 1),
    );
}

// ── 胜利画面 ──

fn draw_victory(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let lang = Lang::from_code(&app.settings.language);
    let title = i18n::t("victory.title", lang);
    let subtitle = i18n::t("victory.subtitle", lang);

    // Calculate remaining seconds
    let remaining_secs = app.victory_countdown
        .map(|until| {
            let left = (until - Instant::now()).as_secs() as u32;
            left.max(0)
        })
        .unwrap_or(0);

    let countdown_text = format!("{}  |  {}", i18n::t("victory.enter", lang), format!("{}", remaining_secs));

    let box_w = 36u16;
    let box_h = 6u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;

    let inner_w = box_w as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));

    // Top
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y, box_w, 1),
    );
    // Title
    let title_line = bordered_line(&title, inner_w, true);
    f.render_widget(
        Paragraph::new(title_line).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Rect::new(box_x, box_y + 1, box_w, 1),
    );
    // Subtitle
    let sub_line = bordered_line(&subtitle, inner_w, true);
    f.render_widget(
        Paragraph::new(sub_line).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Rect::new(box_x, box_y + 2, box_w, 1),
    );
    // Separator
    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y + 3, box_w, 1),
    );
    // Info
    let diff = app.game.difficulty.as_str();
    let elapsed = total_elapsed(app);
    let info_line = format!("│ {:^width$} │", format!("{}  {:02}:{:02}", diff, elapsed / 60, elapsed % 60), width = inner_w - 2);
    f.render_widget(
        Paragraph::new(info_line).style(Style::default().fg(Color::White)),
        Rect::new(box_x, box_y + 4, box_w, 1),
    );
    // Bottom + countdown
    let bot_line = bordered_line(&countdown_text, inner_w, true);
    f.render_widget(
        Paragraph::new(bot_line).style(Style::default().fg(Color::DarkGray)),
        Rect::new(box_x, box_y + 5, box_w, 1),
    );
    // Bottom border
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y + box_h, box_w, 1),
    );
}

// ── 导出选择画面 ──

fn draw_export_select(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let box_w = 34u16;
    let box_h = 5u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;
    let inner_w = box_w as usize - 2;

    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    let title = bordered_line(i18n::t("menu.export", lang), inner_w, true);

    f.render_widget(Paragraph::new(top).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y, box_w, 1));
    f.render_widget(Paragraph::new(title).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 1, box_w, 1));

    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(Paragraph::new(sep).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 2, box_w, 1));

    let opts = [
        i18n::t("export.encrypted", lang),
        i18n::t("export.plaintext", lang),
    ];
    for (i, label) in opts.iter().enumerate() {
        let is_sel = i == app.export_select;
        let prefix = if is_sel { ">" } else { " " };
        let text = format!("{} {}", prefix, label);
        let style = if is_sel {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };
        let line = bordered_line(&text, inner_w, false);
        f.render_widget(Paragraph::new(line).style(style), Rect::new(box_x, box_y + 3 + i as u16, box_w, 1));
    }
    f.render_widget(Paragraph::new(bot).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + box_h, box_w, 1));
}

// ── 导入输入画面 ──

fn draw_import_input(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let box_w = 50u16;
    let box_h = 5u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;
    let inner_w = box_w as usize - 2;

    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    let title = bordered_line(i18n::t("menu.import", lang), inner_w, true);

    f.render_widget(Paragraph::new(top).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y, box_w, 1));
    f.render_widget(Paragraph::new(title).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 1, box_w, 1));

    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(Paragraph::new(sep).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 2, box_w, 1));

    // Input field
    let display_text = if app.import_buffer.is_empty() {
        i18n::t("import.paste", lang).to_string()
    } else {
        let buf = &app.import_buffer;
        let max_chars = inner_w.saturating_sub(6); // reserve for " ..." and padding
        if buf.chars().count() > max_chars {
            let truncated: String = buf.chars().skip(buf.chars().count().saturating_sub(max_chars)).collect();
            format!("...{}", truncated)
        } else {
            buf.clone()
        }
    };
    let input_style = if app.import_buffer.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let input_line = format!("│ {} │", pad_right(&display_text, inner_w - 2));
    f.render_widget(Paragraph::new(input_line).style(input_style), Rect::new(box_x, box_y + 3, box_w, 1));

    let hint = bordered_line("Enter OK / Esc Cancel", inner_w, true);
    f.render_widget(Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)), Rect::new(box_x, box_y + 4, box_w, 1));

    f.render_widget(Paragraph::new(bot).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + box_h, box_w, 1));
}

// ── 生成进度画面 ──

fn draw_generating(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let spinners = ['⠋', '⠙', '⠹', '⠸'];
    let spinner_char = if let Some(ref gen_state) = app.generating {
        spinners[gen_state.spinner as usize]
    } else {
        '⠋'
    };

    let elapsed = if let Some(ref gen_state) = app.generating {
        gen_state.started.elapsed().as_secs()
    } else {
        0
    };

    let lang = Lang::from_code(&app.settings.language);
    let text = format!("  {} {} {}s  ", spinner_char, i18n::t("msg.generating", lang), elapsed);
    let bar_w = 40u16;
    let x = area.x + area.width.saturating_sub(bar_w) / 2;
    let y = area.y + area.height / 2;

    let inner_w = bar_w as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));

    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y - 1, bar_w, 1),
    );

    // 进度条行
    let progress_chars = "█▓▒░";
    let anim_step = (elapsed as usize) % 4;
    let filled = inner_w.min((elapsed as usize * 3).min(inner_w));
    let mut bar = String::new();
    for i in 0..inner_w {
        if i < filled {
            bar.push('█');
        } else if i == filled {
            bar.push(progress_chars.chars().nth(anim_step).unwrap());
        } else {
            bar.push('░');
        }
    }
    let bar_line = format!("│{}│", bar);
    f.render_widget(
        Paragraph::new(bar_line).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y, bar_w, 1),
    );

    // 文字行
    let content = bordered_line(&text, inner_w, true);
    f.render_widget(
        Paragraph::new(content).style(Style::default().fg(Color::White)),
        Rect::new(x, y + 1, bar_w, 1),
    );

    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y + 2, bar_w, 1),
    );
}
