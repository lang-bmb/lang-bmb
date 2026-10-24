//! Deterministic mutation fuzzing of the front end (backlog 1A-09): mutated
//! corpus programs must never make the lexer, parser or checker panic.
//! `BMB_FUZZ_ITERS` raises the iteration count for longer runs and
//! `BMB_FUZZ_SEED` picks another mutant sequence.

use std::fs;
use std::panic;
use std::path::Path;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

const TOKENS: &[&str] = &[
    "fn",
    "spec",
    "mod",
    "let",
    "mut",
    "while",
    "if",
    "else",
    "requires",
    "ensures",
    "decreases",
    "invariant",
    "forall",
    "old",
    "assert",
    "return",
    "for",
    "match",
    "as",
    "#[trigger]",
    "#[uses(",
    "#[effects(pure)]",
    "#[summary(\"s\")]",
    "{",
    "}",
    "(",
    ")",
    "[",
    "]",
    ";",
    ",",
    ":",
    "::",
    "->",
    "==>",
    "&",
    "&mut",
    "|",
    "!",
    "?",
    "=",
    "==",
    "<",
    "<=",
    "+",
    "-",
    "*",
    "/",
    "%",
    "&&",
    "||",
    ".",
    "0",
    "1",
    "18446744073709551615",
    "99999999999999999999999999999999999999999",
    "x",
    "a",
    "i64",
    "u64",
    "usize",
    "bool",
    "()",
    "[i64; 3]",
    "&[i64]",
    "a.len()",
    "\"",
    "'a",
    "/*",
    "vec![1]",
    "\u{00e9}",
    "\u{d55c}",
];

fn mutate(rng: &mut Rng, src: &str, donors: &[String]) -> String {
    let mut b: Vec<u8> = src.as_bytes().to_vec();
    for _ in 0..1 + rng.below(4) {
        let pos = rng.below(b.len() + 1);
        match rng.below(5) {
            0 => {
                let end = (pos + rng.below(40)).min(b.len());
                b.drain(pos..end);
            }
            1 => {
                let t = TOKENS[rng.below(TOKENS.len())];
                b.splice(pos..pos, format!(" {t} ").bytes());
            }
            2 => {
                let end = (pos + rng.below(60)).min(b.len());
                let chunk: Vec<u8> = b[pos..end].to_vec();
                let at = rng.below(b.len() + 1);
                b.splice(at..at, chunk);
            }
            3 => {
                let d = donors[rng.below(donors.len())].as_bytes();
                let s = rng.below(d.len());
                let e = (s + rng.below(80)).min(d.len());
                b.splice(pos..pos, d[s..e].iter().copied());
            }
            _ => {
                const BYTES: &[u8] = b"(){}[];,:<>=!&|*+-/%#'\"\n x0";
                if pos < b.len() {
                    b[pos] = BYTES[rng.below(BYTES.len())];
                }
            }
        }
    }
    // Byte edits may split a UTF-8 character; the lexer only sees valid text.
    String::from_utf8_lossy(&b).into_owned()
}

#[test]
fn front_end_never_panics_on_mutated_programs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut corpus = Vec::new();
    for dir in ["check/ok", "check/err", "verify/pass", "verify/fail", "run"] {
        let mut paths: Vec<_> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        paths.sort();
        for p in paths
            .into_iter()
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        {
            corpus.push(fs::read_to_string(p).unwrap());
        }
    }
    let iters: usize = std::env::var("BMB_FUZZ_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1500);
    // `BMB_FUZZ_SEED` explores other mutants (the nightly run uses the run number).
    let seed: u64 = std::env::var("BMB_FUZZ_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut rng = Rng(0x9E3779B97F4A7C15 ^ seed.wrapping_mul(0xD1B54A32D192ED03) | 1);
    let mut failures = Vec::new();
    let mut clean = 0usize;
    for i in 0..iters {
        let pick = rng.below(corpus.len());
        let src = mutate(&mut rng, &corpus[pick], &corpus);
        let input = src.clone();
        // Silence the panic message only around the compiler call.
        let prev = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind(move || bmb::check_sources(&[("f.bmb".into(), input)]));
        panic::set_hook(prev);
        // Mutants that type-check also go through proof and code generation,
        // whose internal invariants assume a checked program.
        let result = match result {
            Ok(out) if out.ok => {
                clean += 1;
                let input = src.clone();
                let prev = panic::take_hook();
                panic::set_hook(Box::new(|_| {}));
                let r = panic::catch_unwind(move || {
                    bmb::emit_llvm_sources(&[("f.bmb".into(), input)]);
                });
                panic::set_hook(prev);
                r.map(|_| ())
            }
            other => other.map(|_| ()),
        };
        if result.is_err() {
            failures.push((i, src));
            if failures.len() >= 3 {
                break;
            }
        }
    }
    eprintln!("fuzz: {iters} mutants, {clean} type-check cleanly and were also proved and lowered");
    if let Some((i, src)) = failures.first() {
        let path = std::env::temp_dir().join("bmb-fuzz-crash.bmb");
        fs::write(&path, src).unwrap();
        panic!(
            "{} panic(s); first at iteration {i}, input saved to {}",
            failures.len(),
            path.display()
        );
    }
}
