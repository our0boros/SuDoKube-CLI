use crate::render::cube_mesh::CubeCell;
use crate::theme::ThemeColors;
use bevy::prelude::*;
use std::collections::HashSet;
use sudokube_core::cube::{CubeCoord, CubeGrid};
use sudokube_core::game_state::GameState;

/// 根据当前选中状态、高亮数字与主题更新所有格子的视觉。
pub fn update_cell_visuals(
    mut cells: Query<(&CubeCell, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    theme: Res<ThemeColors>,
    game_state: Res<GameState>,
) {
    if !theme.is_changed() && !game_state.is_changed() {
        return;
    }

    let selected = game_state.selected;
    let highlight = game_state.highlight_number;

    let mut related_set = HashSet::<CubeCoord>::new();
    let mut extended_set = HashSet::<CubeCoord>::new();
    if let Some(coord) = selected {
        for c in coord.related() {
            related_set.insert(c);
        }
        for c in coord.extended_related() {
            extended_set.insert(c);
        }
    }

    for (cell, mat_handle) in cells.iter_mut() {
        let color = compute_cell_color(
            cell.coord,
            selected,
            highlight,
            &related_set,
            &extended_set,
            &game_state.grid,
            &theme,
        );
        if let Some(material) = materials.get_mut(&*mat_handle) {
            material.base_color = color;
        }
    }
}

fn compute_cell_color(
    coord: CubeCoord,
    selected: Option<CubeCoord>,
    highlight: Option<u8>,
    related_set: &HashSet<CubeCoord>,
    extended_set: &HashSet<CubeCoord>,
    grid: &CubeGrid,
    theme: &ThemeColors,
) -> Color {
    if selected == Some(coord) {
        return theme.cell_selected;
    }

    if let Some(cell) = grid.get(&coord) {
        if let Some(value) = cell.user_value {
            // 仅根据场上可见数字判断是否冲突，不依赖答案。
            if !cell.given && is_value_conflicting(grid, coord, value) {
                return theme.cell_error;
            }
            if highlight == Some(value) {
                return theme.cell_highlight;
            }
        }
    }

    if related_set.contains(&coord) {
        return theme.cell_related;
    }

    if extended_set.contains(&coord) {
        return mix_colors(theme.cell_related, theme.cell_default, 0.5);
    }

    cell_base_color_static(&coord, theme)
}

/// 检查指定格子的 user_value 是否在其相关约束区域内重复。
fn is_value_conflicting(grid: &CubeGrid, coord: CubeCoord, value: u8) -> bool {
    for other in coord.related() {
        if other == coord {
            continue;
        }
        if let Some(cell) = grid.get(&other) {
            if cell.user_value == Some(value) {
                return true;
            }
        }
    }
    false
}

fn cell_base_color_static(coord: &CubeCoord, theme: &ThemeColors) -> Color {
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

fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let a_srgba = a.to_srgba();
    let b_srgba = b.to_srgba();
    Color::srgba(
        a_srgba.red * (1.0 - t) + b_srgba.red * t,
        a_srgba.green * (1.0 - t) + b_srgba.green * t,
        a_srgba.blue * (1.0 - t) + b_srgba.blue * t,
        a_srgba.alpha * (1.0 - t) + b_srgba.alpha * t,
    )
}
