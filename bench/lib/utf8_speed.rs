// Twin of utf8_speed.bmb (backlog T4-02): the same text, checked 20 times by
// Rust's `std::str::from_utf8` (valid_up_to on error). `rustc -C opt-level=2`.
fn text(n: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(n + 4);
    let mut x: u64 = 12345;
    while v.len() < n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let c = (x / 4294967296) % 8;
        if c < 5 {
            v.push(97);
        } else if c == 5 {
            v.extend_from_slice(&[195, 169]);
        } else if c == 6 {
            v.extend_from_slice(&[226, 130, 172]);
        } else {
            v.extend_from_slice(&[240, 159, 152, 128]);
        }
    }
    v
}

fn main() {
    let v = text(16777216);
    let mut sum: u64 = 0;
    for _ in 0..20 {
        let r = match std::str::from_utf8(std::hint::black_box(&v)) {
            Ok(_) => v.len(),
            Err(e) => e.valid_up_to(),
        };
        sum = sum.wrapping_add(r as u64);
    }
    println!("{}", sum % 1000000007);
}
