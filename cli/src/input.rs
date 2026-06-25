use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::layout::Rect;
use std::time::Duration;
use sudokube_core::cube::{Difficulty, Face};

use crate::{App, AppScreen, MenuItem, current_coord};
use crate::render::{ButtonId, GameLayout, compute_game_layout_from_rect, find_button_at, cell_at, mode_label};
use crate::save::delete_game;
use crate::i18n::{self, Lang};

pub enum EventResult {
    Continue,
    StartGame(sudokube_core::game_state::GameState),
    StartGenerating(Difficulty),
    BackToMenu,
    Quit,
}

pub fn handle_event(app: &mut App, event: Event, area: Rect) -> EventResult {
    match app.screen {
        AppScreen::Menu => handle_menu_event(app, event, area),
        AppScreen::Game => handle_game_event(app, event, area),
        AppScreen::Settings => handle_settings_event(app, event),
        AppScreen::Generating => EventResult::Continue, // 生成中忽略所有输入
    }
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
                        app.screen = AppScreen::Settings;
                        EventResult::Continue
                    }
                };
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(MenuItem::Continue(r)) = app.menu.items.get(app.menu.selected).cloned() {
                    let _ = delete_game(r.id);
                    app.menu = crate::MenuState::new();
                }
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
                            app.screen = AppScreen::Settings;
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

fn handle_settings_event(app: &mut App, event: Event) -> EventResult {
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
                app.screen = AppScreen::Menu;
            }
            _ => {}
        },
        _ => {}
    }
    EventResult::Continue
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

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return EventResult::BackToMenu,
        KeyCode::Char('n') | KeyCode::Char('N') => {
            return EventResult::StartGenerating(app.game.difficulty);
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.render_mode = app.render_mode.toggle();
            let lang = Lang::from_code(&app.settings.language);
            app.set_message(
                format!("{}", mode_label(app.render_mode, lang)),
                Duration::from_secs(2),
            );
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            let coord = current_coord(app);
            if let Some(cell) = app.game.grid.get(&coord) {
                if !cell.given {
                    app.game.set_value(coord, None);
                }
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.guidance = !app.guidance;
            app.set_message(
                format!("辅助模式{}", if app.guidance { "开" } else { "关" }),
                Duration::from_secs(2),
            );
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.game.hint();
            app.set_message("已提示", Duration::from_secs(2));
            if app.game.check_completion() {
                app.set_message("恭喜完成！按 N 开始新局，Q 退出。", Duration::from_secs(5));
            }
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            app.game.undo();
            app.set_message("已撤销", Duration::from_secs(2));
        }
        KeyCode::Backspace | KeyCode::Delete => {
            let coord = current_coord(app);
            app.game.set_value(coord, None);
        }
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let value = c as u8 - b'0';
            let coord = current_coord(app);
            app.game.set_value(coord, Some(value));
            if app.game.check_completion() {
                app.set_message("恭喜完成！按 N 开始新局，Q 退出。", Duration::from_secs(5));
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
            app.current_face = switch_face(app.current_face, 0, -1);
        }
        KeyCode::Down => {
            app.current_face = switch_face(app.current_face, 0, 1);
        }
        KeyCode::Left => {
            app.current_face = switch_face(app.current_face, -1, 0);
        }
        KeyCode::Right => {
            app.current_face = switch_face(app.current_face, 1, 0);
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            app.current_face = Face::Front;
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.current_face = Face::Back;
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.current_face = Face::Left;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.current_face = Face::Right;
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.current_face = Face::Top;
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            app.current_face = Face::Bottom;
        }
        _ => {}
    }

    app.game.selected = Some(current_coord(app));
    EventResult::Continue
}

fn handle_mouse(
    app: &mut App,
    layout: &GameLayout,
    mouse: MouseEvent,
) -> EventResult {
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
            if let Some(btn) = find_button_at(layout, mouse.column, mouse.row) {
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
            app.game.set_value(coord, Some(n));
            if app.game.check_completion() {
                app.set_message("恭喜完成！按 N 开始新局，Q 退出。", Duration::from_secs(5));
            }
        }
        ButtonId::Erase => {
            let coord = current_coord(app);
            if let Some(cell) = app.game.grid.get(&coord) {
                if !cell.given {
                    app.game.set_value(coord, None);
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
            if nv < 0 { (Face::Bottom, (8, u as u8)) }
            else if nv > 8 { (Face::Top, (u as u8, 8)) }
            else if nu < 0 { (Face::Left, (8, v as u8)) }
            else { (Face::Right, (v as u8, 8)) }
        }
        Face::Back => {
            if nv < 0 { (Face::Left, (0, u as u8)) }
            else if nv > 8 { (Face::Right, (u as u8, 0)) }
            else if nu < 0 { (Face::Bottom, (v as u8, 0)) }
            else { (Face::Top, (v as u8, 0)) }
        }
        Face::Top => {
            if nv < 0 { (Face::Back, (u as u8, 8)) }
            else if nv > 8 { (Face::Front, (u as u8, 8)) }
            else if nu < 0 { (Face::Left, (v as u8, 8)) }
            else { (Face::Right, (8, v as u8)) }
        }
        Face::Bottom => {
            if nv < 0 { (Face::Left, (u as u8, 0)) }
            else if nv > 8 { (Face::Right, (0, u as u8)) }
            else if nu < 0 { (Face::Back, (v as u8, 0)) }
            else { (Face::Front, (v as u8, 0)) }
        }
        Face::Left => {
            if nv < 0 { (Face::Bottom, (u as u8, 0)) }
            else if nv > 8 { (Face::Top, (0, u as u8)) }
            else if nu < 0 { (Face::Back, (0, v as u8)) }
            else { (Face::Front, (0, v as u8)) }
        }
        Face::Right => {
            if nv < 0 { (Face::Back, (8, u as u8)) }
            else if nv > 8 { (Face::Front, (8, u as u8)) }
            else if nu < 0 { (Face::Bottom, (v as u8, 8)) }
            else { (Face::Top, (8, v as u8)) }
        }
    }
}

fn switch_face(face: Face, dx: i8, dy: i8) -> Face {
    match face {
        Face::Front => match (dx, dy) {
            (0, -1) => Face::Top, (0, 1) => Face::Bottom,
            (-1, 0) => Face::Left, (1, 0) => Face::Right, _ => face,
        },
        Face::Back => match (dx, dy) {
            (0, -1) => Face::Top, (0, 1) => Face::Bottom,
            (-1, 0) => Face::Right, (1, 0) => Face::Left, _ => face,
        },
        Face::Top => match (dx, dy) {
            (0, -1) => Face::Back, (0, 1) => Face::Front,
            (-1, 0) => Face::Left, (1, 0) => Face::Right, _ => face,
        },
        Face::Bottom => match (dx, dy) {
            (0, -1) => Face::Front, (0, 1) => Face::Back,
            (-1, 0) => Face::Left, (1, 0) => Face::Right, _ => face,
        },
        Face::Left => match (dx, dy) {
            (0, -1) => Face::Top, (0, 1) => Face::Bottom,
            (-1, 0) => Face::Back, (1, 0) => Face::Front, _ => face,
        },
        Face::Right => match (dx, dy) {
            (0, -1) => Face::Top, (0, 1) => Face::Bottom,
            (-1, 0) => Face::Front, (1, 0) => Face::Back, _ => face,
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
    let lang = Lang::from_code(&app.settings.language);
    app.set_message(
        format!("{} {} cells", i18n::t("debug.hint", lang), filled),
        Duration::from_secs(2),
    );
}
