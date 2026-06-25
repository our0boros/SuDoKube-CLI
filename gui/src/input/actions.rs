use crate::app_state::AppState;
use bevy::prelude::*;
use sudokube_core::game_state::GameState;

pub fn handle_keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut game_state: ResMut<GameState>) {
    if keys.just_pressed(KeyCode::Escape) {
        // ESC 由状态切换系统处理，这里只消费事件。
        return;
    }

    if keys.just_pressed(KeyCode::KeyH) {
        game_state.hint();
        return;
    }

    if keys.just_pressed(KeyCode::KeyZ)
        || (keys.just_pressed(KeyCode::KeyZ) && keys.pressed(KeyCode::ControlLeft))
    {
        game_state.undo();
        return;
    }

    if keys.just_pressed(KeyCode::Backspace) || keys.just_pressed(KeyCode::Delete) {
        if let Some(coord) = game_state.selected {
            game_state.set_value(coord, None);
        }
        return;
    }

    // 数字键 1..9。
    for (code, value) in number_keys() {
        if keys.just_pressed(code) {
            if let Some(coord) = game_state.selected {
                game_state.set_value(coord, Some(value));
            }
            return;
        }
    }
}

pub fn handle_game_completion(
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if game_state.check_completion() {
        // 完成后可进入结算或自动下一局逻辑。
        if game_state.auto_next {
            next_state.set(AppState::Loading);
        }
    }
}

fn number_keys() -> [(KeyCode, u8); 9] {
    [
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ]
}
