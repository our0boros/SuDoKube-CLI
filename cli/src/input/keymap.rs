use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;
use std::time::Duration;

use crate::config::Action;
use crate::{App, AppScreen};

use super::EventResult;

pub(super) fn handle_keymap_edit_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    // 检查 debug_mode 变化
    let debug_changed = app.settings_ui.keymap_debug_mode != app.settings.debug_mode;
    if debug_changed {
        app.settings_ui.keymap_debug_mode = app.settings.debug_mode.clone();
        crate::render::rebuild_keymap_actions(app);
    }

    let visible_rows = (area.height as usize).saturating_sub(8);

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
                    if let Some(action) = app
                        .settings_ui
                        .keymap_edit
                        .as_ref()
                        .and_then(|ke| ke.actions.get(idx).copied())
                    {
                        match app.keymap.rebind(action, new_key, new_mods) {
                            Ok(()) => {
                                app.keymap.save_to_db();
                                app.settings_ui.keymap_error = None;
                            }
                            Err("reserved") => {
                                let lang = crate::i18n::Lang::from_code(&app.settings.language);
                                app.settings_ui.keymap_error = Some(
                                    crate::i18n::t("settings.keymap_err_reserved", lang)
                                        .to_string(),
                                );
                            }
                            Err("conflict") => {
                                let lang = crate::i18n::Lang::from_code(&app.settings.language);
                                app.settings_ui.keymap_error = Some(
                                    crate::i18n::t("settings.keymap_err_conflict", lang)
                                        .to_string(),
                                );
                            }
                            Err(_) => {}
                        }
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

    // 鼠标事件处理
    if let Event::Mouse(mouse) = event {
        let ke = match app.settings_ui.keymap_edit.as_ref() {
            Some(ke) => ke,
            None => return EventResult::Continue,
        };
        let total = ke.actions.len();
        let visible_rows = (area.height as usize).saturating_sub(8);
        let scroll = ke.scroll as usize;
        let selected = ke.selected;

        // 计算重置选项的位置
        let _reset_row = total;

        // 计算点击的是哪一行
        let content_start_y = area.y + 2; // 标题占1行，边框1行
        let reset_y = area.y + area.height - 4; // 底部预留3行

        if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
            if mouse.row >= content_start_y && mouse.row < reset_y {
                // 点击在列表区域
                let row_idx = (mouse.row - content_start_y) as usize;
                let item_idx = scroll + row_idx;
                if item_idx < total {
                    if item_idx == selected {
                        // 点击已选中项，进入编辑模式
                        if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                            ke.awaiting_key = true;
                            ke.rebinding_index = Some(item_idx);
                            app.settings_ui.keymap_error = None;
                        }
                    } else {
                        // 选择该项
                        app.settings_ui.keymap_edit.as_mut().unwrap().selected = item_idx;
                        // 自动滚动到可见区域
                        let new_scroll =
                            app.settings_ui.keymap_edit.as_ref().unwrap().scroll as usize;
                        if item_idx < new_scroll {
                            app.settings_ui.keymap_edit.as_mut().unwrap().scroll = item_idx as u16;
                        } else if item_idx >= new_scroll + visible_rows {
                            app.settings_ui.keymap_edit.as_mut().unwrap().scroll =
                                (item_idx - visible_rows + 1) as u16;
                        }
                    }
                }
            } else if mouse.row >= reset_y && mouse.row < reset_y + 1 {
                // 点击重置选项
                if selected == total {
                    // 重置为默认
                    app.keymap.reset_to_default();
                    app.keymap.save_to_db();
                    crate::render::rebuild_keymap_actions(app);
                    let lang = crate::i18n::Lang::from_code(&app.settings.language);
                    app.set_message(
                        crate::i18n::t("settings.keymap_reset_done", lang),
                        Duration::from_secs(2),
                    );
                }
            }
            return EventResult::Continue;
        }

        // 滚轮支持
        if matches!(mouse.kind, MouseEventKind::ScrollUp) {
            if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                if ke.selected > 0 {
                    ke.selected -= 1;
                    // 自动滚动
                    let scroll = ke.scroll as usize;
                    if ke.selected < scroll {
                        ke.scroll = ke.selected as u16;
                    }
                }
            }
            return EventResult::Continue;
        }
        if matches!(mouse.kind, MouseEventKind::ScrollDown) {
            if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                let total = ke.actions.len();
                if ke.selected + 1 <= total {
                    ke.selected += 1;
                    // 自动滚动
                    let scroll = ke.scroll as usize;
                    if ke.selected >= scroll + visible_rows {
                        ke.scroll = (ke.selected - visible_rows + 1) as u16;
                    }
                }
            }
            return EventResult::Continue;
        }
    }

    // 键盘事件处理
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return EventResult::Continue;
        }
        let action = app
            .keymap
            .resolve(AppScreen::KeymapConfig, false, key.code, key.modifiers);
        match action {
            Some(Action::SettingsUp) => {
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    if ke.selected > 0 {
                        ke.selected -= 1;
                        app.settings_ui.keymap_error = None;
                        // 自动滚动
                        let scroll = ke.scroll as usize;
                        if ke.selected < scroll {
                            ke.scroll = ke.selected as u16;
                        }
                    }
                }
            }
            Some(Action::SettingsDown) => {
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    let total = ke.actions.len();
                    if ke.selected < total {
                        ke.selected += 1;
                        app.settings_ui.keymap_error = None;
                        // 自动滚动
                        let scroll = ke.scroll as usize;
                        if ke.selected >= scroll + visible_rows {
                            ke.scroll = (ke.selected - visible_rows + 1) as u16;
                        }
                    }
                }
            }
            Some(Action::Confirm) => {
                if let Some(ref mut ke) = app.settings_ui.keymap_edit {
                    let total = ke.actions.len();
                    if ke.selected == total {
                        // 重置为默认
                        app.keymap.reset_to_default();
                        app.keymap.save_to_db();
                        crate::render::rebuild_keymap_actions(app);
                        let lang = crate::i18n::Lang::from_code(&app.settings.language);
                        app.set_message(
                            crate::i18n::t("settings.keymap_reset_done", lang),
                            Duration::from_secs(2),
                        );
                    } else {
                        // 进入绑定模式
                        ke.awaiting_key = true;
                        ke.rebinding_index = Some(ke.selected);
                        app.settings_ui.keymap_error = None;
                    }
                }
            }
            Some(Action::Cancel) => {
                // 退出键位编辑,回到设置
                app.settings_ui.keymap_edit = None;
                app.settings_ui.keymap_error = None;
                app.screen = AppScreen::Settings;
                app.settings_ui.visible = true;
            }
            _ => {}
        }
    }
    EventResult::Continue
}
