//! `#[uses(..)]` auto-fill (backlog 2D-05).
//!
//! A `spec fn` is opaque to a proof unless the definition names it in
//! `#[uses(..)]` (ADR-0017 §4). A failed obligation's diagnostic already lists
//! the predicates the proof saw only by name (backlog 1D-13). This module
//! turns those hints into a minimal attribute: for each failing definition it
//! reveals its current entries plus every hinted predicate, repeating while
//! newly revealed bodies mention further hidden predicates (bounded), and if
//! that proves the definition it removes entries one at a time in sorted
//! order, keeping each removal that still proves.
//!
//! Every attempt rewrites the attribute text and re-checks and re-proves the
//! program, so the result is exactly what `bmb verify` would see. A
//! definition's obligations depend only on its own `#[uses]` (revealing is not
//! transitive, callees are seen through their contracts), so all failing
//! definitions advance in lockstep, one program verification per step. The
//! runs share an incremental cache (backlog 2D-15, `incremental::Session`,
//! in memory unless `--cache DIR`): a definition whose text and context did
//! not change since an earlier run is not encoded again, and the proof cache
//! keeps the changed definitions' unchanged obligations from being solved
//! again. It runs only on request (`bmb uses`); `check` and
//! `verify` are unchanged.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::ast::{FnKind, Mod};
use crate::cache::ProofCache;
use crate::diag::json_str;
use crate::incremental::Session;
use crate::source::Span;

/// Upper bound on verifications spent on one definition.
pub const MAX_VERIFICATIONS: usize = 32;
/// Upper bound on reveal steps (each may add predicates that newly revealed
/// bodies mention) before the shrinking phase.
pub const MAX_CLOSURE_STEPS: usize = 8;

/// Marker in a diagnostic's fix message after which the hidden predicates
/// are listed (see `verify::with_hint`).
const HINT: &str = "add to #[uses(";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsesStatus {
    /// It failed before; the reported list makes it prove.
    Proved,
    /// It already proves (or it is not an executable definition); untouched.
    Unchanged,
    /// No list built from the hints makes it prove; untouched.
    NotFound,
}

impl UsesStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            UsesStatus::Proved => "proved",
            UsesStatus::Unchanged => "unchanged",
            UsesStatus::NotFound => "not_found",
        }
    }
}

/// A replacement in the original text, as in a diagnostic `fix`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsesEdit {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub struct UsesRow {
    pub def: String,
    /// The definition's `#[uses]` entries after the run, sorted, as written
    /// in the attribute (local names unqualified).
    pub uses: Vec<String>,
    pub status: UsesStatus,
    /// Program verifications this definition took part in (not counting the
    /// first and the final confirmation).
    pub verifications: usize,
    /// The attribute rewrite, for `Proved` rows.
    pub edit: Option<UsesEdit>,
}

pub struct UsesOutput {
    pub rows: Vec<UsesRow>,
    /// Every input file with its rewritten text, in input order.
    pub files: Vec<(String, String)>,
    /// Program verifications in total, including the first and the final one.
    pub verifications: usize,
}

impl UsesOutput {
    /// True when no definition is left failing.
    pub fn ok(&self) -> bool {
        self.rows.iter().all(|r| r.status != UsesStatus::NotFound)
    }

    /// Files whose text changed, in input order.
    pub fn changed<'a>(&'a self, original: &[(String, String)]) -> Vec<&'a str> {
        self.files
            .iter()
            .zip(original)
            .filter(|(a, b)| a.1 != b.1)
            .map(|(a, _)| a.0.as_str())
            .collect()
    }

    /// JSONL: one row per definition with a proof status
    /// `{"def","uses","status","verifications"}` (a `proved` row adds the
    /// location and a `fix` with the `replacement`, as a diagnostic would),
    /// then `{"summary":true,"defs","proved","unchanged","not_found","verifications","changed"}`.
    pub fn report(&self, original: &[(String, String)]) -> String {
        let mut out = String::new();
        let mut counts = [0usize; 3];
        for r in &self.rows {
            counts[r.status as usize] += 1;
            let uses: Vec<String> = r.uses.iter().map(|u| json_str(u)).collect();
            out.push_str(&format!(
                "{{\"def\":{},\"uses\":[{}],\"status\":\"{}\",\"verifications\":{}",
                json_str(&r.def),
                uses.join(","),
                r.status.as_str(),
                r.verifications
            ));
            if let Some(e) = &r.edit {
                out.push_str(&format!(
                    ",\"file\":{},\"line\":{},\"col\":{},\"end_line\":{},\"end_col\":{},\"fix\":{{\"message\":{},\"replacement\":{}}}",
                    json_str(&e.file),
                    e.line,
                    e.col,
                    e.end_line,
                    e.end_col,
                    json_str("set #[uses(..)] to the smallest list found that proves the definition"),
                    json_str(&e.replacement)
                ));
            }
            out.push_str("}\n");
        }
        let changed: Vec<String> = self.changed(original).into_iter().map(json_str).collect();
        out.push_str(&format!(
            "{{\"summary\":true,\"defs\":{},\"proved\":{},\"unchanged\":{},\"not_found\":{},\"verifications\":{},\"changed\":[{}]}}\n",
            self.rows.len(),
            counts[UsesStatus::Proved as usize],
            counts[UsesStatus::Unchanged as usize],
            counts[UsesStatus::NotFound as usize],
            self.verifications,
            changed.join(",")
        ));
        out
    }
}

/// A definition as found in the original text.
struct Target {
    mod_path: String,
    kind: FnKind,
    /// The definition from its `fn` / `spec fn` keyword (attributes precede it).
    span: Span,
    /// Spans of its `#[uses(..)]` attributes.
    attrs: Vec<Span>,
    /// Current entries, as full paths.
    original: BTreeSet<String>,
}

fn walk(m: &Mod, prefix: &str, out: &mut BTreeMap<String, Target>) {
    let mp = format!("{prefix}{}", m.name.name);
    for s in &m.mods {
        walk(s, &format!("{mp}::"), out);
    }
    for f in &m.fns {
        let uses = f.attrs.iter().filter(|a| a.name.name == "uses");
        let original = uses
            .clone()
            .flat_map(|a| a.args.iter())
            .filter_map(|x| match x {
                crate::ast::AttrArg::Ident(i) => Some(full(&i.name, &mp)),
                crate::ast::AttrArg::Str(..) => None,
            })
            .collect();
        out.entry(format!("{mp}::{}", f.name.name))
            .or_insert(Target {
                mod_path: mp.clone(),
                kind: f.kind,
                span: f.span,
                attrs: uses.map(|a| a.span).collect(),
                original,
            });
    }
}

/// Full path of a `#[uses]` entry written in unit `mod_path` (as the prover
/// resolves it, `verify::uses_of`).
fn full(name: &str, mod_path: &str) -> String {
    if name.contains("::") {
        name.to_string()
    } else {
        format!("{mod_path}::{name}")
    }
}

/// How an entry is written in unit `mod_path`: unqualified when local.
fn written(path: &str, mod_path: &str) -> String {
    match path.strip_prefix(&format!("{mod_path}::")) {
        Some(local) if !local.contains("::") => local.to_string(),
        _ => path.to_string(),
    }
}

fn written_list(set: &BTreeSet<String>, mod_path: &str) -> Vec<String> {
    let mut v: Vec<String> = set.iter().map(|p| written(p, mod_path)).collect();
    v.sort();
    v
}

/// Text edits `(lo, hi, replacement)` that make `t`'s attribute list `set`.
fn edits(text: &str, t: &Target, set: &BTreeSet<String>) -> Vec<(usize, usize, String)> {
    let attr = (!set.is_empty())
        .then(|| format!("#[uses({})]", written_list(set, &t.mod_path).join(", ")));
    let line_start = |pos: usize| text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let blank = |s: &str| s.chars().all(|c| c == ' ' || c == '\t');
    let remove = |sp: Span| {
        let (lo, hi) = (sp.lo as usize, sp.hi as usize);
        let ls = line_start(lo);
        let le = text[hi..].find('\n').map_or(text.len(), |i| hi + i);
        if blank(&text[ls..lo]) && blank(&text[hi..le]) && le < text.len() {
            (ls, le + 1, String::new())
        } else {
            let trailing = text[hi..].len() - text[hi..].trim_start_matches([' ', '\t']).len();
            (lo, hi + trailing, String::new())
        }
    };
    let mut out = Vec::new();
    match (t.attrs.first(), attr) {
        (Some(first), Some(a)) => out.push((first.lo as usize, first.hi as usize, a)),
        (Some(first), None) => out.push(remove(*first)),
        (None, Some(a)) => {
            let lo = t.span.lo as usize;
            let indent = &text[line_start(lo)..lo];
            let ins = if blank(indent) {
                format!("{a}\n{indent}")
            } else {
                format!("{a} ")
            };
            out.push((lo, lo, ins));
        }
        (None, None) => {}
    }
    for sp in t.attrs.iter().skip(1) {
        out.push(remove(*sp));
    }
    out
}

fn apply(text: &str, mut es: Vec<(usize, usize, String)>) -> String {
    es.sort_by_key(|e| std::cmp::Reverse((e.0, e.1)));
    let mut s = text.to_string();
    for (lo, hi, r) in es {
        s.replace_range(lo..hi, &r);
    }
    s
}

/// Per-definition result of one program verification.
struct Seen {
    proved: bool,
    /// Hinted hidden predicates, as full paths.
    hints: BTreeSet<String>,
}

/// The caches every verification of one `fill_uses` call shares.
struct Caches<'a> {
    proofs: ProofCache,
    /// Created on the first run; later runs re-key it (backlog 2D-15).
    defs: Option<Session>,
    dir: Option<&'a Path>,
}

/// Checks and proves `files`; `None` when the program does not check.
/// Definitions unchanged since an earlier run (same incremental key) are
/// not encoded again; the result is the same as a full verification.
fn run(files: &[(String, String)], caches: &mut Caches) -> Option<BTreeMap<String, Seen>> {
    let mut fe = crate::frontend(files);
    if fe.diags.has_errors() {
        return None;
    }
    let session = match &mut caches.defs {
        Some(s) => {
            s.rekey(&fe.program, &fe.tables, &fe.sm);
            s
        }
        slot => slot.insert(match caches.dir {
            Some(d) => Session::new(d, &fe.program, &fe.tables, &fe.sm),
            None => Session::in_memory(&fe.program, &fe.tables, &fe.sm),
        }),
    };
    let outcome = crate::verify::verify_program_incremental(
        &fe.program,
        &fe.tables,
        &mut fe.diags,
        Some(&mut caches.proofs),
        session,
    );
    let mut defs = BTreeMap::new();
    for m in &fe.program.mods {
        walk(m, "", &mut defs);
    }
    let mut seen = BTreeMap::new();
    for s in &outcome.statuses {
        let Some(t) = defs.get(&s.path) else {
            continue;
        };
        let sp = t.span;
        let mine: Vec<&crate::diag::Diag> = fe
            .diags
            .items
            .iter()
            .filter(|d| d.span.file == sp.file && d.span.lo >= sp.lo && d.span.hi <= sp.hi)
            .collect();
        let mut hints = BTreeSet::new();
        for d in &mine {
            let Some(msg) = d.fix.as_ref().map(|f| &f.message) else {
                continue;
            };
            let Some(i) = msg.find(HINT) else { continue };
            let rest = &msg[i + HINT.len()..];
            let Some(j) = rest.find(")]") else { continue };
            for n in rest[..j].split(',') {
                let n = n.trim();
                if !n.is_empty() {
                    hints.insert(full(n, &t.mod_path));
                }
            }
        }
        seen.insert(
            s.path.clone(),
            Seen {
                proved: s.failed == 0 && mine.is_empty(),
                hints,
            },
        );
    }
    Some(seen)
}

enum State {
    Closure {
        cand: BTreeSet<String>,
        steps: usize,
    },
    Shrink {
        set: BTreeSet<String>,
        order: Vec<String>,
        idx: usize,
    },
    Done(UsesStatus, BTreeSet<String>),
}

/// Finds, for every definition that fails to prove, a minimal `#[uses]` list
/// built from its current entries and the hidden predicates its diagnostics
/// name (see the module comment). Deterministic: the result depends only on
/// the input. `Err` carries diagnostics when the program does not check.
pub fn fill_uses(files: &[(String, String)]) -> Result<UsesOutput, String> {
    fill_uses_cached(files, None)
}

/// As `fill_uses`, with the proof cache and the incremental cache kept in
/// `cache_dir` (as `bmb verify --cache`): loaded before the first
/// verification and written back after the last. The result is the same.
pub fn fill_uses_cached(
    files: &[(String, String)],
    cache_dir: Option<&Path>,
) -> Result<UsesOutput, String> {
    let fe = crate::frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut targets = BTreeMap::new();
    for m in &fe.program.mods {
        walk(m, "", &mut targets);
    }
    let mut cache = Caches {
        proofs: cache_dir.map(ProofCache::load).unwrap_or_default(),
        defs: None,
        dir: cache_dir,
    };
    let mut total = 1;
    let first = run(files, &mut cache).unwrap_or_default();

    // Only definitions that have a proof status take part.
    let mut states: BTreeMap<String, State> = BTreeMap::new();
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (path, s) in &first {
        let t = &targets[path];
        let st = if s.proved {
            State::Done(UsesStatus::Unchanged, t.original.clone())
        } else if t.kind != FnKind::Exec {
            // A spec fn's proof (termination) does not read `#[uses]`.
            State::Done(UsesStatus::NotFound, t.original.clone())
        } else {
            let cand: BTreeSet<String> = t.original.union(&s.hints).cloned().collect();
            if cand == t.original {
                State::Done(UsesStatus::NotFound, t.original.clone())
            } else {
                State::Closure { cand, steps: 1 }
            }
        };
        states.insert(path.clone(), st);
        counts.insert(path.clone(), 0);
    }

    let texts_for = |sets: &BTreeMap<&str, &BTreeSet<String>>| -> Vec<(String, String)> {
        let mut per_file: Vec<Vec<(usize, usize, String)>> = vec![Vec::new(); files.len()];
        for (path, set) in sets {
            let t = &targets[*path];
            if **set != t.original {
                let f = t.span.file as usize;
                per_file[f].extend(edits(&files[f].1, t, set));
            }
        }
        files
            .iter()
            .zip(per_file)
            .map(|((n, text), es)| (n.clone(), apply(text, es)))
            .collect()
    };

    loop {
        // Out of budget: settle what is open.
        for (path, st) in states.iter_mut() {
            if counts[path] < MAX_VERIFICATIONS {
                continue;
            }
            let orig = targets[path].original.clone();
            match st {
                State::Closure { .. } => *st = State::Done(UsesStatus::NotFound, orig),
                State::Shrink { set, .. } => *st = State::Done(UsesStatus::Proved, set.clone()),
                State::Done(..) => {}
            }
        }
        let trials: BTreeMap<String, BTreeSet<String>> = states
            .iter()
            .filter_map(|(p, st)| match st {
                State::Closure { cand, .. } => Some((p.clone(), cand.clone())),
                State::Shrink { set, order, idx } => {
                    let mut s = set.clone();
                    s.remove(&order[*idx]);
                    Some((p.clone(), s))
                }
                State::Done(..) => None,
            })
            .collect();
        if trials.is_empty() {
            break;
        }
        let sets: BTreeMap<&str, &BTreeSet<String>> =
            trials.iter().map(|(p, s)| (p.as_str(), s)).collect();
        let result = run(&texts_for(&sets), &mut cache);
        total += 1;
        for (path, trial) in trials {
            *counts.get_mut(&path).unwrap() += 1;
            let (proved, hints) = match result.as_ref().and_then(|r| r.get(&path)) {
                Some(s) => (s.proved, s.hints.clone()),
                None => (false, BTreeSet::new()),
            };
            let orig = targets[&path].original.clone();
            let st = states.get_mut(&path).unwrap();
            let next =
                match std::mem::replace(st, State::Done(UsesStatus::NotFound, BTreeSet::new())) {
                    State::Closure { cand, steps } => {
                        if proved {
                            State::Shrink {
                                order: cand.iter().cloned().collect(),
                                set: cand,
                                idx: 0,
                            }
                        } else {
                            let next: BTreeSet<String> = cand.union(&hints).cloned().collect();
                            if next == cand || steps >= MAX_CLOSURE_STEPS {
                                State::Done(UsesStatus::NotFound, orig)
                            } else {
                                State::Closure {
                                    cand: next,
                                    steps: steps + 1,
                                }
                            }
                        }
                    }
                    State::Shrink { set, order, idx } => {
                        let set = if proved { trial } else { set };
                        if idx + 1 < order.len() {
                            State::Shrink {
                                set,
                                order,
                                idx: idx + 1,
                            }
                        } else {
                            State::Done(UsesStatus::Proved, set)
                        }
                    }
                    done => done,
                };
            *st = next;
        }
    }

    // Final sets; a found list equal to the original would contradict the
    // first run, so it does not count as found.
    let mut finals: BTreeMap<String, (UsesStatus, BTreeSet<String>)> = BTreeMap::new();
    for (path, st) in states {
        let orig = targets[&path].original.clone();
        let (status, set) = match st {
            State::Done(UsesStatus::Proved, set) if set == orig => (UsesStatus::NotFound, orig),
            State::Done(s, set) => (s, set),
            _ => unreachable!("every state is settled when no trial remains"),
        };
        finals.insert(path, (status, set));
    }

    // Confirm all found lists together; revert any that does not prove.
    if finals.values().any(|(s, _)| *s == UsesStatus::Proved) {
        let sets: BTreeMap<&str, &BTreeSet<String>> =
            finals.iter().map(|(p, (_, s))| (p.as_str(), s)).collect();
        let check = run(&texts_for(&sets), &mut cache);
        total += 1;
        for (path, (status, set)) in finals.iter_mut() {
            let ok = check
                .as_ref()
                .and_then(|r| r.get(path))
                .is_some_and(|s| s.proved);
            if *status == UsesStatus::Proved && !ok {
                *status = UsesStatus::NotFound;
                *set = targets[path].original.clone();
            }
        }
    }

    if let Some(d) = cache_dir {
        if let Some(s) = &mut cache.defs {
            s.save();
        }
        if let Err(e) = cache.proofs.save(d) {
            eprintln!("bmb: cannot write proof cache in {}: {e}", d.display());
        }
    }

    let sets: BTreeMap<&str, &BTreeSet<String>> =
        finals.iter().map(|(p, (_, s))| (p.as_str(), s)).collect();
    let out_files = texts_for(&sets);
    let rows = finals
        .iter()
        .map(|(path, (status, set))| {
            let t = &targets[path];
            let edit = (*status == UsesStatus::Proved).then(|| {
                let f = t.span.file;
                let es = edits(&files[f as usize].1, t, set);
                let lo = es.iter().map(|e| e.0).min().unwrap_or(0);
                let hi = es.iter().map(|e| e.1).max().unwrap_or(0);
                let local = es
                    .into_iter()
                    .map(|(a, b, r)| (a - lo, b - lo, r))
                    .collect();
                let replacement = apply(&files[f as usize].1[lo..hi], local);
                let (line, col) = fe.sm.line_col(f, lo as u32);
                let (end_line, end_col) = fe.sm.line_col(f, hi as u32);
                UsesEdit {
                    file: files[f as usize].0.clone(),
                    line,
                    col,
                    end_line,
                    end_col,
                    replacement,
                }
            });
            UsesRow {
                def: path.clone(),
                uses: written_list(set, &t.mod_path),
                status: *status,
                verifications: counts[path],
                edit,
            }
        })
        .collect();
    Ok(UsesOutput {
        rows,
        files: out_files,
        verifications: total,
    })
}
