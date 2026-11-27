//! Differential test: the stage0 side of what only the MCP server reaches
//! of the editing surfaces. `edit.bmb` does the same job with the port
//! (`selfhost/ast_json.bmb`, `selfhost/udiff.bmb`, `selfhost/view.bmb`,
//! `selfhost/workspace.bmb`); the cases `tree-*`, `udiff*`, `compact*` and
//! `plan*` compare the two outputs (`tests/port/README.md`).
//!
//! | Arguments | What runs | What is printed |
//! | --- | --- | --- |
//! | `import FILE..` | each line of each file as a JSON document through `ast_json::import` | `json:` and the reader's error; `error:` and the error of `import`; or `unit NAME` and the normal form of each unit (`fmt::format_units`), then `tree same` when `ast_json::export` of the program is the document again, `tree` and the document it gives otherwise |
//! | `patch DOC FILE` | each line of FILE as a JSON Patch on the document of DOC (`ast_json::apply_patch`) | `error:` and the error; or `ok`, the length and the SHA-256 of the patched document, then whether it imports |
//! | `pointer DOC FILE` | each line of FILE as a JSON Pointer (`ast_json::pointer`, `ast_json::resolve` on the document of DOC) | the segments, then the length and SHA-256 of the value (the value itself up to 200 bytes), or the error |
//! | `spans FILE..` | the files parsed together (`workspace::parse_files`), `ast_json::export_spans`, then `ast_json::node_at` at the range of every K-th node (K so that about 200 are asked), at its start, at its first byte and past its end | each span with its pointer, then each answer |
//! | `udiff DIFF TEXT..` | `udiff::parse` of DIFF, then `udiff::apply` of each file's hunks to each TEXT | the patches with their hunks and lines, then each result or error |
//! | `compact FILE..` | `view::compact` of each file | its result |
//! | `plan OLD NEW [NAME FILE]..` | `workspace::relative_sources` of the two workspaces, the files of NEW with each FILE under NAME after them, `workspace::plan_files` | the commit (ok, diagnostics, changed, removed) and the units to write |
//!
//! An example of the `bmb` crate (`compiler/Cargo.toml`), so that it calls
//! the compiler's own functions.

use bmb::ast_json;
use bmb::diag::json_str;
use bmb::json::{self, Value};
use std::io::Write;

fn read(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("cannot read {path}");
            std::process::exit(2);
        }
    }
}

fn names(xs: &[String]) -> String {
    let xs: Vec<String> = xs.iter().map(|x| json_str(x)).collect();
    format!("[{}]", xs.join(","))
}

fn import(out: &mut impl Write, files: &[String]) {
    for f in files {
        for line in read(f).lines() {
            let doc = match json::parse(line) {
                Ok(d) => d,
                Err(e) => {
                    writeln!(out, "json: {e}").unwrap();
                    continue;
                }
            };
            match ast_json::import(&doc) {
                Err(e) => writeln!(out, "error: {e}").unwrap(),
                Ok(p) => {
                    for (unit, text) in bmb::fmt::format_units(&p) {
                        write!(out, "unit {unit}\n{text}").unwrap();
                    }
                    match ast_json::export(&p) {
                        Ok(v) if v.to_json() == doc.to_json() => {
                            writeln!(out, "tree same").unwrap()
                        }
                        Ok(v) => writeln!(out, "tree {}", v.to_json()).unwrap(),
                        Err(e) => writeln!(out, "tree error: {e}").unwrap(),
                    }
                }
            }
        }
    }
}

fn document(out: &mut impl Write, path: &str) -> Option<Value> {
    match json::parse(&read(path)) {
        Ok(d) => Some(d),
        Err(e) => {
            writeln!(out, "json: {e}").unwrap();
            None
        }
    }
}

fn patch(out: &mut impl Write, doc: &str, patches: &str) {
    let Some(doc) = document(out, doc) else {
        return;
    };
    for line in read(patches).lines() {
        let p = match json::parse(line) {
            Ok(p) => p,
            Err(e) => {
                writeln!(out, "json: {e}").unwrap();
                continue;
            }
        };
        let mut work = doc.clone();
        match ast_json::apply_patch(&mut work, &p) {
            Err(e) => writeln!(out, "error: {e}").unwrap(),
            Ok(()) => {
                let text = work.to_json();
                writeln!(
                    out,
                    "ok {} {}",
                    text.len(),
                    bmb::sha256::hex(text.as_bytes())
                )
                .unwrap();
                match ast_json::import(&work) {
                    Err(e) => writeln!(out, "  import error: {e}").unwrap(),
                    Ok(p) => writeln!(out, "  imports {}", p.mods.len()).unwrap(),
                }
            }
        }
    }
}

fn pointer(out: &mut impl Write, doc: &str, pointers: &str) {
    let Some(doc) = document(out, doc) else {
        return;
    };
    for line in read(pointers).lines() {
        match ast_json::pointer(line) {
            Err(e) => writeln!(out, "error: {e}").unwrap(),
            Ok(segs) => {
                writeln!(out, "segs {}", names(&segs)).unwrap();
                match ast_json::resolve(&doc, &segs) {
                    Err(e) => writeln!(out, "  error: {e}").unwrap(),
                    Ok(v) => {
                        let text = v.to_json();
                        writeln!(
                            out,
                            "  value {} {}",
                            text.len(),
                            bmb::sha256::hex(text.as_bytes())
                        )
                        .unwrap();
                        if text.len() <= 200 {
                            writeln!(out, "  {text}").unwrap();
                        }
                    }
                }
            }
        }
    }
}

fn spans(out: &mut impl Write, files: &[String]) {
    let files: Vec<(String, String)> = files.iter().map(|f| (f.clone(), read(f))).collect();
    let (program, sm, diags) = bmb::workspace::parse_files(&files);
    if diags.has_errors() {
        write!(out, "{}", diags.to_jsonl(&sm)).unwrap();
        return;
    }
    let mut marks = ast_json::Spans::new();
    if let Err(e) = ast_json::export_spans(&program, &mut marks) {
        writeln!(out, "error: {e}").unwrap();
    }
    for (s, p) in &marks {
        writeln!(out, "{} {} {} {p}", s.file, s.lo, s.hi).unwrap();
    }
    let step = std::cmp::max(1, marks.len() / 200);
    let mut ask = |file: u32, lo: u32, hi: u32| {
        let at = ast_json::node_at(&marks, file, lo, hi).unwrap_or("none");
        writeln!(out, "at {file} {lo} {hi} -> {at}").unwrap();
    };
    let mut k = 0;
    while k < marks.len() {
        let s = marks[k].0;
        ask(s.file, s.lo, s.hi);
        ask(s.file, s.lo, s.lo);
        ask(s.file, s.lo, if s.lo < s.hi { s.lo + 1 } else { s.lo });
        ask(s.file, s.hi, s.hi.saturating_add(1));
        k += step;
    }
}

fn shown(p: &Option<String>) -> String {
    p.as_ref().map_or("null".to_string(), |p| json_str(p))
}

fn udiff(out: &mut impl Write, diff: &str, texts: &[String]) {
    let patches = match bmb::udiff::parse(&read(diff)) {
        Ok(ps) => ps,
        Err(e) => {
            writeln!(out, "error: {e}").unwrap();
            return;
        }
    };
    for p in &patches {
        writeln!(out, "patch old={} new={}", shown(&p.old), shown(&p.new)).unwrap();
        for h in &p.hunks {
            let start = h.old_start.map_or("none".to_string(), |a| a.to_string());
            writeln!(
                out,
                "hunk {} start={start} before={} after={}",
                json_str(&h.header),
                h.before().len(),
                h.after().len()
            )
            .unwrap();
            for (c, l) in &h.lines {
                writeln!(out, "{c}{}", json_str(l)).unwrap();
            }
        }
    }
    for t in texts {
        let text = read(t);
        for (k, p) in patches.iter().enumerate() {
            match bmb::udiff::apply(&text, &p.hunks) {
                Ok(after) => writeln!(out, "applied {k} {}", json_str(&after)).unwrap(),
                Err(e) => writeln!(out, "apply error {k}: {e}").unwrap(),
            }
        }
    }
}

fn plan(out: &mut impl Write, old: &str, new: &str, more: &[String]) {
    let old = match bmb::workspace::relative_sources(std::path::Path::new(old)) {
        Ok(fs) => fs,
        Err(e) => {
            writeln!(out, "error: {e}").unwrap();
            return;
        }
    };
    let mut new = match bmb::workspace::relative_sources(std::path::Path::new(new)) {
        Ok(fs) => fs,
        Err(e) => {
            writeln!(out, "error: {e}").unwrap();
            return;
        }
    };
    for pair in more.chunks(2) {
        if pair.len() == 2 {
            new.push((pair[0].clone(), read(&pair[1])));
        }
    }
    for (name, _) in &new {
        writeln!(out, "file {}", json_str(name)).unwrap();
    }
    let (c, units) = bmb::workspace::plan_files(&old, &new);
    writeln!(out, "ok {}", c.ok).unwrap();
    write!(out, "{}", c.diagnostics).unwrap();
    writeln!(out, "changed {}", names(&c.changed)).unwrap();
    writeln!(out, "removed {}", names(&c.removed)).unwrap();
    match units {
        None => writeln!(out, "no units").unwrap(),
        Some(units) => {
            for (unit, text) in units {
                write!(out, "unit {unit}\n{text}").unwrap();
            }
        }
    }
}

fn main() {
    // The stack of the compiler's own commands (`bmb::with_stack`): a tree
    // at the nesting limit is walked on it.
    bmb::with_stack(run)
}

fn run() {
    let args: Vec<String> = std::env::args().collect();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    match args.get(1).map(String::as_str) {
        Some("import") => import(&mut out, &args[2..]),
        Some("patch") if args.len() == 4 => patch(&mut out, &args[2], &args[3]),
        Some("pointer") if args.len() == 4 => pointer(&mut out, &args[2], &args[3]),
        Some("spans") => spans(&mut out, &args[2..]),
        Some("udiff") if args.len() > 2 => udiff(&mut out, &args[2], &args[3..]),
        Some("compact") => {
            for f in &args[2..] {
                write!(out, "{}", bmb::view::compact(&read(f))).unwrap();
            }
        }
        Some("plan") if args.len() > 3 => plan(&mut out, &args[2], &args[3], &args[4..]),
        _ => {
            eprintln!(
                "usage: edit_ref import FILE.. | patch DOC FILE | pointer DOC FILE | spans FILE.. | udiff DIFF TEXT.. | compact FILE.. | plan OLD NEW [NAME FILE].."
            );
            std::process::exit(2);
        }
    }
}
