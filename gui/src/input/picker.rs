use crate::render::digit_cubes::DigitCube;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::window::Window;
use sudokube_core::cube::{CubeCoord, Face};
use sudokube_core::game_state::GameState;

#[derive(Component)]
pub struct PickerCamera;

pub fn pick_cell(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<PickerCamera>>,
    cubes: Query<(&DigitCube, &GlobalTransform, &Visibility)>,
    mut game_state: ResMut<GameState>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor) else {
        return;
    };

    // 优先检测可见的数字小方块。
    let mut best_cube: Option<(f32, CubeCoord)> = None;
    for (cube, gt, visibility) in cubes.iter() {
        if *visibility != Visibility::Visible {
            continue;
        }
        let center = gt.translation();
        if let Some(t) = ray_sphere_intersection(&ray, center, 0.22) {
            if best_cube.map_or(true, |(best_t, _)| t < best_t) {
                best_cube = Some((t, cube.coord));
            }
        }
    }

    let picked = if let Some((_, coord)) = best_cube {
        Some(coord)
    } else {
        intersect_cube_plane(&ray)
    };

    if let Some(coord) = picked {
        if game_state.selected == Some(coord) {
            game_state.selected = None;
            game_state.highlight_number = None;
        } else {
            game_state.selected = Some(coord);
            // 只根据场上已显示的数字进行高亮，防止泄露隐藏答案。
            game_state.highlight_number = game_state.grid.get(&coord).and_then(|c| c.user_value);
        }
    }
}

fn ray_sphere_intersection(ray: &Ray3d, center: Vec3, radius: f32) -> Option<f32> {
    let dir = *ray.direction;
    let oc = ray.origin - center;
    let a = dir.dot(dir);
    let b = 2.0 * oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return None;
    }
    let t = (-b - discriminant.sqrt()) / (2.0 * a);
    if t >= 0.0 {
        Some(t)
    } else {
        let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
        if t2 >= 0.0 { Some(t2) } else { None }
    }
}

fn intersect_cube_plane(ray: &Ray3d) -> Option<CubeCoord> {
    let mut best: Option<(f32, CubeCoord)> = None;

    for face in Face::ALL.iter() {
        let (plane_pos, rot) = face_plane(face);
        let plane_normal = rot * Vec3::Z;
        let denom = ray.direction.dot(plane_normal);
        if denom.abs() < 1e-5 {
            continue;
        }
        let t = (plane_pos - ray.origin).dot(plane_normal) / denom;
        if t < 0.0 {
            continue;
        }
        let hit = ray.origin + ray.direction * t;
        let local = rot.inverse() * (hit - plane_pos);
        let u_f = local.x + 4.0;
        let v_f = local.y + 4.0;
        if u_f < -0.5 || u_f > 8.5 || v_f < -0.5 || v_f > 8.5 {
            continue;
        }
        let u = ((u_f + 0.5).floor() as u8).min(8);
        let v = ((v_f + 0.5).floor() as u8).min(8);
        let coord = face.to_cube(u, v);

        if best.map_or(true, |(best_t, _)| t < best_t) {
            best = Some((t, coord));
        }
    }

    best.map(|(_, coord)| coord)
}

fn face_plane(face: &Face) -> (Vec3, Quat) {
    let distance = 4.5;
    match face {
        Face::Right => (
            Vec3::X * distance,
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
        Face::Left => (
            Vec3::NEG_X * distance,
            Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
        ),
        Face::Top => (
            Vec3::Y * distance,
            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        ),
        Face::Bottom => (
            Vec3::NEG_Y * distance,
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
        Face::Front => (Vec3::Z * distance, Quat::IDENTITY),
        Face::Back => (
            Vec3::NEG_Z * distance,
            Quat::from_rotation_y(std::f32::consts::PI),
        ),
    }
}
