// spectral-norm reference (backlog T3-04): the Benchmarks Game algorithm,
// written from the published description (the official Rust submissions are
// not reachable from this environment, backlog T3-01). Single-threaded:
// A(i, j) = 1 / ((i + j)(i + j + 1) / 2 + i + 1), ten rounds of
// v = AtA u, u = AtA v, then sqrt(u.v / v.v).
//
//   rustc -O spectralnorm.rs && ./spectralnorm 100
//
// Standard check output for n = 100: 1.274219991.

fn a(i: usize, j: usize) -> f64 {
    let k = i + j;
    1.0 / ((k * (k + 1)) / 2 + i + 1) as f64
}

fn mul(v: &[f64], out: &mut [f64], transpose: bool) {
    let n = v.len();
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            let aij = if transpose { a(j, i) } else { a(i, j) };
            sum += aij * v[j];
        }
        out[i] = sum;
    }
}

fn mul_atav(v: &[f64], out: &mut [f64], tmp: &mut [f64]) {
    mul(v, tmp, false);
    mul(tmp, out, true);
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(100);
    let mut u = vec![1.0; n];
    let mut v = vec![0.0; n];
    let mut tmp = vec![0.0; n];
    for _ in 0..10 {
        mul_atav(&u, &mut v, &mut tmp);
        mul_atav(&v, &mut u, &mut tmp);
    }
    let mut vbv = 0.0;
    let mut vv = 0.0;
    for i in 0..n {
        vbv += u[i] * v[i];
        vv += v[i] * v[i];
    }
    println!("{:.9}", (vbv / vv).sqrt());
}
