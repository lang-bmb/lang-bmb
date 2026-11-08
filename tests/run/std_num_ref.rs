// Rust translation of tests/run/std_num.bmb (backlog 4B-02), for its
// expected output (`rustc --edition 2021`).
use std::fmt::Write as _;

fn show_u(r: Option<u64>) {
    match r {
        None => println!("none"),
        Some(v) => println!("{v}"),
    }
}

fn show_i(r: Option<i64>) {
    match r {
        None => println!("none"),
        Some(v) => println!("{v}"),
    }
}

fn main() {
    show_u("0".parse().ok());
    show_u("+42".parse().ok());
    show_u("18446744073709551615".parse().ok());
    show_u("18446744073709551616".parse().ok());
    show_u("".parse().ok());
    show_u("+".parse().ok());
    show_u("-1".parse().ok());
    show_u("12a".parse().ok());
    show_u("007".parse().ok());
    show_i("-9223372036854775808".parse().ok());
    show_i("9223372036854775807".parse().ok());
    show_i("9223372036854775808".parse().ok());
    show_i("-9223372036854775809".parse().ok());
    show_i("+5".parse().ok());
    show_i("-".parse().ok());
    show_u(u64::from_str_radix("ff", 16).ok());
    show_u(u64::from_str_radix("+DeadBeef", 16).ok());
    show_u(u64::from_str_radix("ffffffffffffffff", 16).ok());
    show_u(u64::from_str_radix("10000000000000000", 16).ok());
    show_u(u64::from_str_radix("0x10", 16).ok());
    let mut s = String::new();
    write!(s, "{}", 0u64).unwrap();
    s.push(' ');
    write!(s, "{}", u64::MAX).unwrap();
    s.push(' ');
    write!(s, "{}", i64::MIN).unwrap();
    s.push(' ');
    write!(s, "{}", 120i64).unwrap();
    s.push(' ');
    write!(s, "{:x}", 3735928559u64).unwrap();
    s.push(' ');
    write!(s, "{:x}", 0u64).unwrap();
    println!("{s}");
}
