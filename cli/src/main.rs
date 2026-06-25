use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor,
        SetForegroundColor,
    },
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
    ExecutableCommand, QueueableCommand,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use sudokube_core::cube::{CubeCoord, Difficulty, Face};
use sudokube_core::game_state::GameState;
use sudokube_core::puzzle::generate_puzzle;
use sudokube_core::wfc::WfcGenerator;

const GRID_START_ROW: u16 = 2;
const MESSAGE_ROW: u16 = 41;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    /// 标准 ASCII 字符模式，横向更宽，兼容性最好。
    Standard,
    /// 等宽框线字符模式，与旧样式一致。
    Monospace,
}

impl RenderMode {
    fn toggle(self) -> Self {
        match self {
            RenderMode::Standard => RenderMode::Monospace,
            RenderMode::Monospace => RenderMode::Standard,
        }
    }

    fn metrics(self) -> Metrics {
        match self {
            RenderMode::Standard => Metrics {
                cell_width: 5,
                cell_height: 3,
                h_thin: "-",
                h_thick: "=",
                v_thin: "|",
                v_thick: "#",
                cross_thin: "+",
                cross_thick: "#",
                top_left: ",",
                top_right: ".",
                mid_left: "|",
                mid_right: "|",
                bot_left: "`",
                bot_right: "'",
            },
            RenderMode::Monospace => Metrics {
                cell_width: 3,
                cell_height: 3,
                h_thin: "─",
                h_thick: "═",
                v_thin: "│",
                v_thick: "║",
                cross_thin: "┼",
                cross_thick: "╬",
                top_left: "╔",
                top_right: "╗",
                mid_left: "╟",
                mid_right: "╢",
                bot_left: "╚",
                bot_right: "╝",
            },
        }
    }
}

struct Metrics {
    cell_width: u16,
    cell_height: u16,
    h_thin: &'static str,
    h_thick: &'static str,
    v_thin: &'static str,
    v_thick: &'static str,
    cross_thin: &'static str,
    cross_thick: &'static str,
    top_left: &'static str,
    top_right: &'static str,
    mid_left: &'static str,
    mid_right: &'static str,
    bot_left: &'static str,
    bot_right: &'static str,
}

struct CliState {
    game: GameState,
    current_face: Face,
    cursor: (u8, u8),
    render_mode: RenderMode,
    last_blink: Instant,
    blink_on: bool,
    message: String,
    // 用于懒刷新与局部重绘的快照。
    prev_cursor: (u8, u8),
    prev_face: Face,
    prev_blink_on: bool,
    prev_render_mode: RenderMode,
    prev_timer_text: String,
    prev_message: String,
    prev_grid_hash: u64,
    dirty: bool,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    let result = run_game(&mut stdout);

    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    result
}

fn run_game(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut state = new_game();
    state.message = "WASD 移动光标，↑↓←→/FBLRTU 切换面，1-9 填数，H 提示，Z 撤销，M 切换字符模式，N 新局，Q 退出".to_string();
    state.dirty = true;

    loop {
        let now = Instant::now();
        if now.duration_since(state.last_blink) >= Duration::from_millis(500) {
            state.blink_on = !state.blink_on;
            state.last_blink = now;
            state.dirty = true;
        }

        render(stdout, &mut state)?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if handle_key(&mut state, key) {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn new_game() -> CliState {
    let mut rng = StdRng::from_entropy();
    let mut generator = WfcGenerator::new();
    let solution = generator.generate(&mut rng).expect("生成题解失败");
    let grid = generate_puzzle(&solution, Difficulty::Medium, &mut rng);
    let mut game = GameState::new(grid, solution, Difficulty::Medium);
    game.started_at = now_secs();
    game.selected = Some(Face::Front.to_cube(4, 4));
    CliState {
        game,
        current_face: Face::Front,
        cursor: (4, 4),
        render_mode: RenderMode::Standard,
        last_blink: Instant::now(),
        blink_on: true,
        message: String::new(),
        prev_cursor: (4, 4),
        prev_face: Face::Front,
        prev_blink_on: true,
        prev_render_mode: RenderMode::Standard,
        prev_timer_text: String::new(),
        prev_message: String::new(),
        prev_grid_hash: 0,
        dirty: true,
    }
}

fn now_secs() -> f64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn handle_key(state: &mut CliState, key: KeyEvent) -> bool {
    let shift_or_none = key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT;

    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('n') | KeyCode::Char('N') => {
            *state = new_game();
            state.dirty = true;
            return false;
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            state.render_mode = state.render_mode.toggle();
            state.dirty = true;
            state.message = format!("已切换为 {:?} 模式", state.render_mode);
            return false;
        }
        KeyCode::Char('h') | KeyCode::Char('H') => {
            state.game.hint();
            state.message = "已提示".to_string();
            state.dirty = true;
        }
        KeyCode::Char('z') | KeyCode::Char('Z') => {
            state.game.undo();
            state.message = "已撤销".to_string();
            state.dirty = true;
        }
        KeyCode::Backspace | KeyCode::Delete => {
            let coord = state.current_face.to_cube(state.cursor.0, state.cursor.1);
            state.game.set_value(coord, None);
            state.dirty = true;
        }
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let value = c as u8 - b'0';
            let coord = state.current_face.to_cube(state.cursor.0, state.cursor.1);
            state.game.set_value(coord, Some(value));
            state.dirty = true;
            if state.game.check_completion() {
                state.message = "恭喜完成！按 N 开始新局，Q 退出。".to_string();
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') if shift_or_none => {
            if state.cursor.1 > 0 {
                state.cursor.1 -= 1;
                state.dirty = true;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') if shift_or_none => {
            if state.cursor.0 > 0 {
                state.cursor.0 -= 1;
                state.dirty = true;
            }
        }
        KeyCode::Char('s') | KeyCode::Char('S') if shift_or_none => {
            if state.cursor.1 < 8 {
                state.cursor.1 += 1;
                state.dirty = true;
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') if shift_or_none => {
            if state.cursor.0 < 8 {
                state.cursor.0 += 1;
                state.dirty = true;
            }
        }
        KeyCode::Up => {
            state.current_face = switch_face(state.current_face, 0, -1);
            state.dirty = true;
        }
        KeyCode::Down => {
            state.current_face = switch_face(state.current_face, 0, 1);
            state.dirty = true;
        }
        KeyCode::Left => {
            state.current_face = switch_face(state.current_face, -1, 0);
            state.dirty = true;
        }
        KeyCode::Right => {
            state.current_face = switch_face(state.current_face, 1, 0);
            state.dirty = true;
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.current_face = Face::Front;
            state.dirty = true;
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            state.current_face = Face::Back;
            state.dirty = true;
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            state.current_face = Face::Left;
            state.dirty = true;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            state.current_face = Face::Right;
            state.dirty = true;
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            state.current_face = Face::Top;
            state.dirty = true;
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            state.current_face = Face::Bottom;
            state.dirty = true;
        }
        _ => {}
    }

    let coord = state.current_face.to_cube(state.cursor.0, state.cursor.1);
    state.game.selected = Some(coord);
    false
}

fn switch_face(face: Face, dx: i8, dy: i8) -> Face {
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

// ---------------------------------------------------------------------------
// 渲染与懒刷新
// ---------------------------------------------------------------------------

fn render(stdout: &mut io::Stdout, state: &mut CliState) -> io::Result<()> {
    let metrics = state.render_mode.metrics();
    let grid_hash = compute_grid_hash(state);
    let timer_text = format_timer(state);

    let need_full = state.dirty
        && (state.render_mode != state.prev_render_mode
            || state.current_face != state.prev_face
            || grid_hash != state.prev_grid_hash);

    if need_full {
        stdout.queue(Clear(ClearType::All))?;
        stdout.queue(MoveTo(0, 0))?;
        render_header(stdout, state, &timer_text)?;
        render_grid(stdout, state, &metrics)?;
        render_message(stdout, state)?;
        stdout.flush()?;

        state.prev_cursor = state.cursor;
        state.prev_face = state.current_face;
        state.prev_blink_on = state.blink_on;
        state.prev_render_mode = state.render_mode;
        state.prev_timer_text = timer_text;
        state.prev_message = state.message.clone();
        state.prev_grid_hash = grid_hash;
        state.dirty = false;
        return Ok(());
    }

    if state.dirty {
        // 同面内的局部更新：光标移动或数值变化。
        if state.current_face == state.prev_face && state.render_mode == state.prev_render_mode {
            if state.cursor != state.prev_cursor {
                render_cell(stdout, state, &metrics, state.prev_cursor.0, state.prev_cursor.1)?;
                render_cell(stdout, state, &metrics, state.cursor.0, state.cursor.1)?;
            } else if state.blink_on != state.prev_blink_on {
                render_cell(stdout, state, &metrics, state.cursor.0, state.cursor.1)?;
            }

            if grid_hash != state.prev_grid_hash {
                // 数值变化时重绘该格；若变化较大可回退为全屏。
                render_cell(stdout, state, &metrics, state.cursor.0, state.cursor.1)?;
            }
        }

        if timer_text != state.prev_timer_text {
            render_header(stdout, state, &timer_text)?;
        }

        if state.message != state.prev_message {
            render_message(stdout, state)?;
        }

        stdout.flush()?;

        state.prev_cursor = state.cursor;
        state.prev_face = state.current_face;
        state.prev_blink_on = state.blink_on;
        state.prev_render_mode = state.render_mode;
        state.prev_timer_text = timer_text;
        state.prev_message = state.message.clone();
        state.prev_grid_hash = grid_hash;
        state.dirty = false;
    }

    Ok(())
}

fn render_header(stdout: &mut io::Stdout, state: &CliState, timer_text: &str) -> io::Result<()> {
    stdout.queue(MoveTo(0, 0))?;
    stdout
        .queue(SetForegroundColor(Color::Cyan))?
        .queue(Print("SuDoKube CLI"))?
        .queue(ResetColor)?;
    stdout.queue(Print(format!(
        " | [{}] 面: {} | 难度: {} | 时间: {} | M 切换模式",
        mode_label(state.render_mode),
        face_name(state.current_face),
        state.game.difficulty.as_str(),
        timer_text
    )))?;
    // 清除行尾旧内容。
    stdout.queue(Clear(ClearType::UntilNewLine))?;
    Ok(())
}

fn render_message(stdout: &mut io::Stdout, state: &CliState) -> io::Result<()> {
    stdout.queue(MoveTo(0, MESSAGE_ROW))?;
    stdout.queue(Print(&state.message))?;
    stdout.queue(Clear(ClearType::UntilNewLine))?;
    Ok(())
}

fn render_grid(stdout: &mut io::Stdout, state: &CliState, metrics: &Metrics) -> io::Result<()> {
    for v in 0..9u8 {
        let sep_row = grid_separator_row(v);
        stdout.queue(MoveTo(0, sep_row))?;
        print_separator(stdout, metrics, v == 0, v == 8)?;

        for line in 0..metrics.cell_height {
            let row = grid_cell_row(v, line);
            stdout.queue(MoveTo(0, row))?;
            for u in 0..9u8 {
                let v_line = if u % 3 == 0 {
                    metrics.v_thick
                } else {
                    metrics.v_thin
                };
                stdout.queue(Print(v_line))?;
                print_cell_content(stdout, state, metrics, u, v, line)?;
            }
            stdout.queue(Print(metrics.v_thick))?;
            stdout.queue(Clear(ClearType::UntilNewLine))?;
            stdout.queue(Print("\n"))?;
        }
    }

    // 底部粗分隔线。
    let bot_row = grid_separator_row(9);
    stdout.queue(MoveTo(0, bot_row))?;
    print_separator(stdout, metrics, true, true)?;
    Ok(())
}

fn render_cell(
    stdout: &mut io::Stdout,
    state: &CliState,
    metrics: &Metrics,
    u: u8,
    v: u8,
) -> io::Result<()> {
    for line in 0..metrics.cell_height {
        let row = grid_cell_row(v, line);
        let col = 1 + u as u16 * (metrics.cell_width + 1);
        stdout.queue(MoveTo(col, row))?;
        print_cell_content(stdout, state, metrics, u, v, line)?;
    }
    Ok(())
}

fn print_separator(
    stdout: &mut io::Stdout,
    metrics: &Metrics,
    is_top: bool,
    is_bottom: bool,
) -> io::Result<()> {
    for u in 0..9u8 {
        let is_major = u % 3 == 0;
        let cross = if is_major {
            metrics.cross_thick
        } else {
            metrics.cross_thin
        };
        let h = if is_major { metrics.h_thick } else { metrics.h_thin };

        if u == 0 {
            let corner = if is_top {
                metrics.top_left
            } else if is_bottom {
                metrics.bot_left
            } else {
                metrics.mid_left
            };
            stdout.queue(Print(corner))?;
        } else {
            stdout.queue(Print(cross))?;
        }
        stdout.queue(Print(h.repeat(metrics.cell_width as usize)))?;
    }

    let corner = if is_top {
        metrics.top_right
    } else if is_bottom {
        metrics.bot_right
    } else {
        metrics.mid_right
    };
    stdout.queue(Print(corner))?;
    stdout.queue(Clear(ClearType::UntilNewLine))?;
    Ok(())
}

fn print_cell_content(
    stdout: &mut io::Stdout,
    state: &CliState,
    metrics: &Metrics,
    u: u8,
    v: u8,
    line: u16,
) -> io::Result<()> {
    let coord = state.current_face.to_cube(u, v);
    let cell = state.game.grid.get(&coord);
    let selected = state.cursor == (u, v);
    let is_given = cell.map(|c| c.given).unwrap_or(false);
    let value = cell.and_then(|c| c.user_value);

    let mid_line = metrics.cell_height / 2;
    let mut content = " ".repeat(metrics.cell_width as usize);
    if line == mid_line {
        if let Some(n) = value {
            let s = (b'0' + n) as char;
            let idx = metrics.cell_width as usize / 2;
            content.replace_range(idx..idx + 1, &s.to_string());
        }
    }

    if selected && state.blink_on {
        stdout
            .queue(SetBackgroundColor(Color::White))?
            .queue(SetForegroundColor(Color::Black))?;
    } else if selected {
        stdout
            .queue(SetBackgroundColor(Color::Grey))?
            .queue(SetForegroundColor(Color::White))?;
    } else if is_given {
        stdout
            .queue(SetAttribute(Attribute::Bold))?
            .queue(SetForegroundColor(Color::Yellow))?;
    } else if value.map_or(false, |n| is_conflicting(state, coord, n)) {
        stdout.queue(SetForegroundColor(Color::Red))?;
    }

    stdout.queue(Print(content))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

fn grid_separator_row(v: u8) -> u16 {
    GRID_START_ROW + v as u16 * 4
}

fn grid_cell_row(v: u8, line: u16) -> u16 {
    GRID_START_ROW + 1 + v as u16 * 4 + line
}

fn compute_grid_hash(state: &CliState) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    state.current_face.hash(&mut hasher);
    for v in 0..9u8 {
        for u in 0..9u8 {
            let coord = state.current_face.to_cube(u, v);
            if let Some(cell) = state.game.grid.get(&coord) {
                coord.hash(&mut hasher);
                cell.given.hash(&mut hasher);
                cell.user_value.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

fn format_timer(state: &CliState) -> String {
    let elapsed = if state.game.started_at > 0.0 && !state.game.completed {
        now_secs() - state.game.started_at
    } else {
        state.game.elapsed_seconds as f64
    };
    let minutes = (elapsed / 60.0) as u64;
    let seconds = (elapsed % 60.0) as u64;
    format!("{:02}:{:02}", minutes, seconds)
}

fn is_conflicting(state: &CliState, coord: CubeCoord, value: u8) -> bool {
    for other in coord.related() {
        if other == coord {
            continue;
        }
        if let Some(cell) = state.game.grid.get(&other) {
            if cell.user_value == Some(value) {
                return true;
            }
        }
    }
    false
}

fn face_name(face: Face) -> &'static str {
    match face {
        Face::Front => "F 前",
        Face::Back => "B 后",
        Face::Left => "L 左",
        Face::Right => "R 右",
        Face::Top => "T 上",
        Face::Bottom => "U 下",
    }
}

fn mode_label(mode: RenderMode) -> &'static str {
    match mode {
        RenderMode::Standard => "标准",
        RenderMode::Monospace => "等距",
    }
}
