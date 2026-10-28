//! Normal form and definition hash fuzzing (backlog 1D-22, ADR-0019).
//!
//! Programs come from the soundness generator (`common::scalar`: random i64
//! functions with contracts, spread over several units that call each other)
//! and from the corpus. For each one, layout variants are made: definitions
//! and units reordered, units moved across files, comments, blank lines,
//! indentation and line breaks between tokens added. Claims:
//!
//! - `fmt` is idempotent, and every variant has the same normal form;
//! - definition hashes (id, body, iface) are the same for every variant;
//! - renaming any one name (definition, unit, type, field, variant,
//!   parameter, local, label) leaves every identity, and an edit of a literal
//!   changes the edited definition's (ADR-0059 §1, T1-ID-01);
//! - the normal form proves with the same status lines and lowers to the same
//!   IR as the original text.
//!
//! `BMB_FMT_FUZZ_ITERS` raises the number of generated programs (the first
//! `BMB_FMT_FUZZ_PROVE` of them, default 2, are also proved and lowered) and
//! `BMB_FMT_FUZZ_SEED` picks another sequence.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

mod common;

use common::scalar::*;

type Files = Vec<(String, String)>;

fn env(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn shuffle<T>(rng: &mut Rng, v: &mut [T]) {
    for i in (1..v.len()).rev() {
        v.swap(i, rng.below(i + 1));
    }
}

/// A generated unit: its name and top-level items (each a complete
/// definition with its attributes, indented for the unit body).
struct Unit {
    name: String,
    items: Vec<String>,
}

/// Units of 1..=3 generated functions each; some functions call a function
/// of their own unit by its short name and some call another unit's function
/// by full path, so names resolve differently per unit (hash qualification).
fn gen_program(rng: &mut Rng) -> Vec<Unit> {
    let n = 1 + rng.below(3);
    let mut units = Vec::new();
    for u in 0..n {
        let mut items = Vec::new();
        let count = 1 + rng.below(3);
        for k in 0..count {
            let f = gen_func(rng, k);
            items.extend(
                f.text()
                    .split("\n\n")
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| format!("{}\n", s.trim_end())),
            );
        }
        for c in 0..rng.below(3) {
            let target = rng.below(n);
            let callee = rng.below(if target == u { count } else { 1 });
            let path = if target == u {
                format!("f{callee}")
            } else {
                format!("u{target}::f0")
            };
            let mut t = String::new();
            let _ = writeln!(
                t,
                "    #[effects(pure)]\n    fn c{c}(x: i64, y: i64) -> i64 {{\n        {path}(x, y)\n    }}"
            );
            items.push(t);
        }
        // A lemma and a function that calls it in a `proof` block
        // (ADR-0062): ghost code in the normal form and the hashes.
        if rng.chance(40) {
            items.push(
                "    spec fn q(x: i64, y: i64) -> bool {
        x - y <= x + y || y < 0
    }
"
                .into(),
            );
            items.push(
                "    #[uses(q)]
    proof fn l(x: i64, y: i64)
        ensures q(x, y),
    {
        if y < 0 {
            assert(y < 0);
        }
    }
"
                .into(),
            );
            items.push(
                "    #[effects(pure)]
    fn p(x: i64, y: i64) -> i64 {
        proof {
            l(x, y);
        }
        x
    }
"
                .into(),
            );
        }
        units.push(Unit {
            name: format!("u{u}"),
            items,
        });
    }
    units
}

fn unit_text(u: &Unit, summary: &str) -> String {
    format!(
        "#[summary(\"{summary}\")]\nmod {} {{\n{}}}\n",
        u.name,
        u.items.join("\n")
    )
}

fn one(text: String) -> Files {
    vec![("p.bmb".to_string(), text)]
}

fn normal(files: &Files) -> BTreeMap<String, String> {
    match bmb::format_sources(files) {
        Ok(units) => units,
        Err(d) => panic!("program must check:\n{d}\n{files:?}"),
    }
}

/// The normal form as files, one per unit.
fn normal_files(files: &Files) -> Files {
    normal(files)
        .into_iter()
        .map(|(u, t)| (format!("{u}.bmb"), t))
        .collect()
}

fn hashes(files: &Files) -> Vec<String> {
    let mut v: Vec<String> = bmb::hash_sources(files)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    v.sort();
    v
}

/// Adds comments, blank lines, indentation and line breaks between tokens.
/// Lines holding a string literal (`#[summary("..")]`) only get comments and
/// blank lines around them, so the literal is untouched.
fn relayout(rng: &mut Rng, text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if rng.chance(15) {
            out.push('\n');
        }
        if rng.chance(10) {
            out.push_str("// note\n");
        }
        let body = if line.contains('"') {
            line.to_string()
        } else {
            let mut b = String::new();
            if rng.chance(30) {
                b.push_str("  ");
            }
            for (i, word) in line.split(' ').enumerate() {
                if i > 0 {
                    b.push_str(match rng.below(10) {
                        0 => "\n",
                        1 => "   ",
                        2 => " // c\n ",
                        _ => " ",
                    });
                }
                b.push_str(word);
            }
            b
        };
        out.push_str(&body);
        if rng.chance(15) {
            out.push_str(" // trailing");
        }
        out.push('\n');
    }
    out
}

/// Spreads whole-unit texts over 1..=3 files in random order.
fn spread(rng: &mut Rng, mut texts: Vec<String>) -> Files {
    shuffle(rng, &mut texts);
    let n = 1 + rng.below(3);
    let mut files: Vec<String> = vec![String::new(); n];
    for t in texts {
        let k = rng.below(n);
        files[k].push_str(&t);
        files[k].push('\n');
    }
    files
        .into_iter()
        .enumerate()
        .map(|(i, t)| (format!("f{i}.bmb"), t))
        .collect()
}

/// Same program, different layout: the normal form and the hashes must not move.
fn check_variants(rng: &mut Rng, what: &str, base: &Files, units: Option<&[Unit]>) {
    let want = normal(base);
    let want_hashes = hashes(base);
    // Idempotence: the normal form of the normal form is itself.
    let nf = normal_files(base);
    assert_eq!(normal(&nf), want, "{what}: fmt is not idempotent");
    for v in 0..4 {
        let unit_texts: Vec<String> = match units {
            // Generated programs: definitions reordered inside every unit too.
            Some(units) => units
                .iter()
                .map(|u| {
                    let mut items = u.items.clone();
                    shuffle(rng, &mut items);
                    unit_text(
                        &Unit {
                            name: u.name.clone(),
                            items,
                        },
                        "s",
                    )
                })
                .collect(),
            None => nf.iter().map(|(_, t)| t.clone()).collect(),
        };
        let mut files = spread(rng, unit_texts);
        if v > 0 {
            for f in &mut files {
                f.1 = relayout(rng, &f.1);
            }
        }
        let ctx = || format!("{what}, variant {v}:\n{files:#?}");
        assert_eq!(normal(&files), want, "normal form moved: {}", ctx());
        assert_eq!(hashes(&files), want_hashes, "hashes moved: {}", ctx());
    }
}

/// The identifier tokens of `text` outside string literals, and `text` with
/// every token `from` written `to` (a consistent rename of one name: a
/// definition, unit, type, field, variant, parameter, local or label).
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    rename_word(text, "", "", &mut out);
    out
}

fn rename_word(text: &str, from: &str, to: &str, seen: &mut Vec<String>) -> String {
    let mut out = String::with_capacity(text.len());
    let cs: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '"' {
            // A string literal stays as it is.
            out.push(c);
            i += 1;
            while i < cs.len() && cs[i] != '"' {
                if cs[i] == '\\' && i + 1 < cs.len() {
                    out.push(cs[i]);
                    i += 1;
                }
                out.push(cs[i]);
                i += 1;
            }
            if i < cs.len() {
                out.push(cs[i]);
                i += 1;
            }
        } else if c == '/' && cs.get(i + 1) == Some(&'/') {
            while i < cs.len() && cs[i] != '\n' {
                out.push(cs[i]);
                i += 1;
            }
        } else if c.is_alphanumeric() || c == '_' {
            let start = i;
            while i < cs.len() && (cs[i].is_alphanumeric() || cs[i] == '_') {
                i += 1;
            }
            let w: String = cs[start..i].iter().collect();
            // `_` is no name (a wildcard).
            if !w.starts_with(|c: char| c.is_ascii_digit()) && w != "_" {
                seen.push(w.clone());
            }
            out.push_str(if w == from { to } else { &w });
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// `bmb hash` rows by path, the path renamed with `rn`.
fn ids(files: &Files, rn: impl Fn(&str) -> String) -> BTreeMap<String, String> {
    hashes(files)
        .into_iter()
        .map(|l| {
            let path = l.split('"').nth(3).unwrap().to_string();
            let rest = l[l.find(",\"id\"").unwrap()..].to_string();
            (rn(&path), rest)
        })
        .collect()
}

/// Renaming any one name (ADR-0059 §1) leaves every identity, body hash and
/// interface hash as it was; the renamed definition keeps its own under its
/// new path. Returns how many renames were checked.
fn check_renames(rng: &mut Rng, what: &str, base: &Files, max: usize) -> usize {
    let want = ids(base, str::to_string);
    let mut words: Vec<String> = base.iter().flat_map(|(_, t)| tokens(t)).collect();
    words.sort();
    words.dedup();
    shuffle(rng, &mut words);
    let mut checked = 0;
    for (k, w) in words.iter().enumerate() {
        if checked == max {
            break;
        }
        let to = format!("{w}_rn{k}");
        let files: Files = base
            .iter()
            .map(|(n, t)| (n.clone(), rename_word(t, w, &to, &mut Vec::new())))
            .collect();
        // Built-in names and keywords do not rename (the program stops checking).
        if !bmb::check_sources(&files).ok {
            continue;
        }
        let got = ids(&files, str::to_string);
        let mapped = ids(base, |p| rename_word(p, w, &to, &mut Vec::new()));
        assert_eq!(got, mapped, "{what}: renaming `{w}` moved an identity");
        assert_eq!(got.len(), want.len(), "{what}");
        checked += 1;
    }
    checked
}

/// A meaning-changing edit (an integer literal inside one top-level
/// function of the normal form) changes that function's identity.
fn check_edit(rng: &mut Rng, what: &str, base: &Files) -> bool {
    let nf = normal_files(base);
    let want = ids(&nf, str::to_string);
    // (file, unit, function name, line range of its block)
    let mut blocks = Vec::new();
    for (fi, (name, text)) in nf.iter().enumerate() {
        let unit = name.trim_end_matches(".bmb");
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let l = lines[i];
            let sig = l
                .strip_prefix("    fn ")
                .or_else(|| l.strip_prefix("    spec fn "));
            if let Some(rest) = sig {
                let fname: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let end = (i..lines.len()).find(|&j| lines[j] == "    }").unwrap_or(i);
                blocks.push((fi, format!("{unit}::{fname}"), i, end));
                i = end;
            }
            i += 1;
        }
    }
    shuffle(rng, &mut blocks);
    for (fi, path, lo, hi) in blocks {
        let lines: Vec<String> = nf[fi].1.lines().map(String::from).collect();
        for li in lo..=hi {
            let line = &lines[li];
            let Some((at, n)) = line
                .match_indices(|c: char| c.is_ascii_digit())
                .map(|(a, _)| a)
                .find_map(|a| {
                    let before = line[..a].chars().last();
                    if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                        return None;
                    }
                    let d: String = line[a..]
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    let after = line[a + d.len()..].chars().next();
                    if after.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                        return None;
                    }
                    d.parse::<u64>().ok().filter(|v| *v < 100).map(|v| (a, v))
                })
            else {
                continue;
            };
            let digits = n.to_string().len();
            let mut changed = lines.clone();
            changed[li] = format!("{}{}{}", &line[..at], n + 1, &line[at + digits..]);
            let mut files = nf.clone();
            files[fi].1 = changed.join("\n") + "\n";
            if !bmb::check_sources(&files).ok {
                continue;
            }
            let got = ids(&files, str::to_string);
            assert_ne!(
                got.get(&path),
                want.get(&path),
                "{what}: editing a literal of {path} kept its identity"
            );
            return true;
        }
    }
    false
}

fn statuses(out: &str) -> Vec<String> {
    out.lines()
        .filter(|l| l.starts_with("{\"def\""))
        .map(String::from)
        .collect()
}

/// The normal form proves and lowers exactly like the original text.
fn check_proof_and_ir(what: &str, base: &Files) {
    let nf = normal_files(base);
    // Two independent proofs of the same program: run them side by side.
    let (a, b) = std::thread::scope(|s| {
        let a = s.spawn(|| bmb::with_stack(|| bmb::emit_llvm_sources(base)));
        let b = bmb::with_stack(|| bmb::emit_llvm_sources(&nf));
        (a.join().unwrap(), b)
    });
    assert_eq!(
        statuses(&a.verify.output),
        statuses(&b.verify.output),
        "{what}: status lines differ after fmt\n{base:?}"
    );
    assert!(a.ir.is_some(), "{what}: no IR\n{}", a.verify.output);
    assert_eq!(a.ir, b.ir, "{what}: IR differs after fmt");
    assert_eq!(a.checks, b.checks, "{what}: kept checks differ after fmt");
}

#[test]
fn generated_programs_keep_normal_form_hashes_proofs_and_ir() {
    let iters = env("BMB_FMT_FUZZ_ITERS", 24);
    let prove = env("BMB_FMT_FUZZ_PROVE", 2);
    let seed = env("BMB_FMT_FUZZ_SEED", 0);
    for i in 0..iters {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(i);
        let mut rng = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let units = gen_program(&mut rng);
        let base = one(units.iter().map(|u| unit_text(u, "s")).collect());
        let what = format!("seed {seed} program {i}");
        check_variants(&mut rng, &what, &base, Some(&units));
        if i < prove {
            check_proof_and_ir(&what, &base);
        }
    }
}

/// Identities have no names (ADR-0059 §1, T1-ID-01): random renames of
/// definitions, units, types, fields, variants, parameters, locals and
/// labels in generated and corpus programs leave every identity, and a
/// literal edit changes the edited definition's.
#[test]
fn renames_keep_identities_and_edits_change_them() {
    let iters = env("BMB_FMT_FUZZ_ITERS", 24);
    let seed = env("BMB_FMT_FUZZ_SEED", 0);
    let (mut renames, mut edits) = (0, 0);
    for i in 0..iters {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(i);
        let mut rng = Rng(s.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let units = gen_program(&mut rng);
        let base = one(units.iter().map(|u| unit_text(u, "s")).collect());
        let what = format!("seed {seed} program {i}");
        renames += check_renames(&mut rng, &what, &base, 4);
        edits += usize::from(check_edit(&mut rng, &what, &base));
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut rng = Rng(seed.wrapping_mul(0xA24BAED4963EE407) ^ 0x2545F491 | 1);
    let mut files = 0;
    for dir in ["check/ok", "verify/pass", "run"] {
        let mut paths: Vec<_> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        paths.sort();
        for p in paths {
            let base = one(fs::read_to_string(&p).unwrap());
            let what = p.display().to_string();
            renames += check_renames(
                &mut rng,
                &what,
                &base,
                env("BMB_FMT_FUZZ_RENAMES", 3) as usize,
            );
            edits += usize::from(check_edit(&mut rng, &what, &base));
            files += 1;
        }
    }
    assert!(files > 10, "corpus too small: {files}");
    assert!(
        renames > 200 && edits > 50,
        "renames {renames}, edits {edits}"
    );
}

/// Corpus programs (units moved across files and relaid out; their
/// definitions stay in place) keep their normal form and hashes.
#[test]
fn corpus_programs_keep_normal_form_and_hashes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let seed = env("BMB_FMT_FUZZ_SEED", 0);
    let mut rng = Rng(seed.wrapping_mul(0xD1B54A32D192ED03) ^ 0x5DEECE66D | 1);
    let mut n = 0;
    for dir in ["check/ok", "verify/pass", "verify/fail", "run"] {
        let mut paths: Vec<_> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        paths.sort();
        for p in paths {
            let base = one(fs::read_to_string(&p).unwrap());
            check_variants(&mut rng, &p.display().to_string(), &base, None);
            n += 1;
        }
    }
    assert!(n > 10, "corpus too small: {n}");
}
