use crossterm::style::{Color, Stylize};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Position {
    pub sym: char,
    pub color: Option<(u8, u8, u8)>,
}

#[derive(Debug)]
pub struct Buffer {
    pub b: Vec<Vec<Position>>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn color_line(line: &str, color: Option<(u8, u8, u8)>) -> String {
            if let Some((r, g, b)) = color {
                let front_color =
                    if (r as f64 * 299.0 + g as f64 * 587.0 + b as f64 * 114.0) / 1000.0 > 128.0 {
                        Color::Black
                    } else {
                        Color::White
                    };
                let styled = line.to_owned().with(front_color).on(Color::Rgb { r, g, b });
                format!("{}", styled)
            } else {
                line.to_string()
            }
        }
        // always use colors when to_string is called, as we expect it to be used in TUI or ANSI aware context
        let colored = true;

        // cut top lines containing only spaces
        let mut spaces_only = true;

        // capacity is just a bit above typical value for colored kbd
        let mut last_color: Option<(u8, u8, u8)> = None;
        for line in self.b.iter() {
            if spaces_only {
                for p in line {
                    if p.sym != ' ' {
                        spaces_only = false;
                        break;
                    }
                }
            }
            if !spaces_only {
                let mut colored_substring = String::new();
                for p in line.iter() {
                    if colored && last_color != p.color {
                        let _ = write!(f, "{}", &color_line(&colored_substring, last_color));
                        colored_substring.truncate(0);
                        last_color = p.color;
                    }
                    colored_substring.push(p.sym);
                }
                let _ = writeln!(f, "{}", &color_line(&colored_substring, last_color));
                last_color = None;
            }
        }
        Ok(())
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            b: Vec::<Vec<Position>>::new(),
        }
    }

    pub fn put(&mut self, x: usize, y: usize, c: char, color: &Option<(u8, u8, u8)>) {
        while self.b.len() < y + 1 {
            let v = Vec::<Position>::new();
            self.b.push(v);
        }
        if self.b[y].len() < x + 1 {
            self.b[y].resize(
                x + 1,
                Position {
                    sym: ' ',
                    color: None,
                },
            );
        }
        self.b[y][x] = Position {
            sym: c,
            color: *color,
        };
    }

    pub fn dump(&self) {
        print!("{}", self);
    }
}
