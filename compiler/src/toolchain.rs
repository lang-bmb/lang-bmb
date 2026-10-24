//! Pinned external tools (INF-02, INF-03). Versions come from
//! `scripts/toolchain.env`, the single source of truth. A missing tool or a
//! different version is an error, never a warning: the proof gate must not run
//! on an unknown solver.

use std::ffi::OsString;
use std::process::Command;
use std::sync::Mutex;

const ENV: &str = include_str!("../../scripts/toolchain.env");

pub fn pinned(key: &str) -> &'static str {
    ENV.lines()
        .filter_map(|l| l.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim())
        .unwrap_or_else(|| panic!("{key} missing from scripts/toolchain.env"))
}

pub fn z3_version() -> &'static str {
    pinned("Z3_VERSION")
}

pub fn llvm_major() -> &'static str {
    pinned("LLVM_MAJOR")
}

/// Extracts `X.Y.Z` from `Z3 version X.Y.Z - 64 bit`.
pub fn parse_z3_version(out: &str) -> Option<&str> {
    out.split_whitespace()
        .skip_while(|w| *w != "version")
        .nth(1)
}

/// Extracts the major version from `llc --version` output.
pub fn parse_llvm_major(out: &str) -> Option<&str> {
    let rest = out.split("LLVM version ").nth(1)?;
    rest.split('.').next()
}

fn run(cmd: &str, arg: &str) -> Result<String, String> {
    let out = Command::new(cmd)
        .arg(arg)
        .output()
        .map_err(|e| format!("`{cmd}` not found: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Ok(version) if `z3` on PATH is exactly the pinned version.
///
/// A successful check is remembered for the process as long as `PATH` is
/// unchanged (backlog 1D-18): every proof run asks, and `z3 --version` is a
/// process start. Failures are not remembered, so a missing or wrong solver is
/// reported on every call.
pub fn check_z3() -> Result<String, String> {
    static OK: Mutex<Option<(Option<OsString>, String)>> = Mutex::new(None);
    let path = std::env::var_os("PATH");
    let mut ok = OK.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((p, v)) = ok.as_ref()
        && *p == path
    {
        return Ok(v.clone());
    }
    let v = check_z3_uncached()?;
    *ok = Some((path, v.clone()));
    Ok(v)
}

fn check_z3_uncached() -> Result<String, String> {
    let out = run("z3", "--version")?;
    let found =
        parse_z3_version(&out).ok_or_else(|| format!("cannot read z3 version from {out:?}"))?;
    if found == z3_version() {
        Ok(found.to_string())
    } else {
        Err(format!(
            "z3 {found} found, {} required (run scripts/setup.sh)",
            z3_version()
        ))
    }
}

/// Ok(major) if `llc` on PATH has the pinned major version.
pub fn check_llvm() -> Result<String, String> {
    let out = run("llc", "--version")?;
    let found = parse_llvm_major(&out).ok_or_else(|| "cannot read llc version".to_string())?;
    if found == llvm_major() {
        Ok(found.to_string())
    } else {
        Err(format!(
            "LLVM {found} found, {} required (run scripts/setup.sh)",
            llvm_major()
        ))
    }
}

/// Compiles LLVM IR to an executable with the pinned toolchain.
pub fn build_executable(ir: &str, out: &str) -> Result<(), String> {
    check_llvm()?;
    let ll = format!("{out}.ll");
    std::fs::write(&ll, ir).map_err(|e| format!("cannot write {ll}: {e}"))?;
    let res = Command::new("clang")
        .args(["-O2", "-Wno-override-module", "-o", out, &ll])
        .output()
        .map_err(|e| format!("cannot run clang: {e}"))?;
    let _ = std::fs::remove_file(&ll);
    if res.status.success() {
        Ok(())
    } else {
        Err(format!(
            "clang failed: {}",
            String::from_utf8_lossy(&res.stderr)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_versions() {
        assert_eq!(
            parse_z3_version("Z3 version 4.15.4 - 64 bit\n"),
            Some("4.15.4")
        );
        assert_eq!(parse_z3_version("garbage"), None);
        assert_eq!(
            parse_llvm_major("LLVM (http://llvm.org/):\n  Ubuntu LLVM version 18.1.3\n"),
            Some("18")
        );
    }

    #[test]
    fn pinned_values_present() {
        assert!(!z3_version().is_empty());
        assert!(!llvm_major().is_empty());
    }
}
