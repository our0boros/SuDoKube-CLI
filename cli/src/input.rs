use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;
use std::time::Duration;
use sudokube_core::cube::{Difficulty, Face};

use crate::i18n::{self, Lang};
use crate::render::{
    ButtonId, GameLayout, PagerAction, cell_at, compute_game_layout_from_rect, find_button_at,
    mode_label, pager_action_at,
};
use crate::save::delete_game;
use crate::{App, AppScreen, MenuItem, current_coord};
use std::time::Instant;
use sudokube_core::game_state::GameState;

pub enum EventResult {
    Continue,
    StartGame(sudokube_core::game_state::GameState),
    StartGenerating(Difficulty),
    BackToMenu,
    Quit,
}

pub fn handle_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    // 优先级 1: 删除确认弹窗
    if app.confirm_delete_id.is_some() {
        return handle_confirm_delete_event(app, event);
    }
    // 优先级 2: 当 Settings 弹窗可见时,所有事件都路由到 settings handler（弹窗覆盖菜单）
    if app.screen == AppScreen::Menu && app.settings_ui.visible {
        return handle_settings_event(app, event, area);
    }
    match app.screen {
        AppScreen::Menu => handle_menu_event(app, event, area),
        AppScreen::Game => handle_game_event(app, event, area),
        AppScreen::Settings => handle_settings_event(app, event, area),
        AppScreen::Generating => handle_generating_event(app, event),
        AppScreen::Victory => handle_victory_event(app, event),
        AppScreen::ExportSelect => handle_export_select_event(app, event),
        AppScreen::ImportInput => handle_import_input_event(app, event),
    }
}

/// 删除确认弹窗: Y / Enter 确认, N / Esc 取消
fn handle_confirm_delete_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return EventResult::Continue;
        }
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let Some(id) = app.confirm_delete_id.take() {
                    let _ = delete_game(id);
                    app.menu = crate::MenuState::new();
                    let lang = Lang::from_code(&app.settings.language);
                    app.set_message(i18n::t("menu.deleted", lang), Duration::from_secs(2));
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.confirm_delete_id = None;
            }
            _ => {}
        }
    }
    EventResult::Continue
}

fn handle_victory_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {
            *app = App::new_menu();
            return EventResult::Continue;
        }
    }
    EventResult::Continue
}

/// Generating 屏: 允许按 Esc 中断这次生成,回到菜单
fn handle_generating_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
            // 丢弃生成中的结果,回到菜单,并显示取消提示
            app.generating = None;
            let lang = Lang::from_code(&app.settings.language);
            let msg = i18n::t("msg.gen_cancelled", lang).to_string();
            *app = App::new_menu();
            app.set_message(msg, Duration::from_secs(3));
            return EventResult::Continue;
        }
    }
    EventResult::Continue
}

fn handle_export_select_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return EventResult::Continue;
        }
        match key.code {
            KeyCode::Up => {
                if app.export_select > 0 {
                    app.export_select -= 1;
                }
            }
            KeyCode::Down => {
                if app.export_select < 1 {
                    app.export_select += 1;
                }
            }
            KeyCode::Enter => {
                // "Export All": export every stored game (finished + unfinished)
                // as a single bundle string.
                let lang = Lang::from_code(&app.settings.language);
                let records = crate::save::load_history(1000).unwrap_or_default();
                if records.is_empty() {
                    app.screen = AppScreen::Menu;
                    app.set_message(i18n::t("export.empty", lang), Duration::from_secs(2));
                } else {
                    let encrypted = app.export_select == 0;
                    let data = crate::save::export_records(&records, encrypted);
                    if crate::save::copy_to_clipboard(&data) {
                        app.screen = AppScreen::Menu;
                        app.set_message(i18n::t("export.copied", lang), Duration::from_secs(2));
                    } else {
                        app.screen = AppScreen::Menu;
                        app.set_message(i18n::t("export.fail", lang), Duration::from_secs(2));
                    }
                }
            }
            KeyCode::Esc => {
                app.screen = AppScreen::Menu;
            }
            _ => {}
        }
    }
    EventResult::Continue
}

fn handle_import_input_event(app: &mut App, event: Event) -> EventResult {
    // 粘贴 5s 行为: 5s 之前正常右滑显示,5s 后立即"完成"粘贴 — 停止接受后续 paste 事件,
    // 缓冲内容冻结为最终结果,用户可按 Enter 提交。普通按键和 Backspace 仍可用。
    const PASTE_BURST_GAP: Duration = Duration::from_millis(100);
    const PASTE_MAX_DURATION: Duration = Duration::from_secs(5);
    let now = Instant::now();

    // 检测是否已超过 5s:若是则标记 finalize,后续 paste 事件被忽略
    let paste_finalized = if !app.import_buffer.is_empty() {
        if let Some(started) = app.import_paste_started {
            now.duration_since(started) > PASTE_MAX_DURATION
        } else {
            false
        }
    } else {
        false
    };

    // 非突发输入时重置 burst 起点
    let in_burst_now = app
        .import_last_input
        .map(|t| now.duration_since(t) < PASTE_BURST_GAP)
        .unwrap_or(false);
    if !in_burst_now && !paste_finalized {
        app.import_paste_started = Some(now);
    }

    if let Event::Key(key) = event.clone() {
        if key.kind != KeyEventKind::Press {
            return EventResult::Continue;
        }
        match key.code {
            KeyCode::Enter => {
                let data = app.import_buffer.trim().to_string();
                let lang = Lang::from_code(&app.settings.language);
                if let Some(games) = crate::save::import_games(&data) {
                    let coords: Vec<sudokube_core::cube::CubeCoord> =
                        sudokube_core::cube::iter_surface_coords().collect();
                    let mut imported = 0usize;
                    for (diff_str, answer_str, puzzle_str, given_str) in games {
                        let answer = crate::save::deserialize_solution_from(&answer_str, &coords);
                        let puzzle_grid =
                            crate::save::deserialize_grid_from(&puzzle_str, &given_str, &coords);
                        let difficulty = match diff_str.as_str() {
                            "easy" => Difficulty::Easy,
                            "hard" => Difficulty::Hard,
                            _ => Difficulty::Medium,
                        };
                        let mut game = GameState::new(puzzle_grid, answer, difficulty);
                        game.id = None;
                        if crate::save::save_game(&game).is_ok() {
                            imported += 1;
                        }
                    }
                    if imported > 0 {
                        *app = App::new_menu();
                        app.set_message(
                            format!("{} ({})", i18n::t("import.success", lang), imported),
                            Duration::from_secs(2),
                        );
                    } else {
                        app.set_message(i18n::t("import.fail", lang), Duration::from_secs(2));
                        app.screen = AppScreen::Menu;
                    }
                } else {
                    app.set_message(i18n::t("import.fail", lang), Duration::from_secs(2));
                    app.screen = AppScreen::Menu;
                }
                app.import_paste_started = None;
                app.import_last_input = None;
            }
            KeyCode::Esc => {
                app.import_buffer.clear();
                app.import_paste_started = None;
                app.import_last_input = None;
                app.screen = AppScreen::Menu;
            }
            KeyCode::Char(c) => {
                // 5s 后仍允许单个字符输入(覆盖式修正),不算黏贴
                app.import_buffer.push(c);
                app.import_last_input = Some(now);
            }
            KeyCode::Backspace => {
                app.import_buffer.pop();
                app.import_last_input = Some(now);
            }
            _ => {}
        }
    }
    // Handle paste event from terminal (Ctrl+V, right-click paste, etc.)
    if let Event::Paste(s) = event {
        if paste_finalized {
            // 5s 后已 finalize,忽略后续 paste,内容定格
        } else {
            app.import_buffer.push_str(&s);
        }
        app.import_last_input = Some(now);
    }
    EventResult::Continue
}

fn handle_menu_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    match event {
        Event::Resize(_, _) => {}
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return EventResult::Quit,
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                if app.menu.selected > 0 {
                    app.menu.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                if app.menu.selected + 1 < app.menu.items.len() {
                    app.menu.selected += 1;
                }
            }
            KeyCode::Enter => {
                let item = app.menu.items[app.menu.selected].clone();
                return match item {
                    MenuItem::NewGame(d) => EventResult::StartGenerating(d),
                    MenuItem::Continue(r) => EventResult::StartGame(crate::continue_game(&r)),
                    MenuItem::Settings => {
                        // 弹窗模式: 在菜单上叠加,而不是切到独立屏幕
                        app.settings_ui.visible = true;
                        EventResult::Continue
                    }
                    MenuItem::Export => {
                        app.screen = AppScreen::ExportSelect;
                        EventResult::Continue
                    }
                    MenuItem::Import => {
                        app.import_buffer.clear();
                        app.import_paste_started = None;
                        app.import_last_input = None;
                        app.screen = AppScreen::ImportInput;
                        EventResult::Continue
                    }
                };
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(MenuItem::Continue(r)) = app.menu.items.get(app.menu.selected).cloned()
                {
                    // Alt+D: 直接删除,跳过确认
                    if key.modifiers.contains(KeyModifiers::ALT) {
                        let _ = delete_game(r.id);
                        app.menu = crate::MenuState::new();
                        let lang = Lang::from_code(&app.settings.language);
                        app.set_message(i18n::t("menu.deleted", lang), Duration::from_secs(2));
                    } else {
                        // 普通 D: 弹出确认框
                        app.confirm_delete_id = Some(r.id);
                    }
                }
            }
            KeyCode::Delete | KeyCode::Backspace => {
                if let Some(MenuItem::Continue(r)) = app.menu.items.get(app.menu.selected).cloned()
                {
                    app.confirm_delete_id = Some(r.id);
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Single game export
                if let Some(MenuItem::Continue(r)) = app.menu.items.get(app.menu.selected).cloned()
                {
                    let game = crate::continue_game(&r);
                    let encrypted = true;
                    let data = crate::save::export_game(&game, encrypted);
                    if crate::save::copy_to_clipboard(&data) {
                        let lang = Lang::from_code(&app.settings.language);
                        app.set_message(
                            i18n::t("export.copied", lang),
                            std::time::Duration::from_secs(2),
                        );
                    }
                }
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                app.import_buffer.clear();
                app.import_paste_started = None;
                app.import_last_input = None;
                app.screen = AppScreen::ImportInput;
            }
            _ => {}
        },
        Event::Mouse(mouse) => {
            if let Some(idx) = menu_item_at(app, mouse.column, mouse.row, area) {
                if idx != app.menu.selected {
                    app.menu.selected = idx;
                }
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let item = app.menu.items[idx].clone();
                    return match item {
                        MenuItem::NewGame(d) => EventResult::StartGenerating(d),
                        MenuItem::Continue(r) => EventResult::StartGame(crate::continue_game(&r)),
                        MenuItem::Settings => {
                            // 弹窗模式: 在菜单上叠加,而不是切到独立屏幕
                            app.settings_ui.visible = true;
                            EventResult::Continue
                        }
                        MenuItem::Export => {
                            app.screen = AppScreen::ExportSelect;
                            EventResult::Continue
                        }
                        MenuItem::Import => {
                            app.import_buffer.clear();
                            app.import_paste_started = None;
                            app.import_last_input = None;
                            app.screen = AppScreen::ImportInput;
                            EventResult::Continue
                        }
                    };
                }
            }
        }
        _ => {}
    }
    EventResult::Continue
}

fn menu_item_at(app: &App, _col: u16, row: u16, area: Rect) -> Option<usize> {
    let logo_h = 10u16; // LOGO 行数
    let items_len = app.menu.items.len() as u16;
    let total_h = logo_h + 1 + items_len + 2 + 2 + 1;
    let start_y = area.y + area.height.saturating_sub(total_h) / 2;
    let box_y = start_y + logo_h + 1;
    let menu_start_row = box_y + 1;
    let idx = row.saturating_sub(menu_start_row) as usize;
    if idx < app.menu.items.len() {
        Some(idx)
    } else {
        None
    }
}

fn handle_settings_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Up => {
                if app.settings_ui.selected > 0 {
                    app.settings_ui.selected -= 1;
                }
            }
            KeyCode::Down => {
                if app.settings_ui.selected + 1 < app.settings_ui.fields.len() {
                    app.settings_ui.selected += 1;
                }
            }
            KeyCode::Left => {
                let idx = app.settings_ui.selected;
                app.settings_ui.fields[idx].cycle_prev();
                app.settings_ui.apply_to(&mut app.settings);
                app.settings.save_to_db();
            }
            KeyCode::Right => {
                let idx = app.settings_ui.selected;
                app.settings_ui.fields[idx].cycle_next();
                app.settings_ui.apply_to(&mut app.settings);
                app.settings.save_to_db();
            }
            KeyCode::Enter | KeyCode::Esc => {
                app.settings.save_to_db();
                app.settings_ui.visible = false;
                // 弹窗模式: 切回菜单而非停留在独立 Settings 屏
                if app.screen == AppScreen::Settings {
                    app.screen = AppScreen::Menu;
                }
            }
            _ => {}
        },
        Event::Mouse(mouse) => {
            let layout = crate::render::compute_settings_popup_layout(area, app);
            // 更新悬停状态
            if matches!(mouse.kind, MouseEventKind::Moved) {
                let mut found: Option<(usize, Option<crate::SettingsArrow>)> = None;
                for (i, f) in layout.fields.iter().enumerate() {
                    if rect_contains(f.left_arrow_rect, mouse.column, mouse.row) {
                        found = Some((i, Some(crate::SettingsArrow::Left)));
                        break;
                    } else if rect_contains(f.right_arrow_rect, mouse.column, mouse.row) {
                        found = Some((i, Some(crate::SettingsArrow::Right)));
                        break;
                    } else if rect_contains(f.label_rect, mouse.column, mouse.row)
                        || rect_contains(f.value_rect, mouse.column, mouse.row)
                    {
                        found = Some((i, None));
                        break;
                    }
                }
                if let Some((i, arrow)) = found {
                    app.settings_ui.hover_field = Some(i);
                    app.settings_ui.hover_arrow = arrow;
                } else {
                    app.settings_ui.hover_field = None;
                    app.settings_ui.hover_arrow = None;
                }
            }
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                // 检查箭头点击
                for (i, f) in layout.fields.iter().enumerate() {
                    if rect_contains(f.left_arrow_rect, mouse.column, mouse.row) {
                        app.settings_ui.selected = i;
                        app.settings_ui.fields[i].cycle_prev();
                        app.settings_ui.apply_to(&mut app.settings);
                        app.settings.save_to_db();
                        return EventResult::Continue;
                    } else if rect_contains(f.right_arrow_rect, mouse.column, mouse.row) {
                        app.settings_ui.selected = i;
                        app.settings_ui.fields[i].cycle_next();
                        app.settings_ui.apply_to(&mut app.settings);
                        app.settings.save_to_db();
                        return EventResult::Continue;
                    } else if rect_contains(f.label_rect, mouse.column, mouse.row)
                        || rect_contains(f.value_rect, mouse.column, mouse.row)
                    {
                        app.settings_ui.selected = i;
                        return EventResult::Continue;
                    }
                }
                // 点击弹窗外部：关闭
                if !rect_contains(layout.popup_area, mouse.column, mouse.row) {
                    app.settings.save_to_db();
                    app.settings_ui.visible = false;
                    if app.screen == AppScreen::Settings {
                        app.screen = AppScreen::Menu;
                    }
                }
            }
            // 滚轮支持
            if matches!(mouse.kind, MouseEventKind::ScrollUp) {
                if app.settings_ui.selected > 0 {
                    app.settings_ui.selected -= 1;
                }
            } else if matches!(mouse.kind, MouseEventKind::ScrollDown) {
                if app.settings_ui.selected + 1 < app.settings_ui.fields.len() {
                    app.settings_ui.selected += 1;
                }
            }
        }
        _ => {}
    }
    EventResult::Continue
}

fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

fn handle_game_event(app: &mut App, event: Event, area: Rect) -> EventResult {
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
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let shift_or_none = key.modifiers.is_empty() || shift;
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    // Alt+H: Debug hint all blanks on current face
    if alt && (key.code == KeyCode::Char('h') || key.code == KeyCode::Char('H')) {
        if app.settings.debug_mode == "on" {
            debug_hint_face(app);
        }
        return EventResult::Continue;
    }

    // 按钮栏翻页快捷键: '[' 上一页, ']' 下一页
    if key.code == KeyCode::Char('[') || key.code == KeyCode::Char('{') {
        if app.btn_page > 0 {
            app.btn_page -= 1;
            app.hover_button = None;
        }
        return EventResult::Continue;
    }
    if key.code == KeyCode::Char(']') || key.code == KeyCode::Char('}') {
        // 简单按 (current+1) 处理,布局计算时会自动夹紧
        app.btn_page = app.btn_page.saturating_add(1);
        app.hover_button = None;
        return EventResult::Continue;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return EventResult::BackToMenu,
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.push_log("New Game", 50);
            return EventResult::StartGenerating(app.game.difficulty);
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.render_mode = app.render_mode.toggle();
            let lang = Lang::from_code(&app.settings.language);
            let label = mode_label(app.render_mode, lang);
            app.set_message(label.to_string(), Duration::from_secs(2));
            app.push_log(format!("Mode: {}", label), 50);
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('x') | KeyCode::Char('X') => {
            let coord = current_coord(app);
            if let Some(cell) = app.game.grid.get(&coord) {
                if !cell.given {
                    let old = cell.user_value;
                    app.game.set_value(coord, None);
                    app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, None);
                    app.push_log("Erase", 50);
                }
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.guidance = !app.guidance;
            let lang = Lang::from_code(&app.settings.language);
            let key = if app.guidance {
                "msg.guide_on"
            } else {
                "msg.guide_off"
            };
            app.set_message(i18n::t(key, lang).to_string(), Duration::from_secs(2));
            app.push_log(i18n::t(key, lang), 50);
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.game.hint();
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(i18n::t("debug.hint", lang), 50);
            if app.game.check_completion() {
                app.game.completed = true;
                app.screen = AppScreen::Victory;
                app.victory_countdown = Some(Instant::now() + Duration::from_secs(3));
                let _ = crate::save::save_game(&app.game);
                return EventResult::Continue;
            }
            app.set_message("已提示", Duration::from_secs(2));
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            app.game.undo();
            app.push_log("Undo", 50);
            app.set_message("已撤销", Duration::from_secs(2));
        }
        KeyCode::Backspace | KeyCode::Delete => {
            let coord = current_coord(app);
            let old = app.game.grid.get(&coord).and_then(|c| c.user_value);
            app.game.set_value(coord, None);
            app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, None);
            app.push_log("Erase", 50);
        }
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let value = c as u8 - b'0';
            let coord = current_coord(app);
            let old = app.game.grid.get(&coord).and_then(|c| c.user_value);
            app.game.set_value(coord, Some(value));
            app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, Some(value));
            app.push_log(
                format!(
                    "Placed {} at R{}C{}",
                    value,
                    app.cursor.1 + 1,
                    app.cursor.0 + 1
                ),
                50,
            );
            if app.game.check_completion() {
                app.game.completed = true;
                app.screen = AppScreen::Victory;
                app.victory_countdown = Some(Instant::now() + Duration::from_secs(3));
                let _ = crate::save::save_game(&app.game);
                return EventResult::Continue;
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') if shift_or_none => {
            move_cursor_with_wrap(app, 0, -1);
        }
        KeyCode::Char('a') | KeyCode::Char('A') if shift_or_none => {
            move_cursor_with_wrap(app, -1, 0);
        }
        KeyCode::Char('s') | KeyCode::Char('S') if shift_or_none => {
            move_cursor_with_wrap(app, 0, 1);
        }
        KeyCode::Char('d') | KeyCode::Char('D') if shift_or_none => {
            move_cursor_with_wrap(app, 1, 0);
        }
        KeyCode::Up => {
            let prev = app.current_face;
            app.current_face = switch_face(app.current_face, 0, -1);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
            }
        }
        KeyCode::Down => {
            let prev = app.current_face;
            app.current_face = switch_face(app.current_face, 0, 1);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
            }
        }
        KeyCode::Left => {
            let prev = app.current_face;
            app.current_face = switch_face(app.current_face, -1, 0);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
            }
        }
        KeyCode::Right => {
            let prev = app.current_face;
            app.current_face = switch_face(app.current_face, 1, 0);
            if app.current_face != prev {
                let lang = Lang::from_code(&app.settings.language);
                app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            app.current_face = Face::Front;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.current_face = Face::Back;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.current_face = Face::Left;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.current_face = Face::Right;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.current_face = Face::Top;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            app.current_face = Face::Bottom;
            let lang = Lang::from_code(&app.settings.language);
            app.push_log(format!("→ {}", face_label(app.current_face, lang)), 50);
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
                app.current_face = cycle_face_horizontal(app.current_face, true);
            } else {
                app.current_face = cycle_face_vertical(app.current_face, true);
            }
        }
        MouseEventKind::ScrollDown => {
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                app.current_face = cycle_face_horizontal(app.current_face, false);
            } else {
                app.current_face = cycle_face_vertical(app.current_face, false);
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
            let old = app.game.grid.get(&coord).and_then(|c| c.user_value);
            app.game.set_value(coord, Some(n));
            app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, Some(n));
            if app.game.check_completion() {
                app.game.completed = true;
                app.screen = AppScreen::Victory;
                app.victory_countdown = Some(Instant::now() + Duration::from_secs(3));
                let _ = crate::save::save_game(&app.game);
                return EventResult::Continue;
            }
        }
        ButtonId::Erase => {
            let coord = current_coord(app);
            if let Some(cell) = app.game.grid.get(&coord) {
                if !cell.given {
                    let old = cell.user_value;
                    app.game.set_value(coord, None);
                    app.adjust_digit_remaining(coord.x, coord.y, coord.z, old, None);
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
            app.guidance = !app.guidance;
            app.set_message(
                format!("辅助模式{}", if app.guidance { "开" } else { "关" }),
                Duration::from_secs(2),
            );
        }
        ButtonId::ToggleMode => {
            app.render_mode = app.render_mode.toggle();
            let lang = Lang::from_code(&app.settings.language);
            app.set_message(
                format!("{}", mode_label(app.render_mode, lang)),
                Duration::from_secs(2),
            );
        }
        ButtonId::Quit => return EventResult::BackToMenu,
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
        ButtonId::Quit => "Menu",
    };
    app.push_log(label, 50);
}

fn face_label(face: Face, _lang: Lang) -> &'static str {
    match face {
        Face::Front => "F",
        Face::Back => "B",
        Face::Left => "L",
        Face::Right => "R",
        Face::Top => "T",
        Face::Bottom => "U",
    }
}

fn cycle_face_vertical(face: Face, forward: bool) -> Face {
    let ring = [Face::Front, Face::Top, Face::Back, Face::Bottom];
    cycle_in_ring(&ring, face, forward)
}

fn cycle_face_horizontal(face: Face, forward: bool) -> Face {
    let ring = [Face::Front, Face::Left, Face::Back, Face::Right];
    cycle_in_ring(&ring, face, forward)
}

fn cycle_in_ring(ring: &[Face], face: Face, forward: bool) -> Face {
    if let Some(pos) = ring.iter().position(|&f| f == face) {
        let delta = if forward { 1 } else { ring.len() - 1 };
        ring[(pos + delta) % ring.len()]
    } else if forward {
        ring[0]
    } else {
        ring[ring.len() - 1]
    }
}

// ── 光标移动 ──

fn move_cursor_with_wrap(app: &mut App, dx: i8, dy: i8) {
    let (face, cursor) = move_on_surface(app.current_face, app.cursor, dx, dy);
    if face != app.current_face || cursor != app.cursor {
        app.current_face = face;
        app.cursor = cursor;
    }
}

fn move_on_surface(face: Face, cursor: (u8, u8), dx: i8, dy: i8) -> (Face, (u8, u8)) {
    let (u, v) = (cursor.0 as i8, cursor.1 as i8);
    let nu = u + dx;
    let nv = v + dy;

    if (0..9).contains(&nu) && (0..9).contains(&nv) {
        return (face, (nu as u8, nv as u8));
    }

    match face {
        Face::Front => {
            if nv < 0 {
                (Face::Bottom, (8, u as u8))
            } else if nv > 8 {
                (Face::Top, (u as u8, 8))
            } else if nu < 0 {
                (Face::Left, (8, v as u8))
            } else {
                (Face::Right, (v as u8, 8))
            }
        }
        Face::Back => {
            if nv < 0 {
                (Face::Left, (0, u as u8))
            } else if nv > 8 {
                (Face::Right, (u as u8, 0))
            } else if nu < 0 {
                (Face::Bottom, (v as u8, 0))
            } else {
                (Face::Top, (v as u8, 0))
            }
        }
        Face::Top => {
            if nv < 0 {
                (Face::Back, (u as u8, 8))
            } else if nv > 8 {
                (Face::Front, (u as u8, 8))
            } else if nu < 0 {
                (Face::Left, (v as u8, 8))
            } else {
                (Face::Right, (8, v as u8))
            }
        }
        Face::Bottom => {
            if nv < 0 {
                (Face::Left, (u as u8, 0))
            } else if nv > 8 {
                (Face::Right, (0, u as u8))
            } else if nu < 0 {
                (Face::Back, (v as u8, 0))
            } else {
                (Face::Front, (v as u8, 0))
            }
        }
        Face::Left => {
            if nv < 0 {
                (Face::Bottom, (u as u8, 0))
            } else if nv > 8 {
                (Face::Top, (0, u as u8))
            } else if nu < 0 {
                (Face::Back, (0, v as u8))
            } else {
                (Face::Front, (0, v as u8))
            }
        }
        Face::Right => {
            if nv < 0 {
                (Face::Back, (8, u as u8))
            } else if nv > 8 {
                (Face::Front, (8, u as u8))
            } else if nu < 0 {
                (Face::Bottom, (v as u8, 8))
            } else {
                (Face::Top, (8, v as u8))
            }
        }
    }
}

fn switch_face(face: Face, dx: i8, dy: i8) -> Face {
    match face {
        Face::Front => match (dx, dy) {
            (0, -1) => Face::Top,
            (0, 1) => Face::Bottom,
            (-1, 0) => Face::Left,
            (1, 0) => Face::Right,
            _ => face,
        },
        Face::Back => match (dx, dy) {
            (0, -1) => Face::Top,
            (0, 1) => Face::Bottom,
            (-1, 0) => Face::Right,
            (1, 0) => Face::Left,
            _ => face,
        },
        Face::Top => match (dx, dy) {
            (0, -1) => Face::Back,
            (0, 1) => Face::Front,
            (-1, 0) => Face::Left,
            (1, 0) => Face::Right,
            _ => face,
        },
        Face::Bottom => match (dx, dy) {
            (0, -1) => Face::Front,
            (0, 1) => Face::Back,
            (-1, 0) => Face::Left,
            (1, 0) => Face::Right,
            _ => face,
        },
        Face::Left => match (dx, dy) {
            (0, -1) => Face::Top,
            (0, 1) => Face::Bottom,
            (-1, 0) => Face::Back,
            (1, 0) => Face::Front,
            _ => face,
        },
        Face::Right => match (dx, dy) {
            (0, -1) => Face::Top,
            (0, 1) => Face::Bottom,
            (-1, 0) => Face::Front,
            (1, 0) => Face::Back,
            _ => face,
        },
    }
}

/// Debug: Fill all blank cells on the current face with solution values
fn debug_hint_face(app: &mut App) {
    let face = app.current_face;
    let mut filled = 0u32;
    for u in 0..9u8 {
        for v in 0..9u8 {
            let coord = face.to_cube(u, v);
            if let Some(cell) = app.game.grid.get(&coord) {
                if !cell.given && cell.user_value.is_none() {
                    if let Some(&sol) = app.game.solution.get(&coord) {
                        if let Some(cell) = app.game.grid.get_mut(&coord) {
                            cell.user_value = Some(sol);
                            filled += 1;
                        }
                    }
                }
            }
        }
    }
    if app.game.check_completion() {
        app.game.completed = true;
        app.screen = AppScreen::Victory;
        app.victory_countdown = Some(Instant::now() + Duration::from_secs(3));
        let _ = crate::save::save_game(&app.game);
        return;
    }
    let lang = Lang::from_code(&app.settings.language);
    app.set_message(
        format!("{} {} cells", i18n::t("debug.hint", lang), filled),
        Duration::from_secs(2),
    );
}
