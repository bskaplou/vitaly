use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Widget},
};

use super::{
    BORDER_COLOR_ACTIVE, SELECTED_BGCOLOR_ACTIVE, SELECTED_BGCOLOR_INACTIVE,
    SELECTED_COLOR_ACTIVE, SELECTED_COLOR_INACTIVE,
};

pub struct LayerSelector {
    pub count: u8,
    pub selected: u8,
    pub is_active: bool,
}

impl Widget for LayerSelector {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.is_active {
            Style::default().fg(BORDER_COLOR_ACTIVE)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title("Layer")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let mut current_x = inner_area.x;
        for i in 0..self.count {
            let is_selected = i == self.selected;
            let text = format!(" {} ", i);
            let text_len = text.len() as u16;

            if current_x + text_len > inner_area.right() {
                break;
            }

            let mut style = Style::default();
            if is_selected {
                style = style.add_modifier(Modifier::REVERSED);
                if self.is_active {
                    style = style.bg(SELECTED_COLOR_ACTIVE).fg(SELECTED_BGCOLOR_ACTIVE);
                } else {
                    style = style
                        .bg(SELECTED_COLOR_INACTIVE)
                        .fg(SELECTED_BGCOLOR_INACTIVE);
                }
            }

            let span = Span::styled(text, style);
            buf.set_span(current_x, inner_area.y, &span, text_len);
            current_x += text_len;
        }
    }
}
