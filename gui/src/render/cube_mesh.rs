use crate::GameWorld;
use crate::theme::ThemeColors;
use bevy::prelude::*;
use sudokube_core::cube::{CubeCoord, Face, iter_surface_coords};

/// 6 个面背景共享的材质句柄，用于主题切换时更新颜色。
#[derive(Resource)]
pub struct FaceMaterial(pub Handle<StandardMaterial>);

/// 标记一个渲染用格子实体。
#[derive(Component)]
pub struct CubeCell {
    pub coord: CubeCoord,
}

/// 生成完整的 3D 立方体表面：6 个面背景 + 386 个格子。
pub fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Res<ThemeColors>,
) {
    // 共享几何体。格子使用薄立方体，使其看起来是贴在立方体表面的方块。
    let cell_mesh: Handle<Mesh> = meshes.add(Cuboid::new(0.95, 0.95, 0.15));
    let face_mesh: Handle<Mesh> = meshes.add(Plane3d::new(Vec3::Z, Vec2::new(4.5, 4.5)).mesh());
    let face_material = materials.add(StandardMaterial {
        base_color: theme.face_background,
        ..default()
    });
    commands.insert_resource(FaceMaterial(face_material.clone()));

    // 6 个面背景平面，放在立方体表面略靠内，用于填充格子之间的缝隙。
    for face in Face::ALL.iter() {
        let (pos, rot) = face_transform(face, 4.55);
        commands.spawn((
            GameWorld,
            Mesh3d(face_mesh.clone()),
            MeshMaterial3d(face_material.clone()),
            Transform::from_translation(pos).with_rotation(rot),
        ));
    }

    // 386 个唯一表面坐标对应的立方体格子（6 个 9x9 面的 486 个面坐标去重后得到）。
    for coord in iter_surface_coords() {
        let (pos, rot) = cell_transform(&coord);
        let cell_color = cell_base_color(&coord, &theme);
        let material = materials.add(StandardMaterial {
            base_color: cell_color,
            ..default()
        });

        commands.spawn((
            GameWorld,
            Mesh3d(cell_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(pos).with_rotation(rot),
            CubeCell { coord },
            GlobalTransform::default(),
        ));
    }
}

fn face_transform(face: &Face, distance: f32) -> (Vec3, Quat) {
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

pub fn cell_transform(coord: &CubeCoord) -> (Vec3, Quat) {
    let face = if coord.x == 8 {
        Face::Right
    } else if coord.x == 0 {
        Face::Left
    } else if coord.y == 8 {
        Face::Top
    } else if coord.y == 0 {
        Face::Bottom
    } else if coord.z == 8 {
        Face::Front
    } else {
        Face::Back
    };

    let u = match face {
        Face::Right => coord.y,
        Face::Left => coord.z,
        Face::Top => coord.x,
        Face::Bottom => coord.z,
        Face::Front => coord.x,
        Face::Back => coord.y,
    };
    let v = match face {
        Face::Right => coord.z,
        Face::Left => coord.y,
        Face::Top => coord.z,
        Face::Bottom => coord.x,
        Face::Front => coord.y,
        Face::Back => coord.x,
    };

    let (base_pos, rot) = face_transform(&face, 4.60);
    let right = rot * Vec3::X;
    let up = rot * Vec3::Y;
    let local_u = u as f32 - 4.0;
    let local_v = v as f32 - 4.0;
    let pos = base_pos + right * local_u + up * local_v;
    (pos, rot)
}

fn cell_base_color(coord: &CubeCoord, theme: &ThemeColors) -> Color {
    let face_coords = coord.to_face_coords();
    let Some(fc) = face_coords.first() else {
        return theme.cell_default;
    };
    let bu = fc.u / 3;
    let bv = fc.v / 3;
    let parity = (bu + bv + fc.u + fc.v) % 2;
    if parity == 0 {
        theme.cell_default
    } else {
        theme.cell_alt
    }
}
