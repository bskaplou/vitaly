use crate::{common, keymap, protocol};
use anyhow::{Result, anyhow};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use hidapi::{DeviceInfo, HidApi};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Color,
};
use std::io;

mod keymap_widget;
mod layer_keymap;
mod layer_selector;
mod sidebar;
use keymap_widget::KeymapWidget;
use sidebar::Sidebar;

const BORDER_COLOR_ACTIVE: Color = Color::Cyan;
const SELECTED_BGCOLOR_ACTIVE: Color = Color::Cyan;
const SELECTED_BGCOLOR_INACTIVE: Color = Color::DarkGray;
const SELECTED_COLOR_ACTIVE: Color = Color::Black;
const SELECTED_COLOR_INACTIVE: Color = Color::White;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveWidget {
    LayerSelector,
    Keymap,
    Keys,
    Combos,
    TapDance,
    KeyOverrides,
    Macros,
}

pub struct App {
    pub should_quit: bool,
    pub layer_count: u8,
    pub selected_layer: u8,
    pub buttons: Vec<keymap::Button>,
    pub keys: protocol::Keymap,
    pub combos: Vec<protocol::Combo>,
    pub tap_dances: Vec<protocol::TapDance>,
    pub macros: Vec<protocol::Macro>,
    pub key_overrides: Vec<protocol::KeyOverride>,
    pub vial_version: u32,
    pub active_widget: ActiveWidget,
    pub selected_button: usize,
}

impl App {
    pub fn new(
        layer_count: u8,
        buttons: Vec<keymap::Button>,
        keys: protocol::Keymap,
        combos: Vec<protocol::Combo>,
        tap_dances: Vec<protocol::TapDance>,
        macros: Vec<protocol::Macro>,
        key_overrides: Vec<protocol::KeyOverride>,
        vial_version: u32,
    ) -> Self {
        Self {
            should_quit: false,
            layer_count,
            selected_layer: 0,
            buttons,
            keys,
            combos,
            tap_dances,
            macros,
            key_overrides,
            vial_version,
            active_widget: ActiveWidget::LayerSelector,
            selected_button: 0,
        }
    }

    pub fn render(&self, f: &mut Frame) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(66), Constraint::Percentage(34)])
            .split(size);

        let keymap_widget = KeymapWidget {
            layer_count: self.layer_count,
            selected_layer: self.selected_layer,
            buttons: &self.buttons,
            keys: &self.keys,
            vial_version: self.vial_version,
            active_widget: self.active_widget,
            selected_button: self.selected_button,
        };
        f.render_widget(keymap_widget, chunks[0]);

        let sidebar = Sidebar {
            active_widget: self.active_widget,
            selected_layer: self.selected_layer,
            selected_button: self.buttons.get(self.selected_button),
            keys: &self.keys,
            combos: &self.combos,
            tap_dances: &self.tap_dances,
            macros: &self.macros,
            key_overrides: &self.key_overrides,
            vial_version: self.vial_version,
        };
        f.render_widget(sidebar, chunks[1]);
    }

    fn navigate_keymap(&mut self, dx: f64, dy: f64) {
        if self.buttons.is_empty() {
            return;
        }

        let current = &self.buttons[self.selected_button];
        let cx = current.x + current.w / 2.0;
        let cy = current.y + current.h / 2.0;

        let mut best_idx = self.selected_button;
        let mut min_dist = f64::MAX;

        // Weight perpendicular distance more heavily to prefer moving in the primary direction
        let weight = 4.0;

        for (i, btn) in self.buttons.iter().enumerate() {
            if i == self.selected_button {
                continue;
            }

            let bx = btn.x + btn.w / 2.0;
            let by = btn.y + btn.h / 2.0;

            let diff_x = bx - cx;
            let diff_y = by - cy;

            let is_candidate = if dx > 0.1 {
                diff_x > 0.1 // Right
            } else if dx < -0.1 {
                diff_x < -0.1 // Left
            } else if dy > 0.1 {
                diff_y > 0.1 // Down
            } else {
                diff_y < -0.1 // Up
            };

            if is_candidate {
                let dist = if dx.abs() > 0.1 {
                    // Moving horizontal: weight vertical distance
                    diff_x.abs() + diff_y.abs() * weight
                } else {
                    // Moving vertical: weight horizontal distance
                    diff_y.abs() + diff_x.abs() * weight
                };

                if dist < min_dist {
                    min_dist = dist;
                    best_idx = i;
                }
            }
        }
        self.selected_button = best_idx;
    }

    pub fn handle_event(&mut self) -> io::Result<bool> {
        let mut rerender = false;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                KeyCode::Tab => {
                    self.active_widget = match self.active_widget {
                        ActiveWidget::LayerSelector => ActiveWidget::Keymap,
                        _ => ActiveWidget::LayerSelector,
                    };
                    rerender = true;
                }
                KeyCode::Left => match self.active_widget {
                    ActiveWidget::LayerSelector => {
                        if self.selected_layer > 0 {
                            self.selected_layer -= 1;
                            rerender = true;
                        }
                    }
                    ActiveWidget::Keymap => {
                        self.navigate_keymap(-1.0, 0.0);
                        rerender = true;
                    }
                    _ => {}
                },
                KeyCode::Right => match self.active_widget {
                    ActiveWidget::LayerSelector => {
                        if self.selected_layer < self.layer_count.saturating_sub(1) {
                            self.selected_layer += 1;
                            rerender = true;
                        }
                    }
                    ActiveWidget::Keymap => {
                        self.navigate_keymap(1.0, 0.0);
                        rerender = true;
                    }
                    _ => {}
                },
                KeyCode::Up => match self.active_widget {
                    ActiveWidget::LayerSelector => {
                        if self.selected_layer < self.layer_count.saturating_sub(1) {
                            self.selected_layer += 1;
                            rerender = true;
                        }
                    }
                    ActiveWidget::Keymap => {
                        self.navigate_keymap(0.0, -1.0);
                        rerender = true;
                    }
                    _ => {}
                }
                KeyCode::Down => match self.active_widget {
                    ActiveWidget::LayerSelector => {
                        if self.selected_layer > 0 {
                            self.selected_layer -= 1;
                            rerender = true;
                        }
                    }
                    ActiveWidget::Keymap => {
                        self.navigate_keymap(0.0, 1.0);
                        rerender = true;
                    }
                    _ => {}
                }
                _ => {}
            }
        }
        Ok(rerender)
    }
}

pub fn run(api: &HidApi, device: &DeviceInfo) -> Result<()> {
    let dev = api.open_path(device.path())?;

    // Get capabilities
    let caps = protocol::scan_capabilities(&dev)?;

    // Load meta
    let meta = common::load_meta(&dev, &caps, &None)?;

    // Load layout options and buttons
    let layout_options_state = protocol::load_layout_options(&dev)?;
    let layout_options =
        protocol::LayoutOptions::from_json(layout_options_state, &meta["layouts"]["labels"])?;
    let buttons = keymap::keymap_to_buttons(&meta["layouts"]["keymap"], &layout_options)?;

    // Load keymap keys
    let cols = meta["matrix"]["cols"]
        .as_u64()
        .ok_or(anyhow!("matrix/cols not found in meta"))? as u8;
    let rows = meta["matrix"]["rows"]
        .as_u64()
        .ok_or(anyhow!("matrix/rows not found in meta"))? as u8;
    let keys = protocol::load_layers_keys(&dev, caps.layer_count, rows, cols)?;

    // Load combos
    let combos = protocol::load_combos(&dev, caps.combo_count)?;

    // Load tap dances
    let tap_dances = protocol::load_tap_dances(&dev, caps.tap_dance_count)?;

    // Load macros
    let macros = protocol::load_macros(&dev, caps.macro_count, caps.macro_buffer_size)?;

    // Load key overrides
    let key_overrides = protocol::load_key_overrides(&dev, caps.key_override_count)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(
        caps.layer_count,
        buttons,
        keys,
        combos,
        tap_dances,
        macros,
        key_overrides,
        caps.vial_version,
    );

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        Err(err.into())
    } else {
        Ok(())
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut rerender = true;
    while !app.should_quit {
        if rerender {
            terminal.draw(|f| {
                app.render(f);
            })?;
        }

        rerender = app.handle_event()?;
    }
    Ok(())
}
