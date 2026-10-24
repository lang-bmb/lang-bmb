//! Compiler identity for the incremental cache (backlog 2D-10): a SHA-256
//! over every file under `src/` and the pinned toolchain file, so any change
//! to the compiler invalidates every cached definition.

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
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../scripts/toolchain.env");
    println!("cargo:rerun-if-changed=build.rs");
}
