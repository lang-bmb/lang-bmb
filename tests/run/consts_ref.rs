// Rust translation of tests/run/consts.bmb (backlog 2V-19), for its
// expected output (`rustc --edition 2021`).
mod konst {
    pub const A: char = 'a';
    pub const BIG: u64 = u64::MAX / 2 + 1;
    pub const DOUBLE: usize = SIZE * 2;
    pub const MASK: u32 = (1 << 8) - 1;
    pub const NEG: i16 = -300;
    pub const ON: bool = SIZE > 3 && MASK == 255;
    pub const SIZE: usize = 4;

    pub fn main() {
        let z: [u8; SIZE] = zeros();
        println!("{}", z.len());
        println!("{}", masked(1000));
        println!("{}", NEG as i64);
        println!("{}", BIG);
        if ON {
            println!("{}", DOUBLE);
        }
        if A == 'a' {
            println!("a");
        }
        super::other::show();
    }

    fn masked(x: u32) -> u32 {
        x & MASK
    }

    fn zeros() -> [u8; SIZE] {
        [0; SIZE]
    }
}

mod other {
    const LIMIT: u64 = crate::konst::BIG - 1;

    pub fn show() {
        println!("{}", LIMIT);
        let b: [u8; crate::konst::SIZE] = [7; crate::konst::SIZE];
        println!("{}", b[crate::konst::SIZE - 1]);
    }
}

fn main() {
    konst::main();
}
