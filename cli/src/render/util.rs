//! 渲染公共工具函数

use ratatui::{layout::Rect, style::Color};

use sudokube_core::cube::Face;

use crate::i18n::{self, Lang};
use crate::{App, total_elapsed};

use super::types::{ButtonId, GameLayout, PagerAction, RenderMode};

// ── 鼠标点击检测 ──

pub fn find_button_at(layout: &GameLayout, col: u16, row: u16) -> Option<ButtonId> {
    for btn in &layout.buttons {
        if row >= btn.row
            && row < btn.row + btn.height.max(1)
            && col >= btn.col
            && col < btn.col + btn.width
        {
            return Some(btn.id);
        }
    }
    None
}

pub fn shop_item_at(layout: &GameLayout, col: u16, row: u16) -> Option<usize> {
    let panel = layout.shop_area;
    if col < panel.x || col >= panel.x + panel.width {
        return None;
    }
    if row < panel.y || row >= panel.y + panel.height {
        return None;
    }
    let inner_y = row.saturating_sub(panel.y + 1);
    let item_top = 2;
    if inner_y < item_top {
        return None;
    }
    let item_row = (inner_y - item_top) / 3;
    if item_row >= 4 {
        return None;
    }
    Some(item_row as usize)
}

pub fn pager_action_at(layout: &GameLayout, col: u16, row: u16) -> Option<PagerAction> {
    if let Some(pager) = &layout.pager {
        if row == pager.prev_rect.y && col == pager.prev_rect.x && pager.total_pages > 1 {
            return Some(PagerAction::Prev);
        }
        if row == pager.next_rect.y && col == pager.next_rect.x && pager.total_pages > 1 {
            return Some(PagerAction::Next);
        }
    }
    None
}

pub fn cell_at(layout: &GameLayout, cw: usize, ch: usize, col: u16, row: u16) -> Option<(u8, u8)> {
    let gx = col.saturating_sub(layout.grid_area.x);
    let gy = row.saturating_sub(layout.grid_area.y);
    if gy % (ch as u16 + 1) == 0 {
        return None;
    }
    let v = (gy / (ch as u16 + 1)) as u8;
    if v >= 9 {
        return None;
    }
    if gx % (cw as u16 + 1) == 0 {
        return None;
    }
    let u = (gx / (cw as u16 + 1)) as u8;
    if u >= 9 {
        return None;
    }
    Some((u, v))
}

// ── 文本格式化 ──

pub fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

pub fn pad_right(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if dw >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - dw))
    }
}

pub fn pad_center(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if dw >= width {
        return s.to_string();
    }
    let total = width - dw;
    let left = total / 2;
    let right = total - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

pub fn bordered_line(content: &str, inner_w: usize, center: bool) -> String {
    let padded = if center {
        pad_center(content, inner_w)
    } else {
        pad_right(content, inner_w)
    };
    format!("│{}│", padded)
}

// ── 游戏工具 ──

pub fn format_timer(app: &App) -> String {
    let elapsed = total_elapsed(app);
    format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
}

pub fn is_wrong(app: &App, coord: sudokube_core::cube::CubeCoord, value: u8) -> bool {
    app.game
        .solution
        .get(&coord)
        .map_or(true, |&sol| sol != value)
}

// ── 面相关工具 ──

pub fn face_name(face: Face, lang: Lang) -> &'static str {
    match face {
        Face::Front => i18n::t("game.face_front", lang),
        Face::Back => i18n::t("game.face_back", lang),
        Face::Left => i18n::t("game.face_left", lang),
        Face::Right => i18n::t("game.face_right", lang),
        Face::Top => i18n::t("game.face_top", lang),
        Face::Bottom => i18n::t("game.face_bottom", lang),
    }
}

pub fn face_to_color(face: Face) -> Color {
    match face {
        Face::Front => Color::Red,
        Face::Back => Color::Blue,
        Face::Left => Color::Green,
        Face::Right => Color::Yellow,
        Face::Top => Color::Magenta,
        Face::Bottom => Color::Cyan,
    }
}

pub fn wasd_neighbor_faces(face: Face) -> (Face, Face, Face, Face, Face) {
    match face {
        Face::Front => (Face::Bottom, Face::Top, Face::Left, Face::Right, Face::Back),
        Face::Back => (
            Face::Left,
            Face::Right,
            Face::Bottom,
            Face::Top,
            Face::Front,
        ),
        Face::Top => (
            Face::Back,
            Face::Front,
            Face::Left,
            Face::Right,
            Face::Bottom,
        ),
        Face::Bottom => (Face::Left, Face::Right, Face::Back, Face::Front, Face::Top),
        Face::Left => (
            Face::Bottom,
            Face::Top,
            Face::Back,
            Face::Front,
            Face::Right,
        ),
        Face::Right => (Face::Back, Face::Front, Face::Bottom, Face::Top, Face::Left),
    }
}

pub fn arrow_neighbor_faces(face: Face) -> (Face, Face, Face, Face, Face) {
    match face {
        Face::Front => (Face::Top, Face::Bottom, Face::Left, Face::Right, Face::Back),
        Face::Back => (
            Face::Top,
            Face::Bottom,
            Face::Right,
            Face::Left,
            Face::Front,
        ),
        Face::Top => (
            Face::Back,
            Face::Front,
            Face::Left,
            Face::Right,
            Face::Bottom,
        ),
        Face::Bottom => (Face::Front, Face::Back, Face::Left, Face::Right, Face::Top),
        Face::Left => (
            Face::Top,
            Face::Bottom,
            Face::Back,
            Face::Front,
            Face::Right,
        ),
        Face::Right => (Face::Top, Face::Bottom, Face::Front, Face::Back, Face::Left),
    }
}

// ── 颜色 ──

pub fn parse_color(name: &str) -> Color {
    match name {
        "black" => Color::Black,
        "darkgray" => Color::DarkGray,
        "white" => Color::White,
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "red" => Color::Red,
        "yellow" => Color::Yellow,
        "gray" => Color::Gray,
        _ => Color::Cyan,
    }
}

// ── 溢出检测 ──

pub fn needs_scrollbar_mode(area: Rect, app: &App) -> bool {
    if app.render_mode == RenderMode::Scrollbar {
        return false;
    }
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let grid_h = (1 + 9 * (ch + 1)) as u16;
    let grid_w = (1 + 9 * (cw + 1)) as u16;
    let center_panel_w = grid_w + 2;
    let min_h: u16 = grid_h + 8;
    let min_w: u16 = center_panel_w + 24;
    area.height < min_h || area.width < min_w
}
