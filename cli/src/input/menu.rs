use crossterm::event::{Event, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::config::Action;
use crate::i18n::{self, Lang};
use crate::save::delete_game;
use crate::{App, AppScreen, MenuItem};
use std::time::Duration;

use super::EventResult;

pub(super) fn handle_menu_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    match event {
        Event::Resize(_, _) => {}
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            let action = app
                .keymap
                .resolve(app.screen, false, key.code, key.modifiers);
            match action {
                Some(Action::Quit) => return EventResult::Quit,
                Some(Action::MenuUp) => {
                    if app.menu.selected > 0 {
                        app.menu.selected -= 1;
                    }
                }
                Some(Action::MenuDown) => {
                    if app.menu.selected + 1 < app.menu.items.len() {
                        app.menu.selected += 1;
                    }
                }
                Some(Action::MenuSelect) => {
                    let item = app.menu.items[app.menu.selected].clone();
                    return match item {
                        MenuItem::NewGame(d) => EventResult::StartGenerating(d),
                        MenuItem::Continue(r) => EventResult::StartGame(crate::continue_game(&r)),
                        MenuItem::Settings => {
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
                Some(Action::MenuDelete) => {
                    if let Some(MenuItem::Continue(r)) =
                        app.menu.items.get(app.menu.selected).cloned()
                    {
                        if key.modifiers.contains(KeyModifiers::ALT) {
                            let _ = delete_game(r.id);
                            app.menu = crate::MenuState::new();
                            let lang = Lang::from_code(&app.settings.language);
                            app.set_message(i18n::t("menu.deleted", lang), Duration::from_secs(2));
                        } else {
                            app.confirm_delete_id = Some(r.id);
                        }
                    }
                }
                Some(Action::MenuExport) => {
                    if let Some(MenuItem::Continue(r)) =
                        app.menu.items.get(app.menu.selected).cloned()
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
                Some(Action::MenuImport) => {
                    app.import_buffer.clear();
                    app.import_paste_started = None;
                    app.import_last_input = None;
                    app.screen = AppScreen::ImportInput;
                }
                _ => {}
            }
        }
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
