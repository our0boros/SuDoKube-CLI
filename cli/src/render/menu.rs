use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::util::*;
use crate::i18n::{self, Lang};
use crate::{App, MenuItem};

// ── 常量 ──

const LOGO: &[&str] = &[
    " ██████  █    ██ ▓█████▄  ▒█████   ██ ▄█▀ █    ██  ▄▄▄▄   ▓█████   ",
    " ▒██    ▒  ██  ▓██▒▒██▀ ██▌▒██▒  ██▒ ██▄█▒  ██  ▓██▒▓█████▄ ▓█   ▀ ",
    " ░ ▓██▄   ▓██  ▒██░░██   █▌▒██░  ██▒▓███▄░ ▓██  ▒██░▒██▒ ▄██▒███   ",
    "   ▒   ██▒▓▓█  ░██░░▓█▄   ▌▒██   ██░▓██ █▄ ▓▓█  ░██░▒██░█▀  ▒▓█  ▄ ",
    " ▒██████▒▒▒▒█████▓ ░▒████▓ ░ ████▓▒░▒██▒ █▄▒▒█████▓ ░▓█  ▀█▓░▒████▒",
    " ▒ ▒▓▒ ▒ ░░▒▓▒ ▒ ▒  ▒▒▓  ▒ ░ ▒░▒░▒░ ▒ ▒▒ ▓▒░▒▓▒ ▒ ▒ ░▒▓███▀▒░░ ▒░ ░",
    " ░ ░▒  ░ ░░░▒░ ░ ░  ▒ ░▒▓░░ ▒░▒░▒░  ░ ▒ ▒░ ░░▒░ ░ ░    ▒ ▒  ░ ░  ░",
    " ░    ░  ░ ░  ░    ░ ▒ ░▓▓ ░  ░░░ ░░  ░ ░    ░░░ ░ ░  ░  ▒ ░      ░",
    "      ░    ░       ░ ░▒▓▓░      ░ ░        ░       ░     ░ ░  ░     ░",
    "              ░       ░▒▓▓░      ░ ░        ░       ░     ░ ░  ░     ░",
];

pub(super) fn draw_menu(f: &mut Frame, app: &App) {
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
            // 截断到 area 范围内,避免超出缓冲区
            let avail = area.width.saturating_sub(col - area.x);
            let w = logo_width.min(avail);
            if w > 0 {
                f.render_widget(
                    Paragraph::new(*line).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Rect::new(col, row, w, 1),
                );
            }
        }
    }

    let box_y = start_y + logo_h + 1;

    // 计算菜单项文本
    let lang = Lang::from_code(&app.settings.language);
    let item_texts: Vec<String> = app
        .menu
        .items
        .iter()
        .map(|item| match item {
            MenuItem::NewGame(d) => {
                let diff_key = match d {
                    sudokube_core::cube::Difficulty::Easy => "game.diff_easy",
                    sudokube_core::cube::Difficulty::Medium => "game.diff_medium",
                    sudokube_core::cube::Difficulty::Hard => "game.diff_hard",
                };
                format!(
                    "{} - {}",
                    i18n::t("menu.new_easy", lang)
                        .split(" - ")
                        .next()
                        .unwrap_or("New"),
                    i18n::t(diff_key, lang)
                )
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
                    i18n::t("menu.continue", lang),
                    name,
                    r.difficulty,
                    r.started_at.format("%m-%d").to_string(),
                    remaining,
                    total,
                    r.elapsed_seconds / 60,
                    r.elapsed_seconds % 60,
                    if r.completed {
                        i18n::t("menu.victory", lang)
                    } else {
                        i18n::t("menu.in_progress", lang)
                    }
                )
            }
        })
        .collect();

    let max_text_w = item_texts
        .iter()
        .map(|t| display_width(t) + 4)
        .max()
        .unwrap_or(20);
    let box_w = (max_text_w + 4).min(area.width as usize); // ╭ + " " + text + " " + ╮; 裁剪到屏幕宽度

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
    draw_menu_box(
        f,
        box_x,
        box_y,
        box_w as u16,
        &item_texts,
        app.menu.selected,
        area,
    );

    // 侧边栏：胜利记录
    if has_sidebar {
        let sidebar_x = box_x + box_w as u16 + 2;
        let victories = &app.menu.victories;
        let total_v = victories.len();
        let easy_count = victories
            .iter()
            .filter(|r| r.difficulty == "简单" || r.difficulty == "easy")
            .count();
        let med_count = victories
            .iter()
            .filter(|r| r.difficulty == "中等" || r.difficulty == "medium")
            .count();
        let hard_count = victories
            .iter()
            .filter(|r| r.difficulty == "困难" || r.difficulty == "hard")
            .count();

        // 侧边栏高 = 顶 + 1(标题) + 1(分隔) + 1(统计) + 1(分隔) + N(列表) + 1(底)
        let list_rows = victories
            .len()
            .min(area.bottom().saturating_sub(box_y + 5).saturating_sub(2) as usize);
        let sidebar_h = 5 + list_rows as u16 + 1;
        let sidebar_rect = Rect::new(sidebar_x, box_y, sidebar_w, sidebar_h);

        let block = Block::bordered()
            .title(Line::from(Span::styled(
                format!(" {} ", i18n::t("menu.sidebar_title", lang)),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        f.render_widget(block.clone(), sidebar_rect);
        let inner = block.inner(sidebar_rect);
        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let mut lines: Vec<Line> = Vec::with_capacity(inner.height as usize);
        // 统计行
        let stats = format!(
            " {}:{} {} {}:{} {} {}:{}",
            i18n::t("menu.sidebar_total", lang),
            total_v,
            i18n::t("game.diff_easy", lang)
                .chars()
                .next()
                .unwrap_or('E'),
            easy_count,
            i18n::t("game.diff_medium", lang)
                .chars()
                .next()
                .unwrap_or('M'),
            med_count,
            i18n::t("game.diff_hard", lang)
                .chars()
                .next()
                .unwrap_or('H'),
            hard_count
        );
        lines.push(Line::from(Span::styled(
            stats,
            Style::default().fg(Color::Yellow),
        )));
        // 分隔线
        lines.push(Line::from(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::Yellow),
        )));
        // 胜利列表
        for r in victories.iter().take(list_rows) {
            let name = if app.settings.naming_mode == "vivid" {
                i18n::vivid_name(r.id, lang)
            } else {
                format!("#{}", r.id)
            };
            let diff_short = match r.difficulty.as_str() {
                "简单" | "easy" => i18n::t("game.diff_easy", lang),
                "困难" | "hard" => i18n::t("game.diff_hard", lang),
                _ => i18n::t("game.diff_medium", lang),
            };
            let text = format!(
                " {} {} {:02}:{:02}",
                name,
                diff_short,
                r.elapsed_seconds / 60,
                r.elapsed_seconds % 60
            );
            lines.push(Line::from(Span::styled(
                text,
                Style::default().fg(Color::DarkGray),
            )));
        }

        f.render_widget(Paragraph::new(lines), inner);
    }

    // 提示文字 - 固定在屏幕底部
    let hint_row = area.bottom().saturating_sub(1);
    if !app.message.is_empty() {
        // Show message if present (use display_width so CJK chars count as 2)
        let msg_dw = display_width(&app.message) as u16;
        let msg_w = msg_dw.min(area.width);
        let msg_col = area.x + area.width.saturating_sub(msg_dw) / 2;
        f.render_widget(
            Paragraph::new(app.message.as_str()).style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Rect::new(msg_col, hint_row, msg_w, 1),
        );
    } else {
        // 提示行:左侧金币 + 右侧操作提示
        let gold_text = format!("💰 {}", app.gold);
        let hint = i18n::t("menu.hint_nav", lang);
        // 左:金币 (靠左)
        let gold_w = display_width(&gold_text) as u16;
        if area.width >= gold_w + 4 {
            f.render_widget(
                Paragraph::new(gold_text.as_str()).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Rect::new(area.x + 1, hint_row, gold_w, 1),
            );
        }
        // 右:操作提示
        let hint_dw = display_width(hint) as u16;
        let hint_w = hint_dw.min(area.width);
        let hint_col = area.x + area.width.saturating_sub(hint_dw + 1);
        f.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::White)),
            Rect::new(hint_col, hint_row, hint_w, 1),
        );
    }
}

pub(super) fn draw_menu_box(
    f: &mut Frame,
    x: u16,
    y: u16,
    w: u16,
    items: &[String],
    selected: usize,
    area: Rect,
) {
    if y >= area.bottom() {
        return;
    }
    let h = items.len() as u16 + 2; // 顶 + items + 底
    if y + h > area.bottom() {
        return;
    }

    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let block_area = Rect::new(x, y, w, h);
    f.render_widget(block.clone(), block_area);

    let inner = block.inner(block_area);
    if inner.height == 0 {
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(items.len());
    for (i, text) in items.iter().enumerate() {
        let prefix = if i == selected { "▸ " } else { "  " };
        let line_text = format!("{}{}", prefix, text);
        let style = if i == selected {
            Style::default().bg(Color::White).fg(Color::Black)
        } else {
            Style::default().bg(Color::Black).fg(Color::White)
        };
        lines.push(Line::from(Span::styled(line_text, style)));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

// ── 游戏画面 ──
