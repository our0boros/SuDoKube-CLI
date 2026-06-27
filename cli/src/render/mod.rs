mod cube3d;
mod game;
mod menu;
pub mod overlay;
mod settings;
pub mod types;
mod util;

pub use game::compute_game_layout_from_rect;
pub use overlay::rebuild_keymap_actions;
pub use settings::compute_settings_popup_layout;
pub use types::{ButtonId, GameLayout, PagerAction, RenderMode, mode_label};
pub use util::{cell_at, find_button_at, needs_scrollbar_mode, pager_action_at, shop_item_at};

use crate::{App, AppScreen};
use ratatui::Frame;

/// 主渲染入口：根据当前屏幕分发绘制
pub fn draw(f: &mut Frame, app: &mut App) {
    match app.screen {
        AppScreen::Menu => {
            menu::draw_menu(f, app);
            if app.settings_ui.visible {
                settings::draw_settings_overlay(f, app);
            }
        }
        AppScreen::Game => {
            game::draw_game(f, app);
        }
        AppScreen::Settings => {
            app.settings_ui.visible = true;
            menu::draw_menu(f, app);
            settings::draw_settings_overlay(f, app);
        }
        AppScreen::Generating => overlay::draw_generating(f, app),
        AppScreen::Victory => overlay::draw_victory(f, app),
        AppScreen::ExportSelect => {
            menu::draw_menu(f, app);
            overlay::draw_export_overlay(f, app);
        }
        AppScreen::ImportInput => {
            menu::draw_menu(f, app);
            overlay::draw_import_overlay(f, app);
        }
        AppScreen::KeymapConfig => {
            overlay::draw_keymap_config(f, app);
        }
    }

    if app.confirm_delete_id.is_some() {
        overlay::draw_confirm_delete_overlay(f, app);
    }
}
