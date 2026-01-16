use crate::{keycodes, protocol};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct TapDanceInformer<'a> {
    pub current_keycode: u16,
    pub tap_dances: &'a Vec<protocol::TapDance>,
    pub vial_version: u32,
}

impl<'a> TapDanceInformer<'a> {
    pub fn new_if_applicable(
        current_keycode: Option<u16>,
        tap_dances: &'a Vec<protocol::TapDance>,
        vial_version: u32,
    ) -> Option<Self> {
        if let Some(kc) = current_keycode
            && let Some(idx) = keycodes::is_tapdance(kc)
                && tap_dances.iter().any(|t| t.index == idx) {
                    return Some(Self {
                        current_keycode: kc,
                        tap_dances,
                        vial_version,
                    });
                }
        None
    }
}

impl<'a> Widget for TapDanceInformer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let block = Block::default()
            .title("Tap Dance")
            .borders(Borders::ALL)
            .border_style(style);

        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(idx) = keycodes::is_tapdance(self.current_keycode) {
            if let Some(td) = self.tap_dances.iter().find(|t| t.index == idx) {
                let mut lines = Vec::new();
                lines.push(Line::from(vec![Span::styled(
                    format!("Index: {}", td.index),
                    Style::default().fg(Color::Yellow),
                )]));
                if td.tap != 0 {
                    lines.push(Line::from(vec![
                        Span::raw("Tap: "),
                        Span::styled(
                            keycodes::qid_to_name(td.tap, self.vial_version),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
                if td.hold != 0 {
                    lines.push(Line::from(vec![
                        Span::raw("Hold: "),
                        Span::styled(
                            keycodes::qid_to_name(td.hold, self.vial_version),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
                if td.double_tap != 0 {
                    lines.push(Line::from(vec![
                        Span::raw("Double tap: "),
                        Span::styled(
                            keycodes::qid_to_name(td.double_tap, self.vial_version),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
                if td.tap_hold != 0 {
                    lines.push(Line::from(vec![
                        Span::raw("Tap hold: "),
                        Span::styled(
                            keycodes::qid_to_name(td.tap_hold, self.vial_version),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
                lines.push(Line::from(vec![
                    Span::raw("Term: "),
                    Span::styled(
                        format!("{}ms", td.tapping_term),
                        Style::default().fg(Color::Cyan),
                    ),
                ]));

                Paragraph::new(lines).render(inner_area, buf);
            } else {
                Paragraph::new("Tap Dance not found").render(inner_area, buf);
            }
        } else {
            Paragraph::new("Not a Tap Dance key").render(inner_area, buf);
        }
    }
}