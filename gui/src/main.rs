use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future::{block_on, poll_once};
use rand::SeedableRng;
use rand::rngs::StdRng;

mod app_state;
mod input;
mod render;
mod save;
mod theme;
mod ui;

use app_state::AppState;
use input::actions::{handle_game_completion, handle_keyboard_input};
use input::picker::pick_cell;
use render::camera::{spawn_camera, update_camera};
use render::cell_visual::update_cell_visuals;
use render::cube_mesh::{FaceMaterial, spawn_cube};
use render::digit_cubes::{
    DigitTextures, load_digit_textures, spawn_digit_cubes, update_digit_cubes,
};
use save::db::save_game;
use sudokube_core::cube::{CubeGrid, Difficulty};
use sudokube_core::game_state::GameState;
use sudokube_core::puzzle::generate_puzzle;
use sudokube_core::theme::Theme;
use sudokube_core::wfc::WfcGenerator;
use theme::{ThemeBackground, ThemeBorder, ThemeColors, ThemeText};
use ui::history_panel::{
    handle_history_buttons, spawn_history_panel, toggle_history, update_history_visibility,
};
use ui::hud::{GameFont, handle_hud_buttons, spawn_hud, update_timer};
use ui::loading::{despawn_loading_ui, spawn_loading_ui, update_loading_ui};

#[derive(Component)]
pub struct GameWorld;

#[derive(Resource)]
pub struct GenerationTask(
    pub  Task<
        Option<(
            CubeGrid,
            std::collections::HashMap<sudokube_core::cube::CubeCoord, u8>,
        )>,
    >,
);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(Theme::Dark)
        .insert_resource(ThemeColors::for_theme(Theme::Dark))
        .insert_resource(ClearColor(ThemeColors::for_theme(Theme::Dark).background))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::Loading), (start_generation, spawn_loading_ui).chain())

        .add_systems(
            Update,
            (poll_generation, update_loading_ui).run_if(in_state(AppState::Loading)),
        )
        .add_systems(OnExit(AppState::Loading), despawn_loading_ui)
        .add_systems(
            OnEnter(AppState::InGame),
            (
                load_digit_textures,
                spawn_cube,
                spawn_digit_cubes,
                spawn_hud,
                spawn_history_panel,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (apply_theme_update,)
                .run_if(in_state(AppState::InGame).or(in_state(AppState::History))),
        )
        .add_systems(Update, (update_camera,).run_if(in_state(AppState::InGame)))
        .add_systems(Update, (pick_cell,).run_if(in_state(AppState::InGame)))
        .add_systems(
            Update,
            (handle_keyboard_input,).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (handle_hud_buttons,).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (update_cell_visuals,).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (update_digit_cubes,).run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, (update_timer,).run_if(in_state(AppState::InGame)))
        .add_systems(
            Update,
            (handle_game_completion,).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                toggle_history,
                update_history_visibility,
                handle_history_buttons,
            )
                .run_if(in_state(AppState::InGame).or(in_state(AppState::History))),
        )
        .add_systems(OnExit(AppState::InGame), despawn_game_world)
        .run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    spawn_camera(commands.reborrow());
    let font_handle: Handle<Font> = asset_server.load("fonts/FiraSans-Regular.ttf");
    commands.insert_resource(GameFont(font_handle));
    bevy::log::info!("setup complete, transitioning to Loading");
    next_state.set(AppState::Loading);
}

fn start_generation(mut commands: Commands, existing: Option<Res<GameState>>) {
    bevy::log::info!("start_generation called");
    if let Some(old) = existing {
        let _ = save_game(&*old);
    }

    let pool = AsyncComputeTaskPool::get();
    let task = pool.spawn(async move {
        let mut rng = StdRng::from_entropy();
        let mut generator = WfcGenerator::new();
        let solution = generator.generate(&mut rng)?;
        let grid = generate_puzzle(&solution, Difficulty::Medium, &mut rng);
        Some((grid, solution))
    });
    commands.insert_resource(GenerationTask(task));
    bevy::log::info!("GenerationTask inserted");
}

fn poll_generation(
    mut commands: Commands,
    task: Option<ResMut<GenerationTask>>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    bevy::log::info!("poll_generation called");
    let Some(mut task) = task else {
        bevy::log::warn!("GenerationTask not found, skipping poll");
        return;
    };
    if let Some(result) = block_on(poll_once(&mut task.0)) {
        bevy::log::info!("generation task finished");
        commands.remove_resource::<GenerationTask>();
        if let Some((grid, solution)) = result {
            let mut state = GameState::new(grid, solution, Difficulty::Medium);
            state.started_at = time.elapsed_secs_f64();
            commands.insert_resource(state);
            next_state.set(AppState::InGame);
        } else {
            bevy::log::warn!("generation failed, retrying");
            next_state.set(AppState::Loading);
        }
    }
}

fn despawn_game_world(mut commands: Commands, query: Query<Entity, With<GameWorld>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<DigitTextures>();
}

fn apply_theme_update(
    theme: Res<Theme>,
    mut colors: ResMut<ThemeColors>,
    mut clear_color: ResMut<ClearColor>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    face_material: Res<FaceMaterial>,
    mut bg_query: Query<(&ThemeBackground, &mut BackgroundColor)>,
    mut border_query: Query<(&ThemeBorder, &mut BorderColor)>,
    mut text_query: Query<(&ThemeText, &mut TextColor)>,
) {
    if theme.is_changed() {
        let new = ThemeColors::for_theme(*theme);
        *colors = new.clone();
        clear_color.0 = new.background;

        if let Some(mat) = materials.get_mut(&face_material.0) {
            mat.base_color = new.face_background;
        }

        for (key, mut bg) in bg_query.iter_mut() {
            bg.0 = new.color(key.0);
        }
        for (key, mut border) in border_query.iter_mut() {
            *border = BorderColor::all(new.color(key.0));
        }
        for (key, mut text) in text_query.iter_mut() {
            text.0 = new.color(key.0);
        }
    }
}
