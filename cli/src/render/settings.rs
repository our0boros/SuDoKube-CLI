use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::types::*;
use super::util::*;
use crate::App;
use crate::i18n::{self, Lang};

pub fn compute_settings_popup_layout(area: Rect, app: &mut App) -> SettingsPopupLayout {
    let total_fields = app.settings_ui.fields.len();
    let desired_w = 46u16;
    let min_content_h = 6u16;
    let max_content_h = (area.height as i32 - 4).max(3) as u16;
    let content_h = (total_fields as u16 + 2).clamp(min_content_h, max_content_h);
    let box_h = content_h + 2 + 2;
    let box_w = desired_w.min(area.width.saturating_sub(2).max(10));

    let popup_area = Rect::new(
        area.x + area.width.saturating_sub(box_w) / 2,
        area.y + area.height.saturating_sub(box_h) / 2,
        box_w,
        box_h,
    );

    let block = Block::bordered().borders(Borders::ALL);
    let inner = block.inner(popup_area);
    let content_area = Rect::new(
        inner.x,
        inner.y,
        inner.width,
        inner.height.saturating_sub(1),
    );

    // 计算滚动位置
    let visible_rows = content_area.height as i32;
    let mut scroll = app.settings_ui.scroll as i32;
    if (app.settings_ui.selected as i32) < scroll {
        scroll = app.settings_ui.selected as i32;
    }
    if (app.settings_ui.selected as i32) >= scroll + visible_rows {
        scroll = app.settings_ui.selected as i32 - visible_rows + 1;
    }
    if scroll < 0 {
        scroll = 0;
    }
    let max_scroll = (total_fields as i32 - visible_rows).max(0);
    if scroll > max_scroll {
        scroll = max_scroll;
    }
    app.settings_ui.scroll = scroll as u16;

    let lang = Lang::from_code(&app.settings.language);
    let mut fields = Vec::new();
    for (i, field) in app.settings_ui.fields.iter().enumerate() {
        let row_in_view = i as i32 - scroll;
        if row_in_view < 0 || row_in_view >= visible_rows {
            continue;
        }
        let y = content_area.y + row_in_view as u16;
        let display_value = if field.label == "Naming Mode" {
            i18n::t(&format!("naming.{}", field.value), lang).to_string()
        } else {
            field.value.clone()
        };
        let label_text = format!(" {} ", field.label);
        let label_w = display_width(&label_text);
        let value_w = display_width(&display_value);
        // 计算 left_pad 同 draw 逻辑
        let inner_w = content_area.width as i32;
        let left_pad = (inner_w - label_w as i32 - 1 - 1 - value_w as i32 - 1 - 1).max(0) as u16;
        // 标签矩形
        let label_x = content_area.x;
        // 左侧装饰所在位置：label + left_pad
        let left_arrow_x = label_x + label_w as u16 + left_pad;
        let value_x = left_arrow_x + 2;
        let right_arrow_x = value_x + value_w as u16 + 2;
        fields.push(SettingsFieldLayout {
            row_y: y,
            label_rect: Rect::new(label_x, y, label_w as u16, 1),
            left_arrow_rect: Rect::new(left_arrow_x, y, 1, 1),
            value_rect: Rect::new(value_x, y, value_w as u16, 1),
            right_arrow_rect: Rect::new(right_arrow_x, y, 1, 1),
        });
    }

    SettingsPopupLayout {
        popup_area,
        content_area,
        fields,
    }
}

/// 设置弹窗（Popup 模式，叠加在任意画面上）
pub(super) fn draw_settings_overlay(f: &mut Frame, app: &App) {
    let area = f.area();
    let lang = Lang::from_code(&app.settings.language);

    // 弹窗尺寸
    let fields = &app.settings_ui.fields;
    let total_fields = fields.len();

    // 期望弹窗大小（基于内容）
    let desired_w = 46u16;
    let min_content_h = 6u16; // 标题1 + 分隔1 + 内容至少6 + 提示1 + 上下边框2 = 11
    let max_content_h = (area.height as i32 - 4).max(3) as u16; // 上下边框 + 提示共 4 行
    let content_h = (total_fields as u16 + 2).clamp(min_content_h, max_content_h);
    let box_h = content_h + 2 + 2; // 内容 + 上下边框(2) + 标题(1) + 提示(1)
    let box_w = desired_w.min(area.width.saturating_sub(2).max(10));

    // 居中弹窗
    let popup_area = Rect::new(
        area.x + area.width.saturating_sub(box_w) / 2,
        area.y + area.height.saturating_sub(box_h) / 2,
        box_w,
        box_h,
    );

    // 清除背景
    f.render_widget(Clear, popup_area);

    // 弹窗主块
    let block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" {} ", i18n::t("settings.title", lang)),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    f.render_widget(block.clone(), popup_area);
    let inner = block.inner(popup_area);

    // 提示行
    let hint_h = 1u16;
    let content_area = Rect::new(
        inner.x,
        inner.y,
        inner.width,
        inner.height.saturating_sub(hint_h),
    );
    let hint_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(hint_h),
        inner.width,
        hint_h,
    );

    // 自动滚动：保证 selected 在可见区域内
    let visible_rows = content_area.height as i32;
    let mut scroll = app.settings_ui.scroll as i32;
    if (app.settings_ui.selected as i32) < scroll {
        scroll = app.settings_ui.selected as i32;
    }
    if (app.settings_ui.selected as i32) >= scroll + visible_rows {
        scroll = app.settings_ui.selected as i32 - visible_rows + 1;
    }
    if scroll < 0 {
        scroll = 0;
    }
    let max_scroll = (total_fields as i32 - visible_rows).max(0);
    if scroll > max_scroll {
        scroll = max_scroll;
    }

    // 渲染字段
    for (i, field) in fields.iter().enumerate() {
        let row_in_view = i as i32 - scroll;
        if row_in_view < 0 || row_in_view >= visible_rows {
            continue;
        }
        let y = content_area.y + row_in_view as u16;
        let is_selected = i == app.settings_ui.selected;
        let is_hover = app.settings_ui.hover_field == Some(i);

        // 显示值
        let display_value = if field.label == "Naming Mode" {
            i18n::t(&format!("naming.{}", field.value), lang).to_string()
        } else {
            field.value.clone()
        };

        // 计算 ◁ / ▷ 边缘淡化
        let at_min = field.option_index == 0;
        let at_max = field.option_index + 1 >= field.options.len();
        let hl_fg = if is_selected {
            Color::Black
        } else {
            Color::White
        };
        let hl_bg = if is_selected {
            Color::White
        } else {
            Color::Black
        };
        let decor_fg_normal = if is_selected || is_hover {
            Color::Black
        } else {
            Color::Cyan
        };
        let decor_bg = if is_selected || is_hover {
            Color::Yellow
        } else {
            Color::Black
        };

        // 标签部分
        let label_text = format!(" {} ", field.label);
        let label_w = display_width(&label_text);
        let inner_w = content_area.width as i32;

        // ◁ / 值 / ▷
        let left_arrow_style = Style::default()
            .fg(if at_min { decor_bg } else { decor_fg_normal })
            .bg(decor_bg)
            .add_modifier(Modifier::BOLD);
        let right_arrow_style = Style::default()
            .fg(if at_max { decor_bg } else { decor_fg_normal })
            .bg(decor_bg)
            .add_modifier(Modifier::BOLD);

        let value_style = Style::default().fg(hl_fg).bg(hl_bg);
        let label_style = if is_selected || is_hover {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(label_text, label_style));
        let left_pad = (inner_w
            - label_w as i32
            - 1
            - 1
            - display_width(&display_value) as i32
            - 1
            - 1)
        .max(0) as usize;
        if left_pad > 0 {
            spans.push(Span::styled(" ".repeat(left_pad), label_style));
        }
        let left_hover =
            is_hover && app.settings_ui.hover_arrow == Some(crate::SettingsArrow::Left);
        let right_hover =
            is_hover && app.settings_ui.hover_arrow == Some(crate::SettingsArrow::Right);
        let left_sym = if left_hover { "◁" } else { "‹" };
        let right_sym = if right_hover { "▷" } else { "›" };
        // 仍然使用 ◁/▷ 但 hover 时加粗
        let left_symbol = if left_hover { "◁" } else { "‹" };
        let right_symbol = if right_hover { "▷" } else { "›" };
        let la = if left_hover {
            Style::default()
                .fg(Color::Red)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            left_arrow_style
        };
        let ra = if right_hover {
            Style::default()
                .fg(Color::Red)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            right_arrow_style
        };
        let _ = (left_sym, right_sym); // suppress unused
        spans.push(Span::styled(left_symbol, la));
        spans.push(Span::styled(" ", label_style));
        spans.push(Span::styled(display_value.clone(), value_style));
        spans.push(Span::styled(" ", label_style));
        spans.push(Span::styled(right_symbol, ra));
        // 填充剩余
        let used = label_w + left_pad + 1 + 1 + display_width(&display_value) + 1 + 1;
        if used < inner_w as usize {
            spans.push(Span::styled(
                " ".repeat(inner_w as usize - used),
                label_style,
            ));
        }

        f.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(content_area.x, y, content_area.width, 1),
        );
    }

    // 提示
    f.render_widget(
        Paragraph::new(i18n::t("settings.hint", lang)).style(Style::default().fg(Color::DarkGray)),
        hint_area,
    );

    // 溢出检测：内容超过可见区域时显示 scrollbar
    if (total_fields as i32) > visible_rows {
        use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(Color::Cyan))
            .track_style(Style::default().fg(Color::DarkGray));
        let mut state = ScrollbarState::new(total_fields).position(scroll as usize);
        f.render_stateful_widget(
            scrollbar,
            Rect::new(
                content_area.x + content_area.width.saturating_sub(1),
                content_area.y,
                1,
                content_area.height,
            ),
            &mut state,
        );
    }
}

// ── 胜利画面 ──
