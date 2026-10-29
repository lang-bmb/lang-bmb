// Rust translation of tests/run/mandel.bmb (backlog T2-03), for its expected
// output: `rustc -O mandel_ref.rs && ./mandel_ref > mandel.out`.
use std::io::Write;

fn escape(cr: f64, ci: f64, limit: u32) -> u32 {
    let mut zr = 0.0f64;
    let mut zi = 0.0f64;
    let mut n = 0;
    while n < limit && zr * zr + zi * zi <= 4.0 {
        let t = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = t;
        n += 1;
    }
    n
}

fn shade(n: u32) -> u8 {
    if n >= 64 {
        b'#'
    } else if n >= 16 {
        b'+'
    } else if n >= 6 {
        b'.'
    } else {
        b' '
    }
}

fn main() {
    let (w, h) = (48u32, 20u32);
    let mut out = std::io::stdout();
    for y in 0..h {
        let mut line: Vec<u8> = Vec::new();
        for x in 0..w {
            let cr = -2.2 + 3.0 * f64::from(x) / f64::from(w);
            let ci = -1.2 + 2.4 * f64::from(y) / f64::from(h);
            line.push(shade(escape(cr, ci, 64)));
        }
        line.push(b'\n');
        out.write_all(&line).unwrap();
    }
}
