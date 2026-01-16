use crate::{keycodes, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct CombosInformer<'a> {
    pub current_keycode: u16,
    pub combos: &'a Vec<protocol::Combo>,
    pub vial_version: u32,
}

impl<'a> CombosInformer<'a> {
    pub fn new_if_applicable(
        current_keycode: Option<u16>,
        combos: &'a Vec<protocol::Combo>,
        vial_version: u32,
    ) -> Option<Self> {
        if let Some(kc) = current_keycode {
            let has_combos = combos.iter().any(|c| {
                !c.is_empty()
                    && (c.key1 == kc || c.key2 == kc || c.key3 == kc || c.key4 == kc)
            });
            if has_combos {
                return Some(Self {
                    current_keycode: kc,
                    combos,
                    vial_version,
                });
            }
        }
        None
    }
}

impl<'a> Widget for CombosInformer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let block = Block::default()
            .title("Combos")
            .borders(Borders::ALL)
            .border_style(style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Filter combos
        let matching_combos: Vec<&protocol::Combo> = self
            .combos
            .iter()
            .filter(|c| {
                !c.is_empty()
                    && (c.key1 == self.current_keycode
                        || c.key2 == self.current_keycode
                        || c.key3 == self.current_keycode
                        || c.key4 == self.current_keycode)
            })
            .collect();

        if matching_combos.is_empty() {
            Paragraph::new("No matching combos").render(inner_area, buf);
        } else {
            let mut lines = Vec::new();
            for combo in matching_combos {
                // Format combo string: K1 + K2 + ... = OUT
                let mut parts = Vec::new();
                if combo.key1 != 0 {
                    parts.push(keycodes::qid_to_name(combo.key1, self.vial_version));
                }
                if combo.key2 != 0 {
                    parts.push(keycodes::qid_to_name(combo.key2, self.vial_version));
                }
                if combo.key3 != 0 {
                    parts.push(keycodes::qid_to_name(combo.key3, self.vial_version));
                }
                if combo.key4 != 0 {
                    parts.push(keycodes::qid_to_name(combo.key4, self.vial_version));
                }

                let input_str = parts.join(" + ");
                let output_str = keycodes::qid_to_name(combo.output, self.vial_version);

                lines.push(Line::from(vec![
                    Span::raw(format!("{}: ", combo.index)),
                    Span::styled(input_str, Style::default().fg(Color::Yellow)),
                    Span::raw(" = "),
                    Span::styled(output_str, Style::default().fg(Color::Green)),
                ]));

                if lines.len() as u16 >= inner_area.height {
                    break;
                }
            }
            Paragraph::new(lines).render(inner_area, buf);
        }
    }
}