use crate::{keycodes, keymap, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct EncoderInformer<'a> {
    pub selected_layer: u8,
    pub selected_button: &'a keymap::Button,
    pub encoders: &'a Vec<Vec<protocol::Encoder>>,
    pub vial_version: u32,
}

impl<'a> EncoderInformer<'a> {
    pub fn new_if_applicable(
        selected_layer: u8,
        selected_button: Option<&'a keymap::Button>,
        encoders: &'a Vec<Vec<protocol::Encoder>>,
        vial_version: u32,
    ) -> Option<Self> {
        if let Some(button) = selected_button
            && button.encoder {
                return Some(Self {
                    selected_layer,
                    selected_button: button,
                    encoders,
                    vial_version,
                });
            }
        None
    }
}

impl<'a> Widget for EncoderInformer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let block = Block::default()
            .title("Encoder")
            .borders(Borders::ALL)
            .border_style(style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let button = self.selected_button;
        let encoder_index = button.wire_x as usize;
        let is_cw = button.wire_y == 1;

        if let Some(layer_encoders) = self.encoders.get(self.selected_layer as usize)
            && let Some(encoder) = layer_encoders.get(encoder_index) {
                let keycode = if is_cw { encoder.cw } else { encoder.ccw };
                let keycode_name = keycodes::qid_to_name(keycode, self.vial_version);

                let lines = vec![
                    Line::from(vec![
                        Span::raw("Layer: "),
                        Span::styled(
                            format!("{}", self.selected_layer),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Index: "),
                        Span::styled(
                            format!("{}", encoder_index),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Direction: "),
                        Span::styled(
                            if is_cw { "CW" } else { "CCW" },
                            Style::default().fg(Color::Cyan),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Keycode: "),
                        Span::styled(keycode_name, Style::default().fg(Color::Green)),
                    ]),
                    Line::from(vec![
                        Span::raw("Hex: "),
                        Span::styled(
                            format!("{:#06x}", keycode),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]),
                ];
                Paragraph::new(lines).render(inner_area, buf);
                return;
            }
        Paragraph::new("Encoder data not found").render(inner_area, buf);
    }
}