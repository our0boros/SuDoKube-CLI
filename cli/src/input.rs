use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use std::time::Duration;
use sudokube_core::cube::Face;

use crate::{AppScreen, CliState, MenuItem, current_coord, new_game};
use crate::render::{ButtonId, cell_at, find_button_at, mode_label};
use crate::save::delete_game;

pub enum EventResult {
    Continue,
    StartGame(sudokube_core::game_state::GameState),
    BackToMenu,
    Quit,
}

pub fn handle_event(state: &mut CliState, event: Event) -> EventResult {
    match state.screen {
        AppScreen::Menu => handle_menu_event(state, event),
        AppScreen::Game => handle_game_event(state, event),
    }
}

fn handle_menu_event(state: &mut CliState, event: Event) -> EventResult {
    match event {
        Event::Resize(_, _) => {
            state.dirty = true;
        }
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => return EventResult::Quit,
            KeyCode::Up => {
                if state.menu.selected > 0 {
                    state.menu.selected -= 1;
                    state.dirty = true;
                }
            }
            KeyCode::Down => {
                if state.menu.selected + 1 < state.menu.items.len() {
                    state.menu.selected += 1;
                    state.dirty = true;
                }
            }
            KeyCode::Enter => {
                let item = state.menu.items[state.menu.selected].clone();
                return match item {
                    MenuItem::NewGame(d) => EventResult::StartGame(new_game(d)),
                    MenuItem::Continue(r) => {
                        EventResult::StartGame(crate::continue_game(&r))
                    }
                };
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(MenuItem::Continue(r)) = state.menu.items.get(state.menu.selected).cloned() {
                    let _ = delete_game(r.id);
                    state.menu = crate::MenuState::new();
                    state.dirty = true;
                }
            }
            _ => {}
        },
        Event::Mouse(mouse) => {
            if let Some(idx) = menu_item_at(state, mouse.column, mouse.row) {
                if idx != state.menu.selected {
                    state.menu.selected = idx;
                    state.dirty = true;
                }
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let item = state.menu.items[idx].clone();
                    return match item {
                        MenuItem::NewGame(d) => EventResult::StartGame(new_game(d)),
                        MenuItem::Continue(r) => EventResult::StartGame(crate::continue_game(&r)),
                    };
                }
            }
        }
        _ => {}
    }
    EventResult::Continue
}

fn menu_item_at(state: &CliState, _col: u16, row: u16) -> Option<usize> {
    // 菜单项渲染位置较复杂，简化处理：按固定估算行。
    let term_rows = state.prev_term_size.1;
    let start_row = term_rows / 4 + 2;
    let idx = row.saturating_sub(start_row) as usize;
    if idx < state.menu.items.len() {
        Some(idx)
    } else {
        None
    }
}

fn handle_game_event(state: &mut CliState, event: Event) -> EventResult {
    let metrics = state.render_mode.metrics();
    let layout = crate::render::compute_layout_for_state(state);

    match event {
        Event::Resize(_, _) => {
            state.dirty = true;
        }
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            return handle_key(state, key);
        }
        Event::Mouse(mouse) => {
            return handle_mouse(state, &layout, &metrics, mouse);
        }
        _ => {}
    }
    EventResult::Continue
}

fn handle_key(state: &mut CliState, key: KeyEvent) -> EventResult {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let shift_or_none = key.modifiers.is_empty() || shift;

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return EventResult::BackToMenu,
        KeyCode::Char('n') | KeyCode::Char('N') => {
            state.game = new_game(state.game.difficulty);
            state.current_face = Face::Front;
            state.cursor = (4, 4);
            state.dirty = true;
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            state.render_mode = state.render_mode.toggle();
            state.set_message(
                format!("已切换为 {} 模式", mode_label(state.render_mode)),
                Duration::from_secs(2),
            );
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            state.theme = state.theme.toggle();
            state.dirty = true;
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            state.guidance = !state.guidance;
            state.set_message(
                format!("辅助模式{}", if state.guidance { "开" } else { "关" }),
                Duration::from_secs(2),
            );
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            state.game.hint();
            state.set_message("已提示", Duration::from_secs(2));
            if state.game.check_completion() {
                state.set_message("恭喜完成！按 N 开始新局，Q 退出。", Duration::from_secs(5));
            }
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            state.game.undo();
            state.set_message("已撤销", Duration::from_secs(2));
        }
        KeyCode::Backspace | KeyCode::Delete => {
            let coord = current_coord(state);
            state.game.set_value(coord, None);
            state.dirty = true;
        }
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let value = c as u8 - b'0';
            let coord = current_coord(state);
            state.game.set_value(coord, Some(value));
            state.dirty = true;
            if state.game.check_completion() {
                state.set_message("恭喜完成！按 N 开始新局，Q 退出。", Duration::from_secs(5));
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') if shift_or_none => {
            move_cursor_with_wrap(state, 0, -1);
        }
        KeyCode::Char('a') | KeyCode::Char('A') if shift_or_none => {
            move_cursor_with_wrap(state, -1, 0);
        }
        KeyCode::Char('s') | KeyCode::Char('S') if shift_or_none => {
            move_cursor_with_wrap(state, 0, 1);
        }
        KeyCode::Char('d') | KeyCode::Char('D') if shift_or_none => {
            move_cursor_with_wrap(state, 1, 0);
        }
        KeyCode::Up => {
            state.current_face = switch_face(state.current_face, 0, -1);
            state.dirty = true;
        }
        KeyCode::Down => {
            state.current_face = switch_face(state.current_face, 0, 1);
            state.dirty = true;
        }
        KeyCode::Left => {
            state.current_face = switch_face(state.current_face, -1, 0);
            state.dirty = true;
        }
        KeyCode::Right => {
            state.current_face = switch_face(state.current_face, 1, 0);
            state.dirty = true;
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.current_face = Face::Front;
            state.dirty = true;
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            state.current_face = Face::Back;
            state.dirty = true;
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            state.current_face = Face::Left;
            state.dirty = true;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            state.current_face = Face::Right;
            state.dirty = true;
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            state.current_face = Face::Top;
            state.dirty = true;
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            state.current_face = Face::Bottom;
            state.dirty = true;
        }
        _ => {}
    }

    state.game.selected = Some(current_coord(state));
    EventResult::Continue
}

fn handle_mouse(
    state: &mut CliState,
    layout: &crate::render::Layout,
    metrics: &crate::render::Metrics,
    mouse: MouseEvent,
) -> EventResult {
    match mouse.kind {
        MouseEventKind::Moved => {
            let new_hover = find_button_at(layout, mouse.column, mouse.row);
            if new_hover != state.hover_button {
                state.hover_button = new_hover;
                state.dirty = true;
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(btn) = find_button_at(layout, mouse.column, mouse.row) {
                return execute_button(state, btn);
            }
            if let Some((u, v)) = cell_at(layout, metrics, mouse.column, mouse.row) {
                state.cursor = (u, v);
                state.dirty = true;
            }
        }
        MouseEventKind::ScrollUp => {
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                state.current_face = cycle_face_horizontal(state.current_face, true);
            } else {
                state.current_face = cycle_face_vertical(state.current_face, true);
            }
            state.dirty = true;
        }
        MouseEventKind::ScrollDown => {
            if mouse.modifiers.contains(KeyModifiers::ALT) {
                state.current_face = cycle_face_horizontal(state.current_face, false);
            } else {
                state.current_face = cycle_face_vertical(state.current_face, false);
            }
            state.dirty = true;
        }
        _ => {}
    }
    state.game.selected = Some(current_coord(state));
    EventResult::Continue
}

fn execute_button(state: &mut CliState, btn: ButtonId) -> EventResult {
    match btn {
        ButtonId::Number(n) => {
            let coord = current_coord(state);
            state.game.set_value(coord, Some(n));
            state.dirty = true;
            if state.game.check_completion() {
                state.set_message("恭喜完成！按 N 开始新局，Q 退出。", Duration::from_secs(5));
            }
        }
        ButtonId::Hint => {
            state.game.hint();
            state.set_message("已提示", Duration::from_secs(2));
        }
        ButtonId::Undo => {
            state.game.undo();
            state.set_message("已撤销", Duration::from_secs(2));
        }
        ButtonId::ToggleGuidance => {
            state.guidance = !state.guidance;
            state.set_message(
                format!("辅助模式{}", if state.guidance { "开" } else { "关" }),
                Duration::from_secs(2),
            );
        }
        ButtonId::NewGame => {
            state.game = new_game(state.game.difficulty);
            state.current_face = Face::Front;
            state.cursor = (4, 4);
            state.dirty = true;
        }
        ButtonId::ToggleMode => {
            state.render_mode = state.render_mode.toggle();
            state.set_message(
                format!("已切换为 {} 模式", mode_label(state.render_mode)),
                Duration::from_secs(2),
            );
        }
        ButtonId::ToggleTheme => {
            state.theme = state.theme.toggle();
            state.dirty = true;
        }
        ButtonId::Quit => return EventResult::BackToMenu,
    }
    state.game.selected = Some(current_coord(state));
    EventResult::Continue
}

/// 垂直环：前 -> 上 -> 后 -> 下 -> 前。
fn cycle_face_vertical(face: Face, forward: bool) -> Face {
    let ring = [Face::Front, Face::Top, Face::Back, Face::Bottom];
    cycle_in_ring(&ring, face, forward)
}

/// 横向环：前 -> 左 -> 后 -> 右 -> 前。
fn cycle_face_horizontal(face: Face, forward: bool) -> Face {
    let ring = [Face::Front, Face::Left, Face::Back, Face::Right];
    cycle_in_ring(&ring, face, forward)
}

fn cycle_in_ring(ring: &[Face], face: Face, forward: bool) -> Face {
    if let Some(pos) = ring.iter().position(|&f| f == face) {
        let delta = if forward { 1 } else { ring.len() - 1 };
        ring[(pos + delta) % ring.len()]
    } else {
        // 当前面不在环内时，回到环的起点或终点。
        if forward {
            ring[0]
        } else {
            ring[ring.len() - 1]
        }
    }
}

/// 移动光标，到达边缘时切换到相邻面并保持几何连续性。
fn move_cursor_with_wrap(state: &mut CliState, dx: i8, dy: i8) {
    let (face, cursor) = move_on_surface(state.current_face, state.cursor, dx, dy);
    if face != state.current_face || cursor != state.cursor {
        state.current_face = face;
        state.cursor = cursor;
        state.dirty = true;
    }
}

/// 根据当前面局部坐标和移动方向，返回新的面和光标位置（光标已位于相邻面内部一格）。
fn move_on_surface(face: Face, cursor: (u8, u8), dx: i8, dy: i8) -> (Face, (u8, u8)) {
    let (u, v) = (cursor.0 as i8, cursor.1 as i8);
    let nu = u + dx;
    let nv = v + dy;

    if (0..9).contains(&nu) && (0..9).contains(&nv) {
        return (face, (nu as u8, nv as u8));
    }

    // 出界时切换到相邻面。映射基于 to_cube 的几何关系推导，确保来回移动可逆。
    match face {
        Face::Front => {
            if nv < 0 {
                (Face::Bottom, (7, u as u8))
            } else if nv > 8 {
                (Face::Top, (u as u8, 7))
            } else if nu < 0 {
                (Face::Left, (7, v as u8))
            } else {
                (Face::Right, (v as u8, 7))
            }
        }
        Face::Back => {
            if nv < 0 {
                (Face::Left, (1, u as u8))
            } else if nv > 8 {
                (Face::Right, (u as u8, 1))
            } else if nu < 0 {
                (Face::Bottom, (1, v as u8))
            } else {
                (Face::Top, (v as u8, 1))
            }
        }
        Face::Top => {
            if nv < 0 {
                (Face::Back, (7, u as u8))
            } else if nv > 8 {
                (Face::Front, (u as u8, 7))
            } else if nu < 0 {
                (Face::Left, (v as u8, 7))
            } else {
                (Face::Right, (7, v as u8))
            }
        }
        Face::Bottom => {
            if nv < 0 {
                (Face::Left, (u as u8, 1))
            } else if nv > 8 {
                (Face::Right, (1, u as u8))
            } else if nu < 0 {
                (Face::Back, (1, v as u8))
            } else {
                (Face::Front, (v as u8, 1))
            }
        }
        Face::Left => {
            if nv < 0 {
                (Face::Bottom, (u as u8, 1))
            } else if nv > 8 {
                (Face::Top, (1, u as u8))
            } else if nu < 0 {
                (Face::Back, (v as u8, 1))
            } else {
                (Face::Front, (1, v as u8))
            }
        }
        Face::Right => {
            if nv < 0 {
                (Face::Back, (u as u8, 7))
            } else if nv > 8 {
                (Face::Front, (7, u as u8))
            } else if nu < 0 {
                (Face::Bottom, (v as u8, 7))
            } else {
                (Face::Top, (7, v as u8))
            }
        }
    }
}

/// 方向键切换面（不移动光标）。
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
