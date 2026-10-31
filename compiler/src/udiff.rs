//! Unified diffs (backlog T1-06, ADR-0073): the file diff edit surface
//! (`edit_file`). Parsing and applying follow `git apply --recount`: a hunk
//! is its lines, the counts in `@@ -a,b +c,d @@` are not trusted, the line
//! number is where to look first, and context and removed lines must match
//! the file exactly (no fuzz, whitespace included).

/// The changes to one file. A path is `None` for `/dev/null` (a created
/// or deleted file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    pub old: Option<String>,
    pub new: Option<String>,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// The header as written, for messages.
    pub header: String,
    /// The old start line (1-based) from the header, if it gives one.
    pub old_start: Option<usize>,
    /// `(' ' | '-' | '+', text)`.
    pub lines: Vec<(char, String)>,
}

impl Hunk {
    /// The lines the hunk expects in the file (context and removed).
    pub fn before(&self) -> Vec<&str> {
        self.lines
            .iter()
            .filter(|(c, _)| *c != '+')
            .map(|(_, l)| l.as_str())
            .collect()
    }

    /// The lines it leaves there (context and added).
    pub fn after(&self) -> Vec<&str> {
        self.lines
            .iter()
            .filter(|(c, _)| *c != '-')
            .map(|(_, l)| l.as_str())
            .collect()
    }
}

/// A path in a `---`/`+++` line: `/dev/null`, or the path without git's
/// `a/`·`b/` prefix and without a trailing timestamp.
fn header_path(s: &str) -> Option<String> {
    let s = s.split('\t').next().unwrap_or("").trim_end();
    if s == "/dev/null" {
        return None;
    }
    let s = s
        .strip_prefix("a/")
        .or_else(|| s.strip_prefix("b/"))
        .unwrap_or(s);
    Some(s.to_string())
}

/// `@@ -a[,b] +c[,d] @@ ..` -> `Some(a)`; `@@ .. @@` without numbers -> `None`.
fn old_start(header: &str) -> Result<Option<usize>, String> {
    let rest = header.trim_start_matches('@').trim_start();
    let Some(r) = rest.strip_prefix('-') else {
        return Ok(None);
    };
    let num: String = r.chars().take_while(char::is_ascii_digit).collect();
    if num.is_empty() {
        return Err(format!(
            "hunk header `{header}`: expected `@@ -LINE,COUNT +LINE,COUNT @@`"
        ));
    }
    num.parse::<usize>()
        .map(Some)
        .map_err(|_| format!("hunk header `{header}`: line number out of range"))
}

/// Git metadata lines that may stand between files.
fn metadata(l: &str) -> bool {
    [
        "diff ",
        "index ",
        "new file mode",
        "deleted file mode",
        "old mode",
        "new mode",
        "similarity index",
    ]
    .iter()
    .any(|p| l.starts_with(p))
}

/// Parses a unified diff. Text before the first `---` line is ignored (a
/// commit message, `diff --git` lines); inside a file, every line belongs to
/// a hunk. An empty line in a hunk is an empty context line; empty lines at
/// a hunk's end are dropped.
pub fn parse(diff: &str) -> Result<Vec<FilePatch>, String> {
    let lines: Vec<&str> = diff
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    let header_at = |i: usize| {
        lines[i].starts_with("--- ") && lines.get(i + 1).is_some_and(|l| l.starts_with("+++ "))
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("@@") {
            return Err(format!(
                "line {}: a hunk before any `--- PATH` / `+++ PATH` lines",
                i + 1
            ));
        }
        if !header_at(i) {
            i += 1;
            continue;
        }
        let old = header_path(&lines[i][4..]);
        let new = header_path(&lines[i + 1][4..]);
        i += 2;
        let mut hunks = Vec::new();
        while i < lines.len() && lines[i].starts_with("@@") {
            let header = lines[i].to_string();
            let start = old_start(&header)?;
            i += 1;
            let mut body: Vec<(char, String, bool)> = Vec::new();
            while i < lines.len() && !lines[i].starts_with("@@") && !header_at(i) {
                let l = lines[i];
                match l.chars().next() {
                    None => body.push((' ', String::new(), true)),
                    Some(c @ (' ' | '-' | '+')) => body.push((c, l[1..].to_string(), false)),
                    Some('\\') => {}
                    Some(_) => break,
                }
                i += 1;
            }
            while body.last().is_some_and(|(_, _, empty)| *empty) {
                body.pop();
            }
            if body.is_empty() {
                return Err(format!("hunk `{header}` has no lines"));
            }
            hunks.push(Hunk {
                header,
                old_start: start,
                lines: body.into_iter().map(|(c, l, _)| (c, l)).collect(),
            });
        }
        if i < lines.len() && !header_at(i) && !metadata(lines[i]) {
            if lines[i..].iter().all(|l| l.trim().is_empty()) {
                i = lines.len();
            } else {
                return Err(format!(
                    "line {}: `{}` is not a diff line (in a hunk, each line starts with ' ' for context, '-' or '+')",
                    i + 1,
                    lines[i]
                ));
            }
        }
        if hunks.is_empty() && new.is_some() {
            return Err(format!(
                "`--- {}` / `+++ {}` has no hunk (`@@ -LINE,COUNT +LINE,COUNT @@`)",
                old.as_deref().unwrap_or("/dev/null"),
                new.as_deref().unwrap_or("/dev/null")
            ));
        }
        out.push(FilePatch { old, new, hunks });
    }
    Ok(out)
}

/// Applies `hunks` to `text` in order. Each hunk's context and removed
/// lines must appear in the file, after the previous hunk, exactly; among
/// several places the one nearest the header's line number is taken, and
/// without a line number a hunk must match in one place only.
pub fn apply(text: &str, hunks: &[Hunk]) -> Result<String, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<&str> = Vec::new();
    let mut pos = 0;
    for (k, h) in hunks.iter().enumerate() {
        let before = h.before();
        let which = || format!("hunk {} (`{}`)", k + 1, h.header);
        let at = if before.is_empty() {
            // Only added lines: the header says after which line.
            match h.old_start {
                Some(a) if a >= pos && a <= lines.len() => a,
                Some(a) => {
                    return Err(format!(
                        "{}: line {a} is outside the file ({} lines) or before the previous hunk",
                        which(),
                        lines.len()
                    ));
                }
                None if lines.is_empty() => 0,
                None => {
                    return Err(format!(
                        "{}: a hunk with only added lines needs a line number (`@@ -LINE,0 +..`)",
                        which()
                    ));
                }
            }
        } else {
            let found: Vec<usize> = (pos..=lines.len().saturating_sub(before.len()))
                .filter(|&p| {
                    p + before.len() <= lines.len() && lines[p..p + before.len()] == before[..]
                })
                .collect();
            match (found.as_slice(), h.old_start) {
                ([], _) => {
                    return Err(format!("{}: {}", which(), mismatch(&lines[pos..], &before)));
                }
                ([p], _) => *p,
                (ps, Some(a)) => *ps
                    .iter()
                    .min_by_key(|&&p| (p as i64 - (a as i64 - 1)).abs())
                    .unwrap(),
                (ps, None) => {
                    let at: Vec<String> = ps.iter().map(|p| (p + 1).to_string()).collect();
                    return Err(format!(
                        "{}: matches at lines {}; give the line number or more context",
                        which(),
                        at.join(", ")
                    ));
                }
            }
        };
        out.extend_from_slice(&lines[pos..at]);
        out.extend(h.after());
        pos = at + before.len();
    }
    out.extend_from_slice(&lines[pos..]);
    if out.is_empty() {
        return Ok(String::new());
    }
    Ok(out.join("\n") + "\n")
}

/// Why `before` is not in `lines`.
fn mismatch(lines: &[&str], before: &[&str]) -> String {
    if let Some(l) = before.iter().find(|l| !lines.contains(l)) {
        let loose = lines.iter().any(|x| x.trim() == l.trim());
        return format!(
            "the context or removed line `{l}` is not in the file{}",
            if loose {
                " (it is, with other whitespace: indentation is part of the line)"
            } else {
                ""
            }
        );
    }
    "its context and removed lines are in the file, but not together in this order".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "a\nb\nc\nd\ne\nb\nc\n";

    fn one(diff: &str) -> Vec<Hunk> {
        let ps = parse(diff).unwrap();
        assert_eq!(ps.len(), 1);
        ps[0].hunks.clone()
    }

    #[test]
    fn applies_with_recounted_hunks_and_the_nearest_match() {
        // Wrong counts are ignored; `b c` occurs twice and the header picks.
        let h = one("--- a/src/x.bmb\n+++ b/src/x.bmb\n@@ -6,9 +6,9 @@\n b\n-c\n+C\n");
        assert_eq!(apply(FILE, &h).unwrap(), "a\nb\nc\nd\ne\nb\nC\n");
        let h = one("--- a/src/x.bmb\n+++ b/src/x.bmb\n@@ -2 +2 @@\n b\n-c\n+C\n");
        assert_eq!(apply(FILE, &h).unwrap(), "a\nb\nC\nd\ne\nb\nc\n");
        // Without a line number an ambiguous hunk is refused.
        let h = one("--- src/x.bmb\n+++ src/x.bmb\n@@ @@\n b\n-c\n+C\n");
        assert!(
            apply(FILE, &h)
                .unwrap_err()
                .contains("matches at lines 2, 6")
        );
        let h = one("--- src/x.bmb\n+++ src/x.bmb\n@@ @@\n d\n+D\n");
        assert_eq!(apply(FILE, &h).unwrap(), "a\nb\nc\nd\nD\ne\nb\nc\n");
    }

    #[test]
    fn refuses_what_does_not_match_and_says_why() {
        let h = one("--- a/x\n+++ b/x\n@@ -1,2 +1,2 @@\n a\n-z\n+y\n");
        assert!(
            apply(FILE, &h)
                .unwrap_err()
                .contains("`z` is not in the file")
        );
        let h = one("--- a/x\n+++ b/x\n@@ -1,2 +1,2 @@\n  a\n-b\n");
        assert!(apply(FILE, &h).unwrap_err().contains("other whitespace"));
        let h = one("--- a/x\n+++ b/x\n@@ -1,2 +1,2 @@\n a\n-c\n");
        assert!(apply(FILE, &h).unwrap_err().contains("not together"));
    }

    #[test]
    fn parses_files_creation_deletion_and_rejects_stray_lines() {
        let ps = parse(
            "diff --git a/src/x.bmb b/src/x.bmb\n--- /dev/null\n+++ b/src/n.bmb\n@@ -0,0 +1,2 @@\n+x\n+y\n\ndiff --git a/src/o.bmb b/src/o.bmb\n--- a/src/o.bmb\n+++ /dev/null\n",
        )
        .unwrap();
        assert_eq!(ps[0].old, None);
        assert_eq!(ps[0].new.as_deref(), Some("src/n.bmb"));
        assert_eq!(apply("", &ps[0].hunks).unwrap(), "x\ny\n");
        assert_eq!(ps[1].old.as_deref(), Some("src/o.bmb"));
        assert_eq!(ps[1].new, None);
        // An empty line inside a hunk is empty context.
        let h = one("--- a/x\n+++ b/x\n@@ -1 +1 @@\n a\n\n+q\n");
        assert_eq!(apply("a\n\nb\n", &h).unwrap(), "a\n\nq\nb\n");
        // A line that forgot its prefix is an error, not the end of the diff.
        let e = parse("--- a/x\n+++ b/x\n@@ -1 +1 @@\n a\n}\n+q\n").unwrap_err();
        assert!(e.contains("line 5"), "{e}");
        assert!(parse("@@ -1 +1 @@\n a\n").is_err());
    }
}
