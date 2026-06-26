use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Widget,
};

/// 按钮状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonState {
    #[default]
    Normal,
    Selected,
    Hovered,
    Active,
}

/// 按钮主题
#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    pub text: Color,
    pub background: Color,
    pub highlight: Color,
    pub shadow: Color,
}

pub const THEME_PRIMARY: ButtonTheme = ButtonTheme {
    text: Color::White,
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
};

pub const THEME_SUCCESS: ButtonTheme = ButtonTheme {
    text: Color::White,
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
};

pub const THEME_DANGER: ButtonTheme = ButtonTheme {
    text: Color::White,
    background: Color::Rgb(144, 48, 48),
    highlight: Color::Rgb(192, 64, 64),
    shadow: Color::Rgb(96, 32, 32),
};

pub const THEME_NEUTRAL: ButtonTheme = ButtonTheme {
    text: Color::White,
    background: Color::DarkGray,
    highlight: Color::Gray,
    shadow: Color::Black,
};

/// 自定义按钮 Widget
///
/// 该 Widget 在 ratatui 的 `custom-widget` 示例基础上扩展，新增：
/// - `edge` 选项：可在按钮左右两侧自动附加 `◁` / `▷` 装饰，
///   装饰在不可用（edge faded）时前景色与背景色一致。
/// - `compact` 模式：仅渲染单行，不绘制 ▔/▁ 上下边框。
/// - `state` 与 `theme` 完全可定制，参考 `custom-widget` 示例。
#[derive(Debug, Clone)]
pub struct Button<'a> {
    pub label: Line<'a>,
    pub theme: ButtonTheme,
    pub state: ButtonState,
    pub show_border: bool,
    /// 在按钮左侧渲染 `◁` 装饰
    pub left_edge: Option<EdgeDecor>,
    /// 在按钮右侧渲染 `▷` 装饰
    pub right_edge: Option<EdgeDecor>,
}

/// 装饰图标（用于按钮左右侧 ◁ / ▷）。
/// `faded = true` 时将前景色与背景色设为相同，达到"不可用"的视觉提示。
#[derive(Debug, Clone, Copy)]
pub struct EdgeDecor {
    pub ch: char,
    pub faded: bool,
}

impl EdgeDecor {
    pub const fn new(ch: char) -> Self {
        Self { ch, faded: false }
    }
    pub const fn faded(mut self) -> Self {
        self.faded = true;
        self
    }
}

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: THEME_NEUTRAL,
            state: ButtonState::Normal,
            show_border: true,
            left_edge: None,
            right_edge: None,
        }
    }

    pub const fn theme(mut self, theme: ButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    pub const fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    pub const fn border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    pub const fn left_edge(mut self, decor: EdgeDecor) -> Self {
        self.left_edge = Some(decor);
        self
    }

    pub const fn right_edge(mut self, decor: EdgeDecor) -> Self {
        self.right_edge = Some(decor);
        self
    }

    fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            ButtonState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            ButtonState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Hovered => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }

    /// 计算按钮在区域内实际占用的列数（包含左右装饰）。
    pub fn width(&self, area_width: u16) -> u16 {
        let edges = self.left_edge.is_some() as u16 + self.right_edge.is_some() as u16;
        area_width.saturating_sub(edges)
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let (background, text, shadow, highlight) = self.colors();
        let bg_style = Style::new().bg(background).fg(text);

        // 填充背景
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf[(x, y)].set_style(bg_style);
            }
        }

        // 渲染上边框高光
        if self.show_border && area.height > 2 {
            for x in area.left()..area.right() {
                buf[(x, area.top())]
                    .set_symbol("▔")
                    .set_fg(highlight)
                    .set_bg(background);
            }
        }

        // 渲染下边框阴影
        if self.show_border && area.height > 1 {
            for x in area.left()..area.right() {
                buf[(x, area.bottom() - 1)]
                    .set_symbol("▁")
                    .set_fg(shadow)
                    .set_bg(background);
            }
        }

        // 装饰图标（左右边缘）—— 单独占用一格
        if let Some(decor) = self.left_edge {
            let fg = if decor.faded { background } else { highlight };
            let style = Style::new().fg(fg).bg(background).add_modifier(Modifier::BOLD);
            let mut tmp = [0u8; 4];
            let sym = decor.ch.encode_utf8(&mut tmp);
            buf[(area.left(), area.y + (area.height.saturating_sub(1)) / 2)]
                .set_symbol(sym)
                .set_style(style);
        }
        if let Some(decor) = self.right_edge {
            let fg = if decor.faded { background } else { highlight };
            let style = Style::new().fg(fg).bg(background).add_modifier(Modifier::BOLD);
            let mut tmp = [0u8; 4];
            let sym = decor.ch.encode_utf8(&mut tmp);
            buf[(area.right().saturating_sub(1), area.y + (area.height.saturating_sub(1)) / 2)]
                .set_symbol(sym)
                .set_style(style);
        }

        // 标签居中（避开左右装饰）
        let label_x = area.x
            + self.left_edge.is_some() as u16
            + (self.width(area.width).saturating_sub(self.label.width() as u16)) / 2;
        let label_y = area.y + (area.height.saturating_sub(1)) / 2;

        let max_label_w = self
            .width(area.width)
            .saturating_sub((self.label.width() as u16).saturating_sub(self.width(area.width)));
        buf.set_line(label_x, label_y, &self.label, area.width.max(max_label_w));
    }
}
