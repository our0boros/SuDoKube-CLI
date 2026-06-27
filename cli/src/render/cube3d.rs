//! 3D 立方体渲染

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use sudokube_core::cube::Face;

use crate::i18n::Lang;
use crate::App;

use super::types::GameLayout;
use super::util::{arrow_neighbor_faces, face_name, face_to_color, parse_color};

pub fn draw_3d_cube(f: &mut Frame, layout: &GameLayout, app: &App) {
    let bg = parse_color(&app.settings.bg_color);
    let face_color = face_to_color(app.current_face);

    // 1) 外层 Block::bordered — 第一层边框
    if layout.cube_outer_frame.width >= 4 && layout.cube_outer_frame.height >= 3 {
        let outer_block = Block::bordered()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(face_color))
            .style(Style::default().bg(bg));
        f.render_widget(outer_block, layout.cube_outer_frame);
    }

    // 2) 内部方向指示条（实心方块 ▒▒）
    let (up_face, down_face, left_face, right_face, _back_face) =
        arrow_neighbor_faces(app.current_face);
    let up_color = face_to_color(up_face);
    let down_color = face_to_color(down_face);
    let left_color = face_to_color(left_face);
    let right_color = face_to_color(right_face);

    // 顶方向条
    if layout.cube_dir_top.width > 0 && layout.cube_dir_top.height > 0 {
        let line: String = "▒".repeat(layout.cube_dir_top.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(up_color).bg(bg)),
            layout.cube_dir_top,
        );
    }
    // 底方向条
    if layout.cube_dir_bot.width > 0 && layout.cube_dir_bot.height > 0 {
        let line: String = "▒".repeat(layout.cube_dir_bot.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(down_color).bg(bg)),
            layout.cube_dir_bot,
        );
    }
    // 左方向条
    if layout.cube_dir_left.width > 0 && layout.cube_dir_left.height > 0 {
        for row in 0..layout.cube_dir_left.height {
            f.render_widget(
                Paragraph::new("▒").style(Style::default().fg(left_color).bg(bg)),
                Rect::new(layout.cube_dir_left.x, layout.cube_dir_left.y + row, 1, 1),
            );
        }
    }
    // 右方向条
    if layout.cube_dir_right.width > 0 && layout.cube_dir_right.height > 0 {
        for row in 0..layout.cube_dir_right.height {
            f.render_widget(
                Paragraph::new("▒").style(Style::default().fg(right_color).bg(bg)),
                Rect::new(layout.cube_dir_right.x, layout.cube_dir_right.y + row, 1, 1),
            );
        }
    }

    // 3) 旋转 cube
    let area = layout.cube_area;
    if area.width < 4 || area.height < 4 {
        return;
    }

    let placeholder = Block::default().style(Style::default().bg(bg));
    let content_area = placeholder.inner(area);

    let cx = content_area.x as f64 + content_area.width as f64 / 2.0;
    let cy = content_area.y as f64 + content_area.height as f64 / 2.0;
    let scale_factor: f64 = app.settings.cube_scale.parse().unwrap_or(0.38);
    let scale_x = content_area.width as f64 * scale_factor;
    let scale_y = content_area.height as f64 * scale_factor;

    let cos_y = app.cube_angle_y.cos();
    let sin_y = app.cube_angle_y.sin();
    let cos_x = app.cube_angle_x.cos();
    let sin_x = app.cube_angle_x.sin();

    let verts: [(f64, f64, f64); 8] = [
        (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0), (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
        (-1.0, -1.0, 1.0),  (1.0, -1.0, 1.0),  (1.0, 1.0, 1.0),  (-1.0, 1.0, 1.0),
    ];

    let project = |x: f64, y: f64, z: f64| {
        let x1 = x * cos_y + z * sin_y;
        let z1 = -x * sin_y + z * cos_y;
        let y1 = y * cos_x - z1 * sin_x;
        let z2 = y * sin_x + z1 * cos_x;
        (cx + x1 * scale_x, cy - y1 * scale_y, z2)
    };

    let proj: Vec<(f64, f64, f64)> = verts.iter().map(|&(x, y, z)| project(x, y, z)).collect();

    let faces: [([usize; 4], Face); 6] = [
        ([4, 5, 6, 7], Face::Front),  ([0, 3, 2, 1], Face::Back),
        ([0, 4, 7, 3], Face::Left),   ([1, 2, 6, 5], Face::Right),
        ([3, 7, 6, 2], Face::Top),    ([0, 1, 5, 4], Face::Bottom),
    ];

    let mut sorted_faces: Vec<([usize; 4], Face)> = faces.to_vec();
    sorted_faces.sort_by(|a, b| {
        let za: f64 = a.0.iter().map(|&i| proj[i].2).sum::<f64>() / 4.0;
        let zb: f64 = b.0.iter().map(|&i| proj[i].2).sum::<f64>() / 4.0;
        za.partial_cmp(&zb).unwrap()
    });

    for (indices, face) in &sorted_faces {
        let color = face_to_color(*face);
        let pts: Vec<(f64, f64)> = indices.iter().map(|&i| (proj[i].0, proj[i].1)).collect();

        let min_x = pts.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let max_x = pts.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        let min_y = pts.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let max_y = pts.iter().map(|p| p.1).fold(f64::MIN, f64::max);

        for py in (min_y as u16)..=(max_y as u16) {
            for px in (min_x as u16)..=(max_x as u16) {
                if px < content_area.x || px >= content_area.x + content_area.width { continue; }
                if py < content_area.y || py >= content_area.y + content_area.height { continue; }
                if point_in_quad(px as f64, py as f64, &pts) {
                    let style = if *face == app.current_face {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };
                    let ch = if *face == app.current_face { '●' } else { '░' };
                    f.render_widget(Paragraph::new(ch.to_string()).style(style), Rect::new(px, py, 1, 1));
                }
            }
        }

        for i in 0..4 {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % 4];
            draw_line(f, x1, y1, x2, y2, color, content_area);
        }
    }

    // Orbiting sphere
    let face_center_3d = match app.current_face {
        Face::Front => (0.0, 0.0, 1.8),   Face::Back => (0.0, 0.0, -1.8),
        Face::Left => (-1.8, 0.0, 0.0),    Face::Right => (1.8, 0.0, 0.0),
        Face::Top => (0.0, 1.8, 0.0),      Face::Bottom => (0.0, -1.8, 0.0),
    };
    let (sx, sy, _) = project(face_center_3d.0, face_center_3d.1, face_center_3d.2);
    let sphere_color = face_to_color(app.current_face);
    let sx_u = sx as u16;
    let sy_u = sy as u16;
    if sx_u >= content_area.x && sx_u < content_area.x + content_area.width
        && sy_u >= content_area.y && sy_u < content_area.y + content_area.height
    {
        f.render_widget(
            Paragraph::new("◉").style(Style::default().fg(sphere_color).add_modifier(Modifier::BOLD)),
            Rect::new(sx_u, sy_u, 1, 1),
        );
    }

    // Face label
    let lang = Lang::from_code(&app.settings.language);
    let label = face_name(app.current_face, lang);
    let label_w = label.chars().count() as u16;
    let label_x = layout.cube_outer_frame.x + layout.cube_outer_frame.width.saturating_sub(label_w) / 2;
    let label_y = layout.cube_outer_frame.y + layout.cube_outer_frame.height.saturating_sub(1);
    if label_y > layout.cube_outer_frame.y {
        let max_x = layout.cube_outer_frame.x + layout.cube_outer_frame.width;
        let w = label_w.min(max_x.saturating_sub(label_x));
        let start_offset = label_x.saturating_sub(layout.cube_outer_frame.x) as usize;
        let clipped: String = label.chars().skip(start_offset).take(w as usize).collect();
        if w > 0 && !clipped.is_empty() {
            f.render_widget(
                Paragraph::new(clipped).style(Style::default().fg(face_to_color(app.current_face)).bg(bg)),
                Rect::new(label_x.max(layout.cube_outer_frame.x), label_y, w, 1),
            );
        }
    }
}

fn point_in_quad(px: f64, py: f64, pts: &[(f64, f64)]) -> bool {
    if pts.len() != 4 { return false; }
    let mut inside = true;
    for i in 0..4 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % 4];
        let cross = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if cross > 0.0 { inside = false; break; }
    }
    if inside { return true; }
    inside = true;
    for i in 0..4 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % 4];
        let cross = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if cross < 0.0 { inside = false; break; }
    }
    inside
}

fn draw_line(f: &mut Frame, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, bounds: Rect) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let steps = dx.max(dy).ceil() as u16;
    if steps == 0 { return; }
    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = (x1 + (x2 - x1) * t) as u16;
        let y = (y1 + (y2 - y1) * t) as u16;
        if x >= bounds.x && x < bounds.x + bounds.width && y >= bounds.y && y < bounds.y + bounds.height {
            f.render_widget(Paragraph::new("·").style(Style::default().fg(color)), Rect::new(x, y, 1, 1));
        }
    }
}
