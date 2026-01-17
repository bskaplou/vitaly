use crate::{keycodes, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct KeyOverridesInformer<'a> {
    pub current_keycode: u16,
    pub key_overrides: &'a Vec<protocol::KeyOverride>,
    pub selected_layer: u8,
    pub vial_version: u32,
}

impl<'a> KeyOverridesInformer<'a> {
    pub fn new_if_applicable(
        current_keycode: Option<u16>,
        key_overrides: &'a Vec<protocol::KeyOverride>,
        selected_layer: u8,
        vial_version: u32,
    ) -> Option<Self> {
        if let Some(kc) = current_keycode {
            let has_override = key_overrides.iter().any(|ko| {
                !ko.is_empty() && ko.trigger == kc && ((ko.layers & 1 << selected_layer) != 0)
            });
            if has_override {
                return Some(Self {
                    current_keycode: kc,
                    key_overrides,
                    selected_layer,
                    vial_version,
                });
            }
        }
        None
    }
}

impl<'a> Widget for KeyOverridesInformer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let block = Block::default()
            .title("Key Override")
            .borders(Borders::ALL)
            .border_style(style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let matching_kos: Vec<&protocol::KeyOverride> = self
            .key_overrides
            .iter()
            .filter(|ko| {
                !ko.is_empty()
                    && ko.trigger == self.current_keycode
                    && ((ko.layers & 1 << self.selected_layer) != 0)
            })
            .collect();

        if matching_kos.is_empty() {
            Paragraph::new("No matching key overrides").render(inner_area, buf);
        } else {
            let mut lines = Vec::new();
            for ko in matching_kos {
                lines.push(Line::from(vec![Span::styled(
                    format!("Index: {}", ko.index),
                    Style::default().fg(Color::Yellow),
                )]));
                lines.push(Line::from(vec![
                    Span::raw("Trigger: "),
                    Span::styled(
                        keycodes::qid_to_name(ko.trigger, self.vial_version),
                        Style::default().fg(Color::Green),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::raw("Replacement: "),
                    Span::styled(
                        keycodes::qid_to_name(ko.replacement, self.vial_version),
                        Style::default().fg(Color::Green),
                    ),
                ]));

                if ko.trigger_mods != 0 {
                    lines.push(Line::from(vec![
                        Span::raw("Trig Mods: "),
                        Span::styled(
                            keycodes::bitmod_to_name(ko.trigger_mods),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]));
                }

                let mut opts = Vec::new();
                if ko.ko_enabled {
                    opts.push("Enabled");
                }
                if !opts.is_empty() {
                    lines.push(Line::from(vec![
                        Span::raw("Options: "),
                        Span::styled(opts.join(", "), Style::default().fg(Color::Gray)),
                    ]));
                }

                if lines.len() as u16 >= inner_area.height {
                    break;
                }
            }
            Paragraph::new(lines).render(inner_area, buf);
        }
    }
}
