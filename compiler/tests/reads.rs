//! `bmb reads` (backlog T1-27, ADR-0065 §2): each definition's proof key as
//! rows, the read set of its proof. Through the command line: the rows are
//! the key the incremental cache uses (a warm `verify --stats` re-encodes
//! exactly the definitions whose rows changed), and they follow the contract
//! firewall (a callee's body is not read, its contract is; a revealed spec
//! fn's body is read by the revealing proof only).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bmb(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap()
}

/// The string value of `key` in one JSON line.
fn field(line: &str, key: &str) -> Option<String> {
    let at = line.find(&format!("\"{key}\":"))? + key.len() + 3;
    let v = line[at..].strip_prefix('"')?;
    Some(v[..v.find('"')?].to_string())
}

/// Per definition: (key, rows as (kind, path) -> digest).
type Reads = BTreeMap<String, (String, BTreeMap<(String, String), String>)>;

fn reads(src: &Path) -> Reads {
    bmb(&["reads", src.to_str().unwrap()])
        .lines()
        .filter(|l| l.starts_with("{\"def\":"))
        .map(|l| {
            let at = l.find("\"reads\":[").unwrap() + 9;
            let rows = l[at..l.len() - 2]
                .split("},{")
                .filter(|r| !r.is_empty())
                .map(|r| {
                    (
                        (field(r, "kind").unwrap(), field(r, "path").unwrap()),
                        field(r, "digest").unwrap(),
                    )
                })
                .collect();
            (field(l, "def").unwrap(), (field(l, "key").unwrap(), rows))
        })
        .collect()
}

fn ws(dir: &Path, name: &str, text: &str) -> PathBuf {
    let d = dir.join(name);
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src").join("m.bmb"), text).unwrap();
    d
}

const BASE: &str = "#[summary(\"reads\")]
mod m {
    spec fn small(x: i64) -> bool { x < 100 }

    spec fn pos(x: i64) -> bool { x > 0 }

    #[effects(pure)]
    #[uses(small)]
    fn f(x: i64) -> i64
        requires small(x),
    {
        x + 1
    }

    #[effects(pure)]
    fn g() -> i64 {
        f(1)
    }

    #[effects(pure)]
    #[uses(small)]
    fn h() -> i64 {
        f(2)
    }
}
";

#[test]
fn reads_follow_the_contract_firewall() {
    let dir = std::env::temp_dir().join(format!("bmb-reads-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let base = reads(&ws(&dir, "base", BASE));
    let row =
        |r: &Reads, d: &str, k: &str, p: &str| r[d].1.get(&(k.to_string(), p.to_string())).cloned();
    // What each proof reads: `g` sees `f`'s contract and `small` by name,
    // `h` reveals `small`.
    assert!(row(&base, "m::g", "callee", "m::f").is_some());
    assert!(row(&base, "m::g", "decl", "m::small").is_some());
    assert!(row(&base, "m::g", "spec", "m::small").is_none());
    assert!(row(&base, "m::h", "spec", "m::small").is_some());
    assert_eq!(
        row(&base, "m::h", "spec", "m::small"),
        row(&base, "m::small", "self", "m::small"),
        "a revealed body is the spec fn's own text"
    );
    // A body edit of `f`: its own row changes, no caller's.
    let body = reads(&ws(&dir, "body", &BASE.replace("x + 1", "x + 2")));
    assert_ne!(base["m::f"], body["m::f"]);
    assert_eq!(base["m::g"], body["m::g"]);
    assert_eq!(base["m::h"], body["m::h"]);
    // A contract edit: every caller's `callee` row (and the new
    // declaration it reaches).
    let contract = reads(&ws(
        &dir,
        "contract",
        &BASE.replace("requires small(x),", "requires small(x), pos(x),"),
    ));
    for d in ["m::g", "m::h"] {
        assert_ne!(
            row(&base, d, "callee", "m::f"),
            row(&contract, d, "callee", "m::f"),
            "{d}"
        );
        assert!(row(&contract, d, "decl", "m::pos").is_some(), "{d}");
        assert_eq!(
            row(&base, d, "self", d),
            row(&contract, d, "self", d),
            "{d}"
        );
    }
    // A spec fn body edit: the proofs that reveal it (`f`, `h`), not `g`,
    // which sees it by name.
    let spec = reads(&ws(&dir, "spec", &BASE.replace("x < 100", "x < 50")));
    assert_ne!(base["m::f"], spec["m::f"]);
    assert_ne!(base["m::h"], spec["m::h"]);
    assert_eq!(base["m::g"], spec["m::g"]);
    let _ = fs::remove_dir_all(&dir);
}

/// The rows are the cache's key: two versions give a definition the same
/// rows exactly when they give it the same key, and a warm `verify` encodes
/// exactly the definitions whose key changed (the corpus case that changes
/// contracts under their readers, `tests/weaken/readers`).
#[test]
fn reads_are_the_incremental_proof_key() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/weaken/readers");
    let dir = std::env::temp_dir().join(format!("bmb-reads-key-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let mk = |name: &str, version: &str| {
        let d = dir.join(name).join("src");
        fs::create_dir_all(&d).unwrap();
        fs::copy(corpus.join(version), d.join("m.bmb")).unwrap();
        fs::copy(corpus.join("callers.bmb"), d.join("c.bmb")).unwrap();
        dir.join(name)
    };
    let (a, b) = (mk("a", "old.bmb"), mk("b", "new.bmb"));
    let (ra, rb) = (reads(&a), reads(&b));
    let mut changed = BTreeSet::new();
    for (d, (key, rows)) in &rb {
        let (key0, rows0) = &ra[d];
        assert_eq!(key == key0, rows == rows0, "{d}");
        if key != key0 {
            changed.insert(d.clone());
        }
    }
    assert!(changed.contains("c::dec_opaque") && !changed.contains("m::pos"));
    let cache = dir.join("cache");
    let stats = dir.join("stats.json");
    let c = cache.to_str().unwrap();
    bmb(&["verify", "--cache", c, a.to_str().unwrap()]);
    bmb(&[
        "verify",
        "--cache",
        c,
        "--stats",
        stats.to_str().unwrap(),
        b.to_str().unwrap(),
    ]);
    let s = fs::read_to_string(&stats).unwrap();
    let at = s.find("\"encoded\":[").unwrap() + 11;
    let encoded: BTreeSet<String> = s[at..at + s[at..].find(']').unwrap()]
        .split(',')
        .filter(|x| !x.is_empty())
        .map(|x| x.trim_matches('"').to_string())
        .collect();
    // Spec fns have keys but no proof of their own to encode.
    let exec: BTreeSet<String> = changed
        .into_iter()
        .filter(|d| !["m::pos", "m::gt0", "m::big", "m::mid"].contains(&d.as_str()))
        .collect();
    assert_eq!(encoded, exec);
    let _ = fs::remove_dir_all(&dir);
}
