
use sudokube_core::cube::Face;

use crate::i18n::{self, Lang};
use crate::App;

use std::time::Duration;


pub(super) fn face_label(face: Face, _lang: Lang) -> &'static str {
    match face {
        Face::Front => "F",
        Face::Back => "B",
        Face::Left => "L",
        Face::Right => "R",
        Face::Top => "T",
        Face::Bottom => "U",
    }
}

pub(super) fn cycle_face_vertical(face: Face, forward: bool) -> Face {
    let ring = [Face::Front, Face::Top, Face::Back, Face::Bottom];
    cycle_in_ring(&ring, face, forward)
}

pub(super) fn cycle_face_horizontal(face: Face, forward: bool) -> Face {
    let ring = [Face::Front, Face::Left, Face::Back, Face::Right];
    cycle_in_ring(&ring, face, forward)
}

pub(super) fn cycle_in_ring(ring: &[Face], face: Face, forward: bool) -> Face {
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

pub(super) fn move_cursor_with_wrap(app: &mut App, dx: i8, dy: i8) {
    let (face, cursor) = move_on_surface(app.current_face, app.cursor, dx, dy);
    if face != app.current_face || cursor != app.cursor {
        app.current_face = face;
        app.cursor = cursor;
    }
}

pub fn move_on_surface(face: Face, cursor: (u8, u8), dx: i8, dy: i8) -> (Face, (u8, u8)) {
    let (u, v) = (cursor.0 as i8, cursor.1 as i8);
    let nu = u + dx;
    let nv = v + dy;

    if (0..9).contains(&nu) && (0..9).contains(&nv) {
        return (face, (nu as u8, nv as u8));
    }

    match face {
        Face::Front => {
            if nv < 0 {
                (Face::Bottom, (8, u as u8))
            } else if nv > 8 {
                (Face::Top, (u as u8, 8))
            } else if nu < 0 {
                (Face::Left, (8, v as u8))
            } else {
                (Face::Right, (v as u8, 8))
            }
        }
        Face::Back => {
            if nv < 0 {
                (Face::Left, (0, u as u8))
            } else if nv > 8 {
                (Face::Right, (u as u8, 0))
            } else if nu < 0 {
                (Face::Bottom, (v as u8, 0))
            } else {
                (Face::Top, (v as u8, 0))
            }
        }
        Face::Top => {
            if nv < 0 {
                (Face::Back, (u as u8, 8))
            } else if nv > 8 {
                (Face::Front, (u as u8, 8))
            } else if nu < 0 {
                (Face::Left, (v as u8, 8))
            } else {
                (Face::Right, (8, v as u8))
            }
        }
        Face::Bottom => {
            if nv < 0 {
                (Face::Left, (u as u8, 0))
            } else if nv > 8 {
                (Face::Right, (0, u as u8))
            } else if nu < 0 {
                (Face::Back, (v as u8, 0))
            } else {
                (Face::Front, (v as u8, 0))
            }
        }
        Face::Left => {
            if nv < 0 {
                (Face::Bottom, (u as u8, 0))
            } else if nv > 8 {
                (Face::Top, (0, u as u8))
            } else if nu < 0 {
                (Face::Back, (0, v as u8))
            } else {
                (Face::Front, (0, v as u8))
            }
        }
        Face::Right => {
            if nv < 0 {
                (Face::Back, (8, u as u8))
            } else if nv > 8 {
                (Face::Front, (8, u as u8))
            } else if nu < 0 {
                (Face::Bottom, (v as u8, 8))
            } else {
                (Face::Top, (8, v as u8))
            }
        }
    }
}

pub(super) fn switch_face(face: Face, dx: i8, dy: i8) -> Face {
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

/// Debug: Fill all blank cells on the current face with solution values
pub(super) fn debug_hint_face(app: &mut App) {
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
    if app.game.check_completion() {
        app.trigger_victory();
        return;
    }
    let lang = Lang::from_code(&app.settings.language);
    app.set_message(
        format!("{} {} cells", i18n::t("debug.hint", lang), filled),
        Duration::from_secs(2),
    );
}

// ── 3D 方向转换：确保蛇跨面时方向不混乱 ──

/// 每个面 u 方向在3D空间中对应的轴 (dim: 0=x,1=y,2=z) 和符号
fn face_u_axis(face: Face) -> (u8, i8) {
    match face {
        Face::Front  => (0,  1), // +X
        Face::Back   => (1,  1), // +Y
        Face::Right  => (1,  1), // +Y
        Face::Left   => (2,  1), // +Z
        Face::Top    => (0,  1), // +X
        Face::Bottom => (2,  1), // +Z
    }
}

/// 每个面 v 方向在3D空间中对应的轴和符号
fn face_v_axis(face: Face) -> (u8, i8) {
    match face {
        Face::Front  => (1,  1), // +Y
        Face::Back   => (0,  1), // +X
        Face::Right  => (2,  1), // +Z
        Face::Left   => (1,  1), // +Y
        Face::Top    => (2,  1), // +Z
        Face::Bottom => (0,  1), // +X
    }
}

/// 将面局部方向 (du,dv) 转换为3D空间方向 (dx,dy,dz)
pub fn face_dir_to_3d(face: Face, du: i8, dv: i8) -> (i8, i8, i8) {
    let (ud, us) = face_u_axis(face);
    let (vd, vs) = face_v_axis(face);
    let mut d = [0i8; 3];
    d[ud as usize] += us * du;
    d[vd as usize] += vs * dv;
    (d[0], d[1], d[2])
}

/// 将3D空间方向 (dx,dy,dz) 转换为面局部方向 (du,dv)
pub fn dir_3d_to_face(face: Face, dx: i8, dy: i8, dz: i8) -> (i8, i8) {
    let d = [dx, dy, dz];
    let (ud, us) = face_u_axis(face);
    let (vd, vs) = face_v_axis(face);
    let du = d[ud as usize] * us;
    let dv = d[vd as usize] * vs;
    (du, dv)
}

/// 蛇跨面时方向转换：保持3D空间方向不变
pub fn convert_face_dir(old_face: Face, du: i8, dv: i8, new_face: Face) -> (i8, i8) {
    let (dx, dy, dz) = face_dir_to_3d(old_face, du, dv);
    dir_3d_to_face(new_face, dx, dy, dz)
}

// ── 键位映射编辑事件处理 ──

