use super::ActiveWidget;
use crate::{keycodes, keymap, protocol};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct Sidebar<'a> {
    pub active_widget: ActiveWidget,
    pub selected_layer: u8,
    pub selected_button: Option<&'a keymap::Button>,
    pub keys: &'a protocol::Keymap,
    pub combos: &'a Vec<protocol::Combo>,
    pub tap_dances: &'a Vec<protocol::TapDance>,
    pub macros: &'a Vec<protocol::Macro>,
    pub key_overrides: &'a Vec<protocol::KeyOverride>,
    pub vial_version: u32,
}

impl<'a> Widget for Sidebar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
                Constraint::Ratio(1, 5),
            ])
            .split(area);

        // Helper to render a stub and return its inner area
        let render_stub =
            |index: usize, title: &str, widget_enum: ActiveWidget, buf: &mut Buffer| -> Rect {
                let is_active = self.active_widget == widget_enum;
                let style = if is_active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                let block = Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(style);

                let inner = block.inner(chunks[index]);
                block.render(chunks[index], buf);
                inner
            };

        // Render Keys Widget Content
        let keys_area = render_stub(0, "Key", ActiveWidget::Keys, buf);

        let current_keycode = if let Some(button) = self.selected_button {
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

            Paragraph::new(lines).render(keys_area, buf);
            Some(keycode_hex)
        } else {
            Paragraph::new("No button selected").render(keys_area, buf);
            None
        };

        // Render Combos Widget Content
        let combos_area = render_stub(1, "Combos", ActiveWidget::Combos, buf);

        if let Some(keycode) = current_keycode {
            // Filter combos
            let matching_combos: Vec<&protocol::Combo> = self
                .combos
                .iter()
                .filter(|c| {
                    !c.is_empty()
                        && (c.key1 == keycode
                            || c.key2 == keycode
                            || c.key3 == keycode
                            || c.key4 == keycode)
                })
                .collect();

            if matching_combos.is_empty() {
                Paragraph::new("No matching combos").render(combos_area, buf);
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

                    if lines.len() as u16 >= combos_area.height {
                        break;
                    }
                }
                Paragraph::new(lines).render(combos_area, buf);
            }
        } else {
            Paragraph::new("").render(combos_area, buf);
        }

        // Render Tap Dance Widget Content
        let td_area = render_stub(2, "Tap Dance", ActiveWidget::TapDance, buf);

        if self.selected_button.is_some() {
            if let Some(keycode) = current_keycode
                && let Some(idx) = keycodes::is_tapdance(keycode)
            {
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

                    Paragraph::new(lines).render(td_area, buf);
                } else {
                    Paragraph::new("Tap Dance not found").render(td_area, buf);
                }
            } else {
                Paragraph::new("Not a Tap Dance key").render(td_area, buf);
            }
        } else {
            Paragraph::new("").render(td_area, buf);
        }

        // Render Key Overrides Widget Content
        let ko_area = render_stub(3, "Key Override", ActiveWidget::KeyOverrides, buf);

        if let Some(keycode) = current_keycode {
            let matching_kos: Vec<&protocol::KeyOverride> = self
                .key_overrides
                .iter()
                .filter(|ko| {
                    !ko.is_empty()
                        && ko.trigger == keycode
                        && ((ko.layers & 1 << self.selected_layer) != 0)
                })
                .collect();

            if matching_kos.is_empty() {
                Paragraph::new("No matching key overrides").render(ko_area, buf);
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

                    if lines.len() as u16 >= ko_area.height {
                        break;
                    }
                }
                Paragraph::new(lines).render(ko_area, buf);
            }
        } else {
            Paragraph::new("").render(ko_area, buf);
        }

        // Render Macros Widget Content
        let macros_area = render_stub(4, "Macros", ActiveWidget::Macros, buf);

        if self.selected_button.is_some() {
            if let Some(keycode) = current_keycode
                && let Some(idx) = keycodes::is_macro(keycode, self.vial_version)
            {
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
                        if lines.len() as u16 >= macros_area.height {
                            break;
                        }
                    }
                    Paragraph::new(lines).render(macros_area, buf);
                } else {
                    Paragraph::new("Macro not found").render(macros_area, buf);
                }
            } else {
                Paragraph::new("Not a Macro key").render(macros_area, buf);
            }
        } else {
            Paragraph::new("").render(macros_area, buf);
        }
    }
}
