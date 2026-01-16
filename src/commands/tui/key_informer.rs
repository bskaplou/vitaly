use crate::{keymap, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct KeyInformer<'a> {
    pub selected_layer: u8,
    pub selected_button: &'a keymap::Button,
    pub keys: &'a protocol::Keymap,
    pub vial_version: u32,
}

impl<'a> KeyInformer<'a> {
    pub fn new_if_applicable(
        selected_layer: u8,
        selected_button: Option<&'a keymap::Button>,
        keys: &'a protocol::Keymap,
        vial_version: u32,
    ) -> Option<Self> {
        if let Some(button) = selected_button
            && !button.encoder {
                return Some(Self {
                    selected_layer,
                    selected_button: button,
                    keys,
                    vial_version,
                });
            }
        None
    }
}

impl<'a> Widget for KeyInformer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let block = Block::default()
            .title("Key")
            .borders(Borders::ALL)
            .border_style(style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let button = self.selected_button;
        let wire_x = button.wire_x;
        let wire_y = button.wire_y;

        let keycode_hex = self.keys.get(self.selected_layer, wire_x, wire_y);
        let keycode_name = self
            .keys
            .get_long(self.selected_layer, wire_x, wire_y, self.vial_version)
            .unwrap_or_else(|_| "???".to_string());

        let lines = vec![
            Line::from(vec![
                Span::raw("Layer: "),
                Span::styled(
                    format!("{}", self.selected_layer),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Position: "),
                Span::styled(
                    format!("{}, {}", wire_x, wire_y),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Keycode: "),
                Span::styled(keycode_name, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::raw("Hex: "),
                Span::styled(
                    format!("{:#06x}", keycode_hex),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        Paragraph::new(lines).render(inner_area, buf);
    }
}