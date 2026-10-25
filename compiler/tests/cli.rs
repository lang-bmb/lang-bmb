//! Command-line behaviour that no other test covers.

use std::process::{Command, Stdio};

/// Output into a closed pipe (`bmb codes | head -1` once `head` has exited)
/// ends quietly with status 141, as a process killed by SIGPIPE, instead of
/// panicking with a backtrace (backlog 1D-24).
#[test]
fn a_closed_stdout_ends_quietly() {
    let bmb = env!("CARGO_BIN_EXE_bmb");
    for args in [&["codes"][..], &["codes", "--md"]] {
        let (reader, writer) = std::io::pipe().unwrap();
        drop(reader);
        let out = Command::new(bmb)
            .args(args)
            .stdout(writer)
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(141), "{args:?}: {err}");
        assert!(err.is_empty(), "{args:?}: {err}");
    }
}

/// An unknown local name suggests the one visible local within edit
/// distance 2, as a replacement `bmb fix` can apply (backlog 2D-18); two
/// equally near names suggest nothing.
#[test]
fn an_unknown_local_suggests_the_nearest_one() {
    let dir = std::env::temp_dir().join(format!("bmb-cli-near-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let check = |body: &str| {
        let f = dir.join("a.bmb");
        std::fs::write(
            &f,
            format!("#[summary(\"t\")]\nmod a {{\n    #[effects(pure)]\n    fn f(count: i64, other: i64) -> i64 {{\n        {body}\n    }}\n}}\n"),
        )
        .unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("check")
            .arg(&f)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let out = check("cuont");
    assert!(out.contains("\"code\":\"E0301\""), "{out}");
    assert!(
        out.contains("\"fix\":{\"message\":\"did you mean `count`?\",\"replacement\":\"count\"}"),
        "{out}"
    );
    let out = check("zzzzzz");
    assert!(
        out.contains("\"code\":\"E0301\"") && !out.contains("did you mean"),
        "{out}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
