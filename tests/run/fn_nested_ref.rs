// Rust translation of tests/run/fn_nested.bmb (backlog 2V-20e), for its
// expected output (`rustc --edition 2021`).
mod nest {
    fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
        f(x)
    }

    fn twice() -> i64 {
        apply(
            |x: i64| -> i64 {
                let k: i64 = x;
                apply(move |z: i64| -> i64 { z + k }, 3)
            },
            5,
        )
    }

    pub fn main() {
        println!("{}", twice());
    }
}

fn main() {
    nest::main();
}
