//! Compiler identity for the incremental cache (backlog 2D-10): a SHA-256
//! over every file under `src/`, the pinned toolchain file and the standard
//! library, so any change to the compiler invalidates every cached
//! definition. Also the standard library's units, embedded (ADR-0092 §1,
//! backlog 4B-02): `$OUT_DIR/std_units.rs` lists every `lib/std/*.bmb` by
//! its stem.

#[allow(dead_code)]
#[path = "src/sha256.rs"]
mod sha256;

use std::path::{Path, PathBuf};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let mut list = Vec::new();
    files(&root.join("src"), &mut list);
    list.push(root.join("../scripts/toolchain.env"));
    let std_dir = root.join("../lib/std");
    let mut units: Vec<PathBuf> = std::fs::read_dir(&std_dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
                .collect()
        })
        .unwrap_or_default();
    units.sort();
    list.extend(units.iter().cloned());
    list.sort();
    let mut data = Vec::new();
    for p in &list {
        let rel = p.strip_prefix(&root).unwrap_or(p);
        data.extend_from_slice(rel.to_string_lossy().as_bytes());
        data.push(0);
        let body = std::fs::read(p).unwrap_or_default();
        data.extend_from_slice(&(body.len() as u64).to_le_bytes());
        data.extend_from_slice(&body);
    }
    println!("cargo:rustc-env=BMB_SOURCE_DIGEST={}", sha256::hex(&data));
    let mut table = String::from(
        "/// The standard library's units: stem and source text.\npub const STD_UNITS: &[(&str, &str)] = &[\n",
    );
    for u in &units {
        let stem = u.file_stem().unwrap().to_string_lossy();
        let abs = u.canonicalize().unwrap_or_else(|_| u.clone());
        table.push_str(&format!(
            "    ({stem:?}, include_str!({:?})),\n",
            abs.to_string_lossy()
        ));
    }
    table.push_str("];\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("out dir"));
    std::fs::write(out.join("std_units.rs"), table).expect("write std_units.rs");
    println!("cargo:rerun-if-changed=../lib/std");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../scripts/toolchain.env");
    println!("cargo:rerun-if-changed=build.rs");
}
