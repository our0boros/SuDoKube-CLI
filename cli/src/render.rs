use crossterm::{
    cursor::MoveTo,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{Clear, ClearType, size},
    QueueableCommand,
};
use std::io::{self, Write};
use sudokube_core::cube::{CubeCoord, Face};

use crate::{AppScreen, CliState, MenuItem};

/// 渲染字符模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// 标准 ASCII 字符模式，横向更宽，兼容性最好。
    Standard,
    /// 等宽框线字符模式。
    Monospace,
}

impl RenderMode {
    pub fn toggle(self) -> Self {
        match self {
            RenderMode::Standard => RenderMode::Monospace,
            RenderMode::Monospace => RenderMode::Standard,
        }
    }

    pub fn metrics(self) -> Metrics {
        match self {
            RenderMode::Standard => Metrics {
                cell_width: 5,
                cell_height: 3,
                sep: BoxChars {
                    h_thin: '-',
                    h_thick: '=',
                    v_thin: '|',
                    v_thick: '#',
                    cross_thin_thin: '+',
                    cross_thin_thick: '#',
                    cross_thick_thin: '#',
                    cross_thick_thick: '#',
                    top_thin: '-',
                    top_thick: '=',
                    bot_thin: '-',
                    bot_thick: '=',
                },
                top_left: ',',
                top_right: '.',
                mid_left: '|',
                mid_right: '|',
                bot_left: '`',
                bot_right: '\'',
            },
            RenderMode::Monospace => Metrics {
                cell_width: 3,
                cell_height: 3,
                sep: BoxChars {
                    h_thin: '─',
                    h_thick: '═',
                    v_thin: '│',
                    v_thick: '║',
                    cross_thin_thin: '┼',
                    cross_thin_thick: '╫',
                    cross_thick_thin: '╪',
                    cross_thick_thick: '╬',
                    top_thin: '╤',
                    top_thick: '╦',
                    bot_thin: '╧',
                    bot_thick: '╩',
                },
                top_left: '╔',
                top_right: '╗',
                mid_left: '╟',
                mid_right: '╢',
                bot_left: '╚',
                bot_right: '╝',
            },
        }
    }
}

struct BoxChars {
    h_thin: char,
    h_thick: char,
    v_thin: char,
    v_thick: char,
    cross_thin_thin: char,
    cross_thin_thick: char,
    cross_thick_thin: char,
    cross_thick_thick: char,
    top_thin: char,
    top_thick: char,
    bot_thin: char,
    bot_thick: char,
}

pub struct Metrics {
    cell_width: u16,
    cell_height: u16,
    sep: BoxChars,
    top_left: char,
    top_right: char,
    mid_left: char,
    mid_right: char,
    bot_left: char,
    bot_right: char,
}

impl Metrics {
    pub fn grid_width(&self) -> u16 {
        1 + 9 * (self.cell_width + 1)
    }

    pub fn grid_height(&self) -> u16 {
        1 + 9 * (self.cell_height + 1)
    }

    pub fn cell_width(&self) -> u16 {
        self.cell_width
    }

    pub fn cell_height(&self) -> u16 {
        self.cell_height
    }
}

/// 颜色主题。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    fn colors(self) -> ThemeColors {
        match self {
            Theme::Dark => ThemeColors {
                bg: Color::Black,
                fg: Color::White,
                given: Color::Yellow,
                user: Color::Cyan,
                error: Color::Red,
                selected_bg: Color::White,
                selected_fg: Color::Black,
                button_bg: Color::DarkGrey,
                button_fg: Color::White,
                button_hover_bg: Color::White,
                button_hover_fg: Color::Black,
                header: Color::Cyan,
                message: Color::Green,
            },
            Theme::Light => ThemeColors {
                bg: Color::White,
                fg: Color::Black,
                given: Color::Blue,
                user: Color::Magenta,
                error: Color::Red,
                selected_bg: Color::Black,
                selected_fg: Color::White,
                button_bg: Color::Grey,
                button_fg: Color::Black,
                button_hover_bg: Color::DarkGrey,
                button_hover_fg: Color::White,
                header: Color::DarkCyan,
                message: Color::DarkGreen,
            },
        }
    }
}

struct ThemeColors {
    bg: Color,
    fg: Color,
    given: Color,
    user: Color,
    error: Color,
    selected_bg: Color,
    selected_fg: Color,
    button_bg: Color,
    button_fg: Color,
    button_hover_bg: Color,
    button_hover_fg: Color,
    header: Color,
    message: Color,
}

/// 屏幕布局计算结果。
pub struct Layout {
    pub term_cols: u16,
    pub term_rows: u16,
    pub grid_offset: (u16, u16),
    pub info_row: u16,
    pub message_row: u16,
    pub buttons: Vec<ButtonArea>,
    pub too_small: bool,
}

#[derive(Debug, Clone)]
pub struct ButtonArea {
    pub id: ButtonId,
    pub label: String,
    pub col: u16,
    pub row: u16,
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonId {
    Number(u8),
    Hint,
    Undo,
    ToggleGuidance,
    NewGame,
    ToggleMode,
    ToggleTheme,
    Quit,
}

pub fn render(stdout: &mut io::Stdout, state: &mut CliState) -> io::Result<()> {
    let term_size = size()?;
    let metrics = state.render_mode.metrics();
    let layout = compute_layout(term_size, &metrics, state);

    if layout.too_small {
        if state.prev_term_size != term_size || state.dirty {
            stdout.queue(Clear(ClearType::All))?;
            let msg = "当前画面太小，请放大终端";
            let col = term_size.0.saturating_sub(msg.len() as u16) / 2;
            let row = term_size.1 / 2;
            stdout.queue(MoveTo(col, row))?;
            stdout.queue(SetForegroundColor(Color::Red))?;
            stdout.queue(Print(msg))?;
            stdout.queue(ResetColor)?;
            stdout.flush()?;
            state.prev_term_size = term_size;
            state.dirty = false;
        }
        return Ok(());
    }

    let timer_text = format_timer(state);
    let grid_hash = compute_grid_hash(state);

    let need_full = state.dirty
        && (state.screen != state.prev_screen
            || state.render_mode != state.prev_render_mode
            || state.theme != state.prev_theme
            || state.current_face != state.prev_face
            || grid_hash != state.prev_grid_hash
            || term_size != state.prev_term_size);

    if need_full {
        stdout.queue(Clear(ClearType::All))?;
        render_screen_full(stdout, state, &layout, &metrics, &timer_text)?;
    } else if state.dirty {
        render_screen_partial(stdout, state, &layout, &metrics, &timer_text, grid_hash)?;
    }

    if state.dirty {
        state.prev_cursor = state.cursor;
        state.prev_face = state.current_face;
        state.prev_blink_on = state.blink_on;
        state.prev_render_mode = state.render_mode;
        state.prev_theme = state.theme;
        state.prev_timer_text = timer_text;
        state.prev_message.clone_from(&state.message);
        state.prev_grid_hash = grid_hash;
        state.prev_term_size = term_size;
        state.prev_screen = state.screen;
        state.dirty = false;
    }

    Ok(())
}

pub fn compute_layout_for_state(state: &CliState) -> Layout {
    let term_size = size().unwrap_or((80, 24));
    let metrics = state.render_mode.metrics();
    compute_layout(term_size, &metrics, state)
}

fn compute_layout(term_size: (u16, u16), metrics: &Metrics, state: &CliState) -> Layout {
    let (term_cols, term_rows) = term_size;
    let grid_w = metrics.grid_width();
    let grid_h = metrics.grid_height();
    let info_height = 1;
    let message_height = if state.screen == AppScreen::Game { 1 } else { 0 };
    let button_height = 1;
    let total_h = grid_h + info_height + message_height + button_height + 2; // 间距

    let too_small = term_cols < grid_w + 4 || term_rows < total_h;

    let grid_offset = (
        (term_cols.saturating_sub(grid_w)) / 2,
        (term_rows.saturating_sub(total_h)) / 2 + info_height,
    );
    let info_row = grid_offset.1.saturating_sub(1);
    let message_row = grid_offset.1 + grid_h + 1;
    let button_row = message_row + message_height;

    let mut buttons = Vec::new();
    if state.screen == AppScreen::Game {
        let mut col = grid_offset.0;
        for n in 1..=9u8 {
            let label = format!("[{}]", n);
            let width = label.chars().count() as u16;
            buttons.push(ButtonArea {
                id: ButtonId::Number(n),
                label,
                col,
                row: button_row,
                width,
            });
            col += width + 1;
        }
        for (label, id) in [
            ("[H]int", ButtonId::Hint),
            ("[Z]undo", ButtonId::Undo),
            ("[G]uide", ButtonId::ToggleGuidance),
            ("[N]ew", ButtonId::NewGame),
            ("[M]ode", ButtonId::ToggleMode),
            ("[Y]theme", ButtonId::ToggleTheme),
            ("[Q]menu", ButtonId::Quit),
        ] {
            let width = label.chars().count() as u16;
            buttons.push(ButtonArea {
                id,
                label: label.to_string(),
                col,
                row: button_row,
                width,
            });
            col += width + 1;
        }
    }

    Layout {
        term_cols,
        term_rows,
        grid_offset,
        info_row,
        message_row,
        buttons,
        too_small,
    }
}

fn render_screen_full(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
    metrics: &Metrics,
    timer_text: &str,
) -> io::Result<()> {
    let colors = state.theme.colors();
    stdout.queue(SetBackgroundColor(colors.bg))?;
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(ResetColor)?;

    match state.screen {
        AppScreen::Menu => render_menu_full(stdout, state, layout)?,
        AppScreen::Game => {
            render_info_line(stdout, state, layout, timer_text)?;
            render_grid_full(stdout, state, layout, metrics)?;
            render_message_line(stdout, state, layout)?;
            render_buttons(stdout, state, layout)?;
        }
    }
    stdout.flush()?;
    Ok(())
}

fn render_screen_partial(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
    metrics: &Metrics,
    timer_text: &str,
    grid_hash: u64,
) -> io::Result<()> {
    match state.screen {
        AppScreen::Menu => render_menu_full(stdout, state, layout)?,
        AppScreen::Game => {
            if timer_text != state.prev_timer_text {
                render_info_line(stdout, state, layout, timer_text)?;
            }

            if state.current_face == state.prev_face && state.render_mode == state.prev_render_mode {
                if state.cursor != state.prev_cursor {
                    render_cell(stdout, state, layout, metrics, state.prev_cursor.0, state.prev_cursor.1)?;
                    render_cell(stdout, state, layout, metrics, state.cursor.0, state.cursor.1)?;
                } else if state.blink_on != state.prev_blink_on {
                    render_cell(stdout, state, layout, metrics, state.cursor.0, state.cursor.1)?;
                }
            }

            if grid_hash != state.prev_grid_hash {
                // 数值变化时只重绘当前格；冲突高亮变化较小，若需要可扩展为全屏。
                render_cell(stdout, state, layout, metrics, state.cursor.0, state.cursor.1)?;
            }

            if state.message != state.prev_message {
                render_message_line(stdout, state, layout)?;
            }

            render_buttons(stdout, state, layout)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn render_menu_full(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
) -> io::Result<()> {
    let colors = state.theme.colors();
    let title = "SuDoKube CLI";
    let col = (layout.term_cols.saturating_sub(title.chars().count() as u16)) / 2;
    let mut row = layout.term_rows / 4;

    stdout.queue(MoveTo(col, row))?;
    stdout.queue(SetForegroundColor(colors.header))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(title))?;
    stdout.queue(ResetColor)?;
    row += 2;

    for (i, item) in state.menu.items.iter().enumerate() {
        let label = match item {
            MenuItem::NewGame(d) => format!("新游戏 - {}", d.as_str()),
            MenuItem::Continue(r) => format!(
                "继续 #{} | {} | {:02}:{:02} | {}",
                r.id,
                r.difficulty,
                r.elapsed_seconds / 60,
                r.elapsed_seconds % 60,
                if r.completed { "已完成" } else { "进行中" }
            ),
        };
        let prefix = if i == state.menu.selected { "> " } else { "  " };
        let line = format!("{}{}", prefix, label);
        let col = (layout.term_cols.saturating_sub(line.chars().count() as u16)) / 2;
        stdout.queue(MoveTo(col, row + i as u16))?;
        if i == state.menu.selected {
            stdout.queue(SetBackgroundColor(colors.selected_bg))?;
            stdout.queue(SetForegroundColor(colors.selected_fg))?;
        }
        stdout.queue(Print(line))?;
        stdout.queue(ResetColor)?;
    }

    row += state.menu.items.len() as u16 + 2;
    let hint = "↑/↓ 选择，Enter 确认，D 删除记录，Q 退出";
    let col = (layout.term_cols.saturating_sub(hint.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(col, row))?;
    stdout.queue(SetForegroundColor(colors.fg))?;
    stdout.queue(Print(hint))?;
    stdout.queue(ResetColor)?;
    stdout.flush()?;
    Ok(())
}

fn render_info_line(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
    timer_text: &str,
) -> io::Result<()> {
    let colors = state.theme.colors();
    stdout.queue(MoveTo(0, layout.info_row))?;
    stdout.queue(SetForegroundColor(colors.header))?;
    stdout.queue(Print("SuDoKube CLI"))?;
    stdout.queue(ResetColor)?;
    stdout.queue(Print(format!(
        " | [{}] 面: {} | 难度: {} | 时间: {} | 滚轮切换面，Alt+滚轮横向",
        mode_label(state.render_mode),
        face_name(state.current_face),
        state.game.difficulty.as_str(),
        timer_text
    )))?;
    stdout.queue(Clear(ClearType::UntilNewLine))?;
    Ok(())
}

fn render_message_line(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
) -> io::Result<()> {
    let colors = state.theme.colors();
    stdout.queue(MoveTo(0, layout.message_row))?;
    if !state.message.is_empty() {
        stdout.queue(SetForegroundColor(colors.message))?;
        stdout.queue(Print(&state.message))?;
        stdout.queue(ResetColor)?;
    }
    stdout.queue(Clear(ClearType::UntilNewLine))?;
    Ok(())
}

fn render_buttons(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
) -> io::Result<()> {
    let colors = state.theme.colors();
    for btn in &layout.buttons {
        stdout.queue(MoveTo(btn.col, btn.row))?;
        if state.hover_button == Some(btn.id) {
            stdout.queue(SetBackgroundColor(colors.button_hover_bg))?;
            stdout.queue(SetForegroundColor(colors.button_hover_fg))?;
        } else {
            stdout.queue(SetBackgroundColor(colors.button_bg))?;
            stdout.queue(SetForegroundColor(colors.button_fg))?;
        }
        stdout.queue(Print(&btn.label))?;
        stdout.queue(ResetColor)?;
    }
    // 清除按钮行右侧残留。
    if let Some(last) = layout.buttons.last() {
        let end_col = last.col + last.width;
        stdout.queue(MoveTo(end_col, last.row))?;
        stdout.queue(Clear(ClearType::UntilNewLine))?;
    }
    Ok(())
}

fn render_grid_full(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
    metrics: &Metrics,
) -> io::Result<()> {
    for v in 0..9u8 {
        let sep_row = grid_separator_row(layout, v);
        stdout.queue(MoveTo(layout.grid_offset.0, sep_row))?;
        print_separator(stdout, metrics, v == 0, v == 8, v % 3 == 0)?;

        for line in 0..metrics.cell_height {
            let row = grid_cell_row(layout, v, line);
            stdout.queue(MoveTo(layout.grid_offset.0, row))?;
            for u in 0..9u8 {
                let v_line = if u % 3 == 0 {
                    metrics.sep.v_thick
                } else {
                    metrics.sep.v_thin
                };
                stdout.queue(Print(v_line))?;
                print_cell_content(stdout, state, metrics, u, v, line)?;
            }
            stdout.queue(Print(metrics.sep.v_thick))?;
            stdout.queue(Clear(ClearType::UntilNewLine))?;
        }
    }

    let bot_row = grid_separator_row(layout, 9);
    stdout.queue(MoveTo(layout.grid_offset.0, bot_row))?;
    print_separator(stdout, metrics, true, true, true)?;
    Ok(())
}

fn render_cell(
    stdout: &mut io::Stdout,
    state: &CliState,
    layout: &Layout,
    metrics: &Metrics,
    u: u8,
    v: u8,
) -> io::Result<()> {
    for line in 0..metrics.cell_height {
        let row = grid_cell_row(layout, v, line);
        let col = layout.grid_offset.0 + 1 + u as u16 * (metrics.cell_width + 1);
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
    is_thick_h: bool,
) -> io::Result<()> {
    let left_corner = if is_top {
        metrics.top_left
    } else if is_bottom {
        metrics.bot_left
    } else {
        metrics.mid_left
    };
    let right_corner = if is_top {
        metrics.top_right
    } else if is_bottom {
        metrics.bot_right
    } else {
        metrics.mid_right
    };
    stdout.queue(Print(left_corner))?;

    for u in 0..8u8 {
        let is_major_v = (u + 1) % 3 == 0;
        let h = if is_thick_h {
            metrics.sep.h_thick
        } else {
            metrics.sep.h_thin
        };
        stdout.queue(Print(h.to_string().repeat(metrics.cell_width as usize)))?;

        let cross = if is_top {
            if is_major_v {
                metrics.sep.top_thick
            } else {
                metrics.sep.top_thin
            }
        } else if is_bottom {
            if is_major_v {
                metrics.sep.bot_thick
            } else {
                metrics.sep.bot_thin
            }
        } else if is_thick_h {
            if is_major_v {
                metrics.sep.cross_thick_thick
            } else {
                metrics.sep.cross_thick_thin
            }
        } else if is_major_v {
            metrics.sep.cross_thin_thick
        } else {
            metrics.sep.cross_thin_thin
        };
        stdout.queue(Print(cross))?;
    }

    let h = if is_thick_h {
        metrics.sep.h_thick
    } else {
        metrics.sep.h_thin
    };
    stdout.queue(Print(h.to_string().repeat(metrics.cell_width as usize)))?;
    stdout.queue(Print(right_corner))?;
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
    let colors = state.theme.colors();
    let coord = state.current_face.to_cube(u, v);
    let cell = state.game.grid.get(&coord);
    let selected = state.cursor == (u, v);
    let is_given = cell.map(|c| c.given).unwrap_or(false);
    let value = cell.and_then(|c| c.user_value);

    // Guidance 模式：判断当前格是否与选中格同行/列/宫，或有相同数字。
    let (in_same_group, has_same_number) = if state.guidance && !selected {
        let sel_coord = state.current_face.to_cube(state.cursor.0, state.cursor.1);
        let same_row = state.cursor.1 == v;
        let same_col = state.cursor.0 == u;
        let same_box = state.cursor.0 / 3 == u / 3 && state.cursor.1 / 3 == v / 3;
        let in_group = same_row || same_col || same_box;

        let sel_value = state.game.grid.get(&sel_coord).and_then(|c| c.user_value);
        let same_num = value.is_some() && value == sel_value;

        (in_group, same_num)
    } else {
        (false, false)
    };

    let mid_line = metrics.cell_height / 2;
    let mut content: String = " ".repeat(metrics.cell_width as usize);
    if line == mid_line {
        if let Some(n) = value {
            let s: String = if has_same_number && !is_given {
                const CIRCLED: [&str; 9] = ["①","②","③","④","⑤","⑥","⑦","⑧","⑨"];
                CIRCLED[(n - 1) as usize].to_string()
            } else {
                ((b'0' + n) as char).to_string()
            };
            // 居中放置字符。
            let display_width = if has_same_number && !is_given { 2 } else { 1 };
            let padding = metrics.cell_width as usize;
            let start = if padding > display_width {
                (padding - display_width) / 2
            } else {
                0
            };
            content = " ".repeat(padding);
            content.replace_range(start..start + s.len(), &s);
        }
    }

    if selected && state.blink_on {
        stdout
            .queue(SetBackgroundColor(colors.selected_bg))?
            .queue(SetForegroundColor(colors.selected_fg))?;
    } else if selected {
        stdout
            .queue(SetBackgroundColor(Color::Grey))?
            .queue(SetForegroundColor(Color::White))?;
    } else if in_same_group && has_same_number {
        stdout
            .queue(SetBackgroundColor(Color::DarkGreen))?
            .queue(SetForegroundColor(Color::White))?;
    } else if in_same_group {
        stdout
            .queue(SetBackgroundColor(Color::DarkGreen))?
            .queue(SetForegroundColor(colors.fg))?;
    } else if has_same_number {
        stdout
            .queue(SetBackgroundColor(Color::DarkCyan))?
            .queue(SetForegroundColor(Color::White))?;
    } else if is_given {
        stdout
            .queue(SetAttribute(Attribute::Bold))?
            .queue(SetForegroundColor(colors.given))?;
    } else if value.map_or(false, |n| is_conflicting(state, coord, n)) {
        stdout.queue(SetForegroundColor(colors.error))?;
    } else if value.is_some() {
        stdout.queue(SetForegroundColor(colors.user))?;
    } else {
        stdout.queue(SetForegroundColor(colors.fg))?;
    }

    stdout.queue(Print(content))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

fn grid_separator_row(layout: &Layout, v: u8) -> u16 {
    layout.grid_offset.1 + v as u16 * (layout_render_cell_height(layout) + 1)
}

fn grid_cell_row(layout: &Layout, v: u8, line: u16) -> u16 {
    layout.grid_offset.1 + 1 + v as u16 * (layout_render_cell_height(layout) + 1) + line
}

fn layout_render_cell_height(_layout: &Layout) -> u16 {
    // 布局计算时 cell_height 固定为 3，但可通过 metrics 获取；这里保持与 compute_layout 一致。
    3
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
    let elapsed = crate::total_elapsed(state);
    let minutes = elapsed / 60;
    let seconds = elapsed % 60;
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

pub fn mode_label(mode: RenderMode) -> &'static str {
    match mode {
        RenderMode::Standard => "标准",
        RenderMode::Monospace => "等距",
    }
}

pub fn find_button_at(layout: &Layout, col: u16, row: u16) -> Option<ButtonId> {
    for btn in &layout.buttons {
        if row == btn.row && col >= btn.col && col < btn.col + btn.width {
            return Some(btn.id);
        }
    }
    None
}

pub fn cell_at(layout: &Layout, metrics: &Metrics, col: u16, row: u16) -> Option<(u8, u8)> {
    let gx = col.saturating_sub(layout.grid_offset.0);
    let gy = row.saturating_sub(layout.grid_offset.1);

    // 检查是否在分隔线上。
    if gy % (metrics.cell_height + 1) == 0 {
        return None;
    }
    let v = (gy / (metrics.cell_height + 1)) as u8;
    if v >= 9 {
        return None;
    }

    // 检查是否在竖线上。
    if gx % (metrics.cell_width + 1) == 0 {
        return None;
    }
    let u = (gx / (metrics.cell_width + 1)) as u8;
    if u >= 9 {
        return None;
    }

    Some((u, v))
}
