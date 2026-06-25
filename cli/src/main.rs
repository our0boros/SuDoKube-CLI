mod input;
mod render;
mod save;

use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    ExecutableCommand,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::io;
use std::time::{Duration, Instant};
use sudokube_core::cube::{CubeCoord, Difficulty, Face};
use sudokube_core::game_state::GameState;
use sudokube_core::puzzle::generate_puzzle;
use sudokube_core::wfc::WfcGenerator;

use input::{handle_event, EventResult};
use render::{ButtonId, RenderMode, Theme};
use save::{GameRecord, load_unfinished, save_game};

/// 当前处于启动菜单还是游戏内。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Menu,
    Game,
}

/// 启动菜单选项。
#[derive(Debug, Clone)]
pub enum MenuItem {
    NewGame(Difficulty),
    Continue(GameRecord),
}

pub struct MenuState {
    pub items: Vec<MenuItem>,
    pub selected: usize,
}

impl MenuState {
    pub fn new() -> Self {
        let mut items = vec![
            MenuItem::NewGame(Difficulty::Easy),
            MenuItem::NewGame(Difficulty::Medium),
            MenuItem::NewGame(Difficulty::Hard),
        ];
        let unfinished = load_unfinished(20).unwrap_or_default();
        for record in unfinished {
            items.push(MenuItem::Continue(record));
        }
        Self { items, selected: 0 }
    }
}

pub struct CliState {
    pub screen: AppScreen,
    pub menu: MenuState,
    pub game: GameState,
    pub current_face: Face,
    pub cursor: (u8, u8),
    pub render_mode: RenderMode,
    pub theme: Theme,
    pub guidance: bool,
    pub last_blink: Instant,
    pub blink_on: bool,
    pub message: String,
    pub message_until: Option<Instant>,
    pub hover_button: Option<ButtonId>,
    // 懒刷新快照。
    pub prev_screen: AppScreen,
    pub prev_cursor: (u8, u8),
    pub prev_face: Face,
    pub prev_blink_on: bool,
    pub prev_render_mode: RenderMode,
    pub prev_theme: Theme,
    pub prev_timer_text: String,
    pub prev_message: String,
    pub prev_grid_hash: u64,
    pub prev_term_size: (u16, u16),
    pub dirty: bool,
}

impl CliState {
    pub fn start_game(game: GameState) -> Self {
        Self {
            screen: AppScreen::Game,
            menu: MenuState::new(),
            game,
            current_face: Face::Front,
            cursor: (4, 4),
            render_mode: RenderMode::Monospace,
            theme: Theme::Dark,
            guidance: true,
            last_blink: Instant::now(),
            blink_on: true,
            message: String::new(),
            message_until: None,
            hover_button: None,
            prev_screen: AppScreen::Menu,
            prev_cursor: (4, 4),
            prev_face: Face::Front,
            prev_blink_on: true,
            prev_render_mode: RenderMode::Monospace,
            prev_theme: Theme::Dark,
            prev_timer_text: String::new(),
            prev_message: String::new(),
            prev_grid_hash: 0,
            prev_term_size: (0, 0),
            dirty: true,
        }
    }

    pub fn set_message(&mut self, text: impl Into<String>, duration: Duration) {
        self.message = text.into();
        self.message_until = Some(Instant::now() + duration);
        self.dirty = true;
    }

    pub fn clear_message_if_expired(&mut self) {
        if let Some(until) = self.message_until {
            if Instant::now() >= until {
                self.message.clear();
                self.message_until = None;
                self.dirty = true;
            }
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    stdout.execute(EnableMouseCapture)?;

    let result = run_app(&mut stdout);

    let _ = stdout.execute(DisableMouseCapture);
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}

fn run_app(stdout: &mut io::Stdout) -> io::Result<()> {
    let menu = MenuState::new();
    let mut state = CliState {
        screen: AppScreen::Menu,
        menu,
        game: GameState::new(
            sudokube_core::cube::CubeGrid { cells: Default::default() },
            Default::default(),
            Difficulty::Medium,
        ),
        current_face: Face::Front,
        cursor: (4, 4),
        render_mode: RenderMode::Monospace,
        theme: Theme::Dark,
        guidance: true,
        last_blink: Instant::now(),
        blink_on: true,
        message: String::new(),
        message_until: None,
        hover_button: None,
        prev_screen: AppScreen::Game,
        prev_cursor: (4, 4),
        prev_face: Face::Front,
        prev_blink_on: true,
        prev_render_mode: RenderMode::Monospace,
        prev_theme: Theme::Dark,
        prev_timer_text: String::new(),
        prev_message: String::new(),
        prev_grid_hash: 0,
        prev_term_size: (0, 0),
        dirty: true,
    };

    loop {
        let now = Instant::now();
        if now.duration_since(state.last_blink) >= Duration::from_millis(500) {
            state.blink_on = !state.blink_on;
            state.last_blink = now;
            state.dirty = true;
        }
        state.clear_message_if_expired();

        render::render(stdout, &mut state)?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            let event = crossterm::event::read()?;
            match handle_event(&mut state, event) {
                EventResult::Continue => {}
                EventResult::StartGame(game) => {
                    state = CliState::start_game(game);
                }
                EventResult::BackToMenu => {
                    if state.screen == AppScreen::Game {
                        flush_elapsed(&mut state);
                        let _ = save_game(&state.game);
                    }
                    state.screen = AppScreen::Menu;
                    state.menu = MenuState::new();
                    state.dirty = true;
                }
                EventResult::Quit => {
                    if state.screen == AppScreen::Game {
                        flush_elapsed(&mut state);
                        let _ = save_game(&state.game);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

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
    let given: std::collections::HashSet<CubeCoord> = record.given.keys().copied().collect();
    let mut game = GameState::new(
        sudokube_core::cube::CubeGrid::from_solution(record.answer.clone(), &given),
        record.answer.clone(),
        difficulty,
    );
    // 恢复用户已填写的进度（非给定格）。
    for (coord, value) in &record.puzzle {
        if !given.contains(coord) {
            if let Some(cell) = game.grid.get_mut(coord) {
                cell.user_value = Some(*value);
            }
        }
    }
    game.id = Some(record.id);
    game.elapsed_seconds = record.elapsed_seconds as u64;
    game.started_at = now_secs();
    game.selected = Some(Face::Front.to_cube(4, 4));
    game
}

pub fn now_secs() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// 计算当前对局的总用时（包括之前保存的累计时间）。
pub fn total_elapsed(state: &CliState) -> u64 {
    let session = if state.game.started_at > 0.0 {
        (now_secs() - state.game.started_at) as u64
    } else {
        0
    };
    state.game.elapsed_seconds + session
}

/// 将当前会话用时刷入 game.elapsed_seconds 并重置 started_at。
pub fn flush_elapsed(state: &mut CliState) {
    let session = if state.game.started_at > 0.0 {
        (now_secs() - state.game.started_at) as u64
    } else {
        0
    };
    state.game.elapsed_seconds += session;
    state.game.started_at = now_secs();
}

pub fn current_coord(state: &CliState) -> CubeCoord {
    state.current_face.to_cube(state.cursor.0, state.cursor.1)
}
