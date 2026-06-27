use crossterm::event::{Event, KeyEventKind, KeyCode};
use ratatui::layout::Rect;

use crate::config::Action;
use crate::App;

use super::EventResult;

pub(super) fn handle_keymap_edit_event(app: &mut App, event: Event, _area: Rect) -> EventResult {
    if let Some(ref ke) = app.settings_ui.keymap_edit {
        // 正在等待用户按键重新绑定
        if ke.awaiting_key {
            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Press {
                    return EventResult::Continue;
                }
                // Esc 取消绑定
                if key.code == KeyCode::Esc {
                    if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                        ke.awaiting_key = false;
                        ke.rebinding_index = None;
                    }
                    return EventResult::Continue;
                }
                // 记录新绑定
                let new_key = crate::config::keymap::Key::from(key.code);
                let new_mods = crate::config::keymap::ModKey::from(key.modifiers);
                if let Some(idx) = ke.rebinding_index {
                    if let Some(action) = app.settings_ui.keymap_edit.as_ref().and_then(|ke| ke.actions.get(idx).copied()) {
                        app.keymap.rebind(action, new_key, new_mods);
                        app.keymap.save_to_db();
                    }
                }
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    ke.awaiting_key = false;
                    ke.rebinding_index = None;
                }
            }
            return EventResult::Continue;
        }
    }

    // 正常浏览模式
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return EventResult::Continue;
        }
        let action = app.keymap.resolve(app.screen, false, key.code, key.modifiers);
        match action {
            Some(Action::SettingsUp) => {
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    if ke.selected > 0 {
                        ke.selected -= 1;
                    }
                }
            }
            Some(Action::SettingsDown) => {
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    if ke.selected + 1 < ke.actions.len() {
                        ke.selected += 1;
                    }
                }
            }
            Some(Action::Confirm) => {
                // 进入绑定模式: 等待用户按下一个键
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    ke.awaiting_key = true;
                    ke.rebinding_index = Some(ke.selected);
                }
            }
            Some(Action::Cancel) => {
                // 退出键位编辑,回到设置
                app.settings_ui.keymap_edit = None;
            }
            _ => {}
        }
    }
    EventResult::Continue
}
