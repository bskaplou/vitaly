#[derive(Debug)]
pub struct Buffer {
    b: Vec<Vec<char>>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            b: Vec::<Vec<char>>::new(),
        }
    }

    pub fn put(&mut self, x: usize, y: usize, c: char) {
        while self.b.len() < y + 1 {
            let v = Vec::<char>::new();
            self.b.push(v);
        }
        if self.b[y].len() < x + 1 {
            self.b[y].resize(x + 1, ' ');
        }
        self.b[y][x] = c;
    }

    pub fn dump(&self) {
        // cut top lines containing only spaces
        let mut spaces_only = true;
        for line in self.b.iter() {
            if spaces_only {
                for c in line {
                    if *c != ' ' {
                        spaces_only = false;
                        break;
                    }
                }
            }
            if !spaces_only {
                let s: String = line.iter().collect();
                println!("{}", s);
            }
        }
    }
}
