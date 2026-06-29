//! 游戏工具函数：创建新游戏、继续游戏、时间计算等

use std::collections::HashSet;

use rand::SeedableRng;
use rand::rngs::StdRng;

use sudokube_core::cube::{CubeCoord, Difficulty, Face};
use sudokube_core::game_state::GameState;
use sudokube_core::puzzle::generate_puzzle;
use sudokube_core::wfc::WfcGenerator;

use crate::app::App;
use crate::save::GameRecord;

pub fn new_game(difficulty: Difficulty) -> GameState {
    let mut rng = StdRng::from_entropy();
    let mut generator = WfcGenerator::new();
    let solution = generator.generate(&mut rng).expect("生成题解失败");
    let grid = generate_puzzle(&solution, difficulty, &mut rng);
    let mut game = GameState::new(grid, solution, difficulty);
    game.started_at = now_secs();
    game.selected = Some(Face::Front.to_cube(4, 4));
    game
}

pub fn continue_game(record: &GameRecord) -> GameState {
    let difficulty = match record.difficulty.as_str() {
        "简单" => Difficulty::Easy,
        "困难" => Difficulty::Hard,
        _ => Difficulty::Medium,
    };
    let given: HashSet<CubeCoord> = record.given.keys().copied().collect();
    let mut game = GameState::new(
        sudokube_core::cube::CubeGrid::from_solution(record.answer.clone(), &given),
        record.answer.clone(),
        difficulty,
    );
    for (coord, value) in &record.puzzle {
        if !given.contains(coord) {
            if let Some(cell) = game.grid.get_mut(coord) {
                cell.user_value = Some(*value);
            }
        }
    }
    // 恢复草稿数据
    {
        let coords: Vec<CubeCoord> = sudokube_core::cube::iter_surface_coords().collect();
        crate::save::apply_draft_to_grid(&mut game.grid, &record.draft, &coords);
    }
    game.id = Some(record.id);
    game.elapsed_seconds = record.elapsed_seconds as u64;
    game.started_at = now_secs();
    game.selected = Some(Face::Front.to_cube(4, 4));
    game.errors = record.errors;
    game.errors_max = record.errors_max;
    game.frozen = record.frozen;
    if game.frozen {
        game.started_at = 0.0; // 冻结状态不计时
    }
    game
}

pub fn now_secs() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub fn total_elapsed(app: &App) -> u64 {
    let session = if app.game.started_at > 0.0 {
        (now_secs() - app.game.started_at) as u64
    } else {
        0
    };
    app.game.elapsed_seconds + session
}

pub fn flush_elapsed(app: &mut App) {
    let session = if app.game.started_at > 0.0 {
        (now_secs() - app.game.started_at) as u64
    } else {
        0
    };
    app.game.elapsed_seconds += session;
    app.game.started_at = now_secs();
}

pub fn current_coord(app: &App) -> CubeCoord {
    app.current_face.to_cube(app.cursor.0, app.cursor.1)
}
