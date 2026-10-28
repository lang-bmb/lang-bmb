// Reference for tests/lib_run/utf8.bmb (backlog T4-02): the same inputs judged by
// Rust's `std::str::from_utf8`. `rustc -O utf8_ref.rs && ./utf8_ref > utf8.out`.
fn up_to(s: &[u8]) -> usize {
    match std::str::from_utf8(s) {
        Ok(_) => s.len(),
        Err(e) => e.valid_up_to(),
    }
}

fn add(t: &mut (u64, u64), s: &[u8]) {
    let r = up_to(s);
    t.0 += (r == s.len()) as u64;
    t.1 += r as u64;
}

fn main() {
    let mut t = (0, 0);
    for b0 in 0..=255u8 {
        add(&mut t, &[b0]);
    }
    println!("1 {} {}", t.0, t.1);
    let mut t = (0, 0);
    for b0 in 0..=255u8 {
        for b1 in 0..=255u8 {
            add(&mut t, &[b0, b1]);
        }
    }
    println!("2 {} {}", t.0, t.1);
    let mut t = (0, 0);
    for b0 in 0..=255u8 {
        for b1 in 0..=255u8 {
            for b2 in 0..=255u8 {
                add(&mut t, &[b0, b1, b2]);
            }
        }
    }
    println!("3 {} {}", t.0, t.1);
    let edge = [0u8, 127, 128, 143, 144, 159, 160, 191, 192, 255];
    let mut t = (0, 0);
    for b0 in 0..=255u8 {
        for &b1 in &edge {
            for &b2 in &edge {
                for &b3 in &edge {
                    add(&mut t, &[b0, b1, b2, b3]);
                }
            }
        }
    }
    println!("4 {} {}", t.0, t.1);
}
