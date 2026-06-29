use crossterm::event::{
    Event, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;
use sudokube_core::cube::Face;

use crate::config::Action;
use crate::i18n::{self, Lang};
use crate::render::{
    ButtonId, GameLayout, PagerAction, cell_at, compute_game_layout_from_rect, find_button_at,
    mode_label, pager_action_at, shop_item_at,
};
use crate::shop;
use crate::{App, current_coord};
use std::time::Duration;

use super::EventResult;

pub(super) fn handle_game_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    match event {
        Event::Resize(_, _) => {}
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            return handle_key(app, key);
        }
        Event::Mouse(mouse) => {
            let layout = compute_game_layout_from_rect(area, app);
            return handle_mouse(app, &layout, mouse);
        }
        _ => {}
    }
    EventResult::Continue
}

fn handle_key(app: &mut App, key: KeyEvent) -> EventResult {
    // 通过 keymap 系统解析按键为 Action
    let action = app
        .keymap
        .resolve(app.screen, app.snake.is_some(), key.code, key.modifiers);

    // 贪吃蛇小游戏运行中: 方向键控蛇,Esc/Q 退出;禁用其他游戏操作
    if app.snake.is_some() {
        if let Some(snake) = app.snake.as_mut() {
            if let Some(new_dir) = match action {
                Some(Action::SnakeUp) => Some((0i8, -1i8)),
                Some(Action::SnakeDown) => Some((0, 1)),
                Some(Action::SnakeLeft) => Some((-1, 0)),
                Some(Action::SnakeRight) => Some((1, 0)),
                _ => None,
            } {
                // 禁止 180° 反向
                if (new_dir.0 + snake.dir.0, new_dir.1 + snake.dir.1) != (0, 0) {
                    snake.dir = new_dir;
                }
            } else if matches!(action, Some(Action::SnakeQuit)) {
                if let Some(msg) = shop::end_snake_game(app) {
                    app.set_message(msg, Duration::from_secs(2));
                }
            }
        }
        return EventResult::Continue;
    }

    // 冻结状态(容错耗尽): 上下选择购买项, Enter购买, Q退出
    if app.game.frozen {
        match action {
            Some(Action::Quit) => return EventResult::BackToMenu,
            Some(Action::CursorUp) => {
                app.frozen_select = if app.frozen_select == 0 { 1 } else { 0 };
            }
            Some(Action::CursorDown) => {
                app.frozen_select = if app.frozen_select == 0 { 1 } else { 0 };
            }
            Some(Action::Confirm) | Some(Action::ShopBuy) => {
                let item = if app.frozen_select == 0 {
                    crate::shop::ItemType::LocalRevive
                } else {
                    crate::shop::ItemType::GlobalRevive
                };
                let lang = Lang::from_code(&app.settings.language);
                if app.buy_item(item) {
                    let name = i18n::t(item.name_key(), lang);
                    app.set_message(
                        format!("{} {}", i18n::t("shop.bought", lang), name),
                        Duration::from_secs(2),
                    );
                } else {
                    app.set_message(
                        i18n::t("shop.no_gold", lang).to_string(),
                        Duration::from_secs(2),
                    );
                }
            }
            _ => {}
        }
        return EventResult::Continue;
    }

    // Tab: 切换商店焦点
    if matches!(action, Some(Action::ShopFocus)) {
        app.toggle_shop_focus();
        return EventResult::Continue;
    }

    // 商店焦点激活时
    if app.shop_focused {
        let lang = Lang::from_code(&app.settings.language);
        let catalog_len = crate::shop::shop_catalog(app.settings.guide_owned == "on").len();
        let items_per_page: usize = 4;
        match action {
            Some(Action::Cancel) => {
                app.shop_focused = false;
            }
            Some(Action::ShopUp) => {
                if app.shop_selected == 0 {
                    app.shop_selected = catalog_len - 1;
                } else {
                    app.shop_selected -= 1;
                }
                // 自动翻页：若选中项不在当前页，向上翻页
                let page = (app.shop_selected / items_per_page) as u16;
                app.shop_page = page;
            }
            Some(Action::ShopDown) => {
                app.shop_selected = (app.shop_selected + 1) % catalog_len;
                // 自动翻页：若选中项不在当前页，向下翻页
                let page = (app.shop_selected / items_per_page) as u16;
                app.shop_page = page;
            }
            Some(Action::ShopBuy) => {
                if let Some(item) = crate::shop::shop_catalog(app.settings.guide_owned == "on")
                    .get(app.shop_selected)
                    .map(|s| s.item_type)
                {
                    if app.buy_item(item) {
                        let name = i18n::t(item.name_key(), lang);
                        app.set_message(
                            format!("{} {}×1", i18n::t("shop.bought", lang), name),
                            Duration::from_secs(2),
                        );
                    } else {
                        app.set_message(
                            i18n::t("shop.no_gold", lang).to_string(),
                            Duration::from_secs(2),
                        );
                    }
                }
            }
            _ => return EventResult::Continue,
        }
        return EventResult::Continue;
    }

    // 按钮栏翻页
    if matches!(action, Some(Action::BtnPagePrev)) {
        if app.btn_page > 0 {
            app.btn_page -= 1;
            app.hover_button = None;
        }
        return EventResult::Continue;
    }
    if matches!(action, Some(Action::BtnPageNext)) {
        app.btn_page = app.btn_page.saturating_add(1);
        app.hover_button = None;
        return EventResult::Continue;
    }

    match action {
        Some(Action::Quit) => return EventResult::BackToMenu,
        Some(Action::NewGame) => {
            app.push_log("New Game", 50);
            return EventResult::StartGenerating(app.game.difficulty);
        }
        Some(Action::ToggleMode) => {
            let new_mode = app.render_mode.toggle();
            // 如果处于草稿模式且新模式格子尺寸过小,先退出草稿模式防止在看不见的情况下输入
            if app.game.draft_mode {
                let new_cw = new_mode.cell_width(&app.settings);
                let new_ch = new_mode.cell_height();
                if new_cw < 3 || new_ch < 3 {
                    app.game.draft_mode = false;
                    let lang = Lang::from_code(&app.settings.language);
                    app.set_message(
                        i18n::t("msg.draft_off", lang).to_string(),
                        Duration::from_secs(2),
                    );
                }
            }
            app.render_mode = new_mode;
            let lang = Lang::from_code(&app.settings.language);
            let label = mode_label(app.render_mode, lang);
            app.set_message(label.to_string(), Duration::from_secs(2));
            app.push_log(format!("Mode: {}", label), 50);
        }
        Some(Action::ToggleDraft) => {
            // 检查格子尺寸是否足够
            let cw = app.render_mode.cell_width(&app.settings);
            let ch = app.render_mode.cell_height();
            let lang = Lang::from_code(&app.settings.language);
            if cw < 3 || ch < 3 {
                app.set_message(
                    i18n::t("msg.draft_too_small", lang).to_string(),
                    Duration::from_secs(2),
                );
                return EventResult::Continue;
            }
            let was_on = app.game.draft_mode;
            app.game.draft_mode = !was_on;
            // 注意: 不再根据模式开关设置 draft_visible。
            // 草稿内容在两种模式下都可见,仅在格子填入 user_value 时被隐藏(Erase 时恢复)。
            let key = if app.game.draft_mode { "msg.draft_on" } else { "msg.draft_off" };
            app.set_message(i18n::t(key, lang).to_string(), Duration::from_secs(2));
            app.push_log(i18n::t(key, lang), 50);
        }
        Some(Action::Erase) => {
            let coord = current_coord(app);
            // 先读取需要的信息
            let (has_value, old, has_draft) = {
                let cell = app.game.grid.get(&coord);
                match cell {
                    Some(c) if !c.given => (c.user_value.is_some(), c.user_value, !c.draft.is_empty()),
                    _ => (false, None, false),
                }
            };
            if has_value {
                app.game.set_value(coord, None);
                app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, None);
                // 擦除确定值后，如有草稿则标记可见
                if has_draft {
                    if let Some(cell) = app.game.grid.get_mut(&coord) {
                        cell.draft_visible = true;
                    }
                }
                app.push_log("Erase", 50);
            } else if has_draft {
                // 擦除草稿: 清除所有草稿内容并标记可见
                if let Some(cell) = app.game.grid.get_mut(&coord) {
                    cell.draft.clear();
                    cell.draft_visible = true;
                }
                app.push_log("Erase Draft", 50);
            }
        }
        Some(Action::ToggleGuidance) => {
            // Guide 切换: 仅在已购买(owned)时有效;未购则提示锁定
            if app.settings.guide_owned != "on" {
                let lang = Lang::from_code(&app.settings.language);
                app.set_message(i18n::t("guide.locked", lang).to_string(), Duration::from_secs(2));
                return EventResult::Continue;
            }
            let new_val = if app.settings.guide_enabled == "on" { "off" } else { "on" };
            app.settings.guide_enabled = new_val.into();
            app.settings.save_to_db();
            app.guidance = app.settings.guide_active();
            let lang = Lang::from_code(&app.settings.language);
            let key = if app.guidance {
                "msg.guide_on"
            } else {
                "msg.guide_off"
            };
            app.set_message(i18n::t(key, lang).to_string(), Duration::from_secs(2));
            app.push_log(i18n::t(key, lang), 50);
        }
        Some(Action::Hint) => {
            app.game.hint();
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(i18n::t("debug.hint", lang), 50);
            if app.game.check_completion() {
                app.trigger_victory();
                return EventResult::Continue;
            }
            app.set_message("已提示", Duration::from_secs(2));
        }
        Some(Action::Undo) => {
            app.game.undo();
            app.push_log("Undo", 50);
            app.set_message("已撤销", Duration::from_secs(2));
        }
        Some(Action::Number(n)) => {
            let value = n;
            let coord = current_coord(app);
            if app.game.draft_mode {
                // 草稿模式: 切换草稿数字
                if let Some(cell) = app.game.grid.get_mut(&coord) {
                    if cell.given || cell.user_value.is_some() {
                        return EventResult::Continue;
                    }
                    cell.toggle_draft(value);
                    cell.draft_visible = true;
                }
                app.push_log(
                    format!("Draft toggle {} at R{}C{}", value, app.cursor.1 + 1, app.cursor.0 + 1),
                    50,
                );
                return EventResult::Continue;
            }
            let old = app.game.grid.get(&coord).and_then(|c| c.user_value);
            // 跳过已给定/已有相同值的格子
            if let Some(cell) = app.game.grid.get(&coord) {
                if cell.given || cell.user_value == Some(value) {
                    return EventResult::Continue;
                }
            }
            app.game.set_value(coord, Some(value));
            app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, Some(value));
            // 设置确定值后清除该格的草稿
            if let Some(cell) = app.game.grid.get_mut(&coord) {
                cell.draft.clear();
            }
            // 容错检测
            let is_wrong = app.check_and_consume_error(coord, value);
            if is_wrong {
                let lang = Lang::from_code(&app.settings.language);
                let global_rem = app.global_errors_max;
                app.set_message(
                    format!(
                        "{} {}/{}(+{})",
                        crate::i18n::t("status.wrong", lang),
                        app.game.errors,
                        app.game.errors_max,
                        global_rem
                    ),
                    Duration::from_secs(2),
                );
            }
            app.push_log(
                format!(
                    "Placed {} at R{}C{}{}",
                    value,
                    app.cursor.1 + 1,
                    app.cursor.0 + 1,
                    if is_wrong { " ❌ " } else { "" }
                ),
                50,
            );
            if app.game.frozen {
                return EventResult::Continue;
            }
            if app.game.check_completion() {
                app.trigger_victory();
                return EventResult::Continue;
            }
        }
        Some(Action::CursorUp) => super::navigation::move_cursor_with_wrap(app, 0, -1),
        Some(Action::CursorLeft) => super::navigation::move_cursor_with_wrap(app, -1, 0),
        Some(Action::CursorDown) => super::navigation::move_cursor_with_wrap(app, 0, 1),
        Some(Action::CursorRight) => super::navigation::move_cursor_with_wrap(app, 1, 0),
        Some(Action::FaceUp) => {
            let prev = app.current_face;
            app.current_face = super::navigation::switch_face(app.current_face, 0, -1);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(
                    format!(
                        "→ {}",
                        super::navigation::face_label(app.current_face, lang)
                    ),
                    50,
                );
            }
        }
        Some(Action::FaceDown) => {
            let prev = app.current_face;
            app.current_face = super::navigation::switch_face(app.current_face, 0, 1);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(
                    format!(
                        "→ {}",
                        super::navigation::face_label(app.current_face, lang)
                    ),
                    50,
                );
            }
        }
        Some(Action::FaceLeft) => {
            let prev = app.current_face;
            app.current_face = super::navigation::switch_face(app.current_face, -1, 0);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(
                    format!(
                        "→ {}",
                        super::navigation::face_label(app.current_face, lang)
                    ),
                    50,
                );
            }
        }
        Some(Action::FaceRight) => {
            let prev = app.current_face;
            app.current_face = super::navigation::switch_face(app.current_face, 1, 0);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(
                    format!(
                        "→ {}",
                        super::navigation::face_label(app.current_face, lang)
                    ),
                    50,
                );
            }
        }
        Some(Action::FaceFront) => {
            app.current_face = Face::Front;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(
                format!(
                    "→ {}",
                    super::navigation::face_label(app.current_face, lang)
                ),
                50,
            );
        }
        Some(Action::FaceBack) => {
            app.current_face = Face::Back;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(
                format!(
                    "→ {}",
                    super::navigation::face_label(app.current_face, lang)
                ),
                50,
            );
        }
        Some(Action::FaceLeftJump) => {
            app.current_face = Face::Left;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(
                format!(
                    "→ {}",
                    super::navigation::face_label(app.current_face, lang)
                ),
                50,
            );
        }
        Some(Action::FaceRightJump) => {
            app.current_face = Face::Right;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(
                format!(
                    "→ {}",
                    super::navigation::face_label(app.current_face, lang)
                ),
                50,
            );
        }
        Some(Action::FaceTop) => {
            app.current_face = Face::Top;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(
                format!(
                    "→ {}",
                    super::navigation::face_label(app.current_face, lang)
                ),
                50,
            );
        }
        Some(Action::FaceBottom) => {
            app.current_face = Face::Bottom;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(
                format!(
                    "→ {}",
                    super::navigation::face_label(app.current_face, lang)
                ),
                50,
            );
        }
        Some(Action::DebugHintFace) => {
            if app.settings.debug_mode == "on" {
                super::navigation::debug_hint_face(app);
            }
        }
        Some(Action::DebugWin) => {
            if app.settings.debug_mode == "on" {
                debug_win(app);
                return EventResult::Continue;
            }
        }
        Some(Action::DebugGoldUp) => {
            if app.settings.debug_mode == "on" {
                app.gold += 100;
                let _ = crate::save::save_setting("player_gold", &app.gold.to_string());
                app.set_message(format!("💰 +100 → {}", app.gold), Duration::from_secs(2));
            }
        }
        Some(Action::DebugGoldDown) => {
            if app.settings.debug_mode == "on" {
                app.gold = (app.gold - 50).max(0);
                let _ = crate::save::save_setting("player_gold", &app.gold.to_string());
                app.set_message(format!("💰 -50 → {}", app.gold), Duration::from_secs(2));
            }
        }
        _ => {}
    }

    app.game.selected = Some(current_coord(app));
    EventResult::Continue
}

fn apply_pager_action(app: &mut App, layout: &GameLayout, action: PagerAction) {
    let total = layout
        .pager
        .as_ref()
        .map(|p| p.total_pages)
        .unwrap_or(1)
        .max(1) as i32;
    let cur = app.btn_page as i32;
    let new_page = match action {
        PagerAction::Prev => (cur - 1).max(0),
        PagerAction::Next => (cur + 1).min(total - 1).max(0),
    };
    app.btn_page = new_page as u16;
    app.hover_button = None;
}

fn handle_mouse(app: &mut App, layout: &GameLayout, mouse: MouseEvent) -> EventResult {
    // 冻结状态下禁止鼠标交互
    if app.game.frozen {
        return EventResult::Continue;
    }

    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();

    match mouse.kind {
        MouseEventKind::Moved => {
            let new_hover = find_button_at(layout, mouse.column, mouse.row);
            if new_hover != app.hover_button {
                app.hover_button = new_hover;
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            // 商店区点击:优先于按钮/格子
            if let Some(item_idx) = shop_item_at(layout, mouse.column, mouse.row) {
                app.shop_selected = item_idx;
                app.shop_focused = true;
                if let Some(item) = crate::shop::shop_catalog(app.settings.guide_owned == "on")
                    .get(item_idx)
                    .map(|s| s.item_type)
                {
                    let lang = Lang::from_code(&app.settings.language);
                    if app.buy_item(item) {
                        let name = i18n::t(item.name_key(), lang);
                        app.set_message(
                            format!("{} {}×1", i18n::t("shop.bought", lang), name),
                            Duration::from_secs(2),
                        );
                    } else {
                        app.set_message(
                            i18n::t("shop.no_gold", lang).to_string(),
                            Duration::from_secs(2),
                        );
                    }
                }
                return EventResult::Continue;
            }
            // 翻页控件优先
            if let Some(action) = pager_action_at(layout, mouse.column, mouse.row) {
                apply_pager_action(app, layout, action);
                return EventResult::Continue;
            }
            if let Some(btn) = find_button_at(layout, mouse.column, mouse.row) {
                log_button_click(app, btn);
                return execute_button(app, btn);
            }
            if let Some((u, v)) = cell_at(layout, cw, ch, mouse.column, mouse.row) {
                app.cursor = (u, v);
            }
        }
        MouseEventKind::ScrollUp => {
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                app.current_face = super::navigation::cycle_face_horizontal(app.current_face, true);
            } else {
                app.current_face = super::navigation::cycle_face_vertical(app.current_face, true);
            }
        }
        MouseEventKind::ScrollDown => {
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                app.current_face =
                    super::navigation::cycle_face_horizontal(app.current_face, false);
            } else {
                app.current_face = super::navigation::cycle_face_vertical(app.current_face, false);
            }
        }
        _ => {}
    }
    app.game.selected = Some(current_coord(app));
    EventResult::Continue
}

fn execute_button(app: &mut App, btn: ButtonId) -> EventResult {
    match btn {
        ButtonId::Number(n) => {
            let coord = current_coord(app);
            if app.game.draft_mode {
                // 草稿模式: 切换草稿数字
                if let Some(cell) = app.game.grid.get_mut(&coord) {
                    if cell.given || cell.user_value.is_some() {
                        return EventResult::Continue;
                    }
                    cell.toggle_draft(n);
                    cell.draft_visible = true;
                }
                return EventResult::Continue;
            }
            let old = app.game.grid.get(&coord).and_then(|c| c.user_value);
            // 跳过已给定/已有相同值的格子
            if let Some(cell) = app.game.grid.get(&coord) {
                if cell.given || cell.user_value == Some(n) {
                    return EventResult::Continue;
                }
            }
            app.game.set_value(coord, Some(n));
            app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, Some(n));
            // 设置确定值后清除该格的草稿
            if let Some(cell) = app.game.grid.get_mut(&coord) {
                cell.draft.clear();
            }
            // 容错检测
            let is_wrong = app.check_and_consume_error(coord, n);
            if is_wrong {
                let lang = Lang::from_code(&app.settings.language);
                app.set_message(
                    format!(
                        "{} {}/{}(+{})",
                        crate::i18n::t("status.wrong", lang),
                        app.game.errors,
                        app.game.errors_max,
                        app.global_errors_max
                    ),
                    Duration::from_secs(2),
                );
            }
            if app.game.frozen {
                return EventResult::Continue;
            }
            if app.game.check_completion() {
                app.trigger_victory();
                return EventResult::Continue;
            }
        }
        ButtonId::Erase => {
            let coord = current_coord(app);
            let (has_value, old, has_draft) = {
                let cell = app.game.grid.get(&coord);
                match cell {
                    Some(c) if !c.given => (c.user_value.is_some(), c.user_value, !c.draft.is_empty()),
                    _ => (false, None, false),
                }
            };
            if has_value {
                app.game.set_value(coord, None);
                app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, None);
                if has_draft {
                    if let Some(cell) = app.game.grid.get_mut(&coord) {
                        cell.draft_visible = true;
                    }
                }
            } else if has_draft {
                if let Some(cell) = app.game.grid.get_mut(&coord) {
                    cell.draft.clear();
                    cell.draft_visible = true;
                }
            }
        }
        ButtonId::Hint => {
            app.game.hint();
            app.set_message("已提示", Duration::from_secs(2));
        }
        ButtonId::Undo => {
            app.game.undo();
            app.set_message("已撤销", Duration::from_secs(2));
        }
        ButtonId::ToggleGuidance => {
            if app.settings.guide_owned != "on" {
                let lang = Lang::from_code(&app.settings.language);
                app.set_message(i18n::t("guide.locked", lang).to_string(), Duration::from_secs(2));
                return EventResult::Continue;
            }
            let new_val = if app.settings.guide_enabled == "on" { "off" } else { "on" };
            app.settings.guide_enabled = new_val.into();
            app.settings.save_to_db();
            app.guidance = app.settings.guide_active();
            let lang = Lang::from_code(&app.settings.language);
            let key = if app.guidance { "msg.guide_on" } else { "msg.guide_off" };
            app.set_message(i18n::t(key, lang).to_string(), Duration::from_secs(2));
        }
        ButtonId::ToggleMode => {
            let new_mode = app.render_mode.toggle();
            // 如果处于草稿模式且新模式格子尺寸过小,先退出草稿模式防止在看不见的情况下输入
            if app.game.draft_mode {
                let new_cw = new_mode.cell_width(&app.settings);
                let new_ch = new_mode.cell_height();
                if new_cw < 3 || new_ch < 3 {
                    app.game.draft_mode = false;
                    let lang = Lang::from_code(&app.settings.language);
                    app.set_message(
                        i18n::t("msg.draft_off", lang).to_string(),
                        Duration::from_secs(2),
                    );
                }
            }
            app.render_mode = new_mode;
            let lang = Lang::from_code(&app.settings.language);
            app.set_message(
                format!("{}", mode_label(app.render_mode, lang)),
                Duration::from_secs(2),
            );
        }
        ButtonId::ToggleDraft => {
            let cw = app.render_mode.cell_width(&app.settings);
            let ch = app.render_mode.cell_height();
            let lang = Lang::from_code(&app.settings.language);
            if cw < 3 || ch < 3 {
                app.set_message(
                    i18n::t("msg.draft_too_small", lang).to_string(),
                    Duration::from_secs(2),
                );
                return EventResult::Continue;
            }
            let was_on = app.game.draft_mode;
            app.game.draft_mode = !was_on;
            // 注意: 不再根据模式开关设置 draft_visible。
            // 草稿内容在两种模式下都可见,仅在格子填入 user_value 时被隐藏(Erase 时恢复)。
            let key = if app.game.draft_mode { "msg.draft_on" } else { "msg.draft_off" };
            app.set_message(i18n::t(key, lang).to_string(), Duration::from_secs(2));
        }
        ButtonId::Quit => return EventResult::BackToMenu,
        ButtonId::ToolCube => {
            if app.use_tool(shop::ItemType::Cube) {
                return EventResult::Continue;
            }
        }
        ButtonId::ToolSnake3 => {
            if app.use_tool(shop::ItemType::Snake3) {
                return EventResult::Continue;
            }
        }
        ButtonId::ToolSnake5 => {
            if app.use_tool(shop::ItemType::Snake5) {
                return EventResult::Continue;
            }
        }
        ButtonId::ToolFace => {
            if app.use_tool(shop::ItemType::Face) {
                return EventResult::Continue;
            }
        }
        ButtonId::ToolTarget => {
            if app.use_tool(shop::ItemType::Target) {
                return EventResult::Continue;
            }
        }
    }
    app.game.selected = Some(current_coord(app));
    EventResult::Continue
}

// ── 面切换 ──

fn log_button_click(app: &mut App, btn: ButtonId) {
    let label = match btn {
        ButtonId::Number(n) => {
            return app.push_log(
                format!("Placed {} at R{}C{}", n, app.cursor.1 + 1, app.cursor.0 + 1),
                50,
            );
        }
        ButtonId::Erase => "Erase",
        ButtonId::Hint => "Hint",
        ButtonId::Undo => "Undo",
        ButtonId::ToggleGuidance => {
            // Toggle is logged inside execute_button via set_message; pre-log here.
            "Guide"
        }
        ButtonId::ToggleMode => "Mode",
        ButtonId::ToggleDraft => "Draft",
        ButtonId::Quit => "Menu",
        ButtonId::ToolCube => "Tool🎲",
        ButtonId::ToolSnake3 => "Tool🐍3",
        ButtonId::ToolSnake5 => "Tool🐍5",
        ButtonId::ToolFace => "Tool🔀",
        ButtonId::ToolTarget => "Tool❗",
    };
    app.push_log(label, 50);
}

/// Debug: 快速胜利
/// - 贪吃蛇模式：立即吃完所有果实
/// - 数独模式：Hint 所有空格
fn debug_win(app: &mut App) {
    if let Some(snake) = app.snake.as_mut() {
        // 贪吃蛇：直接吃掉所有果实
        snake.score = snake.total_fruits;
        snake.fruits.clear();
        snake.outcome = shop::SnakeOutcome::Won;
        return;
    }
    // 数独模式：填满所有空格
    let coords: Vec<_> = app
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
        .collect();
    for coord in coords {
        if let Some(&ans) = app.game.solution.get(&coord) {
            app.game.set_value(coord, Some(ans));
        }
    }
    if app.game.check_completion() {
        app.trigger_victory();
    }
}
