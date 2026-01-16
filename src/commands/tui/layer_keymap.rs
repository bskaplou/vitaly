use crate::{common, keymap, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Widget},
};

use super::BORDER_COLOR_ACTIVE;

pub struct LayerKeymap<'a> {
    pub buttons: &'a mut Vec<keymap::Button>,
    pub keys: &'a protocol::Keymap,
    pub layer: u8,
    pub vial_version: u32,
    pub is_active: bool,
    pub selected_button: usize,
}

impl<'a> Widget for LayerKeymap<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.is_active {
            Style::default().fg(BORDER_COLOR_ACTIVE)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title("Keymap")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Modify button color in place
        let original_color = if self.selected_button < self.buttons.len() {
            let button = &mut self.buttons[self.selected_button];
            let original = button.color;
            let color = if self.is_active {
                (0, 255, 255) // Cyan
            } else {
                (100, 100, 100) // Dark Gray
            };
            button.color = Some(color);
            Some(original)
        } else {
            None
        };

        if let Ok(data) = common::prepare_layer_render(
            self.keys,
            self.buttons,
            self.layer,
            self.vial_version,
            &None,
        ) {
            // Render grid
            for (y, row) in data.buffer.b.iter().enumerate() {
                let target_y = inner_area.y + y as u16;
                if target_y >= inner_area.bottom() {
                    break;
                }
                for (x, pos) in row.iter().enumerate() {
                    let target_x = inner_area.x + x as u16;
                    if target_x >= inner_area.right() {
                        break;
                    }

                    let mut style = Style::default();
                    if let Some((r, g, b)) = pos.color {
                        // Calculate foreground color for contrast (simple version)
                        let brightness =
                            (r as f64 * 299.0 + g as f64 * 587.0 + b as f64 * 114.0) / 1000.0;
                        let fg = if brightness > 128.0 {
                            Color::Black
                        } else {
                            Color::White
                        };
                        style = style.bg(Color::Rgb(r, g, b)).fg(fg);
                    }

                    if let Some(cell) = buf.cell_mut((target_x, target_y)) {
                        cell.set_char(pos.sym).set_style(style);
                    }
                }
            }

            // Render fat labels below grid if space allows
            let grid_height = data.buffer.b.len() as u16;
            let mut current_y = inner_area.y + grid_height;

            for (idx, fat) in data.fat_labels.iter().enumerate() {
                if current_y >= inner_area.bottom() {
                    break;
                }
                let line = Line::from(format!("*{} - {}", idx + 1, fat));
                buf.set_line(inner_area.x, current_y, &line, inner_area.width);
                current_y += 1;
            }
        }

        // Restore original color
        if let Some(original) = original_color
            && self.selected_button < self.buttons.len() {
                self.buttons[self.selected_button].color = original;
            }
    }
}
