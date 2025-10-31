#[derive(Debug)]
pub struct Buffer {
    b: Vec<Vec<char>>,
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
        for line in self.b.iter() {
            let s: String = line.into_iter().collect();
            println!("{}", s);
        }
    }
}
