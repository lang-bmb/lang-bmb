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
