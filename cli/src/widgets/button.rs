//! Custom Button widget for SuDoKube
//!
//! A styled button widget with hover and active states.
//! Based on ratatui's custom-widget example.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

/// Button state for rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Normal state - default appearance
    Normal,
    /// Selected/Focused state - highlighted
    Selected,
    /// Active/Pressed state
    Active,
    /// Disabled state - grayed out
    Disabled,
}

/// Theme colors for buttons
#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    /// Text color
    pub text: Color,
    /// Background color
    pub background: Color,
    /// Highlight/border color
    pub highlight: Color,
    /// Shadow/pressed effect color
    pub shadow: Color,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            text: Color::White,
            background: Color::DarkGray,
            highlight: Color::Gray,
            shadow: Color::Black,
        }
    }
}

impl ButtonTheme {
    /// Cyan theme (default for game UI)
    pub fn cyan() -> Self {
        Self {
            text: Color::Black,
            background: Color::Cyan,
            highlight: Color::LightCyan,
            shadow: Color::DarkGray,
        }
    }

    /// White theme
    pub fn white() -> Self {
        Self {
            text: Color::Black,
            background: Color::White,
            highlight: Color::Gray,
            shadow: Color::DarkGray,
        }
    }

    /// Green theme (for positive actions)
    pub fn green() -> Self {
        Self {
            text: Color::White,
            background: Color::Green,
            highlight: Color::LightGreen,
            shadow: Color::Black,
        }
    }

    /// Red theme (for destructive actions)
    pub fn red() -> Self {
        Self {
            text: Color::White,
            background: Color::Red,
            highlight: Color::LightRed,
            shadow: Color::Black,
        }
    }

    /// Yellow theme (for warnings/info)
    pub fn yellow() -> Self {
        Self {
            text: Color::Black,
            background: Color::Yellow,
            highlight: Color::LightYellow,
            shadow: Color::Black,
        }
    }
}

/// A custom button widget with label and theme support
#[derive(Debug, Clone)]
pub struct Button<'a> {
    /// Button label text
    label: Line<'a>,
    /// Visual theme
    theme: ButtonTheme,
    /// Current state
    state: ButtonState,
    /// Minimum width (optional)
    min_width: Option<u16>,
}

impl<'a> Button<'a> {
    /// Create a new button with the given label
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: ButtonTheme::default(),
            state: ButtonState::Normal,
            min_width: None,
        }
    }

    /// Set the button theme
    #[must_use]
    pub fn theme(mut self, theme: ButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the button state
    #[must_use]
    pub fn state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    /// Set minimum width
    #[must_use]
    pub fn min_width(mut self, width: u16) -> Self {
        self.min_width = Some(width);
        self
    }

    fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            ButtonState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            ButtonState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            ButtonState::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
            ButtonState::Disabled => (
                Color::DarkGray,
                Color::Gray,
                Color::Black,
                Color::DarkGray,
            ),
        }
    }

    fn effective_width(&self, area_width: u16) -> u16 {
        let content_width = self.label.width() as u16;
        let min = self.min_width.unwrap_or(content_width);
        min.max(content_width).min(area_width)
    }
}

impl Widget for Button<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 2 || area.height < 1 {
            return;
        }

        let (background, text, shadow, highlight) = self.colors();
        let effective_w = self.effective_width(area.width);
        let padding = (area.width - effective_w) / 2;

        // Calculate render area (centered)
        let render_area = Rect::new(
            area.x + padding,
            area.y,
            effective_w,
            area.height,
        );

        // Set background
        buf.set_style(render_area, Style::new().bg(background));

        // Draw top highlight line
        if render_area.height > 1 {
            let highlight_line = "▔".repeat(render_area.width as usize);
            buf.set_string(
                render_area.x,
                render_area.y,
                highlight_line,
                Style::new().fg(highlight).bg(background),
            );
        }

        // Draw bottom shadow line
        if render_area.height > 1 {
            let shadow_line = "▁".repeat(render_area.width as usize);
            buf.set_string(
                render_area.x,
                render_area.y + render_area.height - 1,
                shadow_line,
                Style::new().fg(shadow).bg(background),
            );
        }

        // Draw label centered
        let label_width = self.label.width() as u16;
        let label_x = render_area.x + (render_area.width.saturating_sub(label_width)) / 2;
        let label_y = render_area.y + (render_area.height.saturating_sub(1)) / 2;

        buf.set_line(label_x, label_y, &self.label, render_area.width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let btn = Button::new("Test");
        assert_eq!(btn.state, ButtonState::Normal);
    }

    #[test]
    fn test_button_theme() {
        let btn = Button::new("Test").theme(ButtonTheme::cyan());
        assert_eq!(btn.theme.background, Color::Cyan);
    }

    #[test]
    fn test_button_min_width() {
        let btn = Button::new("Hi").min_width(10);
        assert_eq!(btn.min_width, Some(10));
    }
}
