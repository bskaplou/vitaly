use crate::{common, keymap, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Widget},
};

use super::{
    BORDER_COLOR_ACTIVE, SELECTED_BGCOLOR_ACTIVE, SELECTED_BGCOLOR_INACTIVE,
    SELECTED_COLOR_ACTIVE, SELECTED_COLOR_INACTIVE,
};

pub struct LayerKeymap<'a> {
    pub buttons: &'a Vec<keymap::Button>,
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

            // Highlight selected button
            if self.selected_button < self.buttons.len() {
                let button = &self.buttons[self.selected_button];
                let scale = 4.0;
                let b = button.scale(scale);

                // Calculate bounds (matching src/keymap.rs render_and_dump logic)
                let lu = (b.x.round() as usize, b.y.round() as usize);
                let rb = (
                    (b.x + b.w - 1.0).round() as usize,
                    (b.y + b.h - 1.0).round() as usize,
                );

                let start_x = lu.0;
                let end_x = rb.0;
                let start_y = lu.1;
                let end_y = rb.1;

                let highlight_style = if self.is_active {
                    Style::default()
                        .bg(SELECTED_BGCOLOR_ACTIVE)
                        .fg(SELECTED_COLOR_ACTIVE)
                } else {
                    Style::default()
                        .bg(SELECTED_BGCOLOR_INACTIVE)
                        .fg(SELECTED_COLOR_INACTIVE)
                };

                for y in start_y..=end_y {
                    let target_y = inner_area.y + y as u16;
                    if target_y >= inner_area.bottom() {
                        continue;
                    }

                    for x in start_x..=end_x {
                        let target_x = inner_area.x + x as u16;
                        if target_x >= inner_area.right() {
                            continue;
                        }

                        if let Some(cell) = buf.cell_mut((target_x, target_y)) {
                            cell.set_style(highlight_style);
                        }
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
    }
}
