// Rust translation of tests/run/consts_data.bmb (backlog 2V-19b), for its
// expected output (`rustc --edition 2021`). The types derive `Copy` so a
// const array's element can be passed by value, as BMB's value types are.
mod kd {
    #[derive(Clone, Copy)]
    pub struct Point {
        pub x: i64,
        pub y: i64,
    }

    #[derive(Clone, Copy)]
    pub enum Mode {
        Fast,
        Slow(u8),
    }

    pub const PRIMES: [u64; 5] = [2, 3, 5, 7, 11];
    pub const ORIGIN: Point = Point { x: -3, y: 4 };
    pub const MODES: [Mode; 2] = [Mode::Fast, Mode::Slow(9)];
    pub const THIRD: u64 = PRIMES[2];
    pub const SLOW: u8 = match MODES[1] {
        Mode::Fast => 0,
        Mode::Slow(k) => k,
    };
    pub const MAYBE: Option<u16> = Some(400);
    pub const COUNT: usize = 3;
    pub const ZEROS: [u8; COUNT] = [0; COUNT];
    pub const GRID: [[u8; 2]; 2] = [[1, 2], [3, 4]];
    pub const LEFT: i64 = ORIGIN.x - 1;

    pub fn main() {
        println!("{}", PRIMES[4]);
        println!("{}", THIRD);
        println!("{}", ORIGIN.x);
        println!("{}", ORIGIN.y);
        println!("{}", LEFT);
        println!("{}", SLOW as u64);
        println!("{}", maybe());
        println!("{}", GRID[1][0] as u64);
        println!("{}", ZEROS.len() as u64);
        println!("{}", lookup(3));
        println!("{}", mode_cost(MODES[0]));
        println!("{}", mode_cost(MODES[1]));
        println!("{}", far(ORIGIN));
    }

    fn maybe() -> u64 {
        match MAYBE {
            None => 0,
            Some(v) => v as u64,
        }
    }

    fn lookup(i: usize) -> u64 {
        PRIMES[i]
    }

    fn mode_cost(m: Mode) -> u64 {
        match m {
            Mode::Fast => 1,
            Mode::Slow(k) => k as u64,
        }
    }

    fn far(p: Point) -> i64 {
        p.y - p.x
    }
}

fn main() {
    kd::main();
}
