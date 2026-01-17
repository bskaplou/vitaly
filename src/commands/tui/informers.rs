use super::{
    combos_informer::CombosInformer, encoder_informer::EncoderInformer, key_informer::KeyInformer,
    key_overrides_informer::KeyOverridesInformer, macro_informer::MacroInformer,
    tap_dance_informer::TapDanceInformer,
};
use crate::{keymap, protocol};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

pub struct Informers<'a> {
    pub selected_layer: u8,
    pub selected_button: Option<&'a keymap::Button>,
    pub keys: &'a protocol::Keymap,
    pub encoders: &'a Vec<Vec<protocol::Encoder>>,
    pub combos: &'a Vec<protocol::Combo>,
    pub tap_dances: &'a Vec<protocol::TapDance>,
    pub macros: &'a Vec<protocol::Macro>,
    pub key_overrides: &'a Vec<protocol::KeyOverride>,
    pub vial_version: u32,
}

impl<'a> Widget for Informers<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Pre-calculate current keycode for dependent widgets
        let current_keycode = if let Some(button) = self.selected_button {
            if button.encoder {
                let idx = button.wire_x as usize;
                let is_cw = button.wire_y == 1;
                if let Some(layer_encoders) = self.encoders.get(self.selected_layer as usize) {
                    layer_encoders
                        .get(idx)
                        .map(|enc| if is_cw { enc.cw } else { enc.ccw })
                } else {
                    None
                }
            } else {
                Some(
                    self.keys
                        .get(self.selected_layer, button.wire_x, button.wire_y),
                )
            }
        } else {
            None
        };

        // Instantiate applicable informers
        let key_informer = KeyInformer::new_if_applicable(
            self.selected_layer,
            self.selected_button,
            self.keys,
            self.vial_version,
        );
        let encoder_informer = EncoderInformer::new_if_applicable(
            self.selected_layer,
            self.selected_button,
            self.encoders,
            self.vial_version,
        );
        let combos_informer =
            CombosInformer::new_if_applicable(current_keycode, self.combos, self.vial_version);
        let tap_dance_informer = TapDanceInformer::new_if_applicable(
            current_keycode,
            self.tap_dances,
            self.vial_version,
        );
        let key_overrides_informer = KeyOverridesInformer::new_if_applicable(
            current_keycode,
            self.key_overrides,
            self.selected_layer,
            self.vial_version,
        );
        let macro_informer =
            MacroInformer::new_if_applicable(current_keycode, self.macros, self.vial_version);

        // Count visible widgets
        let mut visible_count = 0;
        if key_informer.is_some() {
            visible_count += 1;
        }
        if encoder_informer.is_some() {
            visible_count += 1;
        }
        if combos_informer.is_some() {
            visible_count += 1;
        }
        if tap_dance_informer.is_some() {
            visible_count += 1;
        }
        if key_overrides_informer.is_some() {
            visible_count += 1;
        }
        if macro_informer.is_some() {
            visible_count += 1;
        }

        if visible_count == 0 {
            return;
        }

        let constraints =
            std::iter::repeat_n(Constraint::Ratio(1, visible_count), visible_count as usize)
                .collect::<Vec<_>>();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut chunk_idx = 0;

        if let Some(w) = key_informer {
            w.render(chunks[chunk_idx], buf);
            chunk_idx += 1;
        }
        if let Some(w) = encoder_informer {
            w.render(chunks[chunk_idx], buf);
            chunk_idx += 1;
        }
        if let Some(w) = combos_informer {
            w.render(chunks[chunk_idx], buf);
            chunk_idx += 1;
        }
        if let Some(w) = tap_dance_informer {
            w.render(chunks[chunk_idx], buf);
            chunk_idx += 1;
        }
        if let Some(w) = key_overrides_informer {
            w.render(chunks[chunk_idx], buf);
            chunk_idx += 1;
        }
        if let Some(w) = macro_informer {
            w.render(chunks[chunk_idx], buf);
        }
    }
}
