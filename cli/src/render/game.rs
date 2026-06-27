use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::i18n::{self, Lang};
use crate::shop;
use crate::{App, AppScreen};
use super::types::*;
use super::util::*;

pub(super) fn draw_game(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let bg = parse_color(&app.settings.bg_color);
    let border = parse_color(&app.settings.border_color);
    // 整体背景
    f.render_widget(Block::default().style(Style::default().bg(bg)), area);

    let layout = compute_game_layout_from_rect(f.area(), app);

    // 左侧：Status / Navigator / Logs
    draw_status_panel(f, &layout, app, bg, border);
    draw_navigator_panel(f, &layout, app, bg, border);
    draw_logs_panel(f, &layout, app, bg, border);

    // 中间：数独面板（含外层 Block + 方向指示 + 网格 + 消息 + 按钮）
    draw_sudoku_grid(f, &layout, app, bg, border);
    draw_message(f, &layout, app, bg);

    // 按钮栏
    draw_button_bar(f, &layout, app, bg, border);

    // 右侧：3D 立方体面板（含外层 Block + 方向指示 + 旋转 cube）
    if app.settings.show_cube == "yes" {
        super::cube3d::draw_3d_cube(f, &layout, app);
    }
    draw_shop_panel(f, &layout, app, bg, border);

    // Popup 叠加层
    if app.screen == AppScreen::Settings {
        super::settings::draw_settings_overlay(f, app);
    } else if app.screen == AppScreen::ImportInput {
        super::overlay::draw_import_overlay(f, app);
    }
}

pub fn compute_game_layout_from_rect(area: Rect, app: &mut App) -> GameLayout {
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let grid_inner_w = (1 + 9 * (cw + 1)) as u16;
    let grid_inner_h = (1 + 9 * (ch + 1)) as u16;
    let grid_h = grid_inner_h + 2; // 数独外框上下各 1
    let grid_w = grid_inner_w + 2; // 数独外框左右各 1

    // 布局参数
    let dir_border: u16 = 1;
    let msg_h = 1u16;
    let btn_h = 3u16;
    let gap = 1u16;

    // 中间面板最小宽度 = 网格 + 两侧方向边
    let center_panel_w = grid_w + dir_border * 2;

    // 列宽配置
    let left_min_w: u16 = 24;
    let left_max_w: u16 = 30;
    let cube_w_cfg: u16 = app.settings.cube_width.parse().unwrap_or(20);
    let right_min_w: u16 = cube_w_cfg + 4;

    // ── 纵向总布局：使用 Flex::Legacy 撑满画面 ──
    let outer_v = Layout::vertical([Constraint::Fill(1)])
        .flex(ratatui::layout::Flex::Legacy)
        .split(area);
    let v_chunk = outer_v[0];

    // ── 横向三列布局：使用 Flex 撑满画面 ──
    let min_w_sum = left_min_w
        .saturating_add(right_min_w)
        .saturating_add(center_panel_w);

    let (left_chunk, center_chunk, right_chunk) = if area.width < min_w_sum {
        // 太窄：只显示中间列，居中
        let h_layout = Layout::horizontal([Constraint::Length(center_panel_w)])
            .flex(ratatui::layout::Flex::Center);
        let h_chunks = h_layout.split(v_chunk);
        (Rect::default(), h_chunks[0], Rect::default())
    } else {
        // 计算左侧实际宽度
        let total = area.width;
        let left_actual = total
            .saturating_sub(center_panel_w + right_min_w)
            .min(left_max_w)
            .max(left_min_w);
        let side_total = total.saturating_sub(left_actual);
        let middle_actual = side_total * 3 / 4;
        let center_w = middle_actual.max(center_panel_w);
        let right_w = (total - left_actual - center_w).max(right_min_w);

        // 使用 Flex::Legacy 撑满画面
        let h_layout = Layout::horizontal([
            Constraint::Length(left_actual),
            Constraint::Length(center_w),
            Constraint::Length(right_w),
        ])
        .flex(ratatui::layout::Flex::Legacy);
        let h_chunks = h_layout.split(v_chunk);
        (h_chunks[0], h_chunks[1], h_chunks[2])
    };

    build_layout(
        app,
        left_chunk,
        center_chunk,
        right_chunk,
        center_panel_w,
        dir_border,
        grid_h,
        grid_w,
        msg_h,
        btn_h,
        gap,
        cube_w_cfg,
        area,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_layout(
    app: &mut App,
    left_chunk: Rect,
    center_chunk: Rect,
    right_chunk: Rect,
    _center_w: u16,
    dir_border: u16,
    grid_h: u16,
    grid_w: u16,
    msg_h: u16,
    btn_h: u16,
    gap: u16,
    _cube_w: u16,
    area: Rect,
) -> GameLayout {
    // 新布局结构：center_column = sudoku_panel + btn_area (垂直排列)
    // sudoku_panel = 外框 → 中间层 ▒ → 内框(隐藏) → 数独网格(居中)

    // ── 按钮栏移到 center_column 外部 ──
    let btn_area_h = btn_h;
    let btn_gap = 1u16;
    let center_v_layout = Layout::vertical([
        Constraint::Fill(1),            // sudoku_panel (自适应)
        Constraint::Length(btn_gap),    // gap
        Constraint::Length(btn_area_h), // btn_area
    ]);
    let center_v_parts = center_v_layout.split(center_chunk);
    let sudoku_panel = center_v_parts[0];
    let _btn_gap = center_v_parts[1];
    let btn_area = center_v_parts[2];

    // ── sudoku_panel 内部布局：外框 → 中间层 ▒ → 内框(隐藏) → 数独(居中) ──
    // 第一步：去掉外框（Block 边框）得到 inner1
    let sudoku_inner1 = Rect::new(
        sudoku_panel.x + 1,
        sudoku_panel.y + 1,
        sudoku_panel.width.saturating_sub(2),
        sudoku_panel.height.saturating_sub(2),
    );

    // 第二步：内框尺寸比外框左右各 -4，上下各 -2（内框隐藏不显示）
    let inner_left_offset = 4u16;
    let inner_top_offset = 2u16;
    let sudoku_inner_frame = Rect::new(
        sudoku_panel.x + 1 + inner_left_offset,
        sudoku_panel.y + 1 + inner_top_offset,
        sudoku_panel.width.saturating_sub(2 + inner_left_offset * 2),
        sudoku_panel.height.saturating_sub(2 + inner_top_offset * 2),
    );

    // 第三步：sudoku_inner1 内部布局：上下方向条 + 中间层
    let center_layout = Layout::vertical([
        Constraint::Length(dir_border),
        Constraint::Fill(1),
        Constraint::Length(dir_border),
    ]);
    let center_parts = center_layout.split(sudoku_inner1);
    let sudoku_dir_top = center_parts[0];
    let sudoku_middle = center_parts[1];
    let sudoku_dir_bot = center_parts[2];

    // 第四步：sudoku_middle 水平布局：左方向条 | 内容区 | 右方向条
    let sudoku_h_layout = Layout::horizontal([
        Constraint::Length(dir_border),
        Constraint::Fill(1),
        Constraint::Length(dir_border),
    ]);
    let sudoku_h_parts = sudoku_h_layout.split(sudoku_middle);
    let sudoku_dir_left = sudoku_h_parts[0];
    let sudoku_content = sudoku_h_parts[1];
    let sudoku_dir_right = sudoku_h_parts[2];

    // 第五步：sudoku_content 内部居中放置数独网格
    let grid_outer_w = grid_w;
    let grid_outer_h = grid_h;
    let grid_x = sudoku_content.x + (sudoku_content.width.saturating_sub(grid_outer_w)) / 2;
    let grid_y = sudoku_content.y + (sudoku_content.height.saturating_sub(grid_outer_h)) / 2;
    let grid_outer = Rect::new(grid_x, grid_y, grid_outer_w, grid_outer_h);

    // 数独外框内部
    let grid_frame = Rect::new(
        grid_outer.x + 1,
        grid_outer.y + 1,
        grid_outer.width.saturating_sub(2),
        grid_outer.height.saturating_sub(2),
    );

    // grid_frame 内部：网格 + 消息
    let frame_v = Layout::vertical([
        Constraint::Length(grid_h),
        Constraint::Length(gap),
        Constraint::Length(msg_h),
    ]);
    let frame_v_parts = frame_v.split(grid_frame);
    let _grid_content = frame_v_parts[0];
    let _msg_gap = frame_v_parts[1];
    let msg_area = frame_v_parts[2];

    // 网格内容（去掉外框）
    let grid_inner_rect = grid_outer.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    // ── 左列：Status / Navigator / Logs ──
    // Status 高度: 4 (标题+面+时间+剩余) + 1 (进度条) + 9 (1~9 数字剩余) + 2 (边框) = 16
    let left_layout = Layout::vertical([
        Constraint::Length(16),
        Constraint::Length(9),
        Constraint::Fill(1),
    ]);
    let left_parts = left_layout.split(left_chunk);
    let status_panel = left_parts[0];
    let navigator_panel = left_parts[1];
    let logs_panel = left_parts[2];

    // ── 右列：3D 立方体面板(削减一半) / 商店 ──
    let right_layout = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Length(1),
        Constraint::Min(4),
    ]);
    let right_parts = right_layout.split(right_chunk);
    let cube_outer_frame = right_parts[0];
    let _cube_gap = right_parts[1];
    let shop_area = right_parts[2];

    // ── cube_outer_frame 内部布局：外框 → 中间层 ▒ → 内框(隐藏) → cube(居中) ──
    let cube_inner1 = Rect::new(
        cube_outer_frame.x + 1,
        cube_outer_frame.y + 1,
        cube_outer_frame.width.saturating_sub(2),
        cube_outer_frame.height.saturating_sub(2),
    );

    // 内框尺寸比外框左右各 -4，上下各 -2
    let cube_inner_frame = Rect::new(
        cube_outer_frame.x + 1 + inner_left_offset,
        cube_outer_frame.y + 1 + inner_top_offset,
        cube_outer_frame.width.saturating_sub(2 + inner_left_offset * 2),
        cube_outer_frame.height.saturating_sub(2 + inner_top_offset * 2),
    );

    // cube_inner1 垂直布局
    let cube_panel_v = Layout::vertical([
        Constraint::Length(dir_border),
        Constraint::Fill(1),
        Constraint::Length(dir_border),
    ]);
    let cube_v_parts = cube_panel_v.split(cube_inner1);
    let cube_dir_top = cube_v_parts[0];
    let cube_middle = cube_v_parts[1];
    let cube_dir_bot = cube_v_parts[2];

    // cube_middle 水平布局
    let cube_middle_h = Layout::horizontal([
        Constraint::Length(dir_border),
        Constraint::Fill(1),
        Constraint::Length(dir_border),
    ]);
    let cube_middle_h_parts = cube_middle_h.split(cube_middle);
    let cube_dir_left = cube_middle_h_parts[0];
    let cube_content = cube_middle_h_parts[1];
    let cube_dir_right = cube_middle_h_parts[2];

    // ── 立方体内容：保持 aspect ratio，居中显示 ──
    let cube_aspect: f64 = app.settings.cube_aspect.parse().unwrap_or(1.0);
    let inner_w = cube_content.width as f64;
    let inner_h = cube_content.height as f64;
    let target_w_from_h = inner_h * cube_aspect * 2.0;
    let target_h_from_w = inner_w / (cube_aspect * 2.0);
    let (cube_w, cube_h) = if target_w_from_h <= inner_w {
        (target_w_from_h, inner_h)
    } else {
        (inner_w, target_h_from_w)
    };
    let cube_area_w = cube_w.floor() as u16;
    let cube_area_h = cube_h.floor() as u16;
    let cube_x = cube_content.x + (cube_content.width.saturating_sub(cube_area_w)) / 2;
    let cube_y = cube_content.y + (cube_content.height.saturating_sub(cube_area_h)) / 2;
    let cube_area = Rect::new(cube_x, cube_y, cube_area_w, cube_area_h);

    // ── 按钮定义（使用自定义 Button Widget）──
    // 道具按钮 label 包含持有数量,例如 "🎲×2"
    let tool_defs: Vec<(String, ButtonId, crate::ButtonTheme)> = [
        (shop::ItemType::Cube, ButtonId::ToolCube, crate::THEME_SUCCESS),
        (shop::ItemType::Snake3, ButtonId::ToolSnake3, crate::THEME_SUCCESS),
        (shop::ItemType::Face, ButtonId::ToolFace, crate::THEME_SUCCESS),
        (shop::ItemType::Snake5, ButtonId::ToolSnake5, crate::THEME_SUCCESS),
        (
            shop::ItemType::Target,
            ButtonId::ToolTarget,
            crate::THEME_SUCCESS,
        ),
    ]
    .iter()
    .map(|(item, id, theme): &(shop::ItemType, ButtonId, crate::ButtonTheme)| {
        let count = app.inventory.get(item).copied().unwrap_or(0);
        let label = format!("{}{}", item.icon(), count);
        (label, *id, *theme)
    })
    .collect();

    let btn_defs: Vec<(String, ButtonId, u16, crate::ButtonTheme)> = (1..=9u8)
        .map(|n| {
            let label = format!("[{}]", n);
            let w = label.chars().count() as u16;
            (label, ButtonId::Number(n), w, crate::THEME_PRIMARY)
        })
        .chain(
            [
                ("[E]rase", ButtonId::Erase, crate::THEME_NEUTRAL),
                ("[H]int", ButtonId::Hint, crate::THEME_SUCCESS),
                ("[Z]Undo", ButtonId::Undo, crate::THEME_NEUTRAL),
                ("[G]uide", ButtonId::ToggleGuidance, crate::THEME_NEUTRAL),
                ("[M]ode", ButtonId::ToggleMode, crate::THEME_NEUTRAL),
                ("[Q]uit", ButtonId::Quit, crate::THEME_DANGER),
            ]
            .iter()
            .map(|(label, id, theme)| {
                let w = label.chars().count() as u16;
                (label.to_string(), *id, w, *theme)
            }),
        )
        .chain(tool_defs.iter().map(|(label, id, theme)| {
            let w = label.chars().count() as u16;
            (label.clone(), *id, w, *theme)
        }))
        .collect();

    // 按钮栏布局 —— 整行无外边框，padding=1 列
    let btn_inner = Rect {
        x: btn_area.x + 1,
        y: btn_area.y,
        width: btn_area.width.saturating_sub(2),
        height: btn_area.height,
    };
    let btn_row = btn_inner.y;
    let btn_height = btn_inner.height;

    // 翻页预留宽度: ◁ (1) + 空格(1) + 页码(3) + 空格(1) + ▷ (1) = 7
    const PAGER_RESERVE_W: u16 = 7;

    // 总按钮宽度 (含按钮间空格)
    let total_btn_w: usize = btn_defs
        .iter()
        .map(|(_, _, w, _)| *w as usize + 1)
        .sum::<usize>()
        .saturating_sub(1);

    // 计算翻页模式
    let single_page_fits = total_btn_w as u16 <= btn_inner.width;
    let available_w = if single_page_fits {
        btn_inner.width
    } else {
        btn_inner.width.saturating_sub(PAGER_RESERVE_W)
    };

    // 计算每页最大按钮数
    let mut page_capacity: usize = 0;
    let mut running: usize = 0;
    for (_, _, w, _) in &btn_defs {
        let add = if page_capacity == 0 { *w as usize } else { 1 + *w as usize };
        if running + add > available_w as usize {
            break;
        }
        running += add;
        page_capacity += 1;
    }
    if page_capacity == 0 {
        page_capacity = btn_defs.len();
    }
    let total_pages: u16 = ((btn_defs.len() + page_capacity - 1) / page_capacity) as u16;
    // 修正 current_page 不超出范围
    let current_page = (app.btn_page as usize).min((total_pages as usize).saturating_sub(1));
    let start_idx = current_page * page_capacity;
    let end_idx = (start_idx + page_capacity).min(btn_defs.len());
    let visible = &btn_defs[start_idx..end_idx];

    // 当前页按钮宽度
    let visible_w: usize = if visible.is_empty() {
        0
    } else {
        visible
            .iter()
            .map(|(_, _, w, _)| *w as usize + 1)
            .sum::<usize>()
            .saturating_sub(1)
    };

    // 居中布局
    let bar_x: u16;
    let mut pager: Option<ButtonPagerLayout> = None;
    if single_page_fits {
        bar_x = btn_inner.x + (btn_inner.width.saturating_sub(visible_w as u16)) / 2;
    } else {
        // 翻页模式: 左/右两侧放翻页控件，按钮居中
        let prev_x = btn_inner.x;
        let page_y = btn_row + (btn_height.saturating_sub(1)) / 2;
        let next_x = btn_inner.x + btn_inner.width.saturating_sub(1);
        let page_label_x = btn_inner.x + 2; // 紧跟 ◁ 之后
        pager = Some(ButtonPagerLayout {
            prev_rect: Rect::new(prev_x, page_y, 1, 1),
            next_rect: Rect::new(next_x, page_y, 1, 1),
            page_label_rect: Rect::new(page_label_x, page_y, 3, 1),
            total_pages,
        });
        // 按钮居中
        let pager_left = prev_x + 1;
        let pager_right = next_x;
        let center_start = pager_left + (pager_right.saturating_sub(pager_left + visible_w as u16)) / 2;
        bar_x = center_start.max(pager_left);
    }

    let mut col = bar_x;
    let mut buttons = Vec::new();
    for (label, id, w, theme) in visible {
        buttons.push(ButtonLayout {
            id: *id,
            label: label.clone(),
            col,
            row: btn_row,
            width: *w,
            height: btn_height,
            theme: *theme,
        });
        col += w + 1;
    }

    // 将 current_page 写回 (防止越界)
    if (app.btn_page as usize) != current_page {
        app.btn_page = current_page as u16;
    }

    GameLayout {
        game_area: area,
        left_column: left_chunk,
        status_panel,
        navigator_panel,
        logs_panel,
        center_column: center_chunk,
        sudoku_outer_frame: sudoku_panel,
        sudoku_dir_top,
        sudoku_dir_bot,
        sudoku_dir_left,
        sudoku_dir_right,
        sudoku_inner_frame,
        grid_area: grid_inner_rect,
        grid_frame,
        msg_area,
        btn_area,
        btn_content_x: bar_x,
        right_column: right_chunk,
        cube_outer_frame,
        cube_dir_top,
        cube_dir_bot,
        cube_dir_left,
        cube_dir_right,
        cube_inner_frame,
        cube_area,
        shop_area,
        buttons,
        pager,
    }
}

/// 绘制带标题的通用面板（使用 Block::bordered 渲染边框与标题）。
/// `lines` 会被渲染到面板内部，超出 (panel.height - 2) 的部分会被截断。
fn draw_framed_panel(
    f: &mut Frame,
    panel: Rect,
    title: &str,
    title_color: Color,
    border_color: Color,
    bg: Color,
    lines: Vec<Line>,
) {
    if panel.width < 4 || panel.height < 3 {
        return;
    }
    let block = Block::bordered()
        .title(Line::from(Span::styled(
            format!(" {} ", title),
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg));

    // 先渲染 Block 自身（边框 + 标题）
    f.render_widget(block.clone(), panel);

    // 再渲染内容到 inner area
    let inner = block.inner(panel);
    let content_h = inner.height as usize;
    for (i, line) in lines.into_iter().take(content_h).enumerate() {
        f.render_widget(
            Paragraph::new(line).style(Style::default().bg(bg).fg(Color::White)),
            Rect::new(inner.x, inner.y + i as u16, inner.width, 1),
        );
    }
}

/// 顶部 Status 面板：游戏名 / 难度 / 时间 / 进度。
fn draw_status_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.status_panel;

    // 贪吃蛇模式:显示蛇状态
    if let Some(snake) = app.snake.as_ref() {
        let secs_left = snake
            .deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_secs();
        let speed_ms = snake.step_interval.as_millis();
        let dir_name = match snake.dir {
            (1, 0) => "→",
            (-1, 0) => "←",
            (0, -1) => "↑",
            (0, 1) => "↓",
            _ => "?",
        };
        let lines = vec![
            Line::from(Span::styled(
                " 🐍 Snake Mode",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!(" ⏱  {}s", secs_left)),
            Line::from(format!(" 🍒 {}/{}", snake.score, snake.total_fruits)),
            Line::from(format!(" 🧱 {}", snake.walls.len())),
            Line::from(format!(" 📏 {}", snake.body.len())),
            Line::from(format!(" ⚡ {}ms", speed_ms)),
            Line::from(format!(" 🧭 {}", dir_name)),
            Line::from(""),
            Line::from(Span::styled(
                format!(" {}", i18n::t("status.inventory", lang)),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )),
        ];
        draw_framed_panel(
            f,
            panel,
            i18n::t("panel.status", lang),
            Color::Cyan,
            border,
            bg,
            lines,
        );
        return;
    }

    let inner_w = (panel.width.saturating_sub(4)) as usize; // 可用文本宽度(去边框去前导空格)
    let total = app.game.grid.cells.len();
    let filled = app
        .game
        .grid
        .cells
        .values()
        .filter(|c| c.user_value.is_some())
        .count();
    let remaining = total - filled;
    let game_name = app
        .game
        .id
        .map_or(i18n::t("game.unnamed", lang).to_string(), |id| {
            if app.settings.naming_mode == "vivid" {
                i18n::vivid_name(id, lang)
            } else {
                format!("#{}", id)
            }
        });
    let progress_pct = if total == 0 {
        0
    } else {
        (filled * 100) / total
    };
    let bar_w = inner_w.saturating_sub(2).max(4);
    let filled_w = (bar_w * filled) / total.max(1);
    let progress_bar = format!(
        "[{}{}]",
        "█".repeat(filled_w),
        "░".repeat(bar_w.saturating_sub(filled_w))
    );

    // 数字 1~9 的剩余数量(防漏题,固定上限 6*9=54)
    // 角落权重 3,边权重 2,面中心权重 1
    let digit_lines: Vec<Line> = (0..9)
        .map(|i| {
            let remain = app.digit_remaining[i];
            let n = (i + 1) as u8;
            // 缺题高亮(<= 0 视为已漏完,需关注)
            let color = if remain <= 0 {
                Color::Red
            } else if remain <= 3 {
                Color::Yellow
            } else {
                Color::White
            };
            Line::from(Span::styled(
                format!(" {}: {:>2}/54 ", n, remain),
                Style::default().fg(color),
            ))
        })
        .collect();

    let mut lines = vec![
        Line::from(vec![Span::styled(
            game_name.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(format!(
            " {}: {}  {}: {}",
            i18n::t("info.face", lang),
            face_name(app.current_face, lang),
            i18n::t("info.diff", lang),
            app.game.difficulty.as_str(),
        )),
        Line::from(format!(
            " {}: {}",
            i18n::t("info.time", lang),
            format_timer(app),
        )),
        Line::from(format!(" {}/{} ({}%)", remaining, total, progress_pct)),
    ];
    lines.extend(digit_lines);
    lines.push(Line::from(progress_bar));
    // ── 道具背包 ──
    let is_debug = app.settings.debug_mode == "on";
    lines.push(Line::from(Span::styled(
        format!(" {}: ", i18n::t("status.inventory", lang)),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    )));
    for item in shop::ItemType::all().iter() {
        let count = app.inventory.get(item).copied().unwrap_or(0);
        // 数量为 0 时显示警告样式(debug 模式不算警告)
        let is_warn = count == 0 && !is_debug;
        let color = if is_debug && count == 0 {
            Color::DarkGray // debug 模式下空也允许使用
        } else if is_warn {
            Color::Red
        } else {
            Color::White
        };
        let warn_mark = if is_warn { "!" } else { " " };
        lines.push(Line::from(Span::styled(
            format!(" {}{} {}", warn_mark, item.icon(), count),
            Style::default().fg(color),
        )));
    }

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.status", lang),
        Color::Cyan,
        border,
        bg,
        lines,
    );
}

/// Navigator 面板：方向键 / WASD / FBLRTU 提示 + 当前面指示。
/// 贪吃蛇模式时替换为蛇操作指导。
fn draw_navigator_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.navigator_panel;

    // 贪吃蛇模式:操作指导
    if app.snake.is_some() {
        let face_color = face_to_color(app.current_face);
        let lines = vec![
            Line::from(Span::styled(
                " 🐍 Snake Controls",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(" ↑↓←→ / WASD"),
            Line::from("   转向"),
            Line::from(""),
            Line::from(" 🍒 吃果实→填答案"),
            Line::from(" 🧱 撞墙→失败"),
            Line::from(" ⬡ 撞自己→失败"),
            Line::from(" ⏱  超时→失败"),
            Line::from(""),
            Line::from(" Esc/Q 退出"),
            Line::from(Span::styled(
                format!(
                    " ● {} ({})",
                    app.current_face.short_code(),
                    face_name(app.current_face, lang)
                ),
                Style::default().fg(face_color).add_modifier(Modifier::BOLD),
            )),
        ];
        draw_framed_panel(
            f,
            panel,
            i18n::t("panel.navigator", lang),
            Color::Green,
            border,
            bg,
            lines,
        );
        return;
    }

    let face_color = face_to_color(app.current_face);
    let (up, down, left, right, _back) = arrow_neighbor_faces(app.current_face);

    let current_marker = format!(
        "● {} ({})",
        app.current_face.short_code(),
        face_name(app.current_face, lang)
    );

    let lines = vec![
        Line::from(format!(" ↑ {}    [W]上", face_name(up, lang))),
        Line::from(format!(" ↓ {}    [S]下", face_name(down, lang))),
        Line::from(format!(" ← {}    [A]左", face_name(left, lang))),
        Line::from(format!(" → {}    [D]右", face_name(right, lang))),
        Line::from(format!(" [F]前 [B]后 [L]左 [R]右")),
        Line::from(format!(" [T]上 [U]下")),
        Line::from(Span::styled(
            current_marker,
            Style::default().fg(face_color).add_modifier(Modifier::BOLD),
        )),
    ];

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.navigator", lang),
        Color::Green,
        border,
        bg,
        lines,
    );
}

/// Logs 面板：最近 N 条操作。
fn draw_logs_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.logs_panel;
    let inner_h = panel.height.saturating_sub(2) as usize;

    // 收集行：新 -> 旧
    let mut entries: Vec<String> = app.action_log.iter().rev().cloned().collect();
    // 如果为空，至少给一行提示
    if entries.is_empty() {
        entries.push(i18n::t("panel.logs_empty", lang).to_string());
    }
    entries.truncate(inner_h);

    let lines: Vec<Line> = entries
        .into_iter()
        .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::White))))
        .collect();

    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.logs", lang),
        Color::Yellow,
        border,
        bg,
        lines,
    );
}

/// 商店预留面板。
fn draw_shop_panel(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, border: Color) {
    let lang = Lang::from_code(&app.settings.language);
    let panel = layout.shop_area;
    if panel.width < 4 || panel.height < 3 {
        return;
    }

    let catalog = crate::shop::shop_catalog();
    let _item_h: u16 = 3; // 每项 3 行(图标名 + 描述 + 价格/数量)
    let total_pages = 1u16; // 4 项一页
    let page = app.shop_page as usize;

    let mut lines: Vec<Line> = Vec::new();

    // 顶部:金币余额
    lines.push(Line::from(vec![
        Span::styled("💰 ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{}", app.gold),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {}/{} ", page + 1, total_pages),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(Span::styled(
        "─".repeat(panel.width.saturating_sub(2) as usize),
        Style::default().fg(Color::Magenta),
    )));

    // 商品列表
    let items_per_page = 4usize;
    let start = page * items_per_page;
    let end = (start + items_per_page).min(catalog.len());
    for (idx, shop_item) in catalog.iter().enumerate().take(end).skip(start) {
        let i = idx; // 全局索引(用于选中态)
        let sel = i == app.shop_selected && app.shop_focused;
        let count = app.inventory.get(&shop_item.item_type).copied().unwrap_or(0);
        let _title_color = if sel {
            Color::White
        } else {
            Color::Magenta
        };
        let bg_style = if sel {
            Style::default().bg(Color::Magenta).fg(Color::White)
        } else {
            Style::default().bg(bg)
        };
        // 第 1 行: 图标 + 名称 + 持有数量
        let name = i18n_item_name(shop_item.item_type, lang);
        let line1 = format!(
            "{} {} ×{}",
            shop_item.item_type.icon(),
            name,
            count
        );
        lines.push(Line::from(Span::styled(line1, bg_style)));
        // 第 2 行: 描述
        let desc = i18n_item_desc(shop_item.item_type, lang);
        lines.push(Line::from(Span::styled(
            format!("  {}", desc),
            Style::default().fg(Color::DarkGray),
        )));
        // 第 3 行: 价格
        let afford = app.gold >= shop_item.price;
        let price_color = if sel {
            Color::White
        } else if afford {
            Color::Yellow
        } else {
            Color::Red
        };
        let price_bg = if sel {
            Style::default().bg(Color::Magenta).fg(Color::White)
        } else {
            Style::default().bg(bg).fg(price_color)
        };
        let price_text = format!("  💰 {}  ", shop_item.price);
        lines.push(Line::from(Span::styled(price_text, price_bg)));
    }

    // 底部提示
    lines.push(Line::from(Span::styled(
        "─".repeat(panel.width.saturating_sub(2) as usize),
        Style::default().fg(Color::Magenta),
    )));
    let hint = if app.shop_focused {
        i18n::t("shop.hint_focused", lang)
    } else {
        i18n::t("shop.hint_unfocused", lang)
    };
    lines.push(Line::from(Span::styled(
        hint,
        Style::default()
            .fg(if app.shop_focused {
                Color::Magenta
            } else {
                Color::DarkGray
            })
            .add_modifier(Modifier::ITALIC),
    )));

    let title_color = if app.shop_focused {
        Color::Magenta
    } else {
        Color::Magenta
    };
    let frame_border = if app.shop_focused {
        Color::Magenta
    } else {
        border
    };
    draw_framed_panel(
        f,
        panel,
        i18n::t("panel.shop", lang),
        title_color,
        frame_border,
        bg,
        lines,
    );
}

/// 商品名称 i18n
fn i18n_item_name(item: crate::shop::ItemType, lang: Lang) -> &'static str {
    i18n::t(item.name_key(), lang)
}
/// 商品描述 i18n
fn i18n_item_desc(item: crate::shop::ItemType, lang: Lang) -> &'static str {
    i18n::t(item.desc_key(), lang)
}

fn draw_sudoku_grid(f: &mut Frame, layout: &GameLayout, app: &App, bg: Color, _border: Color) {
    let cw = app.render_mode.cell_width(&app.settings);
    let ch = app.render_mode.cell_height();
    let ox = layout.grid_area.x;
    let oy = layout.grid_area.y;

    // 当前面颜色
    let face_color = face_to_color(app.current_face);

    // 1) 外层 Block::bordered — 第一层边框
    if layout.sudoku_outer_frame.width >= 4 && layout.sudoku_outer_frame.height >= 3 {
        let outer_block = Block::bordered()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(face_color))
            .style(Style::default().bg(bg));
        f.render_widget(outer_block, layout.sudoku_outer_frame);
    }

    // 2) 内部方向指示条（实心方块 ▒▒▒）— 颜色对应 WASD 邻居面
    let (up_face, down_face, left_face, right_face, back_face) =
        wasd_neighbor_faces(app.current_face);
    let up_color = face_to_color(up_face);
    let down_color = face_to_color(down_face);
    let left_color = face_to_color(left_face);
    let right_color = face_to_color(right_face);
    let back_color = face_to_color(back_face);

    // 顶方向条：整行用 ▒ 填满
    if layout.sudoku_dir_top.width > 0 && layout.sudoku_dir_top.height > 0 {
        let line: String = "▒".repeat(layout.sudoku_dir_top.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(up_color).bg(bg)),
            layout.sudoku_dir_top,
        );
    }
    // 底方向条
    if layout.sudoku_dir_bot.width > 0 && layout.sudoku_dir_bot.height > 0 {
        let line: String = "▒".repeat(layout.sudoku_dir_bot.width as usize);
        f.render_widget(
            Paragraph::new(line).style(Style::default().fg(down_color).bg(bg)),
            layout.sudoku_dir_bot,
        );
    }
    // 左方向条：逐行渲染 ▒（垂直方向条）
    if layout.sudoku_dir_left.width > 0 && layout.sudoku_dir_left.height > 0 {
        for row in 0..layout.sudoku_dir_left.height {
            f.render_widget(
                Paragraph::new("▒").style(Style::default().fg(left_color).bg(bg)),
                Rect::new(
                    layout.sudoku_dir_left.x,
                    layout.sudoku_dir_left.y + row,
                    1,
                    1,
                ),
            );
        }
    }
    // 右方向条：逐行渲染 ▒（垂直方向条）
    if layout.sudoku_dir_right.width > 0 && layout.sudoku_dir_right.height > 0 {
        for row in 0..layout.sudoku_dir_right.height {
            f.render_widget(
                Paragraph::new("▒").style(Style::default().fg(right_color).bg(bg)),
                Rect::new(
                    layout.sudoku_dir_right.x,
                    layout.sudoku_dir_right.y + row,
                    1,
                    1,
                ),
            );
        }
    }

    // 注意：内框不渲染（隐藏），内容直接居中显示

    // 4) ╔═╗ 正式数独框（用 face color 替换 border_color）
    // 逐行逐列绘制网格
    for v in 0..9usize {
        // 分隔行
        let sep_y = oy + v as u16 * (ch as u16 + 1);
        draw_separator(
            f,
            ox,
            sep_y,
            cw,
            v == 0,
            false,
            v % 3 == 0,
            layout.grid_area,
            face_color,
        );

        // 单元格内容行
        for line in 0..ch {
            let row_y = oy + v as u16 * (ch as u16 + 1) + 1 + line as u16;
            draw_cell_row(
                f,
                ox,
                row_y,
                cw,
                ch,
                v,
                line,
                app,
                layout.grid_area,
                bg,
                face_color,
            );
        }
    }

    // 底部分隔行
    let bot_y = oy + 9 * (ch as u16 + 1);
    draw_separator(
        f,
        ox,
        bot_y,
        cw,
        false,
        true,
        true,
        layout.grid_area,
        face_color,
    );

    // 5) Back-face indicator: 实心方块在数独 ╔═╗ 框的右下方
    // ╔═╗ 框位于 layout.grid_frame，框高度 = grid_h = (1 + 9 * (ch + 1)) + 2
    let grid_outer_h = (1 + 9 * (ch as u16 + 1)) + 2;
    let outer_x = layout.grid_frame.x;
    let outer_y = layout.grid_frame.y;
    let outer_w = layout.grid_frame.width;
    let indicator_x = outer_x + outer_w;
    let indicator_y = outer_y + grid_outer_h;
    if indicator_x < f.area().width && indicator_y < f.area().height {
        f.render_widget(
            Paragraph::new("▨").style(Style::default().fg(back_color).bg(bg)),
            Rect::new(indicator_x, indicator_y, 1, 1),
        );
    }
}

fn draw_separator(
    f: &mut Frame,
    x: u16,
    y: u16,
    cw: usize,
    is_top: bool,
    is_bottom: bool,
    is_thick_h: bool,
    bounds: Rect,
    border: Color,
) {
    if y >= bounds.bottom() {
        return;
    }

    let mut buf = String::new();
    // 左角
    buf.push(if is_top {
        '╔'
    } else if is_bottom {
        '╚'
    } else {
        '╟'
    });

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
    buf.push(if is_top {
        '╗'
    } else if is_bottom {
        '╝'
    } else {
        '╢'
    });

    let w = buf.chars().count() as u16;
    f.render_widget(
        Paragraph::new(buf).style(Style::default().fg(border)),
        Rect::new(x, y, w.min(bounds.width), 1),
    );
}

fn draw_cell_row(
    f: &mut Frame,
    x: u16,
    y: u16,
    cw: usize,
    ch: usize,
    v: usize,
    line: usize,
    app: &App,
    bounds: Rect,
    _bg: Color,
    border: Color,
) {
    if y >= bounds.bottom() {
        return;
    }

    let mut spans: Vec<Span> = Vec::new();

    // 贪吃蛇模式: 渲染蛇/果实/墙壁
    let snake_mode = app.snake.is_some();

    for u in 0..9usize {
        // 竖线
        let v_char = if u % 3 == 0 { '║' } else { '│' };
        spans.push(Span::styled(
            v_char.to_string(),
            Style::default().fg(border),
        ));

        // 单元格内容
        let coord = app.current_face.to_cube(u as u8, v as u8);
        let face_pos = (app.current_face, u as u8, v as u8);

        if snake_mode {
            // 贪吃蛇模式渲染
            let mid_line = ch / 2;
            let mut content = " ".repeat(cw);
            let mut style = Style::default().fg(Color::White);

            if let Some(snake) = &app.snake {
                let is_head = snake.body.first() == Some(&face_pos);
                let is_body = !is_head && snake.body.contains(&face_pos);
                let is_fruit = snake.fruits.contains(&face_pos);
                let is_wall = snake.walls.contains(&face_pos);

                if is_head && line == mid_line {
                    let s = "◉".to_string();
                    let start = (cw - 1) / 2;
                    content.replace_range(start..start + 1, &s);
                    style = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);
                } else if is_body {
                    if line == mid_line {
                        let s = "●".to_string();
                        let start = (cw - 1) / 2;
                        content.replace_range(start..start + 1, &s);
                    }
                    style = Style::default().fg(Color::Green);
                } else if is_fruit && line == mid_line {
                    let s = "●".to_string();
                    let start = (cw - 1) / 2;
                    content.replace_range(start..start + 1, &s);
                    style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                } else if is_wall {
                    content = "▓".repeat(cw);
                    style = Style::default().fg(Color::DarkGray);
                } else {
                    // 空格: 在蛇模式下灰显
                    style = Style::default().fg(Color::DarkGray);
                }
            }
            spans.push(Span::styled(content, style));
        } else {
            // 正常数独模式渲染
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
            // 闪烁逻辑:开启时 blink_on 在两态间切换;关闭时使用反色(白底黑字)保持高亮
            let blink_setting_on = app.settings.blink_highlight == "on";
            let blink_on = blink_setting_on && app.blink_on;
            let style = if selected && is_error {
                Style::default().bg(Color::White).fg(Color::Red)
            } else if selected && blink_on {
                Style::default().bg(Color::White).fg(Color::Black)
            } else if selected && blink_setting_on {
                Style::default().bg(Color::Gray).fg(Color::White)
            } else if selected {
                // 闪烁关闭:反色(白底黑字),保证可视
                Style::default().bg(Color::White).fg(Color::Black)
            } else if in_same_group && has_same_number {
                Style::default().bg(guide_same).fg(Color::White)
            } else if in_same_group {
                Style::default().bg(guide_group).fg(Color::White)
            } else if has_same_number {
                Style::default().bg(guide_same).fg(Color::White)
            } else if is_error {
                Style::default().fg(Color::Red)
            } else if is_given {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if value.is_some() {
                Style::default().fg(border)
            } else {
                Style::default().fg(Color::White)
            };

            spans.push(Span::styled(content, style));
        }
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
    if app.message.is_empty() {
        return;
    }
    let style = Style::default().fg(Color::Green);
    f.render_widget(
        Paragraph::new(app.message.as_str()).style(style),
        layout.msg_area,
    );
}

fn draw_button_bar(f: &mut Frame, layout: &GameLayout, app: &App, _bg: Color, border: Color) {
    if layout.btn_area.width < 4 || layout.btn_area.height < 1 {
        return;
    }
    // 贪吃蛇模式下隐藏按钮栏
    if app.snake.is_some() {
        return;
    }

    // 按钮栏两侧装饰（提示符），使用与边框同色
    let deco_style = Style::default().fg(border).add_modifier(Modifier::BOLD);
    if layout.btn_area.height >= 1 {
        let left_x = layout.btn_area.x;
        let right_x = layout.btn_area.x + layout.btn_area.width.saturating_sub(1);
        let y = layout.btn_area.y + layout.btn_area.height / 2;
        if right_x > left_x + 1 {
            f.render_widget(
                Paragraph::new("╶").style(deco_style),
                Rect::new(left_x, y, 1, 1),
            );
            f.render_widget(
                Paragraph::new("╴").style(deco_style),
                Rect::new(right_x, y, 1, 1),
            );
        }
    }

    // 每个按钮独立渲染为自定义 Button widget
    for btn in &layout.buttons {
        let area = Rect::new(btn.col, btn.row, btn.width, btn.height);
        if area.width == 0 || area.height == 0 {
            continue;
        }
        let state = if app.hover_button == Some(btn.id) {
            crate::ButtonState::Hovered
        } else {
            crate::ButtonState::Normal
        };
        let label = Line::from(btn.label.as_str());
        let mut button = crate::Button::new(label)
            .theme(btn.theme)
            .state(state)
            .border(true);
        if btn.height >= 3 {
            // 3D 风格（顶/底高光）
        } else {
            // 高度不足时关闭 border
            button = button.border(false);
        }
        f.render_widget(button, area);
    }

    // 翻页控件 ◁ ▷ 和页码
    if let Some(pager) = &layout.pager {
        let pager_style_normal = Style::default().fg(border).add_modifier(Modifier::BOLD);
        let pager_style_disabled = Style::default().fg(Color::DarkGray);
        let total = pager.total_pages.max(1) as usize;
        let cur = (app.btn_page as usize).min(total.saturating_sub(1));

        // ◁ 左翻页
        let prev_disabled = cur == 0;
        let prev_text = if prev_disabled { " " } else { "◁" };
        f.render_widget(
            Paragraph::new(prev_text).style(if prev_disabled { pager_style_disabled } else { pager_style_normal }),
            pager.prev_rect,
        );

        // 页码标签 (3 字符: 1/3 等)
        let page_text = format!("{}/{}", cur + 1, total);
        f.render_widget(
            Paragraph::new(page_text).style(Style::default().fg(Color::White)),
            pager.page_label_rect,
        );

        // ▷ 右翻页
        let next_disabled = cur + 1 >= total;
        let next_text = if next_disabled { " " } else { "▷" };
        f.render_widget(
            Paragraph::new(next_text).style(if next_disabled { pager_style_disabled } else { pager_style_normal }),
            pager.next_rect,
        );
    }
}


// ── 3D立方体 + 公共工具 + 设置类型 (已迁移至子模块) ──

