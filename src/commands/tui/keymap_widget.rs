use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use super::ActiveWidget;
use super::layer_keymap::LayerKeymap;
use super::layer_selector::LayerSelector;
use crate::{keymap, protocol};

pub struct KeymapWidget<'a> {
    pub layer_count: u8,
    pub selected_layer: u8,
    pub buttons: &'a Vec<keymap::Button>,
    pub keys: &'a protocol::Keymap,
    pub vial_version: u32,
    pub active_widget: ActiveWidget,
    pub selected_button: usize,
}

impl<'a> Widget for KeymapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // Each layer number is formatted as " N " (3 chars).
        // Borders add 2 chars to width.
        let needed_width = (self.layer_count as u16 * 3) + 2;

        let top_row_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(needed_width), Constraint::Min(0)])
            .split(vertical_chunks[0]);

        let layer_selector = LayerSelector {
            count: self.layer_count,
            selected: self.selected_layer,
            is_active: self.active_widget == ActiveWidget::LayerSelector,
        };

        layer_selector.render(top_row_chunks[0], buf);

        let layer_keymap = LayerKeymap {
            buttons: self.buttons,
            keys: self.keys,
            layer: self.selected_layer,
            vial_version: self.vial_version,
            is_active: self.active_widget == ActiveWidget::Keymap,
            selected_button: self.selected_button,
        };
        layer_keymap.render(vertical_chunks[1], buf);
    }
}
