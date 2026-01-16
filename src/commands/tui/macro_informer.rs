use crate::{keycodes, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct MacroInformer<'a> {
    pub current_keycode: u16,
    pub macros: &'a Vec<protocol::Macro>,
    pub vial_version: u32,
}

impl<'a> MacroInformer<'a> {
    pub fn new_if_applicable(
        current_keycode: Option<u16>,
        macros: &'a Vec<protocol::Macro>,
        vial_version: u32,
    ) -> Option<Self> {
        if let Some(kc) = current_keycode
            && let Some(idx) = keycodes::is_macro(kc, vial_version)
                && macros.iter().any(|m| m.index == idx) {
                    return Some(Self {
                        current_keycode: kc,
                        macros,
                        vial_version,
                    });
                }
        None
    }
}

impl<'a> Widget for MacroInformer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let block = Block::default()
            .title("Macro")
            .borders(Borders::ALL)
            .border_style(style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(idx) = keycodes::is_macro(self.current_keycode, self.vial_version) {
            if let Some(macro_obj) = self.macros.iter().find(|m| m.index == idx) {
                let mut lines = Vec::new();
                lines.push(Line::from(vec![Span::styled(
                    format!("Index: {}", macro_obj.index),
                    Style::default().fg(Color::Yellow),
                )]));

                for step in &macro_obj.steps {
                    let text = match step {
                        protocol::MacroStep::Tap(kc) => {
                            format!("Tap({})", keycodes::qid_to_name(*kc, self.vial_version))
                        }
                        protocol::MacroStep::Down(kc) => {
                            format!("Down({})", keycodes::qid_to_name(*kc, self.vial_version))
                        }
                        protocol::MacroStep::Up(kc) => {
                            format!("Up({})", keycodes::qid_to_name(*kc, self.vial_version))
                        }
                        protocol::MacroStep::Delay(ms) => format!("Delay({})", ms),
                        protocol::MacroStep::Text(txt) => format!("Text({})", txt),
                    };
                    lines.push(Line::from(Span::raw(text)));
                    if lines.len() as u16 >= inner_area.height {
                        break;
                    }
                }
                Paragraph::new(lines).render(inner_area, buf);
            } else {
                Paragraph::new("Macro not found").render(inner_area, buf);
            }
        } else {
            Paragraph::new("Not a Macro key").render(inner_area, buf);
        }
    }
}