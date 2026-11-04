//! The fuzzers' builds through the `bmb` command line (backlog 4A-04): the
//! same tests then test whichever compiler is built, and no test depends on
//! the library API for it (HEALTH.md S5).

use std::fs;
use std::path::Path;
use std::process::Command;

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
