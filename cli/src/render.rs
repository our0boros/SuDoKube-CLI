use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
use sudokube_core::cube::Face;
use std::time::Instant;

use crate::{App, AppScreen, MenuItem, total_elapsed, AppSettings};
use crate::i18n::{self, Lang};

// ── 常量 ──

const LOGO: &[&str] = &[
    " ██████  █    ██ ▓█████▄  ▒█████   ██ ▄█▀ █    ██  ▄▄▄▄   ▓█████   ",
    " ▒██    ▒  ██  ▓██▒▒██▀ ██▌▒██▒  ██▒ ██▄█▒  ██  ▓██▒▓█████▄ ▓█   ▀ ",
    " ░ ▓██▄   ▓██  ▒██░░██   █▌▒██░  ██▒▓███▄░ ▓██  ▒██░▒██▒ ▄██▒███   ",
    "   ▒   ██▒▓▓█  ░██░░▓█▄   ▌▒██   ██░▓██ █▄ ▓▓█  ░██░▒██░█▀  ▒▓█  ▄ ",
    " ▒██████▒▒▒▒█████▓ ░▒████▓ ░ ████▓▒░▒██▒ █▄▒▒█████▓ ░▓█  ▀█▓░▒████▒",
    " ▒ ▒▓▒ ▒ ░░▒▓▒ ▒ ▒  ▒▒▓  ▒ ░ ▒░▒░▒░ ▒ ▒▒ ▓▒░▒▓▒ ▒ ▒ ░▒▓███▀▒░░ ▒░ ░",
    " ░ ░▒  ░ ░░░▒░ ░ ░  ░ ▒  ▒   ░ ▒ ▒░ ░ ░▒ ▒░░░▒░ ░ ░ ▒░▒   ░  ░ ░  ░",
    " ░  ░  ░   ░░░ ░ ░  ░ ░  ░ ░ ░ ░ ▒  ░ ░░ ░  ░░░ ░ ░  ░    ░    ░   ",
    "       ░     ░        ░        ░ ░  ░  ░      ░      ░         ░  ░",
    "                    ░                                     ░        ",
];

// ── 类型 ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Standard,
    Monospace,
}

impl RenderMode {
    pub fn toggle(self) -> Self {
        match self {
            RenderMode::Standard => RenderMode::Monospace,
            RenderMode::Monospace => RenderMode::Standard,
        }
    }

    pub fn cell_width(self, settings: &AppSettings) -> usize {
        match self {
            RenderMode::Standard => settings.standard_cell_width,
            RenderMode::Monospace => 3,
        }
    }

    pub fn cell_height(self) -> usize {
        3
    }
}

pub fn mode_label(mode: RenderMode, lang: Lang) -> &'static str {
    match mode {
        RenderMode::Standard => i18n::t("game.mode_standard", lang),
        RenderMode::Monospace => i18n::t("game.mode_mono", lang),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonId {
    Number(u8),
    Erase,
    Hint,
    Undo,
    ToggleGuidance,
    ToggleMode,
    Quit,
}

/// 按钮布局信息（用于鼠标点击检测）。
pub struct ButtonLayout {
    pub id: ButtonId,
    pub label: String,
    pub col: u16,
    pub row: u16,
    pub width: u16,
}

/// 游戏画面布局。
pub struct GameLayout {
    pub grid_area: Rect,
    pub info_area: Rect,
    pub msg_area: Rect,
    pub btn_area: Rect,
    pub cube_area: Rect,
    pub buttons: Vec<ButtonLayout>,
}

// ── 主入口 ──

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        AppScreen::Menu => draw_menu(f, app),
        AppScreen::Game => draw_game(f, app),
        AppScreen::Settings => draw_settings(f, app),
        AppScreen::Generating => draw_generating(f, app),
        AppScreen::Victory => draw_victory(f, app),
        AppScreen::ExportSelect => draw_export_select(f, app),
        AppScreen::ImportInput => draw_import_input(f, app),
    }
}

// ── 菜单画面 ──

fn draw_menu(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let logo_h = LOGO.len() as u16;
    let items_len = app.menu.items.len() as u16;
    let total_h = logo_h + 1 + items_len + 2 + 2 + 1; // logo + gap + box(1+items+1) + gap + hint

    let start_y = area.y + area.height.saturating_sub(total_h) / 2;

    // 绘制 LOGO
    let logo_width = LOGO.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    for (i, line) in LOGO.iter().enumerate() {
        let col = area.x + area.width.saturating_sub(logo_width) / 2;
        let row = start_y + i as u16;
        if row < area.bottom() {
            f.render_widget(
                Paragraph::new(*line).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Rect::new(col, row, logo_width, 1),
            );
        }
    }

    let box_y = start_y + logo_h + 1;

    // 计算菜单项文本
    let lang = Lang::from_code(&app.settings.language);
    let item_texts: Vec<String> = app.menu.items.iter().map(|item| match item {
        MenuItem::NewGame(d) => {
            let diff_key = match d {
                sudokube_core::cube::Difficulty::Easy => "game.diff_easy",
                sudokube_core::cube::Difficulty::Medium => "game.diff_medium",
                sudokube_core::cube::Difficulty::Hard => "game.diff_hard",
            };
            format!("{} - {}", i18n::t("menu.new_easy", lang).split(" - ").next().unwrap_or("New"), i18n::t(diff_key, lang))
        }
        MenuItem::Settings => i18n::t("menu.settings", lang).into(),
        MenuItem::Export => i18n::t("menu.export_all", lang).into(),
        MenuItem::Import => i18n::t("menu.import_all", lang).into(),
        MenuItem::Continue(r) => {
            let total = r.answer.len() + r.puzzle.values().filter(|&&v| v == 0).count();
            let filled = r.puzzle.values().filter(|&&v| v > 0).count();
            let remaining = total.saturating_sub(filled);
            let name = if app.settings.naming_mode == "vivid" {
                i18n::vivid_name(r.id, lang)
            } else {
                format!("#{}", r.id)
            };
            format!(
                "{} {} | {} | {} | {}/{} | {:02}:{:02} | {}",
                i18n::t("menu.continue", lang), name, r.difficulty,
                r.started_at.format("%m-%d").to_string(),
                remaining, total,
                r.elapsed_seconds / 60, r.elapsed_seconds % 60,
                if r.completed { i18n::t("menu.victory", lang) } else { i18n::t("menu.in_progress", lang) }
            )
        }
    }).collect();

    let max_text_w = item_texts.iter().map(|t| display_width(t) + 4).max().unwrap_or(20);
    let box_w = max_text_w + 4; // ╭ + " " + text + " " + ╮

    // 侧边栏宽度
    let sidebar_w: u16 = 28;
    let total_w = box_w as u16 + 2 + sidebar_w; // menu + gap + sidebar

    // If sidebar doesn't fit, skip it
    let has_sidebar = area.width >= total_w;
    let box_x = if has_sidebar {
        area.x + area.width.saturating_sub(total_w) / 2
    } else {
        area.x + area.width.saturating_sub(box_w as u16) / 2
    };

    // 画菜单框
    draw_menu_box(f, box_x, box_y, box_w as u16, &item_texts, app.menu.selected, area);

    // 侧边栏：胜利记录
    if has_sidebar {
        let sidebar_x = box_x + box_w as u16 + 2;
        let victories = &app.menu.victories;
        let inner_sw = sidebar_w as usize - 2;

        // 统计
        let total_v = victories.len();
        let easy_count = victories.iter().filter(|r| r.difficulty == "简单" || r.difficulty == "easy").count();
        let med_count = victories.iter().filter(|r| r.difficulty == "中等" || r.difficulty == "medium").count();
        let hard_count = victories.iter().filter(|r| r.difficulty == "困难" || r.difficulty == "hard").count();

        let top = format!("╭{}╮", "─".repeat(inner_sw));
        f.render_widget(Paragraph::new(top).style(Style::default().fg(Color::Yellow)), Rect::new(sidebar_x, box_y, sidebar_w, 1));

        let title = format!("│{:^width$}│", i18n::t("menu.sidebar_title", lang), width = inner_sw);
        f.render_widget(Paragraph::new(title).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), Rect::new(sidebar_x, box_y + 1, sidebar_w, 1));

        let sep = format!("╟{}╢", "─".repeat(inner_sw));
        f.render_widget(Paragraph::new(sep).style(Style::default().fg(Color::Yellow)), Rect::new(sidebar_x, box_y + 2, sidebar_w, 1));

        // 统计行
        let stats = format!(" {}:{} {} {}:{} {} {}:{}", 
            i18n::t("menu.sidebar_total", lang), total_v,
            i18n::t("game.diff_easy", lang).chars().next().unwrap_or('E'), easy_count,
            i18n::t("game.diff_medium", lang).chars().next().unwrap_or('M'), med_count,
            i18n::t("game.diff_hard", lang).chars().next().unwrap_or('H'), hard_count
        );
        let stats_line = format!("│{:<width$}│", stats, width = inner_sw);
        f.render_widget(Paragraph::new(stats_line).style(Style::default().fg(Color::Yellow)), Rect::new(sidebar_x, box_y + 3, sidebar_w, 1));

        let sep2 = format!("╟{}╢", "─".repeat(inner_sw));
        f.render_widget(Paragraph::new(sep2).style(Style::default().fg(Color::Yellow)), Rect::new(sidebar_x, box_y + 4, sidebar_w, 1));

        // 胜利列表
        let available_rows = area.bottom().saturating_sub(box_y + 5).saturating_sub(2) as usize; // reserve bottom border + hint
        for (i, r) in victories.iter().enumerate().take(available_rows) {
            let name = if app.settings.naming_mode == "vivid" {
                i18n::vivid_name(r.id, lang)
            } else {
                format!("#{}", r.id)
            };
            // Short format: name | diff | time
            let diff_short = match r.difficulty.as_str() {
                "简单" | "easy" => i18n::t("game.diff_easy", lang),
                "困难" | "hard" => i18n::t("game.diff_hard", lang),
                _ => i18n::t("game.diff_medium", lang),
            };
            let text = format!(" {} {} {:02}:{:02}", name, diff_short, r.elapsed_seconds / 60, r.elapsed_seconds % 60);
            let row_text = format!("│{:<width$}│", text, width = inner_sw);
            f.render_widget(
                Paragraph::new(row_text).style(Style::default().fg(Color::DarkGray)),
                Rect::new(sidebar_x, box_y + 5 + i as u16, sidebar_w, 1),
            );
        }

        // 底部
        let bot_y = box_y + 5 + victories.len().min(available_rows) as u16;
        if bot_y < area.bottom() {
            let bot = format!("╰{}╯", "─".repeat(inner_sw));
            f.render_widget(Paragraph::new(bot).style(Style::default().fg(Color::Yellow)), Rect::new(sidebar_x, bot_y, sidebar_w, 1));
        }
    }

    // 提示文字 - 固定在屏幕底部
    let hint_row = area.bottom().saturating_sub(1);
    if !app.message.is_empty() {
        // Show message if present
        let msg_chars = app.message.chars().count() as u16;
        let msg_w = msg_chars.min(area.width);
        let msg_col = area.x + area.width.saturating_sub(msg_chars) / 2;
        f.render_widget(
            Paragraph::new(app.message.as_str()).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Rect::new(msg_col, hint_row, msg_w, 1),
        );
    } else {
        let hint = i18n::t("menu.hint_nav", lang);
        let hint_chars = hint.chars().count() as u16;
        let hint_w = hint_chars.min(area.width);
        let hint_col = area.x + area.width.saturating_sub(hint_chars) / 2;
        f.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::White)),
            Rect::new(hint_col, hint_row, hint_w, 1),
        );
    }
}

fn draw_menu_box(f: &mut Frame, x: u16, y: u16, w: u16, items: &[String], selected: usize, area: Rect) {
    let inner_w = w.saturating_sub(2) as usize; // 去掉两侧 ╭╮
    let _h = items.len() as u16 + 2; // 顶 + items + 底

    // 顶行 ╭─╮
    if y < area.bottom() {
        let top = format!("╭{}╮", "─".repeat(inner_w));
        f.render_widget(
            Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
            Rect::new(x, y, w, 1),
        );
    }

    // 内容行
    for (i, text) in items.iter().enumerate() {
        let row = y + 1 + i as u16;
        if row >= area.bottom() { break; }

        let prefix = if i == selected { "▸ " } else { "  " };
        let line_text = format!("{}{}", prefix, text);
        let line_dw = display_width(&line_text);
        let padding = inner_w.saturating_sub(2 + line_dw);
        let padded = format!(" {}{} ", line_text, " ".repeat(padding));

        let style = if i == selected {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };

        let line_span = Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Cyan)),
            Span::styled(padded, style),
            Span::styled("│", Style::default().fg(Color::Cyan)),
        ]);
        f.render_widget(Paragraph::new(line_span), Rect::new(x, row, w, 1));
    }

    // 底行 ╰─╯
    let bot_row = y + 1 + items.len() as u16;
    if bot_row < area.bottom() {
        let bot = format!("╰{}╯", "─".repeat(inner_w));
        f.render_widget(
            Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
            Rect::new(x, bot_row, w, 1),
        );
    }
}

// ── 游戏画面 ──

fn draw_game(f: &mut Frame, app: &App) {
    let area = f.area();
    let bg = parse_color(&app.settings.bg_color);
    let border = parse_color(&app.settings.border_color);
    // 整体背景
    f.render_widget(
        Block::default().style(Style::default().bg(bg)),
        area,
    );

    let layout = compute_game_layout_from_rect(f.area(), app);

    // 信息栏
    draw_info_panel(f, &layout, app, bg, border);
    // 数独网格
    draw_sudoku_grid(f, &layout, app, bg, border);
    // 消息行
    draw_message(f, &layout, app, bg);
    // 按钮栏
    draw_button_bar(f, &layout, app, bg, border);
    // 3D立方体
    if app.settings.show_cube == "yes" {
        draw_3d_cube(f, &layout, app);
    }
}

pub fn compute_game_layout_from_rect(area: Rect, app: &App) -> GameLayout {
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let grid_inner_w = (1 + 9 * (cw + 1)) as u16;
    let grid_inner_h = (1 + 9 * (ch + 1)) as u16;
    let _grid_w = grid_inner_w + 2; // 外边框左右各1
    let grid_h = grid_inner_h + 2; // 外边框上下各1

    // 信息栏高度 3 行（╭─╮ + 内容 + ╰─╯）
    let info_h = 3u16;
    // 消息行 1 行
    let msg_h = 1u16;
    // 按钮栏高度 3 行
    let btn_h = 3u16;
    let gap = 1u16;
    // Direction indicator borders add 2 rows (top + bottom)
    let dir_border_h = 2u16;

    // 右侧立方体宽度
    let _cube_w = 20u16;

    let total_h = info_h + gap + grid_h + dir_border_h + gap + msg_h + gap + btn_h;

    let vert_offset = area.height.saturating_sub(total_h) / 2;

    let info_y = area.y + vert_offset;
    let grid_y = info_y + info_h + gap + 1; // +1 for top direction border
    let msg_y = grid_y + grid_h + 1 + gap; // +1 for bottom direction border
    let btn_y = msg_y + msg_h + gap;

    // 网格靠左，右侧留给立方体
    let left_margin = 4u16; // space for direction border + outer frame + gap
    let grid_x = area.x + left_margin + 1; // +1 for outer border left

    // 信息栏宽度和位置
    let lang = Lang::from_code(&app.settings.language);
    let total = app.game.grid.cells.len();
    let filled = app.game.grid.cells.values().filter(|c| c.user_value.is_some()).count();
    let remaining = total - filled;
    let game_name = app.game.id.map_or(i18n::t("game.unnamed", lang).to_string(), |id| {
        if app.settings.naming_mode == "vivid" {
            i18n::vivid_name(id, lang)
        } else {
            format!("#{}", id)
        }
    });
    let info_text = format!(
        "  {} [{}] {}:{} {}:{} {}/{} {}:{}  ",
        game_name,
        mode_label(app.render_mode, lang),
        i18n::t("info.face", lang),
        face_name(app.current_face, lang),
        i18n::t("info.diff", lang),
        app.game.difficulty.as_str(),
        remaining,
        total,
        i18n::t("info.time", lang),
        format_timer(app)
    );
    let info_w = display_width(&info_text) as u16 + 4;

    let info_area = Rect::new(area.x, info_y, info_w.min(area.width), info_h);
    let grid_area = Rect::new(grid_x, grid_y, grid_inner_w, grid_inner_h);
    let msg_area = Rect::new(grid_x, msg_y, grid_inner_w, msg_h);
    let btn_area = Rect::new(area.x, btn_y, area.width, btn_h);

    // 3D立方体区域 - 网格右侧，留出方向边框空间
    let cube_w: u16 = app.settings.cube_width.parse().unwrap_or(20);
    let cube_h: u16 = app.settings.cube_height.parse().unwrap_or(18);
    let cube_x = grid_x + grid_inner_w + 5; // 网格外边框+方向边+间距
    let cube_area = Rect::new(cube_x, grid_y + 1, cube_w, cube_h); // +1 for top direction border

    // 按钮布局 - 计算居中后的起始列
    let btn_row = btn_y + 1; // 框内内容行

    // 先收集所有按钮的label和width
    let btn_defs: Vec<(String, ButtonId, u16)> = (1..=9u8)
        .map(|n| {
            let label = format!("[{}]", n);
            let w = label.chars().count() as u16;
            (label, ButtonId::Number(n), w)
        })
        .chain(
            [
                ("[X]Erase", ButtonId::Erase),
                ("[H]Hint", ButtonId::Hint),
                ("[Z]Undo", ButtonId::Undo),
                ("[G]Guide", ButtonId::ToggleGuidance),
                ("[M]Mode", ButtonId::ToggleMode),
                ("[Q]Menu", ButtonId::Quit),
            ].iter().map(|(label, id)| {
                let w = label.chars().count() as u16;
                (label.to_string(), *id, w)
            })
        )
        .collect();

    // 计算按钮内容总宽，与 draw_button_bar 一致
    let total_btn_w: usize = btn_defs.iter().map(|(_, _, w)| *w as usize + 1).sum::<usize>().saturating_sub(1);
    let inner_w = total_btn_w + 4; // 两侧空格
    let bar_w = inner_w + 2; // 两侧 │
    let bar_x = area.x + area.width.saturating_sub(bar_w as u16) / 2;

    // 按钮起始列 = bar_x + 1(│) + 1(空格)
    let mut col = bar_x + 2;
    let mut buttons = Vec::new();
    for (label, id, w) in btn_defs {
        buttons.push(ButtonLayout {
            id,
            label,
            col,
            row: btn_row,
            width: w,
        });
        col += w + 1;
    }

    GameLayout {
        grid_area,
        info_area,
        msg_area,
        btn_area,
        cube_area,
        buttons,
    }
}

fn draw_info_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let total = app.game.grid.cells.len();
    let filled = app.game.grid.cells.values().filter(|c| c.user_value.is_some()).count();
    let remaining = total - filled;
    let game_name = app.game.id.map_or(i18n::t("game.unnamed", lang).to_string(), |id| {
        if app.settings.naming_mode == "vivid" {
            i18n::vivid_name(id, lang)
        } else {
            format!("#{}", id)
        }
    });
    let info_text = format!(
        "  {} [{}] {}:{} {}:{} {}/{} {}:{}  ",
        game_name,
        mode_label(app.render_mode, lang),
        i18n::t("info.face", lang),
        face_name(app.current_face, lang),
        i18n::t("info.diff", lang),
        app.game.difficulty.as_str(),
        remaining,
        total,
        i18n::t("info.time", lang),
        format_timer(app)
    );
    let inner_w = display_width(&info_text);
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));

    let x = layout.info_area.x;
    let y = layout.info_area.y;

    // 顶行
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(border)),
        Rect::new(x, y, (inner_w + 2) as u16, 1),
    );
    // 内容行
    let line = Line::from(vec![
        Span::styled("│", Style::default().fg(border)),
        Span::styled(info_text, Style::default().fg(Color::White).bg(bg)),
        Span::styled("│", Style::default().fg(border)),
    ]);
    f.render_widget(
        Paragraph::new(line),
        Rect::new(x, y + 1, (inner_w + 2) as u16, 1),
    );
    // 底行 ╰─╯
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(border)),
        Rect::new(x, y + 2, (inner_w + 2) as u16, 1),
    );
}

fn draw_sudoku_grid(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let ox = layout.grid_area.x;
    let oy = layout.grid_area.y;

    // 外边框 - 颜色对应当前面
    let face_color = face_to_color(app.current_face);
    let grid_w = layout.grid_area.width;
    let grid_h = layout.grid_area.height;

    // 顶
    let top_outer = format!("╔{}╗", "═".repeat(grid_w as usize));
    f.render_widget(
        Paragraph::new(top_outer).style(Style::default().fg(face_color)),
        Rect::new(ox.saturating_sub(1), oy.saturating_sub(1), grid_w + 2, 1),
    );
    // 左右 + 内容区域两侧
    for row in 0..grid_h {
        let y = oy + row;
        f.render_widget(
            Paragraph::new("║").style(Style::default().fg(face_color)),
            Rect::new(ox.saturating_sub(1), y, 1, 1),
        );
        f.render_widget(
            Paragraph::new("║").style(Style::default().fg(face_color)),
            Rect::new(ox + grid_w, y, 1, 1),
        );
    }
    // 底
    let bot_outer = format!("╚{}╝", "═".repeat(grid_w as usize));
    f.render_widget(
        Paragraph::new(bot_outer).style(Style::default().fg(face_color)),
        Rect::new(ox.saturating_sub(1), oy + grid_h, grid_w + 2, 1),
    );

    // 逐行逐列绘制网格
    for v in 0..9usize {
        // 分隔行
        let sep_y = oy + v as u16 * (ch as u16 + 1);
        draw_separator(f, ox, sep_y, cw, v == 0, false, v % 3 == 0, layout.grid_area, border);

        // 单元格内容行
        for line in 0..ch {
            let row_y = oy + v as u16 * (ch as u16 + 1) + 1 + line as u16;
            draw_cell_row(f, ox, row_y, cw, ch, v, line, app, layout.grid_area, bg, border);
        }
    }

    // 底部分隔行
    let bot_y = oy + 9 * (ch as u16 + 1);
    draw_separator(f, ox, bot_y, cw, false, true, true, layout.grid_area, border);

    // ── Direction indicator borders ──
    // Draw 4 colored edges outside the outer frame to show which face is in each direction
    let outer_x = ox.saturating_sub(1); // left edge of outer frame
    let outer_y = oy.saturating_sub(1); // top edge of outer frame
    let outer_w = grid_w + 2;           // total outer width
    let outer_h = grid_h + 2;           // total outer height

    // Compute neighbor faces based on current face (WASD edge-crossing)
    let (up_face, down_face, left_face, right_face, back_face) = wasd_neighbor_faces(app.current_face);
    let up_color = face_to_color(up_face);
    let down_color = face_to_color(down_face);
    let left_color = face_to_color(left_face);
    let right_color = face_to_color(right_face);
    let back_color = face_to_color(back_face);

    // Top direction border
    if outer_y > 0 {
        let dir_y = outer_y.saturating_sub(1);
        let top_line = "▔".repeat(outer_w as usize);
        f.render_widget(
            Paragraph::new(top_line).style(Style::default().fg(up_color)),
            Rect::new(outer_x, dir_y, outer_w, 1),
        );
    }

    // Bottom direction border
    {
        let dir_y = outer_y + outer_h;
        let bot_line = "▁".repeat(outer_w as usize);
        f.render_widget(
            Paragraph::new(bot_line).style(Style::default().fg(down_color)),
            Rect::new(outer_x, dir_y, outer_w, 1),
        );
    }

    // Left direction border
    if outer_x > 0 {
        let dir_x = outer_x.saturating_sub(1);
        for row in 0..outer_h {
            f.render_widget(
                Paragraph::new("▏").style(Style::default().fg(left_color)),
                Rect::new(dir_x, outer_y + row, 1, 1),
            );
        }
    }

    // Right direction border
    {
        let dir_x = outer_x + outer_w;
        for row in 0..outer_h {
            f.render_widget(
                Paragraph::new("▕").style(Style::default().fg(right_color)),
                Rect::new(dir_x, outer_y + row, 1, 1),
            );
        }
    }

    // ── Back-face indicator: solid block at bottom-right corner ──
    {
        let indicator_x = outer_x + outer_w + 1;
        let indicator_y = outer_y + outer_h;
        f.render_widget(
            Paragraph::new("■").style(Style::default().fg(back_color)),
            Rect::new(indicator_x, indicator_y, 1, 1),
        );
    }
}

fn draw_separator(
    f: &mut Frame, x: u16, y: u16, cw: usize,
    is_top: bool, is_bottom: bool, is_thick_h: bool,
    bounds: Rect, border: Color,
) {
    if y >= bounds.bottom() { return; }

    let mut buf = String::new();
    // 左角
    buf.push(if is_top { '╔' } else if is_bottom { '╚' } else { '╟' });

    for u in 0..9usize {
        let h_char = if is_thick_h { '═' } else { '─' };
        buf.push_str(&h_char.to_string().repeat(cw));

        if u < 8 {
            let is_major_v = (u + 1) % 3 == 0;
            buf.push(if is_top {
                if is_major_v { '╦' } else { '╤' }
            } else if is_bottom {
                if is_major_v { '╩' } else { '╧' }
            } else if is_thick_h {
                if is_major_v { '╬' } else { '╪' }
            } else if is_major_v {
                '╫'
            } else {
                '┼'
            });
        }
    }

    // 右角
    buf.push(if is_top { '╗' } else if is_bottom { '╝' } else { '╢' });

    let w = buf.chars().count() as u16;
    f.render_widget(
        Paragraph::new(buf).style(Style::default().fg(border)),
        Rect::new(x, y, w.min(bounds.width), 1),
    );
}

fn draw_cell_row(
    f: &mut Frame, x: u16, y: u16, cw: usize, ch: usize,
    v: usize, line: usize, app: &App, bounds: Rect, _bg: Color, border: Color,
) {
    if y >= bounds.bottom() { return; }

    let mut spans: Vec<Span> = Vec::new();

    for u in 0..9usize {
        // 竖线
        let v_char = if u % 3 == 0 { '║' } else { '│' };
        spans.push(Span::styled(
            v_char.to_string(),
            Style::default().fg(border),
        ));

        // 单元格内容
        let coord = app.current_face.to_cube(u as u8, v as u8);
        let cell = app.game.grid.get(&coord);
        let selected = app.cursor == (u as u8, v as u8);
        let is_given = cell.map(|c| c.given).unwrap_or(false);
        let value = cell.and_then(|c| c.user_value);
        let is_error = value.map_or(false, |n| is_wrong(app, coord, n));

        let (in_same_group, has_same_number) = if app.guidance && !selected {
            let sel_coord = app.current_face.to_cube(app.cursor.0, app.cursor.1);
            let same_row = app.cursor.1 == v as u8;
            let same_col = app.cursor.0 == u as u8;
            let same_box = app.cursor.0 / 3 == u as u8 / 3 && app.cursor.1 / 3 == v as u8 / 3;
            let in_group = same_row || same_col || same_box;
            let sel_value = app.game.grid.get(&sel_coord).and_then(|c| c.user_value);
            let same_num = value.is_some() && value == sel_value;
            (in_group, same_num)
        } else {
            (false, false)
        };

        let mid_line = ch / 2;
        let mut content = " ".repeat(cw);
        if line == mid_line {
            if let Some(n) = value {
                let s = ((b'0' + n) as char).to_string();
                let start = (cw - 1) / 2;
                content.replace_range(start..start + 1, &s);
            }
        }

        let guide_group = parse_color(&app.settings.guide_group_color);
        let guide_same = parse_color(&app.settings.guide_same_color);
        let style = if selected && is_error {
            Style::default().bg(Color::White).fg(Color::Red)
        } else if selected && app.blink_on {
            Style::default().bg(Color::White).fg(Color::Black)
        } else if selected {
            Style::default().bg(Color::Gray).fg(Color::White)
        } else if in_same_group && has_same_number {
            Style::default().bg(guide_same).fg(Color::White)
        } else if in_same_group {
            Style::default().bg(guide_group).fg(Color::White)
        } else if has_same_number {
            Style::default().bg(guide_same).fg(Color::White)
        } else if is_error {
            Style::default().fg(Color::Red)
        } else if is_given {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if value.is_some() {
            Style::default().fg(border)
        } else {
            Style::default().fg(Color::White)
        };

        spans.push(Span::styled(content, style));
    }

    // 右侧封口
    spans.push(Span::styled("║".to_string(), Style::default().fg(border)));

    let total_w: u16 = spans.iter().map(|s| s.content.chars().count() as u16).sum();
    f.render_widget(
        Paragraph::new(Line::from(spans)),
        Rect::new(x, y, total_w.min(bounds.width), 1),
    );
}

fn draw_message(f: &mut Frame, layout: &GameLayout, app: &App, _bg: Color) {
    if app.message.is_empty() { return; }
    let style = Style::default().fg(Color::Green);
    f.render_widget(
        Paragraph::new(app.message.as_str()).style(style),
        layout.msg_area,
    );
}

fn draw_button_bar(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let x = layout.btn_area.x;
    let y = layout.btn_area.y;
    let w = layout.btn_area.width;

    if w < 4 || layout.btn_area.height < 3 { return; }

    // 计算按钮内容总宽
    let total_btn_w: usize = layout.buttons.iter().map(|b| b.width as usize + 1).sum::<usize>().saturating_sub(1);
    let inner_w = total_btn_w + 4; // 两侧空格
    let bar_w = inner_w + 2; // 两侧 │

    let bar_x = x + w.saturating_sub(bar_w as u16) / 2;

    // 顶行 ╭─╮
    let top = format!("╭{}╮", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(border)),
        Rect::new(bar_x, y, bar_w as u16, 1),
    );

    // 内容行：按钮
    let mut spans: Vec<Span> = vec![Span::styled("│", Style::default().fg(border)), Span::raw(" ")];
    for btn in &layout.buttons {
        let is_hover = app.hover_button == Some(btn.id);
        let style = if is_hover {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(bg).fg(Color::White)
        };
        spans.push(Span::styled(&btn.label, style));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled("│", Style::default().fg(border)));

    f.render_widget(
        Paragraph::new(Line::from(spans)),
        Rect::new(bar_x, y + 1, bar_w as u16, 1),
    );

    // 底行 ╰─╯
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(border)),
        Rect::new(bar_x, y + 2, bar_w as u16, 1),
    );
}

// ── 3D 立方体 ──

fn draw_3d_cube(f: &mut Frame, layout: &GameLayout, app: &App) {
    let area = layout.cube_area;
    if area.width < 10 || area.height < 10 { return; }

    // ── Arrow-key direction indicator border (outer) ──
    let (up_face, down_face, left_face, right_face, _back_face) = arrow_neighbor_faces(app.current_face);
    let up_color = face_to_color(up_face);
    let down_color = face_to_color(down_face);
    let left_color = face_to_color(left_face);
    let right_color = face_to_color(right_face);

    if area.x > 0 && area.y > 0 {
        // Top arrow border
        let top_line = "▔".repeat(area.width as usize);
        f.render_widget(
            Paragraph::new(top_line).style(Style::default().fg(up_color)),
            Rect::new(area.x, area.y.saturating_sub(1), area.width, 1),
        );
        // Bottom arrow border
        let bot_line = "▁".repeat(area.width as usize);
        f.render_widget(
            Paragraph::new(bot_line).style(Style::default().fg(down_color)),
            Rect::new(area.x, area.y + area.height, area.width, 1),
        );
        // Left arrow border
        let dir_x = area.x.saturating_sub(1);
        for row in 0..area.height {
            f.render_widget(
                Paragraph::new("▏").style(Style::default().fg(left_color)),
                Rect::new(dir_x, area.y + row, 1, 1),
            );
        }
        // Right arrow border
        for row in 0..area.height {
            f.render_widget(
                Paragraph::new("▕").style(Style::default().fg(right_color)),
                Rect::new(area.x + area.width, area.y + row, 1, 1),
            );
        }
    }

    // 绘制边框
    let border_color = face_to_color(app.current_face);
    let inner_w = area.width as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(border_color)),
        Rect::new(area.x, area.y, area.width, 1),
    );
    for row in 1..area.height.saturating_sub(1) {
        f.render_widget(
            Paragraph::new("│").style(Style::default().fg(border_color)),
            Rect::new(area.x, area.y + row, 1, 1),
        );
        f.render_widget(
            Paragraph::new("│").style(Style::default().fg(border_color)),
            Rect::new(area.x + area.width - 1, area.y + row, 1, 1),
        );
    }
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(border_color)),
        Rect::new(area.x, area.y + area.height - 1, area.width, 1),
    );

    // 内容区域（去掉边框）
    let content_area = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));

    let cx = content_area.x as f64 + content_area.width as f64 / 2.0;
    let cy = content_area.y as f64 + content_area.height as f64 / 2.0;
    let scale_factor: f64 = app.settings.cube_scale.parse().unwrap_or(0.38);
    // 终端字符高宽比约2:1，Y方向缩小以补偿
    let scale_x = content_area.width as f64 * scale_factor;
    let scale_y = content_area.height as f64 * scale_factor * 0.5;

    let cos_y = app.cube_angle_y.cos();
    let sin_y = app.cube_angle_y.sin();
    let cos_x = app.cube_angle_x.cos();
    let sin_x = app.cube_angle_x.sin();

    // 8 vertices of a unit cube
    let verts: [(f64, f64, f64); 8] = [
        (-1.0, -1.0, -1.0), ( 1.0, -1.0, -1.0),
        ( 1.0,  1.0, -1.0), (-1.0,  1.0, -1.0),
        (-1.0, -1.0,  1.0), ( 1.0, -1.0,  1.0),
        ( 1.0,  1.0,  1.0), (-1.0,  1.0,  1.0),
    ];

    // Project: rotate Y then X, with aspect ratio correction
    let project = |x: f64, y: f64, z: f64| {
        // Rotate around Y axis
        let x1 = x * cos_y + z * sin_y;
        let z1 = -x * sin_y + z * cos_y;
        // Rotate around X axis
        let y1 = y * cos_x - z1 * sin_x;
        let z2 = y * sin_x + z1 * cos_x;
        (cx + x1 * scale_x, cy - y1 * scale_y, z2)
    };

    let proj: Vec<(f64, f64, f64)> = verts.iter().map(|&(x, y, z)| project(x, y, z)).collect();

    // 6 faces: vertex indices + color
    let faces: [([usize; 4], Face); 6] = [
        ([4, 5, 6, 7], Face::Front),  // front (z=1)
        ([0, 3, 2, 1], Face::Back),   // back (z=-1)
        ([0, 4, 7, 3], Face::Left),   // left (x=-1)
        ([1, 2, 6, 5], Face::Right),  // right (x=1)
        ([3, 7, 6, 2], Face::Top),    // top (y=1)
        ([0, 1, 5, 4], Face::Bottom), // bottom (y=-1)
    ];

    // Sort faces by average z (painter's algorithm, far first)
    let mut sorted_faces: Vec<([usize; 4], Face)> = faces.to_vec();
    sorted_faces.sort_by(|a, b| {
        let za: f64 = a.0.iter().map(|&i| proj[i].2).sum::<f64>() / 4.0;
        let zb: f64 = b.0.iter().map(|&i| proj[i].2).sum::<f64>() / 4.0;
        za.partial_cmp(&zb).unwrap()
    });

    // Draw faces
    for (indices, face) in &sorted_faces {
        let color = face_to_color(*face);
        let pts: Vec<(f64, f64)> = indices.iter().map(|&i| (proj[i].0, proj[i].1)).collect();

        // Fill face with colored dots
        let min_x = pts.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let max_x = pts.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        let min_y = pts.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let max_y = pts.iter().map(|p| p.1).fold(f64::MIN, f64::max);

        for py in (min_y as u16)..=(max_y as u16) {
            for px in (min_x as u16)..=(max_x as u16) {
                if px < content_area.x || px >= content_area.x + content_area.width { continue; }
                if py < content_area.y || py >= content_area.y + content_area.height { continue; }
                if point_in_quad(px as f64, py as f64, &pts) {
                    let style = if *face == app.current_face {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };
                    let ch = if *face == app.current_face { '●' } else { '░' };
                    f.render_widget(
                        Paragraph::new(ch.to_string()).style(style),
                        Rect::new(px, py, 1, 1),
                    );
                }
            }
        }

        // Draw edges
        for i in 0..4 {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % 4];
            draw_line(f, x1, y1, x2, y2, color, content_area);
        }
    }

    // Draw orbiting sphere - position on the face's center, projected outward
    let face_center_3d = match app.current_face {
        Face::Front  => (0.0, 0.0, 1.8),
        Face::Back   => (0.0, 0.0, -1.8),
        Face::Left   => (-1.8, 0.0, 0.0),
        Face::Right  => (1.8, 0.0, 0.0),
        Face::Top    => (0.0, 1.8, 0.0),
        Face::Bottom => (0.0, -1.8, 0.0),
    };
    let (sx, sy, _) = project(face_center_3d.0, face_center_3d.1, face_center_3d.2);
    let sphere_color = face_to_color(app.current_face);

    let sx_u = sx as u16;
    let sy_u = sy as u16;
    if sx_u >= content_area.x && sx_u < content_area.x + content_area.width
        && sy_u >= content_area.y && sy_u < content_area.y + content_area.height {
        f.render_widget(
            Paragraph::new("◉").style(Style::default().fg(sphere_color).add_modifier(Modifier::BOLD)),
            Rect::new(sx_u, sy_u, 1, 1),
        );
    }

    // Face label - inside border at bottom
    let lang = Lang::from_code(&app.settings.language);
    let label = face_name(app.current_face, lang);
    let label_x = area.x + area.width.saturating_sub(label.len() as u16) / 2;
    let label_y = area.y + area.height.saturating_sub(1); // 底部边框上一行
    if label_y > area.y {
        f.render_widget(
            Paragraph::new(label).style(Style::default().fg(face_to_color(app.current_face))),
            Rect::new(label_x, label_y, label.len() as u16, 1),
        );
    }
}

fn point_in_quad(px: f64, py: f64, pts: &[(f64, f64)]) -> bool {
    if pts.len() != 4 { return false; }
    // Cross product method
    let mut inside = true;
    for i in 0..4 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % 4];
        let cross = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if cross > 0.0 { inside = false; break; }
    }
    if inside { return true; }
    inside = true;
    for i in 0..4 {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % 4];
        let cross = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
        if cross < 0.0 { inside = false; break; }
    }
    inside
}

fn draw_line(f: &mut Frame, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, bounds: Rect) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let steps = dx.max(dy).ceil() as u16;
    if steps == 0 { return; }

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = (x1 + (x2 - x1) * t) as u16;
        let y = (y1 + (y2 - y1) * t) as u16;
        if x >= bounds.x && x < bounds.x + bounds.width && y >= bounds.y && y < bounds.y + bounds.height {
            f.render_widget(
                Paragraph::new("·").style(Style::default().fg(color)),
                Rect::new(x, y, 1, 1),
            );
        }
    }
}

// ── 公共工具函数 ──

pub fn find_button_at(layout: &GameLayout, col: u16, row: u16) -> Option<ButtonId> {
    for btn in &layout.buttons {
        if row == btn.row && col >= btn.col && col < btn.col + btn.width {
            return Some(btn.id);
        }
    }
    None
}

pub fn cell_at(layout: &GameLayout, cw: usize, ch: usize, col: u16, row: u16) -> Option<(u8, u8)> {
    let gx = col.saturating_sub(layout.grid_area.x);
    let gy = row.saturating_sub(layout.grid_area.y);

    if gy % (ch as u16 + 1) == 0 { return None; }
    let v = (gy / (ch as u16 + 1)) as u8;
    if v >= 9 { return None; }

    if gx % (cw as u16 + 1) == 0 { return None; }
    let u = (gx / (cw as u16 + 1)) as u8;
    if u >= 9 { return None; }

    Some((u, v))
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

fn format_timer(app: &App) -> String {
    let elapsed = total_elapsed(app);
    format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
}

fn is_wrong(app: &App, coord: sudokube_core::cube::CubeCoord, value: u8) -> bool {
    app.game.solution.get(&coord).map_or(true, |&sol| sol != value)
}

fn face_name(face: Face, lang: Lang) -> &'static str {
    match face {
        Face::Front => i18n::t("game.face_front", lang),
        Face::Back => i18n::t("game.face_back", lang),
        Face::Left => i18n::t("game.face_left", lang),
        Face::Right => i18n::t("game.face_right", lang),
        Face::Top => i18n::t("game.face_top", lang),
        Face::Bottom => i18n::t("game.face_bottom", lang),
    }
}

fn face_to_color(face: Face) -> Color {
    match face {
        Face::Front => Color::Red,
        Face::Back => Color::Blue,
        Face::Left => Color::Green,
        Face::Right => Color::Yellow,
        Face::Top => Color::Magenta,
        Face::Bottom => Color::Cyan,
    }
}

/// WASD edge-crossing neighbors: which face you reach when cursor moves off the edge.
/// Based on move_on_surface logic in input.rs.
fn wasd_neighbor_faces(face: Face) -> (Face, Face, Face, Face, Face) {
    match face {
        Face::Front  => (Face::Bottom, Face::Top, Face::Left, Face::Right, Face::Back),
        Face::Back   => (Face::Left, Face::Right, Face::Bottom, Face::Top, Face::Front),
        Face::Top    => (Face::Back, Face::Front, Face::Left, Face::Right, Face::Bottom),
        Face::Bottom => (Face::Left, Face::Right, Face::Back, Face::Front, Face::Top),
        Face::Left   => (Face::Bottom, Face::Top, Face::Back, Face::Front, Face::Right),
        Face::Right  => (Face::Back, Face::Front, Face::Bottom, Face::Top, Face::Left),
    }
}

/// Arrow-key face-switching neighbors: which face you switch to with ↑↓←→.
/// Based on switch_face logic in input.rs.
fn arrow_neighbor_faces(face: Face) -> (Face, Face, Face, Face, Face) {
    match face {
        Face::Front  => (Face::Top, Face::Bottom, Face::Left, Face::Right, Face::Back),
        Face::Back   => (Face::Top, Face::Bottom, Face::Right, Face::Left, Face::Front),
        Face::Top    => (Face::Back, Face::Front, Face::Left, Face::Right, Face::Bottom),
        Face::Bottom => (Face::Front, Face::Back, Face::Left, Face::Right, Face::Top),
        Face::Left   => (Face::Top, Face::Bottom, Face::Back, Face::Front, Face::Right),
        Face::Right  => (Face::Top, Face::Bottom, Face::Front, Face::Back, Face::Left),
    }
}

pub fn parse_color(name: &str) -> Color {
    match name {
        "black" => Color::Black,
        "darkgray" => Color::DarkGray,
        "white" => Color::White,
        "cyan" => Color::Cyan,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "red" => Color::Red,
        "yellow" => Color::Yellow,
        "gray" => Color::Gray,
        _ => Color::Cyan,
    }
}

// ── 设置画面 ──

fn draw_settings(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let box_w = 42u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let fields = &app.settings_ui.fields;
    let content_h = fields.len() as u16;
    let box_h = content_h + 4; // title(2) + content + hint(1) + bottom(1)
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;

    // 标题
    let inner_w = box_w as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, box_y, box_w, 1),
    );

    let title_line = format!("│{:^width$}│", i18n::t("settings.title", lang), width = inner_w);
    f.render_widget(
        Paragraph::new(title_line).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, box_y + 1, box_w, 1),
    );

    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, box_y + 2, box_w, 1),
    );

    // 设置项
    for (i, field) in fields.iter().enumerate() {
        let row = box_y + 3 + i as u16;
        let is_selected = i == app.settings_ui.selected;
        // Translate special values
        let display_value = if field.label == "Naming Mode" {
            i18n::t(&format!("naming.{}", field.value), lang).to_string()
        } else {
            field.value.clone()
        };
        let label_part = format!(" {} ", field.label);
        let value_part = format!(" < {} >", display_value);
        let padding = inner_w.saturating_sub(display_width(&label_part) + display_width(&value_part));
        let line_text = format!("{}{}{}", label_part, " ".repeat(padding), value_part);

        let style = if is_selected {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };

        let line = Line::from(vec![
            Span::styled("│", Style::default().fg(Color::Cyan)),
            Span::styled(line_text, style),
            Span::styled("│", Style::default().fg(Color::Cyan)),
        ]);
        f.render_widget(Paragraph::new(line), Rect::new(box_x, row, box_w, 1));
    }

    // 提示
    let hint_row = box_y + 3 + content_h;
    let hint = format!("│{:^width$}│", i18n::t("settings.hint", lang), width = inner_w);
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        Rect::new(box_x, hint_row, box_w, 1),
    );

    // 底部
    let bot_row = hint_row + 1;
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(box_x, bot_row, box_w, 1),
    );
}

// ── 胜利画面 ──

fn draw_victory(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let lang = Lang::from_code(&app.settings.language);
    let title = i18n::t("victory.title", lang);
    let subtitle = i18n::t("victory.subtitle", lang);

    // Calculate remaining seconds
    let remaining_secs = app.victory_countdown
        .map(|until| {
            let left = (until - Instant::now()).as_secs() as u32;
            left.max(0)
        })
        .unwrap_or(0);

    let countdown_text = format!("{}  |  {}", i18n::t("victory.enter", lang), format!("{}", remaining_secs));

    let box_w = 36u16;
    let box_h = 6u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;

    let inner_w = box_w as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));

    // Top
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y, box_w, 1),
    );
    // Title
    let title_line = format!("│{:^width$}│", title, width = inner_w);
    f.render_widget(
        Paragraph::new(title_line).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Rect::new(box_x, box_y + 1, box_w, 1),
    );
    // Subtitle
    let sub_line = format!("│{:^width$}│", subtitle, width = inner_w);
    f.render_widget(
        Paragraph::new(sub_line).style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Rect::new(box_x, box_y + 2, box_w, 1),
    );
    // Separator
    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y + 3, box_w, 1),
    );
    // Info
    let diff = app.game.difficulty.as_str();
    let elapsed = total_elapsed(app);
    let info_line = format!("│ {:^width$} │", format!("{}  {:02}:{:02}", diff, elapsed / 60, elapsed % 60), width = inner_w - 2);
    f.render_widget(
        Paragraph::new(info_line).style(Style::default().fg(Color::White)),
        Rect::new(box_x, box_y + 4, box_w, 1),
    );
    // Bottom + countdown
    let bot_line = format!("│{:^width$}│", countdown_text, width = inner_w);
    f.render_widget(
        Paragraph::new(bot_line).style(Style::default().fg(Color::DarkGray)),
        Rect::new(box_x, box_y + 5, box_w, 1),
    );
    // Bottom border
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y + box_h, box_w, 1),
    );
}

// ── 导出选择画面 ──

fn draw_export_select(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let box_w = 34u16;
    let box_h = 5u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;
    let inner_w = box_w as usize - 2;

    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    let title = format!("│{:^width$}│", i18n::t("menu.export", lang), width = inner_w);

    f.render_widget(Paragraph::new(top).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y, box_w, 1));
    f.render_widget(Paragraph::new(title).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 1, box_w, 1));

    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(Paragraph::new(sep).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 2, box_w, 1));

    let opts = [
        i18n::t("export.encrypted", lang),
        i18n::t("export.plaintext", lang),
    ];
    for (i, label) in opts.iter().enumerate() {
        let is_sel = i == app.export_select;
        let prefix = if is_sel { ">" } else { " " };
        let text = format!("{} {}", prefix, label);
        let style = if is_sel {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };
        let line = format!("│{:<width$}│", text, width = inner_w);
        f.render_widget(Paragraph::new(line).style(style), Rect::new(box_x, box_y + 3 + i as u16, box_w, 1));
    }
    f.render_widget(Paragraph::new(bot).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + box_h, box_w, 1));
}

// ── 导入输入画面 ──

fn draw_import_input(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let box_w = 50u16;
    let box_h = 5u16;
    let box_x = area.x + area.width.saturating_sub(box_w) / 2;
    let box_y = area.y + area.height.saturating_sub(box_h) / 2;
    let inner_w = box_w as usize - 2;

    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    let title = format!("│{:^width$}│", i18n::t("menu.import", lang), width = inner_w);

    f.render_widget(Paragraph::new(top).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y, box_w, 1));
    f.render_widget(Paragraph::new(title).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 1, box_w, 1));

    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(Paragraph::new(sep).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + 2, box_w, 1));

    // Input field
    let display_text = if app.import_buffer.is_empty() {
        i18n::t("import.paste", lang).to_string()
    } else {
        let buf = &app.import_buffer;
        let max_chars = inner_w.saturating_sub(6); // reserve for " ..." and padding
        if buf.chars().count() > max_chars {
            let truncated: String = buf.chars().skip(buf.chars().count().saturating_sub(max_chars)).collect();
            format!("...{}", truncated)
        } else {
            buf.clone()
        }
    };
    let input_style = if app.import_buffer.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let input_line = format!("│ {:<width$} │", display_text, width = inner_w - 2);
    f.render_widget(Paragraph::new(input_line).style(input_style), Rect::new(box_x, box_y + 3, box_w, 1));

    let hint = format!("│{:^width$}│", "Enter OK / Esc Cancel", width = inner_w);
    f.render_widget(Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)), Rect::new(box_x, box_y + 4, box_w, 1));

    f.render_widget(Paragraph::new(bot).style(Style::default().fg(Color::Cyan)), Rect::new(box_x, box_y + box_h, box_w, 1));
}

// ── 生成进度画面 ──

fn draw_generating(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let spinners = ['⠋', '⠙', '⠹', '⠸'];
    let spinner_char = if let Some(ref gen_state) = app.generating {
        spinners[gen_state.spinner as usize]
    } else {
        '⠋'
    };

    let elapsed = if let Some(ref gen_state) = app.generating {
        gen_state.started.elapsed().as_secs()
    } else {
        0
    };

    let lang = Lang::from_code(&app.settings.language);
    let text = format!("  {} {} {}s  ", spinner_char, i18n::t("msg.generating", lang), elapsed);
    let bar_w = 40u16;
    let x = area.x + area.width.saturating_sub(bar_w) / 2;
    let y = area.y + area.height / 2;

    let inner_w = bar_w as usize - 2;
    let top = format!("╭{}╮", "─".repeat(inner_w));
    let bot = format!("╰{}╯", "─".repeat(inner_w));

    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y - 1, bar_w, 1),
    );

    // 进度条行
    let progress_chars = "█▓▒░";
    let anim_step = (elapsed as usize) % 4;
    let filled = inner_w.min((elapsed as usize * 3).min(inner_w));
    let mut bar = String::new();
    for i in 0..inner_w {
        if i < filled {
            bar.push('█');
        } else if i == filled {
            bar.push(progress_chars.chars().nth(anim_step).unwrap());
        } else {
            bar.push('░');
        }
    }
    let bar_line = format!("│{}│", bar);
    f.render_widget(
        Paragraph::new(bar_line).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y, bar_w, 1),
    );

    // 文字行
    let content = format!("│{:^width$}│", text, width = inner_w);
    f.render_widget(
        Paragraph::new(content).style(Style::default().fg(Color::White)),
        Rect::new(x, y + 1, bar_w, 1),
    );

    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y + 2, bar_w, 1),
    );
}
