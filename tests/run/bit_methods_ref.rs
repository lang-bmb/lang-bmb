// Rust translation of tests/run/bit_methods.bmb (backlog 2N-03), for its
// expected output.
fn fx(h: u64, word: u64) -> u64 {
    (h.rotate_left(5) ^ word).wrapping_mul(0x517cc1b727220a95)
}

fn fx_bytes(s: &[u8]) -> u64 {
    let mut h: u64 = 0;
    for &b in s {
        h = fx(h, b as u64);
    }
    h
}

fn members(set: u64) {
    let mut s = set;
    println!("{}", s.count_ones());
    while s != 0 {
        println!("{}", s.trailing_zeros());
        s &= s - 1;
    }
}

fn next_power(n: u32) -> u32 {
    if n == 1 {
        return 1;
    }
    let z = (n - 1).leading_zeros();
    1 << (32 - z)
}

fn xorshift(x: u64) -> u64 {
    let a = x ^ x.wrapping_shl(13);
    let b = a ^ a.wrapping_shr(7);
    b ^ b.wrapping_shl(17)
}

fn main() {
    println!("{}", fx_bytes(b"hello"));
    println!("{}", fx_bytes(b"The quick brown fox jumps over the lazy dog"));
    members(0x8000000000010025);
    members(0);
    for n in [1u32, 2, 3, 17, 1000, 1048576] {
        println!("{}", next_power(n));
        assert_eq!(next_power(n), n.next_power_of_two());
    }
    let w: u32 = 0x12345678;
    println!("{}", w.swap_bytes());
    let s: i16 = -2;
    println!("{}", s.swap_bytes());
    println!("{}", s.rotate_right(4));
    println!("{}", s.leading_zeros());
    println!("{}", s.count_zeros());
    let mut x: u64 = 88172645463325252;
    for _ in 0..4 {
        x = xorshift(x);
        println!("{}", x);
    }
    let b: u8 = 0x96;
    println!("{}", b.rotate_left(3));
    println!("{}", b.rotate_right(11));
    println!("{}", b.wrapping_shl(9));
}
