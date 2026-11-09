// Rust translation of tests/run/impl_blocks.bmb (backlog 2T-11), for its
// expected output (`rustc --edition 2021`).
struct Cell<T> {
    v: Vec<T>,
    #[allow(dead_code)]
    n: u64,
}

impl<T> Cell<T> {
    fn new(n: u64) -> Cell<T> {
        Cell { v: Vec::new(), n }
    }

    fn size(&self) -> usize {
        self.v.len()
    }
}

impl<T: Copy> Cell<T> {
    fn first(&self) -> Option<T> {
        if !self.v.is_empty() { Some(self.v[0]) } else { None }
    }
}

fn main() {
    let c: Cell<u64> = Cell::new(3);
    println!("{}", c.size());
    let s: Cell<Box<u64>> = Cell::new(4);
    println!("{}", s.size());
    match c.first() {
        None => println!("9"),
        Some(x) => println!("{x}"),
    }
}
