//! ScrollableList widget for SuDoKube
//!
//! A scrollable list widget with optional scrollbar and selection support.

use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style},
    symbols::scrollbar,
    text::Line,
    widgets::{Block, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget},
};

/// State for a scrollable list
#[derive(Debug, Clone)]
pub struct ScrollableListState {
    /// Currently selected item index
    pub selected: usize,
    /// Scroll position (first visible item)
    pub scroll_offset: usize,
    /// Total number of items
    pub item_count: usize,
    /// Number of visible lines (set during render)
    visible_lines: usize,
}

impl Default for ScrollableListState {
    fn default() -> Self {
        Self {
            selected: 0,
            scroll_offset: 0,
            item_count: 0,
            visible_lines: 10,
        }
    }
}

impl ScrollableListState {
    /// Create a new state with the given item count
    pub fn new(item_count: usize) -> Self {
        Self {
            selected: 0,
            scroll_offset: 0,
            item_count,
            visible_lines: 10,
        }
    }

    /// Update item count and adjust scroll if needed
    pub fn set_item_count(&mut self, count: usize) {
        self.item_count = count;
        self.selected = self.selected.min(count.saturating_sub(1));
        self.scroll_offset = self.scroll_offset.min(count.saturating_sub(1));
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.item_count {
            self.selected += 1;
            let max_visible = self.scroll_offset + self.visible_lines;
            if self.selected >= max_visible {
                self.scroll_offset = self.selected.saturating_sub(self.visible_lines) + 1;
            }
        }
    }

    /// Set visible lines (used for scroll calculation)
    pub fn set_visible_lines(&mut self, visible: usize) {
        self.visible_lines = visible;
    }

    /// Get scroll state for scrollbar
    pub fn scrollbar_state(&self) -> ScrollbarState {
        ScrollbarState::new(self.item_count).position(self.scroll_offset)
    }
}

/// Style configuration for scrollable list items
#[derive(Debug, Clone, Copy)]
pub struct ScrollableListStyle {
    /// Normal item text color
    pub text_color: Color,
    /// Selected item background
    pub selected_bg: Color,
    /// Selected item text color
    pub selected_fg: Color,
    /// Scrollbar thumb color
    pub scrollbar_color: Color,
}

impl Default for ScrollableListStyle {
    fn default() -> Self {
        Self {
            text_color: Color::White,
            selected_bg: Color::DarkGray,
            selected_fg: Color::Black,
            scrollbar_color: Color::Gray,
        }
    }
}

impl ScrollableListStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn selected_colors(mut self, bg: Color, fg: Color) -> Self {
        self.selected_bg = bg;
        self.selected_fg = fg;
        self
    }

    pub fn scrollbar_color(mut self, color: Color) -> Self {
        self.scrollbar_color = color;
        self
    }
}

/// A scrollable list widget with selection support
#[derive(Debug, Clone)]
pub struct ScrollableList<'a> {
    /// List items
    items: Vec<Line<'a>>,
    /// Optional block wrapper
    block: Option<Block<'a>>,
    /// Visual style
    style: ScrollableListStyle,
    /// Show scrollbar
    show_scrollbar: bool,
    /// Scrollbar position
    scrollbar_on_left: bool,
}

impl<'a> ScrollableList<'a> {
    /// Create a new scrollable list with items
    pub fn new(items: Vec<Line<'a>>) -> Self {
        Self {
            items,
            block: None,
            style: ScrollableListStyle::default(),
            show_scrollbar: true,
            scrollbar_on_left: false,
        }
    }

    /// Create from string items
    pub fn from_strings<T: Into<Line<'a>>>(items: Vec<T>) -> Self {
        let lines: Vec<Line<'a>> = items.into_iter().map(|s| s.into()).collect();
        Self::new(lines)
    }

    /// Wrap with a block
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the visual style
    #[must_use]
    pub fn style(mut self, style: ScrollableListStyle) -> Self {
        self.style = style;
        self
    }

    /// Show or hide the scrollbar
    #[must_use]
    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    /// Position scrollbar on the left side
    #[must_use]
    pub fn scrollbar_on_left(mut self) -> Self {
        self.scrollbar_on_left = true;
        self
    }

    fn content_area(&self, area: Rect) -> Rect {
        match &self.block {
            Some(block) => block.inner(area),
            None => area,
        }
    }
}

impl<'a> StatefulWidget for ScrollableList<'a> {
    type State = ScrollableListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Render block first if present
        if let Some(ref block) = self.block {
            block.render(area, buf);
        }

        let content = self.content_area(area);
        let content_h = content.height as usize;
        let item_count = self.items.len();

        if item_count == 0 || content_h == 0 {
            return;
        }

        // Update state
        state.item_count = item_count;
        state.set_visible_lines(content_h);

        // Calculate visible range
        let end = (state.scroll_offset + content_h).min(item_count);

        // Render visible items
        for (i, item_idx) in (state.scroll_offset..end).enumerate() {
            let row = content.y + i as u16;
            let is_selected = item_idx == state.selected;

            // Set line style based on selection
            let item_style = if is_selected {
                Style::default()
                    .bg(self.style.selected_bg)
                    .fg(self.style.selected_fg)
            } else {
                Style::default()
                    .bg(Color::Reset)
                    .fg(self.style.text_color)
            };

            // Render the line
            buf.set_line(content.x, row, &self.items[item_idx], content.width);

            // Apply style to entire row
            let row_rect = Rect::new(content.x, row, content.width, 1);
            buf.set_style(row_rect, item_style);
        }

        // Render scrollbar
        if self.show_scrollbar && item_count > content_h {
            let scrollbar_area = content.inner(Margin {
                vertical: 0,
                horizontal: 0,
            });

            let orientation = if self.scrollbar_on_left {
                ScrollbarOrientation::VerticalLeft
            } else {
                ScrollbarOrientation::VerticalRight
            };

            Scrollbar::new(orientation)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(None)
                .end_symbol(None)
                .thumb_style(Style::default().fg(self.style.scrollbar_color))
                .render(scrollbar_area, buf, &mut state.scrollbar_state());
        }
    }
}

/// Simple non-stateful list (all items always visible)
impl<'a> Widget for ScrollableList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = ScrollableListState::default();
        state.item_count = self.items.len();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_state() {
        let mut state = ScrollableListState::new(20);
        state.select_next();
        assert_eq!(state.selected, 1);
        state.select_prev();
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn test_scroll_bounds() {
        let mut state = ScrollableListState::new(5);
        for _ in 0..10 {
            state.select_next();
        }
        assert_eq!(state.selected, 4);
    }
}
