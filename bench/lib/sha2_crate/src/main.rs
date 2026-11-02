// Twin of ../sha256_speed.bmb with the `sha2` crate (backlog T4-01): the same
// data and the same folded output.
use sha2::{Digest, Sha256};

fn main() {
    let n = 16777216;
    let mut v = Vec::with_capacity(n);
    let mut x: u64 = 12345;
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        v.push((x >> 56) as u8);
    }
    let mut acc: u64 = 0;
    for k in 0..8u8 {
        v[0] = k;
        let d = Sha256::digest(std::hint::black_box(&v));
        for b in d {
            acc = acc.wrapping_mul(31).wrapping_add(b as u64);
        }
    }
    println!("{acc}");
}
