mod game;
mod keymap;
mod menu;
mod navigation;
mod settings;

pub use navigation::move_on_surface;
pub use navigation::convert_face_dir;

use crossterm::event::{Event, KeyEventKind, KeyCode};
use ratatui::layout::Rect;
use sudokube_core::cube::Difficulty;
use sudokube_core::game_state::GameState;

use crate::config::Action;
use crate::i18n::{self, Lang};
use crate::save::delete_game;
use crate::{App, AppScreen};
use std::time::{Duration, Instant};

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
        return settings::handle_settings_event(app, event, area);
    }
    match app.screen {
        AppScreen::Menu => menu::handle_menu_event(app, event, area),
        AppScreen::Game => game::handle_game_event(app, event, area),
        AppScreen::Settings => settings::handle_settings_event(app, event, area),
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
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                app.confirm_delete_id = None;
            }
            _ => {}
        }
    }
    EventResult::Continue
}

fn handle_victory_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            let action = app.keymap.resolve(app.screen, false, key.code, key.modifiers);
            if matches!(action, Some(Action::Confirm)) {
                *app = App::new_menu();
                return EventResult::Continue;
            }
        }
    }
    EventResult::Continue
}

/// Generating 屏: 允许按 Esc 中断这次生成,回到菜单
fn handle_generating_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind == KeyEventKind::Press {
            let action = app.keymap.resolve(app.screen, false, key.code, key.modifiers);
            if matches!(action, Some(Action::Cancel)) {
                app.generating = None;
                let lang = Lang::from_code(&app.settings.language);
                let msg = i18n::t("msg.gen_cancelled", lang).to_string();
                *app = App::new_menu();
                app.set_message(msg, Duration::from_secs(3));
                return EventResult::Continue;
            }
        }
    }
    EventResult::Continue
}

fn handle_export_select_event(app: &mut App, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return EventResult::Continue;
        }
        let action = app.keymap.resolve(app.screen, false, key.code, key.modifiers);
        match action {
            Some(Action::ExportUp) => {
                if app.export_select > 0 {
                    app.export_select -= 1;
                }
            }
            Some(Action::ExportDown) => {
                if app.export_select < 1 {
                    app.export_select += 1;
                }
            }
            Some(Action::Confirm) => {
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
            Some(Action::Cancel) => {
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
        let action = app.keymap.resolve(app.screen, false, key.code, key.modifiers);
        match action {
            Some(Action::Confirm) => {
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
            Some(Action::Cancel) => {
                app.import_buffer.clear();
                app.import_paste_started = None;
                app.import_last_input = None;
                app.screen = AppScreen::Menu;
            }
            Some(Action::ImportBackspace) => {
                app.import_buffer.pop();
                app.import_last_input = Some(now);
            }
            _ => {
                // 回退: 普通字符输入(用于导入粘贴)
                if let KeyCode::Char(c) = key.code {
                    app.import_buffer.push(c);
                    app.import_last_input = Some(now);
                }
            }
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

