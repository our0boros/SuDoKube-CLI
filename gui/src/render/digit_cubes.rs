use crate::GameWorld;
use crate::render::cube_mesh::cell_transform;
use crate::theme::ThemeColors;
use ab_glyph::{Font, FontArc, Glyph, PxScale, point};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use sudokube_core::cube::{CubeCoord, iter_surface_coords};
use sudokube_core::game_state::GameState;

/// 1~9 的数字纹理资源。
#[derive(Resource)]
pub struct DigitTextures(pub Vec<Handle<Image>>);

/// 标记一个 3D 数字小方块实体。
#[derive(Component)]
pub struct DigitCube {
    pub coord: CubeCoord,
}

/// 使用 assets/fonts/FiraSans-Regular.ttf 生成 9 张数字纹理。
pub fn load_digit_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let font_data = std::fs::read("assets/fonts/FiraSans-Regular.ttf")
        .expect("failed to read assets/fonts/FiraSans-Regular.ttf");
    let font = FontArc::try_from_vec(font_data).expect("invalid font data");

    let size = 128u32;
    let mut textures = Vec::with_capacity(9);
    for value in 1..=9u8 {
        textures.push(images.add(render_digit_texture(&font, value, size, size)));
    }
    commands.insert_resource(DigitTextures(textures));
}

fn render_digit_texture(font: &FontArc, value: u8, width: u32, height: u32) -> Image {
    let scale = PxScale::from(height as f32 * 0.72);
    let glyph_id = font.glyph_id(value.to_string().chars().next().unwrap());
    let glyph = Glyph {
        id: glyph_id,
        scale,
        position: point(0.0, 0.0),
    };
    let outline = font
        .outline_glyph(glyph)
        .expect("digit glyph not found in font");
    let bounds = outline.px_bounds();

    let offset_x = ((width as f32 - bounds.width()) / 2.0 - bounds.min.x).round() as i32;
    let offset_y = ((height as f32 - bounds.height()) / 2.0 - bounds.min.y).round() as i32;

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    outline.draw(|x, y, c| {
        let px = x as i32 + offset_x;
        let py = y as i32 + offset_y;
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            let idx = ((py as u32 * width + px as u32) * 4) as usize;
            let alpha = (c * 255.0) as u8;
            pixels[idx] = 255;
            pixels[idx + 1] = 255;
            pixels[idx + 2] = 255;
            pixels[idx + 3] = alpha;
        }
    });

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// 为所有格子创建数字小方块池（初始隐藏）。
pub fn spawn_digit_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(0.32, 0.32, 0.08));
    let placeholder = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    for coord in iter_surface_coords() {
        let (pos, rot) = cell_transform(&coord);
        let forward = rot * Vec3::Z;
        let cube_pos = pos + forward * 0.12;

        commands.spawn((
            GameWorld,
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(placeholder.clone()),
            Transform::from_translation(cube_pos).with_rotation(rot),
            DigitCube { coord },
            Visibility::Hidden,
        ));
    }
}

/// 根据当前棋盘状态更新每个小方块的可见性、纹理与颜色。
pub fn update_digit_cubes(
    mut cubes: Query<(
        &DigitCube,
        &mut Visibility,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    textures: Res<DigitTextures>,
    theme: Res<ThemeColors>,
    game_state: Res<GameState>,
) {
    for (cube, mut visibility, mat_handle) in cubes.iter_mut() {
        let Some(mat) = materials.get_mut(&*mat_handle) else {
            continue;
        };
        let Some(cell) = game_state.grid.get(&cube.coord) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        if let Some(value) = cell.user_value {
            let idx = (value - 1) as usize;
            mat.base_color_texture = Some(textures.0[idx].clone());
            mat.base_color = if cell.given {
                theme.text_given
            } else {
                theme.text_user
            };
            mat.alpha_mode = AlphaMode::Blend;
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
