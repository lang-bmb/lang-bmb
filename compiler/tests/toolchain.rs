//! The pinned toolchain must be present in every development and CI environment
//! (run scripts/setup.sh). These tests fail rather than skip when it is not.
//! Through the CLI (`bmb doctor`), so they test whichever compiler is built
//! (backlog 4A-04).

use std::process::Command;

fn doctor() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("doctor")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The `doctor` row of `tool`: its required version equals what was found.
fn pinned(tool: &str) {
    let out = doctor();
    let row = out
        .lines()
        .find(|l| l.contains(&format!("\"tool\":\"{tool}\"")))
        .unwrap_or_else(|| panic!("no {tool} row:\n{out}"));
    assert!(row.contains("\"status\":\"ok\""), "{row}");
    let field = |k: &str| {
        let at = row.find(&format!("\"{k}\":\"")).unwrap() + k.len() + 4;
        row[at..].split('"').next().unwrap().to_string()
    };
    assert_eq!(field("required"), field("detail"), "{row}");
}

#[test]
fn z3_is_pinned_version() {
    pinned("z3");
}

#[test]
fn llvm_is_pinned_major() {
    pinned("llvm");
}
