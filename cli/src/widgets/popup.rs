//! Popup widget for SuDoKube
//!
//! A modal popup widget that can display centered content over the background.
//! Uses Clear widget to hide background content.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

/// Kind of popup border style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    /// Simple single-line border
    Simple,
    /// Double-line border (box drawing)
    Double,
    /// Rounded corners
    Rounded,
}

impl Default for PopupKind {
    fn default() -> Self {
        Self::Simple
    }
}

impl PopupKind {
    fn border_type(&self) -> BorderType {
        match self {
            Self::Simple => BorderType::Plain,
            Self::Double => BorderType::Double,
            Self::Rounded => BorderType::Rounded,
        }
    }
}

/// A popup widget that renders centered content over a cleared background
#[derive(Debug, Clone)]
pub struct Popup<'a> {
    /// Popup title (optional)
    title: Option<&'a str>,
    /// Border and text color
    color: Color,
    /// Background color
    background: Color,
    /// Border style
    kind: PopupKind,
    /// Padding inside the popup
    padding: u16,
}

impl<'a> Popup<'a> {
    /// Create a new popup with default styles
    pub fn new() -> Self {
        Self {
            title: None,
            color: Color::Cyan,
            background: Color::Black,
            kind: PopupKind::Simple,
            padding: 1,
        }
    }

    /// Set the popup title
    #[must_use]
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the border/text color
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the background color
    #[must_use]
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set the border style
    #[must_use]
    pub fn kind(mut self, kind: PopupKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set the padding inside the popup
    #[must_use]
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Calculate the inner area available for content
    pub fn inner_area(&self, outer: Rect) -> Rect {
        let block = self.build_block();
        block.inner(outer)
    }

    fn build_block(&self) -> Block<'a> {
        let mut block = Block::bordered()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(self.color))
            .border_type(self.kind.border_type())
            .style(Style::new().bg(self.background));

        if let Some(title) = self.title {
            block = block.title(format!(" {} ", title));
        }

        if self.padding > 0 {
            block = block.padding(ratatui::widgets::Padding::horizontal(self.padding));
        }

        block
    }
}

impl<'a> Default for Popup<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the background
        Clear.render(area, buf);

        // Build the popup block
        let block = self.build_block();

        // Calculate centered position
        let block_area = block.inner(area);
        let block_width = block_area.width + 2 + self.padding * 2;
        let block_height = block_area.height + 2 + self.padding * 2;

        let x = area.x + area.width.saturating_sub(block_width) / 2;
        let y = area.y + area.height.saturating_sub(block_height) / 2;

        let popup_rect = Rect::new(x, y, block_width, block_height);

        // Render the block
        block.render(popup_rect, buf);
    }
}

/// Render a popup with content widget
pub fn render_popup<'a, W: Widget>(
    frame: &mut ratatui::Frame,
    area: Rect,
    popup: Popup<'a>,
    content: W,
) {
    // Clear background
    Clear.render(area, frame.buffer_mut());

    // Build block for sizing
    let mut block = Block::bordered()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(popup.color))
        .border_type(popup.kind.border_type())
        .style(Style::new().bg(popup.background));

    if let Some(title) = popup.title {
        block = block.title(format!(" {} ", title));
    }

    if popup.padding > 0 {
        block = block.padding(ratatui::widgets::Padding::horizontal(popup.padding));
    }

    // Calculate popup dimensions
    let inner = block.inner(area);
    let popup_w = inner.width + 2;
    let popup_h = inner.height + 2;

    // Center the popup
    let popup_x = area.x + area.width.saturating_sub(popup_w) / 2;
    let popup_y = area.y + area.height.saturating_sub(popup_h) / 2;
    let popup_rect = Rect::new(popup_x, popup_y, popup_w, popup_h);

    // Render block
    block.render(popup_rect, frame.buffer_mut());

    // Render content in inner area
    let content_rect = Rect::new(
        popup_x + 1,
        popup_y + 1,
        popup_w.saturating_sub(2),
        popup_h.saturating_sub(2),
    );
    content.render(content_rect, frame.buffer_mut());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popup_creation() {
        let popup = Popup::new();
        assert_eq!(popup.color, Color::Cyan);
        assert_eq!(popup.background, Color::Black);
    }

    #[test]
    fn test_popup_with_title() {
        let popup = Popup::new().title("Settings");
        assert_eq!(popup.title, Some("Settings"));
    }
}
