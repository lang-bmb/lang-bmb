//! The fuzzers' builds through the `bmb` command line (backlog 4A-04): the
//! same tests then test whichever compiler is built, and no test depends on
//! the library API for it (HEALTH.md S5).

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// `bmb ARGS NAMES..` in a new directory holding `files` (name, text): its
/// success and stdout.
pub fn run(args: &[&str], files: &[(String, String)]) -> (bool, String) {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "bmb-cli-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(&dir)
        .args(args)
        .args(files.iter().map(|(n, _)| n))
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

/// The normal form `bmb fmt` printed, back into its units (name, text): each
/// unit ends at its unindented `}` and the next follows a blank line.
pub fn units(formatted: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let (mut name, mut text) = (String::new(), String::new());
    for line in formatted.lines() {
        if text.is_empty() && line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("mod ") {
            name = rest.trim_end_matches(" {").to_string();
        }
        text.push_str(line);
        text.push('\n');
        if line == "}" {
            out.push((std::mem::take(&mut name), std::mem::take(&mut text)));
        }
    }
    assert!(text.is_empty(), "a unit does not end: {text}");
    out
}

/// What `bmb emit-llvm` printed (the verification rows) and the IR it
/// wrote, when the program checks.
pub struct Emitted {
    pub output: String,
    pub ir: Option<String>,
}

/// `bmb emit-llvm` on `files`, written into `dir` under their names (so the
/// rows name them as given), proving through the proof cache `cache` when
/// given, or keeping every check (`--all-checks`).
pub fn emit_llvm(
    files: &[(String, String)],
    dir: &Path,
    cache: Option<&Path>,
    all_checks: bool,
) -> Emitted {
    fs::create_dir_all(dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    let ll = dir.join("out.ll");
    let _ = fs::remove_file(&ll);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
    cmd.current_dir(dir).arg("emit-llvm");
    if all_checks {
        cmd.arg("--all-checks");
    }
    if let Some(c) = cache {
        cmd.arg("--cache").arg(c);
    }
    let out = cmd
        .args(files.iter().map(|(name, _)| name))
        .arg("-o")
        .arg(&ll)
        .output()
        .unwrap();
    let ir = fs::read_to_string(&ll).ok();
    let _ = fs::remove_file(&ll);
    Emitted {
        output: String::from_utf8_lossy(&out.stdout).into_owned(),
        ir,
    }
}

/// Builds `ir` into `exe` as `bmb build` does (clang `-O2`).
pub fn build(ir: &str, exe: &Path) -> Result<(), String> {
    let ll = exe.with_extension("build.ll");
    fs::write(&ll, ir).unwrap();
    let cc = Command::new("clang")
        .args(["-O2", "-Wno-override-module", "-o"])
        .arg(exe)
        .arg(&ll)
        .output()
        .map_err(|e| format!("cannot run clang: {e}"))?;
    let _ = fs::remove_file(&ll);
    if cc.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed: {}",
            String::from_utf8_lossy(&cc.stderr)
        ))
    }
}
