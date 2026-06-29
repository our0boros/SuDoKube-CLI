use crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

use crate::config::Action;
use crate::{App, AppScreen};

use super::EventResult;

pub(super) fn handle_settings_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    // 键位映射编辑模式: 独立处理
    if app.settings_ui.keymap_edit.is_some() {
        return super::keymap::handle_keymap_edit_event(app, event, area);
    }

    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            // 设置弹窗可能覆盖在 Menu 上，需要用 Settings 画面解析按键
            let action = app
                .keymap
                .resolve(AppScreen::Settings, false, key.code, key.modifiers);
            match action {
                Some(Action::SettingsUp) => {
                    if app.settings_ui.selected > 0 {
                        app.settings_ui.selected -= 1;
                    }
                }
                Some(Action::SettingsDown) => {
                    if app.settings_ui.selected + 1 < app.settings_ui.fields.len() {
                        app.settings_ui.selected += 1;
                    }
                }
                Some(Action::SettingsLeft) => {
                    let idx = app.settings_ui.selected;
                    // Keymap 字段不走左右切换
                    if app.settings_ui.fields[idx].label != "Keymap" {
                        // Guide 字段:未购买时锁定,不能切换
                        if app.settings_ui.fields[idx].label == "Guide"
                            && app.settings.guide_owned != "on"
                        {
                            return EventResult::Continue;
                        }
                        app.settings_ui.fields[idx].cycle_prev();
                        app.settings_ui.apply_to(&mut app.settings);
                        app.settings.save_to_db();
                    }
                }
                Some(Action::SettingsRight) | Some(Action::Confirm) => {
                    let idx = app.settings_ui.selected;
                    // Keymap 字段: 进入映射编辑界面
                    if app.settings_ui.fields[idx].label == "Keymap" {
                        app.settings_ui.visible = false;
                        app.screen = AppScreen::KeymapConfig;
                    } else if matches!(action, Some(Action::SettingsRight)) {
                        // Guide 字段:未购买时锁定,不能切换
                        if app.settings_ui.fields[idx].label == "Guide"
                            && app.settings.guide_owned != "on"
                        {
                            return EventResult::Continue;
                        }
                        app.settings_ui.fields[idx].cycle_next();
                        app.settings_ui.apply_to(&mut app.settings);
                        app.settings.save_to_db();
                    }
                    // Confirm on non-keymap field: 关闭设置
                    if matches!(action, Some(Action::Confirm))
                        && app.settings_ui.fields[idx].label != "Keymap"
                    {
                        app.settings.save_to_db();
                        app.settings_ui.visible = false;
                        if app.screen == AppScreen::Settings {
                            app.screen = AppScreen::Menu;
                        }
                    }
                }
                Some(Action::Cancel) => {
                    app.settings.save_to_db();
                    app.settings_ui.visible = false;
                    if app.screen == AppScreen::Settings {
                        app.screen = AppScreen::Menu;
                    }
                }
                _ => {}
            }
        }
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
                    // Guide 字段未购买时锁定,鼠标点击也不允许切换
                    let guide_locked = app.settings_ui.fields[i].label == "Guide"
                        && app.settings.guide_owned != "on";
                    if rect_contains(f.left_arrow_rect, mouse.column, mouse.row) {
                        app.settings_ui.selected = i;
                        if !guide_locked {
                            app.settings_ui.fields[i].cycle_prev();
                            app.settings_ui.apply_to(&mut app.settings);
                            app.settings.save_to_db();
                        }
                        return EventResult::Continue;
                    } else if rect_contains(f.right_arrow_rect, mouse.column, mouse.row) {
                        app.settings_ui.selected = i;
                        if !guide_locked {
                            app.settings_ui.fields[i].cycle_next();
                            app.settings_ui.apply_to(&mut app.settings);
                            app.settings.save_to_db();
                        }
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
