use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::i18n::{self, Lang};
use crate::{App, total_elapsed};
use super::util::*;

use std::time::Instant;

pub(super) fn draw_confirm_delete_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);

    let box_w = 36u16;
    let box_h = 5u16;
    let popup_area = Rect::new(
        area.x + area.width.saturating_sub(box_w) / 2,
        area.y + area.height.saturating_sub(box_h) / 2,
        box_w,
        box_h,
    );

    f.render_widget(Clear, popup_area);

    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(Span::styled(
            format!(" {} ", i18n::t("menu.delete_confirm", lang)),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block.clone(), popup_area);
    let inner = block.inner(popup_area);
    if inner.height < 2 || inner.width < 4 {
        return;
    }

    let lines = vec![
        Line::from(Span::styled(
            i18n::t("menu.delete_confirm", lang),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            i18n::t("menu.delete_hint", lang),
            Style::default().fg(Color::Yellow),
        )),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

// ── 菜单画面 ──


pub(super) fn draw_victory(f: &mut Frame, app: &App) {
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );

    let lang = Lang::from_code(&app.settings.language);
    let title = i18n::t("victory.title", lang);
    let subtitle = i18n::t("victory.subtitle", lang);

    // Calculate remaining seconds
    let remaining_secs = app
        .victory_countdown
        .map(|until| {
            let left = (until - Instant::now()).as_secs() as u32;
            left.max(0)
        })
        .unwrap_or(0);

    let countdown_text = format!(
        "{}  |  {}",
        i18n::t("victory.enter", lang),
        format!("{}", remaining_secs)
    );

    let box_w = 36u16;
    let box_h = 7u16;
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
    let title_line = bordered_line(&title, inner_w, true);
    f.render_widget(
        Paragraph::new(title_line).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(box_x, box_y + 1, box_w, 1),
    );
    // Subtitle
    let sub_line = bordered_line(&subtitle, inner_w, true);
    f.render_widget(
        Paragraph::new(sub_line).style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(box_x, box_y + 2, box_w, 1),
    );
    // Separator
    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y + 3, box_w, 1),
    );
    // Info: diff + time
    let diff = app.game.difficulty.as_str();
    let elapsed = total_elapsed(app);
    let info_line = format!(
        "│ {:^width$} │",
        format!("{}  {:02}:{:02}", diff, elapsed / 60, elapsed % 60),
        width = inner_w - 2
    );
    f.render_widget(
        Paragraph::new(info_line).style(Style::default().fg(Color::White)),
        Rect::new(box_x, box_y + 4, box_w, 1),
    );
    // 金币收益
    let reward_line = format!(
        "│ {:^width$} │",
        format!("💰 +{} gold  (total: {})", app.last_reward, app.gold),
        width = inner_w - 2
    );
    f.render_widget(
        Paragraph::new(reward_line).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Rect::new(box_x, box_y + 5, box_w, 1),
    );
    // Bottom + countdown
    let bot_line = bordered_line(&countdown_text, inner_w, true);
    f.render_widget(
        Paragraph::new(bot_line).style(Style::default().fg(Color::DarkGray)),
        Rect::new(box_x, box_y + 6, box_w, 1),
    );
    // Bottom border
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Yellow)),
        Rect::new(box_x, box_y + box_h, box_w, 1),
    );
}

// ── 导出选择弹窗 ──

/// 导出选择弹窗(Popup 模式,叠加在菜单画面上)
pub(super) fn draw_export_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);

    // 弹窗尺寸
    let box_w = 40u16;
    let box_h = 6u16;

    // 居中弹窗
    let popup_area = Rect::new(
        area.x + area.width.saturating_sub(box_w) / 2,
        area.y + area.height.saturating_sub(box_h) / 2,
        box_w,
        box_h,
    );

    // 清除背景
    f.render_widget(Clear, popup_area);

    let inner_w = box_w as usize - 2;

    // 顶部边框
    let top = format!("╭{}╮", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y, box_w, 1),
    );

    // 标题
    let title = bordered_line(i18n::t("menu.export", lang), inner_w, true);
    f.render_widget(
        Paragraph::new(title).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y + 1, box_w, 1),
    );

    // 分隔线
    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y + 2, box_w, 1),
    );

    // 选项
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
        let line = bordered_line(&text, inner_w, false);
        f.render_widget(
            Paragraph::new(line).style(style),
            Rect::new(popup_area.x, popup_area.y + 3 + i as u16, box_w, 1),
        );
    }

    // 提示
    let hint = bordered_line("Enter OK / Esc Cancel", inner_w, true);
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        Rect::new(popup_area.x, popup_area.y + 5, box_w, 1),
    );

    // 底部
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y + 6, box_w, 1),
    );
}

// ── 导入输入画面 ──

/// 导入弹窗（Popup 模式，叠加在任意画面上）
pub(super) fn draw_import_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);

    // 弹窗尺寸
    let box_w = 50u16;
    let box_h = 6u16;

    // 居中弹窗
    let popup_area = Rect::new(
        area.x + area.width.saturating_sub(box_w) / 2,
        area.y + area.height.saturating_sub(box_h) / 2,
        box_w,
        box_h,
    );

    // 清除背景
    f.render_widget(Clear, popup_area);

    let inner_w = box_w as usize - 2;

    // 顶部边框
    let top = format!("╭{}╮", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(top).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y, box_w, 1),
    );

    // 标题
    let title = bordered_line(i18n::t("menu.import", lang), inner_w, true);
    f.render_widget(
        Paragraph::new(title).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y + 1, box_w, 1),
    );

    // 分隔线
    let sep = format!("╟{}╢", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y + 2, box_w, 1),
    );

    // 输入框
    let display_text = if app.import_buffer.is_empty() {
        i18n::t("import.paste", lang).to_string()
    } else {
        let buf = &app.import_buffer;
        let max_chars = inner_w.saturating_sub(6);
        if buf.chars().count() > max_chars {
            let truncated: String = buf
                .chars()
                .skip(buf.chars().count().saturating_sub(max_chars))
                .collect();
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
    let input_line = format!("│ {} │", pad_right(&display_text, inner_w - 2));
    f.render_widget(
        Paragraph::new(input_line).style(input_style),
        Rect::new(popup_area.x, popup_area.y + 3, box_w, 1),
    );

    // 提示
    let hint = bordered_line("Enter OK / Esc Cancel", inner_w, true);
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        Rect::new(popup_area.x, popup_area.y + 4, box_w, 1),
    );

    // 底部
    let bot = format!("╰{}╯", "─".repeat(inner_w));
    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(popup_area.x, popup_area.y + 5, box_w, 1),
    );
}

// ── 生成进度画面 ──

pub(super) fn draw_generating(f: &mut Frame, app: &App) {
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
    let text = format!(
        "  {} {} {}s  ",
        spinner_char,
        i18n::t("msg.generating", lang),
        elapsed
    );
    let hint_text = format!("  {}  ", i18n::t("msg.gen_cancel_hint", lang));
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
    let content = bordered_line(&text, inner_w, true);
    f.render_widget(
        Paragraph::new(content).style(Style::default().fg(Color::White)),
        Rect::new(x, y + 1, bar_w, 1),
    );

    // 取消提示行(按 Esc 中断)
    let hint = bordered_line(&hint_text, inner_w, true);
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        Rect::new(x, y + 2, bar_w, 1),
    );

    f.render_widget(
        Paragraph::new(bot).style(Style::default().fg(Color::Cyan)),
        Rect::new(x, y + 3, bar_w, 1),
    );
}



