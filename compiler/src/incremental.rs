//! Contract-firewall incremental build (backlog 2D-10, DESIGN.md §8.1).
//!
//! The proof cache (`cache.rs`) skips the solver, but every run would still
//! encode every definition and lower all of it to IR. This cache keeps, per
//! definition, the result of proving it (status line numbers and the proved
//! obligations) and its IR, under a key built from everything that encoding
//! or lowering reads. A definition whose key is unchanged is neither encoded
//! nor lowered again.
//!
//! **Proof key.** Encoding a definition `D` (`verify::verify_impl`) reads:
//!
//! - the query header: solver options (seed, `rlimit`) and the prelude
//!   (`verify::query_header`);
//! - the declarations of the `spec fn`s its queries mention (backlog 2D-14:
//!   a query declares only those). The key takes the declaration of every
//!   `spec fn` reachable from `D`'s qualified text through executable
//!   callees' contracts and `spec fn` texts, which covers every symbol a
//!   query of `D` can contain: its assumptions and goals come from `D`, the
//!   callee contracts and the revealed `spec fn` bodies, and those name
//!   only what they name. An unrelated `spec fn` therefore changes no key;
//! - `D` itself: its source text (spans of obligations are byte offsets, so
//!   the key takes the raw text between `fn` and the closing brace and the
//!   results are stored relative to its start) and its qualified normal form
//!   with the body (ADR-0019 §2: names resolved to full paths, so a new
//!   definition that changes what a short name means changes the key);
//! - each definition `D`'s qualified text names: an executable callee through
//!   its contract, signature and effects (`fmt::caller_text`: the iface
//!   without `#[uses]`, which reveals bodies only in the callee's own
//!   queries, backlog 2D-22), a `spec fn` revealed by
//!   `#[uses]` or in `D`'s recursive component through its whole text (body
//!   hash), any other `spec fn` only through its declaration (above);
//! - `D`'s recursive component (`decreases` obligations at calls inside it);
//! - the compiler (a digest of its sources, `build.rs`) and the pinned Z3.
//!
//! These include the edges of `query impact` (map.rs): `calls` (callee
//! contract), `uses` (revealed body), `names` (spec signature, through the
//! declarations). So two runs with the same key
//! send the same query text, and the stored verdict is the one the solver
//! (and the proof cache, keyed by that text) would give. `with_audit` re-runs
//! a hit anyway and compares the script digest, status and proved set;
//! `tests/incremental.rs` runs it across edits.
//!
//! Only fully proved definitions are stored: a failure always runs again,
//! with its diagnostics. A component with a failed definition keeps its
//! runtime checks (backlog 1B-31); the IR key therefore includes the proved
//! obligations that code generation may use for `D` (relative spans), so the
//! IR follows any change of verification state elsewhere in the component.
//!
//! **IR key.** `D`'s source text and qualified text, the signature and
//! contract of each executable callee (call types and return types), the
//! usable proved obligations inside `D`, and the compiler identity.
//! An instance of a generic definition (backlog 2T-04) is stored under its
//! own key: the definition's IR key, the instance's text and the data types
//! its type arguments name. A stored fragment carries the instances it calls
//! and the instance types it uses, so a hit still lowers those.
//!
//! **File.** `DIR/defs` next to the proof cache, one entry per (kind,
//! definition): the newest. Every line carries a SHA-256 of its fields; a
//! line that does not parse or whose digest does not match is ignored, as is
//! a file with another format version or compiler identity. Ignored entries
//! only cost a rebuild. Like the proof cache, the file is trusted as much as
//! the source tree.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::ast::{FnDef, FnKind, Program, TypeKind};
use crate::source::{SourceMap, Span};
use crate::verify::{FnStatus, ObKey, Tables};

pub const FILE: &str = "defs";
const MAGIC: &str = "bmb incremental cache v1";

thread_local! {
    static ID_OVERRIDE: RefCell<Option<String>> = const { RefCell::new(None) };
    static AUDIT: Cell<bool> = const { Cell::new(false) };
}

/// Identity of this compiler build: package version and a digest of its
/// sources and pinned toolchain file (`build.rs`).
pub fn compiler_id() -> String {
    ID_OVERRIDE.with(|o| o.borrow().clone()).unwrap_or_else(|| {
        format!(
            "{}+{}",
            env!("CARGO_PKG_VERSION"),
            env!("BMB_SOURCE_DIGEST")
        )
    })
}

/// Test hook: runs `f` as if the compiler identity were `id` (on this
/// thread), to show that a compiler change invalidates every entry.
#[doc(hidden)]
pub fn with_compiler_id<T>(id: &str, f: impl FnOnce() -> T) -> T {
    let prev = ID_OVERRIDE.with(|o| o.replace(Some(id.to_string())));
    let r = f();
    ID_OVERRIDE.with(|o| *o.borrow_mut() = prev);
    r
}

/// Test hook: runs `f` with auditing on (this thread). A cache hit is then
/// recomputed anyway and compared with the stored result; differences are
/// reported in `Stats::audit_mismatches` and the recomputed result is used.
#[doc(hidden)]
pub fn with_audit<T>(f: impl FnOnce() -> T) -> T {
    let prev = AUDIT.with(|a| a.replace(true));
    let r = f();
    AUDIT.with(|a| a.set(prev));
    r
}

/// What the incremental cache did in one run. Paths are in processing order.
#[derive(Default, Clone, Debug)]
pub struct Stats {
    /// Definitions whose obligations were encoded (and solved or looked up
    /// in the proof cache).
    pub encoded: Vec<String>,
    /// Definitions whose proof result came from this cache.
    pub reused: Vec<String>,
    /// Definitions lowered to IR.
    pub lowered: Vec<String>,
    /// Definitions whose IR came from this cache.
    pub ir_reused: Vec<String>,
    /// Lines of the cache file ignored because they were damaged.
    pub damaged: usize,
    /// Audit only: definitions whose stored result differs from the recomputed one.
    pub audit_mismatches: Vec<String>,
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\n', "\\n")
}

/// A path field of a line: escaped, and a space (a trait impl function,
/// `<unit::Sq as unit::Shape>::area`) as `\s` so fields stay separated.
fn escape_path(s: &str) -> String {
    escape(s).replace(' ', "\\s")
}

fn unescape(s: &str) -> Option<String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next()? {
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                's' => out.push(' '),
                _ => return None,
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

fn line_digest(kind: &str, path: &str, key: &str, payload: &str) -> String {
    crate::sha256::hex(format!("{kind}\n{path}\n{key}\n{payload}").as_bytes())
}

/// A line of the cache file not checked yet: (key, digest, escaped payload).
type RawLine = (String, String, String);

/// The cache file: (kind, definition path) -> (key, payload).
#[derive(Default)]
struct Store {
    entries: BTreeMap<(String, String), (String, String)>,
    /// Lazy load only (`Store::load_lazy`): lines not checked yet, by slot,
    /// in file order. A slot's value is its last line whose digest holds,
    /// as in an eager load; it is checked when looked up, written or saved.
    pending: BTreeMap<(String, String), Vec<RawLine>>,
    changed: bool,
    damaged: usize,
}

/// Checks one line's digest; the payload when it holds.
fn check_line(kind: &str, path: &str, (key, sum, payload): &RawLine) -> Option<(String, String)> {
    let payload = unescape(payload)?;
    (line_digest(kind, path, key, &payload) == *sum).then(|| (key.clone(), payload))
}

impl Store {
    /// Loads and checks every line.
    fn load(dir: &Path, id: &str) -> Store {
        let mut s = Store::load_lazy(dir, id);
        for (slot, lines) in std::mem::take(&mut s.pending) {
            s.settle(slot, lines);
        }
        s
    }

    /// Loads the lines without checking their digests (backlog 3A-04): a
    /// session that proves a few definitions checks only the lines it
    /// reads, and the rest when it saves.
    fn load_lazy(dir: &Path, id: &str) -> Store {
        let mut s = Store::default();
        let Ok(text) = std::fs::read_to_string(dir.join(FILE)) else {
            return s;
        };
        let mut lines = text.lines();
        if lines.next() != Some(MAGIC) || lines.next() != Some(&format!("id {id}")) {
            // Another format or another compiler: nothing is usable.
            s.changed = true;
            return s;
        }
        for line in lines {
            let mut it = line.splitn(5, ' ');
            match (it.next(), it.next(), it.next(), it.next(), it.next()) {
                (Some(kind), Some(path), Some(key), Some(sum), Some(payload))
                    if let Some(path) = unescape(path) =>
                {
                    s.pending
                        .entry((kind.to_string(), path))
                        .or_default()
                        .push((key.to_string(), sum.to_string(), payload.to_string()))
                }
                _ => {
                    s.damaged += 1;
                    s.changed = true;
                }
            }
        }
        s
    }

    /// Checks the pending lines of `slot` into `entries`.
    fn settle(&mut self, slot: (String, String), lines: Vec<RawLine>) {
        for raw in &lines {
            match check_line(&slot.0, &slot.1, raw) {
                Some(v) => {
                    self.entries.insert(slot.clone(), v);
                }
                None => {
                    self.damaged += 1;
                    self.changed = true;
                }
            }
        }
    }

    fn get(&self, kind: &str, path: &str, key: &str) -> Option<String> {
        let slot = (kind.to_string(), path.to_string());
        let (k, p) = match self.entries.get(&slot) {
            Some(v) => v.clone(),
            None => self
                .pending
                .get(&slot)?
                .iter()
                .rev()
                .find_map(|raw| check_line(kind, path, raw))?,
        };
        (k == key).then_some(p)
    }

    fn put(&mut self, kind: &str, path: &str, key: String, payload: String) {
        let slot = (kind.to_string(), path.to_string());
        if let Some(lines) = self.pending.remove(&slot) {
            self.settle(slot.clone(), lines);
        }
        if self.entries.get(&slot) != Some(&(key.clone(), payload.clone())) {
            self.entries.insert(slot, (key, payload));
            self.changed = true;
        }
    }

    fn save(&mut self, dir: &Path, id: &str) -> std::io::Result<()> {
        for (slot, lines) in std::mem::take(&mut self.pending) {
            self.settle(slot, lines);
        }
        if !self.changed {
            return Ok(());
        }
        std::fs::create_dir_all(dir)?;
        let mut out = format!("{MAGIC}\nid {id}\n");
        for ((kind, path), (key, payload)) in &self.entries {
            out.push_str(&format!(
                "{kind} {} {key} {} {}\n",
                escape_path(path),
                line_digest(kind, path, key, payload),
                escape(payload)
            ));
        }
        let tmp = dir.join(format!("{FILE}.tmp"));
        crate::workspace::write_new(&tmp, &out)?;
        std::fs::rename(tmp, dir.join(FILE))
    }
}

/// A stored proof result, relocated to the definition's current span.
pub struct ProofHit {
    pub status: FnStatus,
    pub proved: Vec<ObKey>,
    script_sha: String,
}

/// Obligation keys inside `span`, relative to its start. `None` when one
/// lies outside (then nothing is stored).
fn relative(keys: &[ObKey], span: Span) -> Option<Vec<(&'static str, u32, u32)>> {
    let mut v = Vec::new();
    for &(code, file, lo, hi) in keys {
        if file != span.file || lo < span.lo || hi > span.hi || lo > hi {
            return None;
        }
        v.push((code, lo - span.lo, hi - span.lo));
    }
    v.sort();
    v.dedup();
    Some(v)
}

fn rel_text(rel: &[(&'static str, u32, u32)]) -> String {
    if rel.is_empty() {
        return "-".into();
    }
    rel.iter()
        .map(|(c, a, b)| format!("{c}:{a}:{b}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn code_static(code: &str) -> Option<&'static str> {
    crate::diag::CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(c, _)| *c)
}

/// Parses a relative key list and relocates it into `span`.
fn parse_rel(text: &str, span: Span) -> Option<Vec<ObKey>> {
    if text == "-" {
        return Some(Vec::new());
    }
    let len = span.hi - span.lo;
    let mut v = Vec::new();
    for item in text.split(',') {
        let mut it = item.split(':');
        let code = code_static(it.next()?)?;
        let a: u32 = it.next()?.parse().ok()?;
        let b: u32 = it.next()?.parse().ok()?;
        if it.next().is_some() || a > b || b > len {
            return None;
        }
        v.push((code, span.file, span.lo + a, span.lo + b));
    }
    Some(v)
}

/// One run's view of the cache: per-definition keys of the current program,
/// the stored entries, and counters.
pub struct Session {
    /// `None`: in memory only, never saved (`bmb lint`, `bmb uses`).
    dir: Option<PathBuf>,
    id: String,
    store: Store,
    audit: bool,
    proof_keys: BTreeMap<String, String>,
    ir_keys: BTreeMap<String, String>,
    /// Each data type's key line and the types it names, for the keys of
    /// instances (backlog 2T-04).
    adts: BTreeMap<String, (String, Vec<String>)>,
    pub stats: Stats,
}

/// What a lowered fragment needs besides its text (backlog 2T-04): the
/// instances it calls (key, generic definition, substitution of its type
/// parameters) and the instance types it uses (their LLVM declarations).
#[derive(Default, Clone)]
pub struct IrDeps {
    pub wants: Vec<(String, String, BTreeMap<String, TypeKind>)>,
    pub used: Vec<TypeKind>,
}

/// A stored fragment: its IR, its run-time checks and its dependencies.
pub struct IrHit {
    pub ir: String,
    pub checks: u32,
    pub deps: IrDeps,
}

/// The instance a fragment is: its key and its type arguments.
pub type Instance<'a> = Option<(&'a str, &'a [TypeKind])>;

/// `deps` as payload lines (fields split by `\x1f`, types as `ty_str`).
fn deps_text(deps: &IrDeps) -> String {
    let mut out = format!("{}\n", deps.wants.len());
    for (key, callee, sub) in &deps.wants {
        out.push_str(&format!("{key}\x1f{callee}"));
        for (n, t) in sub {
            out.push_str(&format!("\x1f{n}\x1f{}", crate::check::ty_str(t)));
        }
        out.push('\n');
    }
    out.push_str(&format!("{}\n", deps.used.len()));
    for t in &deps.used {
        out.push_str(&format!("{}\n", crate::check::ty_str(t)));
    }
    out
}

/// The dependencies at the start of `text` and the rest.
fn parse_deps(text: &str) -> Option<(IrDeps, &str)> {
    let mut rest = text;
    let mut line = || -> Option<&str> {
        let (l, r) = rest.split_once('\n')?;
        rest = r;
        Some(l)
    };
    let mut deps = IrDeps::default();
    let w: usize = line()?.parse().ok()?;
    for _ in 0..w {
        let l = line()?;
        let mut it = l.split('\x1f');
        let key = it.next()?.to_string();
        let callee = it.next()?.to_string();
        let mut sub = BTreeMap::new();
        while let Some(n) = it.next() {
            sub.insert(n.to_string(), crate::check::parse_ty(it.next()?)?);
        }
        deps.wants.push((key, callee, sub));
    }
    let u: usize = line()?.parse().ok()?;
    for _ in 0..u {
        deps.used.push(crate::check::parse_ty(line()?)?);
    }
    Some((deps, rest))
}

fn section(out: &mut String, tag: &str, text: &str) {
    out.push_str(&format!("{tag} {}\n{text}\n", text.len()));
}

impl Session {
    /// Loads the cache in `dir` and computes the keys of every definition of
    /// a checked program.
    pub fn new(dir: &Path, p: &Program, t: &Tables, sm: &SourceMap) -> Session {
        let id = compiler_id();
        let store = Store::load(dir, &id);
        Session::with_store(Some(dir.to_path_buf()), id, store, p, t, sm, None)
    }

    /// As `new`, computing the keys of the definitions in `only` alone
    /// (backlog 3A-04): a session that proves just those. Each key is the
    /// one `new` computes; the work follows what the definitions can read,
    /// not the size of the program.
    pub fn new_for(
        dir: &Path,
        p: &Program,
        t: &Tables,
        sm: &SourceMap,
        only: &BTreeSet<String>,
    ) -> Session {
        let id = compiler_id();
        let store = Store::load_lazy(dir, &id);
        Session::with_store(Some(dir.to_path_buf()), id, store, p, t, sm, Some(only))
    }

    /// A session that starts empty and keeps its entries in memory only
    /// (backlog 2D-15): tools that verify a program several times with
    /// small edits (`bmb uses`, `bmb lint`) reuse the unchanged definitions
    /// between their own runs through `rekey`.
    pub fn in_memory(p: &Program, t: &Tables, sm: &SourceMap) -> Session {
        Session::with_store(None, compiler_id(), Store::default(), p, t, sm, None)
    }

    /// Recomputes the keys for another version of the program, keeping the
    /// stored entries and counters. A definition whose key is unchanged is
    /// then reused as in a new session over the same cache.
    pub fn rekey(&mut self, p: &Program, t: &Tables, sm: &SourceMap) {
        let (proof_keys, ir_keys) = Session::keys(&self.id, p, t, sm, None);
        self.proof_keys = proof_keys;
        self.ir_keys = ir_keys;
    }

    fn with_store(
        dir: Option<PathBuf>,
        id: String,
        store: Store,
        p: &Program,
        t: &Tables,
        sm: &SourceMap,
        only: Option<&BTreeSet<String>>,
    ) -> Session {
        let (proof_keys, ir_keys) = Session::keys(&id, p, t, sm, only);
        let stats = Stats {
            damaged: store.damaged,
            ..Stats::default()
        };
        let adts = t
            .adts
            .iter()
            .map(|(p, a)| (p.clone(), (adt_line(a), adt_refs(a))))
            .collect();
        Session {
            dir,
            id,
            store,
            audit: AUDIT.with(|a| a.get()),
            proof_keys,
            ir_keys,
            adts,
            stats,
        }
    }

    /// Proof and IR keys of every definition, or of those in `only` (see
    /// the module comment). What a key reads of other definitions (their
    /// hashes, the names their texts mention) is computed on first use, so
    /// the keys of a few definitions cost what those definitions reach.
    fn keys(
        id: &str,
        p: &Program,
        t: &Tables,
        sm: &SourceMap,
        only: Option<&BTreeSet<String>>,
    ) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
        let mut defs = crate::check::defs_of(p, &t.fns);
        defs.sort_by(|a, b| a.0.cmp(&b.0));
        let decls_of_traits = crate::check::trait_decls(p);
        // By path; a later duplicate wins, as in `fmt::hashes` collected
        // into a map (a checked program has none). Trait method
        // declarations are read like executable callees (their signature).
        let by_path: BTreeMap<&str, (&str, &FnDef)> = defs
            .iter()
            .chain(&decls_of_traits)
            .map(|(p, u, f)| (p.as_str(), (u.as_str(), *f)))
            .collect();
        // A trait method names every impl's function too: a call with a
        // known `Self` reads that impl's contract (ADR-0046 §4).
        let impls_of = |w: &str| -> Vec<String> {
            let Some(tr) = t.fns.get(w).and_then(|s| s.trait_of.as_ref()) else {
                return Vec::new();
            };
            let m = w.rsplit("::").next().unwrap_or_default();
            t.trait_impls
                .iter()
                .filter(|i| &i.trait_path == tr)
                .map(|i| i.fn_path(m))
                .filter(|q| by_path.contains_key(q.as_str()))
                .collect()
        };
        // (body hash, caller hash): what another definition's keys take of
        // a spec fn (its whole text) and of an executable function.
        let hash_memo: RefCell<BTreeMap<&str, (String, String)>> = RefCell::default();
        let hash_of = |r: &str| -> (String, String) {
            let (unit, f) = by_path[r];
            hash_memo
                .borrow_mut()
                .entry(by_path.get_key_value(r).unwrap().0)
                .or_insert_with(|| {
                    let body = crate::sha256::hex(
                        crate::fmt::qualified_text(f, unit, &t.fns, true).as_bytes(),
                    );
                    let caller =
                        crate::sha256::hex(crate::fmt::caller_text(f, unit, &t.fns).as_bytes());
                    (body, caller)
                })
                .clone()
        };
        let kinds: BTreeMap<&str, FnKind> = defs
            .iter()
            .chain(&decls_of_traits)
            .map(|(p, _, f)| (p.as_str(), f.kind))
            .collect();
        // Hashed once: it is the same for every definition.
        let header = crate::sha256::hex(crate::verify::query_header().as_bytes());
        let decls = crate::verify::spec_decl_map(p, t);
        // Definitions a text names (full paths).
        let names = |text: &str| -> BTreeSet<String> {
            text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
                .filter(|w| w.contains("::") && kinds.contains_key(w))
                .flat_map(|w| std::iter::once(w.to_string()).chain(impls_of(w)))
                .collect()
        };
        // What another definition's query can read of each definition: a
        // spec fn's whole text, an executable function's surface without
        // `#[uses]` (`fmt::caller_text`).
        let reads_memo: RefCell<BTreeMap<String, BTreeSet<String>>> = RefCell::default();
        let reads = |r: &str| -> BTreeSet<String> {
            let Some(&(unit, f)) = by_path.get(r) else {
                return BTreeSet::new();
            };
            reads_memo
                .borrow_mut()
                .entry(r.to_string())
                .or_insert_with(|| {
                    names(&if f.kind == FnKind::Spec {
                        crate::fmt::qualified_text(f, unit, &t.fns, true)
                    } else {
                        crate::fmt::caller_text(f, unit, &t.fns)
                    })
                })
                .clone()
        };
        let z3 = crate::toolchain::z3_version();
        let mut proof_keys = BTreeMap::new();
        let mut ir_keys = BTreeMap::new();
        for (path, unit, f) in &defs {
            if only.is_some_and(|o| !o.contains(path)) {
                continue;
            }
            let qual = crate::fmt::qualified_text(f, unit, &t.fns, true);
            let src = &sm.file(f.span.file).text;
            let slice = src
                .get(f.span.lo as usize..f.span.hi as usize)
                .unwrap_or_default();
            // Every definition the qualified text names (full paths).
            // A trait impl's function is proved against the trait method's
            // contract too (its refinement obligations).
            let refines = crate::verify::refinement_of(t, path).map(|(decl, _)| decl);
            let refs: BTreeSet<String> = qual
                .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
                .filter(|w| w.contains("::") && *w != path && kinds.contains_key(w))
                .map(str::to_string)
                .chain(refines.clone())
                .flat_map(|w| {
                    let impls = impls_of(&w);
                    std::iter::once(w).chain(impls)
                })
                .filter(|w| w != path)
                .collect();
            // `#[uses(Trait::m)]` reveals every impl's definition of it.
            let uses: BTreeSet<String> = crate::verify::uses_of(f, unit, &t.fns)
                .into_iter()
                .flat_map(|u| {
                    let impls = impls_of(&u);
                    std::iter::once(u).chain(impls)
                })
                .collect();
            let comp: Vec<&str> = match t.recursive.get(path) {
                Some(c) => defs
                    .iter()
                    .filter(|(q, _, _)| t.recursive.get(q) == Some(c))
                    .map(|(q, _, _)| q.as_str())
                    .collect(),
                None => Vec::new(),
            };
            let mut own = String::new();
            section(&mut own, "id", id);
            section(&mut own, "path", path);
            section(&mut own, "unit", unit);
            section(&mut own, "source", slice);
            section(&mut own, "qualified", &qual);
            let mut pk = String::from("proof\n");
            section(&mut pk, "z3", z3);
            section(&mut pk, "header", &header);
            pk.push_str(&own);
            section(&mut pk, "component", &comp.join(" "));
            let mut ik = String::from("ir\n");
            ik.push_str(&own);
            for r in &refs {
                let r = r.as_str();
                let (body, caller) = &hash_of(r);
                match kinds[r] {
                    FnKind::Exec => {
                        let line = format!("{r} {caller}");
                        section(&mut pk, "callee", &line);
                        section(&mut ik, "callee", &line);
                    }
                    FnKind::Spec if uses.contains(r) || comp.contains(&r) => {
                        section(&mut pk, "spec", &format!("{r} {body}"))
                    }
                    // Named only: the query reads its declaration (below).
                    FnKind::Spec => section(&mut pk, "names", r),
                }
            }
            // Declarations the queries can contain (backlog 2D-14): a query
            // declares only the spec fns it mentions, and every symbol it
            // mentions comes from `D`'s text, a callee's contract or a spec
            // fn's text (a revealed body, a measure in the component), so
            // the spec fns reachable along those texts cover it.
            let mut reach: BTreeSet<String> = names(&qual);
            reach.insert(path.clone());
            reach.extend(refines);
            let mut todo: Vec<String> = reach.iter().cloned().collect();
            while let Some(r) = todo.pop() {
                for n in reads(&r) {
                    if reach.insert(n.clone()) {
                        todo.push(n);
                    }
                }
            }
            for r in &reach {
                if let Some(decl) = decls.get(r) {
                    section(&mut pk, "decl", decl);
                }
            }
            // Data types the queries read (fields, payloads: ADR-0027):
            // named in `D`'s text or a text it reaches, closed over their
            // field and payload types.
            let mut texts = vec![(qual.clone(), unit.as_str())];
            for r in &reach {
                if let Some(&(u, g)) = by_path.get(r.as_str()) {
                    texts.push((
                        if g.kind == FnKind::Spec {
                            crate::fmt::qualified_text(g, u, &t.fns, true)
                        } else {
                            crate::fmt::caller_text(g, u, &t.fns)
                        },
                        u,
                    ));
                }
            }
            // A type is written by full path or, in its own unit, by name.
            let mut tys: BTreeSet<String> = BTreeSet::new();
            for (x, u) in &texts {
                for w in x.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':')) {
                    let local = format!("{u}::{w}");
                    if t.adts.contains_key(&local) {
                        tys.insert(local);
                    } else if t.adts.contains_key(w) {
                        tys.insert(w.to_string());
                    }
                }
            }
            let mut todo: Vec<String> = tys.iter().cloned().collect();
            while let Some(ty) = todo.pop() {
                for n in adt_refs(&t.adts[&ty]) {
                    if t.adts.contains_key(&n) && tys.insert(n.clone()) {
                        todo.push(n);
                    }
                }
            }
            for ty in &tys {
                let line = adt_line(&t.adts[ty]);
                section(&mut pk, "type", &line);
                section(&mut ik, "type", &line);
            }
            proof_keys.insert(path.clone(), crate::sha256::hex(pk.as_bytes()));
            ir_keys.insert(path.clone(), crate::sha256::hex(ik.as_bytes()));
        }
        (proof_keys, ir_keys)
    }

    pub fn audit(&self) -> bool {
        self.audit
    }

    /// The stored proof result of `path`, relocated into `span`.
    pub fn proof_lookup(&self, path: &str, span: Span) -> Option<ProofHit> {
        let key = self.proof_keys.get(path)?;
        let payload = self.store.get("P", path, key)?;
        let mut it = payload.split(' ');
        let obligations: usize = it.next()?.parse().ok()?;
        let rlimit_total: u64 = it.next()?.parse().ok()?;
        let rlimit_max: u64 = it.next()?.parse().ok()?;
        let script_sha = it.next()?.to_string();
        let proved = parse_rel(it.next()?, span)?;
        if it.next().is_some() {
            return None;
        }
        Some(ProofHit {
            status: FnStatus {
                path: path.to_string(),
                obligations,
                failed: 0,
                rlimit_total,
                rlimit_max,
            },
            proved,
            script_sha,
        })
    }

    /// Stores the result of a fully proved definition.
    pub fn proof_record(&mut self, status: &FnStatus, span: Span, proved: &[ObKey], script: &str) {
        if status.failed != 0 {
            return;
        }
        let (Some(key), Some(rel)) = (self.proof_keys.get(&status.path), relative(proved, span))
        else {
            return;
        };
        let payload = format!(
            "{} {} {} {} {}",
            status.obligations,
            status.rlimit_total,
            status.rlimit_max,
            crate::sha256::hex(script.as_bytes()),
            rel_text(&rel)
        );
        self.store.put("P", &status.path, key.clone(), payload);
    }

    /// Audit: compares a hit with the recomputed result.
    pub fn proof_audit(
        &mut self,
        hit: &ProofHit,
        status: &FnStatus,
        proved: &[ObKey],
        script: &str,
    ) {
        let mut a: Vec<ObKey> = hit.proved.clone();
        let mut b: Vec<ObKey> = proved.to_vec();
        a.sort();
        a.dedup();
        b.sort();
        b.dedup();
        let same = hit.script_sha == crate::sha256::hex(script.as_bytes())
            && status.failed == 0
            && hit.status.obligations == status.obligations
            && hit.status.rlimit_total == status.rlimit_total
            && hit.status.rlimit_max == status.rlimit_max
            && a == b;
        if !same {
            self.stats.audit_mismatches.push(status.path.clone());
        }
    }

    /// The IR key of definition `path` (or of its instance `inst`) lowered
    /// with the proved obligations `rel`. An instance's key adds its text
    /// and the data types its type arguments name (closed over their
    /// fields): its definition's key covers the rest, since the callees it
    /// reaches through bounds are named there (`impls_of`).
    fn ir_key(
        &self,
        path: &str,
        inst: Instance,
        rel: &[(&'static str, u32, u32)],
    ) -> Option<String> {
        let base = self.ir_keys.get(path)?;
        let mut text = format!("{base}\n{}", rel_text(rel));
        if let Some((key, args)) = inst {
            text.push_str(&format!("\ninstance {key}"));
            let mut tys: BTreeSet<String> = BTreeSet::new();
            let mut todo: Vec<String> = Vec::new();
            fn named(t: &TypeKind, out: &mut Vec<String>) {
                match t {
                    TypeKind::Named { path, args } => {
                        out.push(path.join("::"));
                        args.iter().for_each(|a| named(&a.kind, out));
                    }
                    TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) => {
                        named(&e.kind, out)
                    }
                    TypeKind::Ref { inner, .. } => named(&inner.kind, out),
                    _ => {}
                }
            }
            args.iter().for_each(|a| named(a, &mut todo));
            while let Some(ty) = todo.pop() {
                if let Some((_, refs)) = self.adts.get(&ty)
                    && tys.insert(ty)
                {
                    todo.extend(refs.iter().cloned());
                }
            }
            for ty in &tys {
                text.push_str(&format!("\ntype {}", self.adts[ty].0));
            }
        }
        Some(crate::sha256::hex(text.as_bytes()))
    }

    /// The stored IR of `path` (or of its instance `inst`), lowered with
    /// the usable proved obligations `usable` (those inside `span`).
    pub fn ir_lookup(
        &self,
        path: &str,
        inst: Instance,
        span: Span,
        usable: &[ObKey],
    ) -> Option<IrHit> {
        let key = self.ir_key(path, inst, &relative(usable, span)?)?;
        let slot = inst.map_or(path, |(k, _)| k);
        let payload = self.store.get("I", slot, &key)?;
        let (checks, rest) = payload.split_once('\n')?;
        let (deps, ir) = parse_deps(rest)?;
        Some(IrHit {
            ir: ir.to_string(),
            checks: checks.parse().ok()?,
            deps,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ir_record(
        &mut self,
        path: &str,
        inst: Instance,
        span: Span,
        usable: &[ObKey],
        ir: &str,
        checks: u32,
        deps: &IrDeps,
    ) {
        let Some(key) = relative(usable, span).and_then(|rel| self.ir_key(path, inst, &rel)) else {
            return;
        };
        let slot = inst.map_or(path, |(k, _)| k);
        self.store
            .put("I", slot, key, format!("{checks}\n{}{ir}", deps_text(deps)));
    }

    /// Writes the cache back when something changed.
    pub fn save(&mut self) {
        let Some(dir) = &self.dir else { return };
        if let Err(e) = self.store.save(dir, &self.id) {
            eprintln!(
                "bmb: cannot write incremental cache in {}: {e}",
                dir.display()
            );
        }
    }
}

/// The data types a type declaration's fields and payloads name.
fn adt_refs(a: &crate::check::Adt) -> Vec<String> {
    fn walk(t: &crate::ast::TypeKind, out: &mut Vec<String>) {
        use crate::ast::TypeKind;
        match t {
            TypeKind::Named { path, args } => {
                out.push(path.join("::"));
                args.iter().for_each(|a| walk(&a.kind, out));
            }
            TypeKind::Array(e, _) | TypeKind::Slice(e) => walk(&e.kind, out),
            TypeKind::Ref { inner, .. } => walk(&inner.kind, out),
            _ => {}
        }
    }
    let mut out = Vec::new();
    match &a.kind {
        crate::check::AdtKind::Struct(fs) => fs.iter().for_each(|(_, t)| walk(t, &mut out)),
        crate::check::AdtKind::Enum(vs) => vs
            .iter()
            .flat_map(|(_, ps)| ps.iter())
            .for_each(|t| walk(t, &mut out)),
    }
    out
}

/// A type declaration as a key line: its path, fields or variants, and
/// their resolved types.
fn adt_line(a: &crate::check::Adt) -> String {
    use crate::check::ty_str;
    let body: Vec<String> = match &a.kind {
        crate::check::AdtKind::Struct(fs) => fs
            .iter()
            .map(|(n, t)| format!("{n}: {}", ty_str(t)))
            .collect(),
        crate::check::AdtKind::Enum(vs) => vs
            .iter()
            .map(|(n, ps)| {
                let ps: Vec<String> = ps.iter().map(ty_str).collect();
                format!("{n}({})", ps.join(", "))
            })
            .collect(),
    };
    format!("{} {{ {} }}", a.path, body.join(", "))
}
