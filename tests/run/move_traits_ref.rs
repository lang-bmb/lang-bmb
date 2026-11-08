// Rust translation of tests/run/move_traits.bmb (backlog 2T-09), for its
// expected output (`rustc --edition 2021`).
trait Weigh: Sized {
    fn weight(&self) -> u64;

    fn spend(self) -> u64;

    fn spend_twice_weight(self) -> u64 {
        let w = self.weight();
        let s = self.spend();
        if w > 1000000 || s > 1000000 {
            return 0;
        }
        w + s
    }

    fn forget(self) -> u64 {
        7
    }
}

struct Bag {
    items: Vec<u64>,
}

struct Cell {
    b: Box<u64>,
}

impl Weigh for Bag {
    fn weight(&self) -> u64 {
        self.items.len() as u64
    }

    fn spend(self) -> u64 {
        let v = self.items;
        v.len() as u64
    }
}

impl Weigh for Cell {
    fn weight(&self) -> u64 {
        if *self.b > 1000 {
            return 1000;
        }
        *self.b
    }

    fn spend(self) -> u64 {
        100
    }

    fn forget(self) -> u64 {
        8
    }
}

fn total<T: Weigh>(a: &T, b: T) -> u64 {
    let x = a.weight();
    let y = b.spend();
    if x > 1000000 || y > 1000000 {
        return 0;
    }
    x + y
}

struct Pair<T: Weigh> {
    l: T,
    r: T,
}

fn heavier<T: Weigh>(p: Pair<T>) -> T {
    if p.l.weight() >= p.r.weight() {
        return p.l;
    }
    p.r
}

fn bag(n: u64) -> Bag {
    let mut v = Vec::new();
    for i in 0..n {
        if v.len() < 1000 {
            v.push(i);
        }
    }
    Bag { items: v }
}

fn main() {
    let a = bag(3);
    println!("{}", total(&a, bag(2)));
    println!("{}", a.spend_twice_weight());
    println!("{}", bag(5).forget());
    let c = Cell { b: Box::new(9) };
    println!("{}", total(&c, Cell { b: Box::new(1) }));
    println!("{}", c.forget());
    let h = heavier(Pair { l: bag(4), r: bag(6) });
    println!("{}", h.weight());
    println!("{}", Cell { b: Box::new(2) }.spend_twice_weight());
}
