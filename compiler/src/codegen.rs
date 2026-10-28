//! LLVM text IR generation (backlog 1C-01, 1C-03, 1C-04).
//!
//! Every implicit proof obligation (overflow, division, bounds) has a runtime
//! check here unless the proof gate proved that very obligation. Proved
//! arithmetic carries `nsw`/`nuw`, proved indexing uses `inbounds` without a
//! check, and every `&mut` parameter is `noalias` (ADR-0015). Structs and
//! enums (ADR-0026) live in memory as named LLVM types and are copied where
//! Rust copies them (backlog 2B-06). A `Vec` (ADR-0031 §4) is a header
//! `{ptr, len, cap}` in memory that moves like a data value (copying the
//! header is the move); its buffer is freed where `moves.rs` says, and a
//! temporary `Vec` read but not moved at the end of its statement (backlog
//! 2G-04). Unproved
//! obligations keep a check that traps; `bmb build` refuses such programs
//! anyway, so this matters only for `bmb emit-llvm` on partial proofs.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::ast::*;
use crate::check::ty_str;
use crate::source::Span;
use crate::verify::{ObKey, Tables, enum_variants, struct_fields, value_ty};

#[derive(Clone)]
enum CVal {
    Imm(String),
    Arr {
        ptr: String,
        len: String,
    },
    Unit,
    /// A struct or enum value in memory: its address and type. Whoever keeps
    /// it copies it, where Rust copies (arguments, fields, payloads, `let`,
    /// `=`), so a later write to the source does not reach the copy.
    Ptr {
        ptr: String,
        ty: TypeKind,
    },
    /// The value of an expression that does not finish (`return`): the
    /// code after it is unreachable, so it stands for any value.
    Never,
}

impl CVal {
    fn imm(&self) -> String {
        match self {
            CVal::Imm(s) => s.clone(),
            CVal::Unit => "0".into(),
            CVal::Never => "undef".into(),
            CVal::Arr { .. } | CVal::Ptr { .. } => panic!("aggregate used as scalar"),
        }
    }
}

/// Where a scalar reference argument's temporary is copied back after the
/// call (backlog 2T-08): a place in memory form, or an element of a scalar
/// slice (its element kind and address).
enum Back {
    Mem(String),
    Elem(&'static str, String),
}

#[derive(Clone)]
enum Slot {
    Scalar {
        ptr: String,
        ty: &'static str,
    },
    Arr {
        ptr: String,
        len: String,
        elem: &'static str,
        /// The element type (a struct or enum for slices of them).
        ety: TypeKind,
    },
    /// A struct or enum local or parameter.
    Data {
        ptr: String,
        ty: TypeKind,
    },
}

/// Whether `t` is a slice reference.
fn is_array(t: &TypeKind) -> bool {
    matches!(t, TypeKind::Ref { inner, .. } if matches!(inner.kind, TypeKind::Slice(_)))
}

/// Whether `t` is a struct or enum (through a reference).
fn is_data(t: &TypeKind) -> bool {
    matches!(value_ty(t), TypeKind::Named { .. })
}

/// Whether an array element of type `t` lives in memory as an aggregate:
/// a struct, an enum or a fixed-size array (backlog 2V-04b).
fn is_mem_elem(t: &TypeKind) -> bool {
    is_mem(t) || matches!(value_ty(t), TypeKind::Array(..))
}

/// Whether `t` is a `Vec` (through a reference).
fn is_vec(t: &TypeKind) -> bool {
    matches!(value_ty(t), TypeKind::Vec(_))
}

/// Whether `t` is a `Box` (through a reference).
fn is_box(t: &TypeKind) -> bool {
    matches!(value_ty(t), TypeKind::Box(_))
}

/// Whether a value of type `t` lives in memory and is passed by address:
/// a struct, an enum, a `Vec` header or a `Box` (its heap address).
fn is_mem(t: &TypeKind) -> bool {
    is_data(t) || is_vec(t) || is_box(t)
}

/// The drop glue of type `t` (ADR-0047 §4).
fn glue(t: &TypeKind) -> String {
    format!("@\"bmb_drop.{}\"", ty_str(t))
}

/// The element type of a `Vec`.
fn vec_elem(t: &TypeKind) -> TypeKind {
    match value_ty(t) {
        TypeKind::Vec(e) => e.kind.clone(),
        other => unreachable!("not a Vec: {other:?}"),
    }
}

/// The maximum length of a `Vec<T>` (ADR-0017 §3, as the prover's).
fn vec_len_max(e: &TypeKind) -> &'static str {
    if matches!(e, TypeKind::Bool | TypeKind::Named { .. }) {
        "9223372036854775807"
    } else {
        "1152921504606846975"
    }
}

/// The run-time support of `Vec` (ADR-0031 §4): allocation through libc,
/// growth to twice the capacity (at least 4), and `abort` when a size does
/// not fit in `isize` or the allocator fails.
const VEC_RUNTIME: &str = "declare ptr @realloc(ptr, i64)
define internal i64 @bmb_vec_bytes(i64 %n, i64 %size) {
entry:
  %p = call {i64, i1} @llvm.umul.with.overflow.i64(i64 %n, i64 %size)
  %b = extractvalue {i64, i1} %p, 0
  %o = extractvalue {i64, i1} %p, 1
  %big = icmp ugt i64 %b, 9223372036854775807
  %bad = or i1 %o, %big
  br i1 %bad, label %fail, label %ok
fail:
  call void @abort()
  unreachable
ok:
  ret i64 %b
}
define internal void @bmb_vec_alloc(ptr %h, i64 %n, i64 %size) {
entry:
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
  store i64 0, ptr %lp
  %z = icmp eq i64 %n, 0
  br i1 %z, label %empty, label %some
empty:
  store ptr null, ptr %h
  store i64 0, ptr %cp
  ret void
some:
  %b = call i64 @bmb_vec_bytes(i64 %n, i64 %size)
  %p = call ptr @malloc(i64 %b)
  %null = icmp eq ptr %p, null
  br i1 %null, label %fail, label %ok
fail:
  call void @abort()
  unreachable
ok:
  store ptr %p, ptr %h
  store i64 %n, ptr %cp
  ret void
}
define internal void @bmb_vec_grow(ptr %h, i64 %size) {
entry:
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
  %cap = load i64, ptr %cp
  %z = icmp eq i64 %cap, 0
  %d = shl i64 %cap, 1
  %new = select i1 %z, i64 4, i64 %d
  %b = call i64 @bmb_vec_bytes(i64 %new, i64 %size)
  %buf = load ptr, ptr %h
  %p = call ptr @realloc(ptr %buf, i64 %b)
  %null = icmp eq ptr %p, null
  br i1 %null, label %fail, label %ok
fail:
  call void @abort()
  unreachable
ok:
  store ptr %p, ptr %h
  store i64 %new, ptr %cp
  ret void
}
define internal void @bmb_vec_clone(ptr %dst, ptr %src, i64 %size) {
entry:
  %slp = getelementptr inbounds %bmb.vec, ptr %src, i32 0, i32 1
  %len = load i64, ptr %slp
  call void @bmb_vec_alloc(ptr %dst, i64 %len, i64 %size)
  %db = load ptr, ptr %dst
  %sb = load ptr, ptr %src
  %bytes = mul nuw i64 %len, %size
  call void @llvm.memcpy.p0.p0.i64(ptr %db, ptr %sb, i64 %bytes, i1 false)
  %dlp = getelementptr inbounds %bmb.vec, ptr %dst, i32 0, i32 1
  store i64 %len, ptr %dlp
  ret void
}
";

/// The allocator, for `Vec` and `Box` (ADR-0047 §4): `bmb_box_alloc`
/// aborts when `malloc` fails (a zero size never occurs: `Box<()>` is not
/// a type).
const HEAP_RUNTIME: &str = "declare ptr @malloc(i64)
declare void @free(ptr)
declare void @abort() noreturn nounwind
define internal ptr @bmb_box_alloc(i64 %size) {
entry:
  %p = call ptr @malloc(i64 %size)
  %null = icmp eq ptr %p, null
  br i1 %null, label %fail, label %ok
fail:
  call void @abort()
  unreachable
ok:
  ret ptr %p
}
";

/// The LLVM type of a struct or enum: `%"geo::Point"` (ADR-0026 §6 paths).
fn data_ty(t: &TypeKind) -> String {
    format!("%\"{}\"", ty_str(value_ty(t)))
}

/// The view of variant `v` of enum `t`: its tag and payload.
fn view_ty(t: &TypeKind, v: &str) -> String {
    format!("%\"{}::{v}\"", ty_str(value_ty(t)))
}

/// The in-memory LLVM type of a value of type `t` (`bool` is a byte).
fn mem_ty(t: &TypeKind) -> String {
    match value_ty(t) {
        TypeKind::Vec(_) => "%bmb.vec".into(),
        TypeKind::Box(_) => "ptr".into(),
        TypeKind::Bool | TypeKind::Int(IntTy::U8) => "i8".into(),
        TypeKind::Named { .. } => data_ty(t),
        TypeKind::Array(e, n) => format!("[{n} x {}]", mem_ty(&e.kind)),
        _ => "i64".into(),
    }
}

/// The size of `t` in memory as a constant expression (LLVM's own layout).
fn size_of(t: &TypeKind) -> String {
    format!(
        "ptrtoint (ptr getelementptr ({}, ptr null, i32 1) to i64)",
        mem_ty(t)
    )
}

/// Size and alignment of `t` in memory under the C layout LLVM uses
/// (x86-64): used only to size an enum's payload area.
fn layout(t: &TypeKind, tb: &Tables) -> (u64, u64) {
    fn fields<'a>(fs: impl Iterator<Item = &'a TypeKind>, tb: &Tables) -> (u64, u64) {
        let (mut size, mut align) = (0u64, 1u64);
        for f in fs {
            let (s, a) = layout(f, tb);
            size = size.div_ceil(a) * a + s;
            align = align.max(a);
        }
        (size.div_ceil(align) * align, align)
    }
    match value_ty(t) {
        TypeKind::Bool => (1, 1),
        TypeKind::Unit => (0, 1),
        // `{ptr, len, cap}`.
        TypeKind::Vec(_) => (24, 8),
        TypeKind::Array(e, n) => {
            let (s, a) = layout(&e.kind, tb);
            (s * n, a)
        }
        TypeKind::Named { .. } => {
            if let Some(fs) = struct_fields(t, tb) {
                return fields(fs.iter().map(|(_, ft)| ft), tb);
            }
            let vs = enum_variants(t, tb).unwrap_or_default();
            let tag = TypeKind::Int(IntTy::I64);
            let size = vs
                .iter()
                .map(|(_, ps)| fields(std::iter::once(&tag).chain(ps.iter()), tb).0)
                .max()
                .unwrap_or(8)
                .max(8);
            (size.div_ceil(8) * 8, 8)
        }
        _ => (8, 8),
    }
}

/// LLVM type definitions of every struct and enum (instance) the program
/// uses: a struct's fields in declaration order; an enum as its tag and a
/// payload area, with one view type per variant that has a payload.
fn type_defs(tb: &Tables, extra: &[TypeKind]) -> String {
    fn visit(t: &TypeKind, tb: &Tables, seen: &mut BTreeSet<String>, out: &mut String) {
        // A generic definition's own types: only instances are lowered.
        if crate::check::has_param(t) {
            return;
        }
        match t {
            TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) | TypeKind::Box(e) => {
                return visit(&e.kind, tb, seen, out);
            }
            TypeKind::Ref { inner, .. } => return visit(&inner.kind, tb, seen, out),
            TypeKind::Named { .. } => {}
            _ => return,
        }
        if !seen.insert(ty_str(t)) {
            return;
        }
        if let Some(fs) = struct_fields(t, tb) {
            let parts: Vec<String> = fs.iter().map(|(_, ft)| mem_ty(ft)).collect();
            let _ = writeln!(out, "{} = type {{ {} }}", data_ty(t), parts.join(", "));
            for (_, ft) in &fs {
                visit(ft, tb, seen, out);
            }
            return;
        }
        let Some(vs) = enum_variants(t, tb) else {
            return;
        };
        let words = (layout(t, tb).0 - 8) / 8;
        let _ = writeln!(out, "{} = type {{ i64, [{words} x i64] }}", data_ty(t));
        for (v, ps) in &vs {
            if !ps.is_empty() {
                let parts: Vec<String> = ps.iter().map(mem_ty).collect();
                let _ = writeln!(
                    out,
                    "{} = type {{ i64, {} }}",
                    view_ty(t, v),
                    parts.join(", ")
                );
            }
            for p in ps {
                visit(p, tb, seen, out);
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut out = String::new();
    for t in all_types(tb, extra) {
        visit(t, tb, &mut seen, &mut out);
    }
    out
}

/// The types of the program's values: its expressions, locals and
/// signatures, and the instance types `extra`.
fn all_types<'a>(tb: &'a Tables, extra: &'a [TypeKind]) -> Vec<&'a TypeKind> {
    let mut all: Vec<&TypeKind> = tb.types.values().collect();
    all.extend(tb.locals.values().flat_map(|m| m.values()));
    for sig in tb.fns.values() {
        all.extend(sig.params.iter());
        all.push(&sig.ret);
    }
    all.extend(extra);
    all
}

/// Whether a value of type `t` holds a `Vec` anywhere (through references,
/// fields, payloads, `Box`es, arrays and type arguments).
fn holds_vec(t: &TypeKind, tb: &Tables) -> bool {
    fn go(t: &TypeKind, tb: &Tables, seen: &mut BTreeSet<String>) -> bool {
        match t {
            TypeKind::Vec(_) => true,
            TypeKind::Ref { inner: e, .. }
            | TypeKind::Box(e)
            | TypeKind::Array(e, _)
            | TypeKind::Slice(e) => go(&e.kind, tb, seen),
            TypeKind::Named { args, .. } => {
                if args.iter().any(|a| go(&a.kind, tb, seen)) {
                    return true;
                }
                if !seen.insert(ty_str(t)) {
                    return false;
                }
                if let Some(fs) = struct_fields(t, tb) {
                    return fs.iter().any(|(_, ft)| go(ft, tb, seen));
                }
                enum_variants(t, tb)
                    .is_some_and(|vs| vs.iter().any(|(_, ps)| ps.iter().any(|p| go(p, tb, seen))))
            }
            _ => false,
        }
    }
    go(t, tb, &mut BTreeSet::new())
}

/// The drop glue of every type that owns heap memory (ADR-0047 §4):
/// `@"bmb_drop.<type>"(ptr %p)` drops the value at `%p`. A `Vec` frees
/// its buffer (its elements own nothing yet, backlog 2O-03), a `Box`
/// drops its value and frees it, a struct drops its fields in
/// declaration order, an enum the payloads of its variant in order.
fn drop_glue(tb: &Tables, extra: &[TypeKind]) -> String {
    fn visit(t: &TypeKind, tb: &Tables, seen: &mut BTreeSet<String>, out: &mut String) {
        if crate::check::has_param(t) {
            return;
        }
        match t {
            TypeKind::Slice(e) => return visit(&e.kind, tb, seen, out),
            TypeKind::Ref { inner, .. } => return visit(&inner.kind, tb, seen, out),
            TypeKind::Vec(_) | TypeKind::Box(_) | TypeKind::Named { .. } | TypeKind::Array(..) => {}
            _ => return,
        }
        if !crate::check::is_move(t, &tb.adts) || !seen.insert(ty_str(t)) {
            return;
        }
        let owns = |x: &TypeKind| crate::check::is_move(x, &tb.adts);
        let mut body = String::new();
        let mut inner: Vec<TypeKind> = Vec::new();
        match t {
            // Each element, in order (backlog 2O-03c).
            TypeKind::Array(e, n) => {
                let _ = write!(
                    body,
                    "  br label %head\nhead:\n  %i = phi i64 [ 0, %entry ], [ %j, %body ]\n  %more = icmp ult i64 %i, {n}\n  br i1 %more, label %body, label %done\nbody:\n  %ep = getelementptr inbounds {}, ptr %p, i64 0, i64 %i\n  call void {}(ptr %ep)\n  %j = add nuw i64 %i, 1\n  br label %head\ndone:\n",
                    mem_ty(t),
                    glue(&e.kind)
                );
                inner.push(e.kind.clone());
            }
            TypeKind::Vec(e) => {
                // Each element, in order, then the buffer (ADR-0047 §4).
                if owns(&e.kind) {
                    let _ = write!(
                        body,
                        "  %b0 = load ptr, ptr %p\n  %lp = getelementptr inbounds %bmb.vec, ptr %p, i32 0, i32 1\n  %n = load i64, ptr %lp\n  br label %head\nhead:\n  %i = phi i64 [ 0, %entry ], [ %j, %body ]\n  %more = icmp ult i64 %i, %n\n  br i1 %more, label %body, label %done\nbody:\n  %ep = getelementptr inbounds {}, ptr %b0, i64 %i\n  call void {}(ptr %ep)\n  %j = add nuw i64 %i, 1\n  br label %head\ndone:\n",
                        mem_ty(&e.kind),
                        glue(&e.kind)
                    );
                    inner.push(e.kind.clone());
                }
                body.push_str("  %b = load ptr, ptr %p\n  call void @free(ptr %b)\n");
            }
            TypeKind::Box(e) => {
                body.push_str("  %h = load ptr, ptr %p\n");
                if owns(&e.kind) {
                    let _ = writeln!(body, "  call void {}(ptr %h)", glue(&e.kind));
                    inner.push(e.kind.clone());
                }
                body.push_str("  call void @free(ptr %h)\n");
            }
            _ => {
                if let Some(fs) = struct_fields(t, tb) {
                    for (k, (_, ft)) in fs.iter().enumerate() {
                        if owns(ft) {
                            let _ = writeln!(
                                body,
                                "  %f{k} = getelementptr inbounds {}, ptr %p, i32 0, i32 {k}\n  call void {}(ptr %f{k})",
                                data_ty(t),
                                glue(ft)
                            );
                            inner.push(ft.clone());
                        }
                    }
                } else {
                    let vs = enum_variants(t, tb).unwrap_or_default();
                    let _ = writeln!(body, "  %tag = load i64, ptr %p");
                    let mut cases = String::new();
                    let mut arms = String::new();
                    for (k, (v, ps)) in vs.iter().enumerate() {
                        if !ps.iter().any(owns) {
                            continue;
                        }
                        let _ = write!(cases, " i64 {k}, label %v{k}");
                        let _ = writeln!(arms, "v{k}:");
                        for (j, pt) in ps.iter().enumerate() {
                            if owns(pt) {
                                let _ = writeln!(
                                    arms,
                                    "  %v{k}.{j} = getelementptr inbounds {}, ptr %p, i32 0, i32 {}\n  call void {}(ptr %v{k}.{j})",
                                    view_ty(t, v),
                                    j + 1,
                                    glue(pt)
                                );
                                inner.push(pt.clone());
                            }
                        }
                        let _ = writeln!(arms, "  br label %done");
                    }
                    let _ = write!(
                        body,
                        "  switch i64 %tag, label %done [{cases} ]\n{arms}done:\n"
                    );
                }
            }
        }
        let _ = writeln!(
            out,
            "define internal void {}(ptr %p) {{\nentry:\n{body}  ret void\n}}",
            glue(t)
        );
        for x in &inner {
            visit(x, tb, seen, out);
        }
    }
    let mut seen = BTreeSet::new();
    let mut out = String::new();
    for t in all_types(tb, extra) {
        visit(t, tb, &mut seen, &mut out);
    }
    out
}

/// Whether `t` is an integer type or `bool`.
fn is_scalar_ty(t: &TypeKind) -> bool {
    matches!(
        t,
        TypeKind::Int(IntTy::I64)
            | TypeKind::Int(IntTy::U64)
            | TypeKind::Int(IntTy::U8)
            | TypeKind::Int(IntTy::Usize)
            | TypeKind::Bool
    )
}

fn llty(t: &TypeKind) -> &'static str {
    match t {
        TypeKind::Bool => "i1",
        TypeKind::Unit => "void",
        _ => "i64",
    }
}

fn elem_of(t: &TypeKind) -> TypeKind {
    match t {
        TypeKind::Array(e, _) => e.kind.clone(),
        TypeKind::Ref { inner, .. } => match &inner.kind {
            TypeKind::Slice(e) => e.kind.clone(),
            o => o.clone(),
        },
        o => o.clone(),
    }
}

/// The element kind of an array, slice or `Vec` of `t`: its register type,
/// or `"u8"` for a byte (an `i64` in registers, a byte in memory, ADR-0044).
fn elem_kind(t: &TypeKind) -> &'static str {
    if *t == TypeKind::Int(IntTy::U8) {
        "u8"
    } else {
        llty(t)
    }
}

/// In-memory type of an array element: `bool` and `u8` are bytes, as in
/// Rust and C, so LLVM recognizes fills and vectorizes loads (backlog 1E-09).
fn mem(elem: &str) -> &str {
    if elem == "i1" || elem == "u8" {
        "i8"
    } else {
        elem
    }
}

/// ABI alignment of an array element in memory (`&[i64]` is 8-aligned, as in Rust).
fn align(elem: &str) -> u32 {
    if elem == "i1" || elem == "u8" { 1 } else { 8 }
}

/// Bytes of one element in memory.
fn width(elem: &str) -> u64 {
    u64::from(align(elem))
}

fn fn_sym(path: &str) -> String {
    format!("@\"{path}\"")
}

struct Gen<'p> {
    t: &'p Tables,
    proven: &'p BTreeSet<ObKey>,
    /// The function being lowered: a definition, or an instance of a
    /// generic one (`path<A, B>`, ADR-0046 §5).
    path: String,
    /// The definition whose tables (local types, drops) it reads.
    base: String,
    /// The instance's type arguments, by parameter name.
    tsub: BTreeMap<String, TypeKind>,
    /// Instances of generic functions it calls: (key, definition, type
    /// arguments by parameter name).
    wants: Vec<(String, String, BTreeMap<String, TypeKind>)>,
    /// The instance types it computes (their LLVM type definitions), by
    /// their text.
    used: BTreeMap<String, TypeKind>,
    mod_path: String,
    out: String,
    allocas: String,
    tmp: u32,
    label: u32,
    slots: BTreeMap<String, Slot>,
    /// Runtime checks emitted (unproved obligations), for reporting.
    checks: u32,
    /// Temporaries that own heap memory (a `Vec` header, a `Box`, a data
    /// value holding one) read but not moved, dropped when their region
    /// (statement, block value, arm, operand, loop condition) ends.
    pending: Vec<(String, TypeKind)>,
    /// The byte strings of this function as private constants, emitted with
    /// its IR (so a cached fragment carries them).
    globals: String,
    /// The function's result type, in the instance (`return`).
    ret: TypeKind,
    /// The loops around the code being lowered, innermost last.
    loops: Vec<GLoop>,
}

/// A loop a `break` or `continue` may leave.
struct GLoop {
    label: Option<String>,
    /// Where `continue` goes.
    head: String,
    /// Where `break` goes.
    end: String,
    /// The length of `pending` when the loop began: the temporaries after it
    /// belong to regions inside the loop.
    mark: usize,
    /// Where a `loop`'s `break e` puts its value, and its type.
    slot: Option<String>,
    ty: TypeKind,
}

impl<'p> Gen<'p> {
    fn ty(&mut self, e: &Expr) -> TypeKind {
        let t = self
            .t
            .types
            .get(&(e.span.file, e.span.lo, e.span.hi))
            .cloned()
            .unwrap_or(TypeKind::Error);
        self.inst(&t)
    }

    /// `t` in the instance being lowered.
    fn inst(&mut self, t: &TypeKind) -> TypeKind {
        if self.tsub.is_empty() {
            return t.clone();
        }
        let t = crate::check::subst(t, &self.tsub);
        self.used.insert(ty_str(&t), t.clone());
        t
    }

    /// The type of local `name`, in the instance being lowered.
    fn local_ty(&mut self, name: &str) -> Option<TypeKind> {
        let t = self.t.locals.get(&self.base)?.get(name)?.clone();
        Some(self.inst(&t))
    }

    fn proved(&self, code: &'static str, e: &Expr) -> bool {
        self.proven
            .contains(&(code, e.span.file, e.span.lo, e.span.hi))
    }

    fn t(&mut self) -> String {
        self.tmp += 1;
        format!("%t{}", self.tmp)
    }

    fn l(&mut self, base: &str) -> String {
        self.label += 1;
        format!("{base}{}", self.label)
    }

    fn emit(&mut self, s: impl AsRef<str>) {
        self.out.push_str("  ");
        self.out.push_str(s.as_ref());
        self.out.push('\n');
    }

    fn block(&mut self, label: &str) {
        let _ = writeln!(self.out, "{label}:");
    }

    fn alloca(&mut self, ty: &str) -> String {
        let p = self.t();
        let _ = writeln!(self.allocas, "  {p} = alloca {ty}");
        p
    }

    fn load_elem(&mut self, elem: &str, ptr: &str) -> String {
        let r = self.t();
        self.emit(format!(
            "{r} = load {}, ptr {ptr}, align {}",
            mem(elem),
            align(elem)
        ));
        let b = self.t();
        match elem {
            "i1" => self.emit(format!("{b} = trunc i8 {r} to i1")),
            "u8" => self.emit(format!("{b} = zext i8 {r} to i64")),
            _ => return r,
        }
        b
    }

    fn store_elem(&mut self, elem: &str, v: &str, ptr: &str) {
        if elem == "i1" || elem == "u8" {
            let z = self.t();
            let op = if elem == "i1" { "zext i1" } else { "trunc i64" };
            self.emit(format!("{z} = {op} {v} to i8"));
            self.emit(format!("store i8 {z}, ptr {ptr}, align 1"));
        } else {
            self.emit(format!(
                "store {elem} {v}, ptr {ptr}, align {}",
                align(elem)
            ));
        }
    }

    /// Branches to a trap unless `cond` holds.
    fn check(&mut self, cond: &str, what: &str) {
        self.checks += 1;
        let ok = self.l("ok");
        let bad = self.l("trap");
        self.emit(format!(
            "br i1 {cond}, label %{ok}, label %{bad} ; unproved: {what}"
        ));
        self.block(&bad);
        self.emit("call void @llvm.trap()");
        self.emit("unreachable");
        self.block(&ok);
    }

    /// The function a call names, as the checker resolves it: `f` and
    /// `T::f` (an `impl` function of the unit's type) are local.
    fn resolve(&self, path: &[Ident]) -> String {
        let name = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let local = format!("{}::{name}", self.mod_path);
        match self.t.fns.get(&local) {
            Some(sig)
                if path.len() == 1
                    || (path.len() == 2 && (sig.in_impl || sig.trait_of.is_some())) =>
            {
                local
            }
            _ => name,
        }
    }

    /// Whether a call builds a variant (`Some(x)`, `E::V(..)`): it names no
    /// function.
    fn is_variant(&self, path: &[Ident]) -> bool {
        let target = self.resolve(path);
        !self.t.fns.contains_key(&target)
    }

    // ---------- data values in memory ----------

    fn temp(&mut self, t: &TypeKind) -> String {
        self.alloca(&mem_ty(t))
    }

    /// Loads a scalar of type `t` from data memory.
    fn load_mem(&mut self, ptr: &str, t: &TypeKind) -> String {
        let r = self.t();
        if *t == TypeKind::Bool || *t == TypeKind::Int(IntTy::U8) {
            self.emit(format!("{r} = load i8, ptr {ptr}"));
            let b = self.t();
            let op = if *t == TypeKind::Bool {
                "trunc i8"
            } else {
                "zext i8"
            };
            let to = if *t == TypeKind::Bool { "i1" } else { "i64" };
            self.emit(format!("{b} = {op} {r} to {to}"));
            return b;
        }
        self.emit(format!("{r} = load i64, ptr {ptr}"));
        r
    }

    /// Copies a value of type `t` (`memmove`: `x = x` is allowed).
    fn copy(&mut self, dst: &str, src: &str, t: &TypeKind) {
        self.emit(format!(
            "call void @llvm.memmove.p0.p0.i64(ptr {dst}, ptr {src}, i64 {}, i1 false)",
            size_of(t)
        ));
    }

    /// Stores value `v` of type `t` into memory at `ptr`.
    fn put(&mut self, v: CVal, ptr: &str, t: &TypeKind) {
        match v {
            CVal::Imm(x) if matches!(value_ty(t), TypeKind::Bool | TypeKind::Int(IntTy::U8)) => {
                let z = self.t();
                let op = if *value_ty(t) == TypeKind::Bool {
                    "zext i1"
                } else {
                    "trunc i64"
                };
                self.emit(format!("{z} = {op} {x} to i8"));
                self.emit(format!("store i8 {z}, ptr {ptr}"));
            }
            CVal::Imm(x) => self.emit(format!("store i64 {x}, ptr {ptr}")),
            CVal::Ptr { ptr: src, .. } | CVal::Arr { ptr: src, .. } => self.copy(ptr, &src, t),
            CVal::Unit | CVal::Never => {}
        }
    }

    /// The value of type `t` at `ptr`: a scalar is loaded, an array or a
    /// data value is its address.
    fn read(&mut self, ptr: &str, t: &TypeKind) -> CVal {
        match value_ty(t) {
            TypeKind::Named { .. } | TypeKind::Vec(_) | TypeKind::Box(_) => CVal::Ptr {
                ptr: ptr.to_string(),
                ty: value_ty(t).clone(),
            },
            TypeKind::Array(_, n) => CVal::Arr {
                ptr: ptr.to_string(),
                len: n.to_string(),
            },
            TypeKind::Unit => CVal::Unit,
            s => CVal::Imm(self.load_mem(ptr, &s.clone())),
        }
    }

    /// Address of field `k` of the struct of type `t` at `base`.
    fn field_ptr(&mut self, base: &str, t: &TypeKind, k: usize) -> String {
        let p = self.t();
        self.emit(format!(
            "{p} = getelementptr inbounds {}, ptr {base}, i32 0, i32 {k}",
            data_ty(t)
        ));
        p
    }

    fn field_of(&self, t: &TypeKind, name: &str) -> (usize, TypeKind) {
        let fs = struct_fields(t, self.t).expect("field of a struct");
        let k = fs
            .iter()
            .position(|(n, _)| n == name)
            .expect("checked field");
        (k, fs[k].1.clone())
    }

    /// Variant `v` of enum `t` built from `args`: each payload is copied
    /// as it is evaluated, in order.
    fn variant(&mut self, t: &TypeKind, v: &str, args: &[Expr]) -> CVal {
        let vs = enum_variants(t, self.t).expect("variant of an enum");
        let k = vs
            .iter()
            .position(|(n, _)| n == v)
            .expect("checked variant");
        let tmp = self.temp(t);
        let tag = self.field_ptr(&tmp, t, 0);
        self.emit(format!("store i64 {k}, ptr {tag}"));
        for (j, (a, pt)) in args.iter().zip(&vs[k].1).enumerate() {
            let val = self.expr(a);
            let p = self.t();
            self.emit(format!(
                "{p} = getelementptr inbounds {}, ptr {tmp}, i32 0, i32 {}",
                view_ty(t, v),
                j + 1
            ));
            self.put(val, &p, pt);
        }
        CVal::Ptr {
            ptr: tmp,
            ty: value_ty(t).clone(),
        }
    }

    /// The address and type of a place: a variable, a field, `*p`, an
    /// element of a slice of data values.
    fn place(&mut self, e: &Expr) -> (String, TypeKind) {
        match &e.kind {
            ExprKind::Var(id) => match self.slots[&id.name].clone() {
                Slot::Data { ptr, ty } => (ptr, ty),
                _ => unreachable!("a scalar place has its own path"),
            },
            // `*b` of a `Box` is the place at its address; `*r` of a
            // reference (only a parameter is one) is the place it names.
            ExprKind::Deref(x) => {
                let (p, t) = self.place(x);
                let reference = matches!(&x.kind, ExprKind::Var(id)
                    if matches!(self.local_ty(&id.name), Some(TypeKind::Ref { .. })));
                match t {
                    TypeKind::Box(inner) if !reference => (self.unbox(&p), inner.kind),
                    t => (p, t),
                }
            }
            ExprKind::Ref { expr: x, .. } => self.place(x),
            ExprKind::Field { base, name } => {
                let (mut bp, mut bt) = self.place(base);
                while let TypeKind::Box(inner) = bt {
                    bp = self.unbox(&bp);
                    bt = inner.kind;
                }
                let (k, ft) = self.field_of(&bt, &name.name);
                (self.field_ptr(&bp, &bt, k), ft)
            }
            ExprKind::Index { base, index } if !matches!(base.kind, ExprKind::Var(_)) => {
                // An element of an array or `Vec` field.
                let (bp, bt) = self.place(base);
                let (buf, len, el) = match value_ty(&bt).clone() {
                    TypeKind::Array(el, n) => (bp, n.to_string(), el.kind),
                    TypeKind::Vec(el) => {
                        let (b, l) = self.vec_view(&bp);
                        (b, l, el.kind)
                    }
                    _ => unreachable!("indexing an array or Vec field"),
                };
                let i = self.expr(index).imm();
                let p = self.data_elem(e, &buf, &len, &i, &el);
                (p, el)
            }
            ExprKind::Index { base, index } => {
                let ExprKind::Var(id) = &base.kind else {
                    unreachable!("indexing a named slice")
                };
                let (ptr, len, ety) = match self.slots[&id.name].clone() {
                    Slot::Arr { ptr, len, ety, .. } => (ptr, len, ety),
                    Slot::Data { ptr, ty } => {
                        let (b, l) = self.vec_view(&ptr);
                        (b, l, vec_elem(&ty))
                    }
                    Slot::Scalar { .. } => unreachable!(),
                };
                let i = self.expr(index).imm();
                let p = self.data_elem(e, &ptr, &len, &i, &ety);
                (p, ety)
            }
            _ => unreachable!("not a place"),
        }
    }

    /// Address of element `i` of a slice of data values (bounds-checked
    /// unless proved).
    fn data_elem(&mut self, e: &Expr, ptr: &str, len: &str, i: &str, ety: &TypeKind) -> String {
        let note = if self.proved("E0805", e) {
            " ; proved E0805"
        } else {
            let c = self.t();
            self.emit(format!("{c} = icmp ult i64 {i}, {len}"));
            self.check(&c, "index may be out of bounds");
            ""
        };
        let p = self.t();
        self.emit(format!(
            "{p} = getelementptr inbounds {}, ptr {ptr}, i64 {i}{note}",
            mem_ty(ety)
        ));
        p
    }

    /// A `match`: the tag selects the arm; each binding is a copy of its
    /// payload position; the arms' values meet in one slot. A `match` that
    /// moves its scrutinee (`MoveInfo::consumed`) drops the owned payloads
    /// an arm does not bind at its start, and the bindings still owned at
    /// its end; one that only reads a temporary drops it afterwards.
    fn match_expr(&mut self, e: &Expr, scrut: &Expr, arms: &[Arm]) -> CVal {
        let rt = self.ty(e);
        let slot = self.result_slot(&rt);
        let sv = self.expr(scrut);
        let CVal::Ptr { ptr: sp, ty: st } = sv.clone() else {
            unreachable!("match on an enum value")
        };
        let info = self.t.moves.get(&self.base);
        let consumed =
            info.is_some_and(|m| m.consumed.contains(&(e.span.file, e.span.lo, e.span.hi)));
        if !consumed {
            self.read_temp(scrut, &sv, &st);
        }
        let vs = enum_variants(&st, self.t).expect("match on an enum");
        let tagp = self.field_ptr(&sp, &st, 0);
        let tag = self.t();
        self.emit(format!("{tag} = load i64, ptr {tagp}"));
        let labels: Vec<String> = arms.iter().map(|_| self.l("arm")).collect();
        let end = self.l("endmatch");
        let cases: Vec<String> = arms
            .iter()
            .zip(&labels)
            .map(|(a, l)| {
                let v = &a.pat.path.last().unwrap().name;
                let k = vs.iter().position(|(n, _)| n == v).expect("checked arm");
                format!("i64 {k}, label %{l}")
            })
            .collect();
        self.emit(format!(
            "switch i64 {tag}, label %{} [ {} ]",
            labels.last().unwrap(),
            cases.join(" ")
        ));
        for (a, l) in arms.iter().zip(&labels) {
            self.block(l);
            let v = a.pat.path.last().unwrap().name.clone();
            let ps = vs.iter().find(|(n, _)| *n == v).unwrap().1.clone();
            let mut write_back: Vec<(String, String, TypeKind)> = Vec::new();
            for (j, (b, pt)) in a.pat.binds.iter().zip(&ps).enumerate() {
                let bound = b.is_some();
                if !bound && !(consumed && crate::check::is_move(pt, &self.t.adts)) {
                    continue;
                }
                let fp = self.t();
                self.emit(format!(
                    "{fp} = getelementptr inbounds {}, ptr {sp}, i32 0, i32 {}",
                    view_ty(&st, &v),
                    j + 1
                ));
                match b {
                    // A binding of a `&mut` match (ADR-0055) is the payload's
                    // place; a scalar is read into a register slot and
                    // written back at the end of the arm.
                    Some(id)
                        if matches!(
                            self.local_ty(&id.name),
                            Some(TypeKind::Ref { mutable: true, .. })
                        ) =>
                    {
                        let slot = match value_ty(pt).clone() {
                            t if is_mem(&t) => Slot::Data { ptr: fp, ty: t },
                            TypeKind::Array(e, n) => Slot::Arr {
                                ptr: fp,
                                len: n.to_string(),
                                elem: elem_kind(&e.kind),
                                ety: e.kind.clone(),
                            },
                            t => {
                                let lt = llty(&t);
                                let tmp = self.alloca(lt);
                                let x = self.load_mem(&fp, &t);
                                self.emit(format!("store {lt} {x}, ptr {tmp}"));
                                write_back.push((tmp.clone(), fp, t));
                                Slot::Scalar { ptr: tmp, ty: lt }
                            }
                        };
                        self.slots.insert(id.name.clone(), slot);
                    }
                    Some(id) => self.bind(&id.name, &fp, pt),
                    None => self.drop_at(&fp, pt),
                }
            }
            let v = self.region(|g| g.expr(&a.body));
            for (tmp, fp, t) in write_back {
                let x = self.t();
                self.emit(format!("{x} = load {}, ptr {tmp}", llty(&t)));
                self.put(CVal::Imm(x), &fp, &t);
            }
            self.store_result(&slot, v, &rt);
            let owned = self
                .t
                .moves
                .get(&self.base)
                .and_then(|m| {
                    m.arm_drops
                        .get(&(a.body.span.file, a.body.span.lo, a.body.span.hi))
                })
                .cloned()
                .unwrap_or_default();
            for n in owned {
                if let Some(Slot::Data { ptr, ty }) = self.slots.get(&n).cloned() {
                    self.drop_at(&ptr, &ty);
                }
            }
            self.emit(format!("br label %{end}"));
        }
        self.block(&end);
        self.load_result(slot, &rt)
    }

    /// A new local `name` holding a copy of the value of type `t` at `src`.
    fn bind(&mut self, name: &str, src: &str, t: &TypeKind) {
        let slot = match value_ty(t) {
            t if is_mem(t) => {
                let ptr = self.temp(t);
                self.copy(&ptr, src, t);
                Slot::Data {
                    ptr,
                    ty: value_ty(t).clone(),
                }
            }
            TypeKind::Array(e, n) => {
                let ptr = self.temp(t);
                self.copy(&ptr, src, t);
                Slot::Arr {
                    ptr,
                    len: n.to_string(),
                    elem: elem_kind(&e.kind),
                    ety: e.kind.clone(),
                }
            }
            s => {
                let s = s.clone();
                let v = self.load_mem(src, &s);
                let lt = llty(&s);
                let ptr = self.alloca(lt);
                self.emit(format!("store {lt} {v}, ptr {ptr}"));
                Slot::Scalar { ptr, ty: lt }
            }
        };
        self.slots.insert(name.to_string(), slot);
    }

    /// Where branches of an `if` or `match` of type `t` leave their value.
    fn result_slot(&mut self, t: &TypeKind) -> Option<String> {
        match t {
            TypeKind::Unit | TypeKind::Error => None,
            t if is_mem(t) => Some(self.temp(t)),
            t => Some(self.alloca(llty(t))),
        }
    }

    fn store_result(&mut self, slot: &Option<String>, v: CVal, t: &TypeKind) {
        match slot {
            _ if matches!(v, CVal::Never) => {}
            Some(s) if is_mem(t) => self.put(v, s, t),
            Some(s) => self.emit(format!("store {} {}, ptr {s}", llty(t), v.imm())),
            None => {}
        }
    }

    fn load_result(&mut self, slot: Option<String>, t: &TypeKind) -> CVal {
        match slot {
            Some(s) if is_mem(t) => CVal::Ptr {
                ptr: s,
                ty: value_ty(t).clone(),
            },
            Some(s) => {
                let r = self.t();
                self.emit(format!("{r} = load {}, ptr {s}", llty(t)));
                CVal::Imm(r)
            }
            None => CVal::Unit,
        }
    }

    /// Whether the result is written to the caller's memory (a data or
    /// array result).
    fn ret_in_memory(&self) -> bool {
        is_mem(&self.ret) || matches!(self.ret, TypeKind::Array(..))
    }

    /// Returns `rv` from the function.
    fn emit_return(&mut self, rv: CVal) {
        if self.ret_in_memory() {
            let t = self.ret.clone();
            self.put(rv, "%\"ret.out\"", &t);
            self.emit("ret void");
            return;
        }
        match llty(&self.ret) {
            "void" => self.emit("ret void"),
            ret => {
                let v = rv.imm();
                self.emit(format!("ret {ret} {v}"));
            }
        }
    }

    /// `return e` (ADR-0049 §4, §5): the owned temporaries of the regions
    /// around it and the locals the move check left owned on this exit are
    /// dropped, then the function returns; what follows is unreachable.
    fn return_expr(&mut self, e: &Expr, x: Option<&Expr>) -> CVal {
        let v = match x {
            Some(x) => self.expr(x),
            None => CVal::Unit,
        };
        // The value may point into a buffer dropped here.
        let v = self.detach(v);
        for (h, t) in self.pending.clone() {
            self.drop_at(&h, &t);
        }
        let exits = self
            .t
            .moves
            .get(&self.base)
            .and_then(|m| m.exits.get(&(e.span.file, e.span.lo, e.span.hi)))
            .cloned()
            .unwrap_or_default();
        for p in exits {
            self.drop_path(&p);
        }
        self.emit_return(v);
        let after = self.l("after_return");
        self.block(&after);
        CVal::Never
    }

    /// `x?` (ADR-0049 §5): a tag test; on `None`/`Err` the function
    /// returns `None`/`Err(e)` with the operand's error moved into it, after
    /// the drops of a `return` here; otherwise the value is the payload.
    fn try_expr(&mut self, e: &Expr, x: &Expr) -> CVal {
        let sv = self.expr(x);
        let CVal::Ptr { ptr: sp, ty: st } = sv.clone() else {
            unreachable!("`?` on an enum value")
        };
        let consumed = self
            .t
            .moves
            .get(&self.base)
            .is_some_and(|m| m.consumed.contains(&(e.span.file, e.span.lo, e.span.hi)));
        if !consumed {
            self.read_temp(x, &sv, &st);
        }
        let (ok, bad) = crate::check::try_variants(&st).expect("checked `?`");
        let vs = enum_variants(&st, self.t).expect("an `Option` or a `Result`");
        let k_ok = vs.iter().position(|(n, _)| n == ok).expect("variant");
        let tagp = self.field_ptr(&sp, &st, 0);
        let tag = self.t();
        self.emit(format!("{tag} = load i64, ptr {tagp}"));
        let is_ok = self.t();
        self.emit(format!("{is_ok} = icmp eq i64 {tag}, {k_ok}"));
        let (ok_l, bad_l) = (self.l("try_ok"), self.l("try_out"));
        self.emit(format!("br i1 {is_ok}, label %{ok_l}, label %{bad_l}"));
        // The returned `None` / `Err(e)`.
        self.block(&bad_l);
        let rt = self.ret.clone();
        let rvs = enum_variants(&rt, self.t).expect("an `Option` or a `Result` result");
        let k_bad = rvs.iter().position(|(n, _)| n == bad).expect("variant");
        let out = self.temp(&rt);
        let out_tag = self.field_ptr(&out, &rt, 0);
        self.emit(format!("store i64 {k_bad}, ptr {out_tag}"));
        if let Some((_, ps)) = vs.iter().find(|(n, _)| n == bad)
            && let [pt] = ps.as_slice()
        {
            let (src, dst) = (self.t(), self.t());
            self.emit(format!(
                "{src} = getelementptr inbounds {}, ptr {sp}, i32 0, i32 1",
                view_ty(&st, bad)
            ));
            self.emit(format!(
                "{dst} = getelementptr inbounds {}, ptr {out}, i32 0, i32 1",
                view_ty(&rt, bad)
            ));
            self.copy(&dst, &src, pt);
        }
        for (h, t) in self.pending.clone() {
            self.drop_at(&h, &t);
        }
        let exits = self
            .t
            .moves
            .get(&self.base)
            .and_then(|m| m.exits.get(&(e.span.file, e.span.lo, e.span.hi)))
            .cloned()
            .unwrap_or_default();
        for p in exits {
            self.drop_path(&p);
        }
        self.emit_return(CVal::Ptr { ptr: out, ty: rt });
        // The payload.
        self.block(&ok_l);
        let pt = vs[k_ok].1[0].clone();
        let fp = self.t();
        self.emit(format!(
            "{fp} = getelementptr inbounds {}, ptr {sp}, i32 0, i32 1",
            view_ty(&st, ok)
        ));
        match value_ty(&pt).clone() {
            t if is_mem(&t) => {
                let tmp = self.temp(&t);
                self.copy(&tmp, &fp, &t);
                CVal::Ptr { ptr: tmp, ty: t }
            }
            TypeKind::Array(_, n) => {
                let tmp = self.temp(&pt);
                self.copy(&tmp, &fp, &pt);
                CVal::Arr {
                    ptr: tmp,
                    len: n.to_string(),
                }
            }
            t => CVal::Imm(self.load_mem(&fp, &t)),
        }
    }

    /// A `loop` (ADR-0049 §5): its value is the `break e` that leaves it.
    fn loop_expr(&mut self, e: &Expr, label: &Option<Ident>, body: &Block) -> CVal {
        let ty = self.ty(e);
        let slot = if ty == TypeKind::Never {
            None
        } else {
            self.result_slot(&ty)
        };
        let head = self.l("loop");
        let end = self.l("endloop");
        self.emit(format!("br label %{head}"));
        self.block(&head);
        self.loops.push(GLoop {
            label: label.as_ref().map(|l| l.name.clone()),
            head: head.clone(),
            end: end.clone(),
            mark: self.pending.len(),
            slot: slot.clone(),
            ty: ty.clone(),
        });
        self.block_expr(body);
        self.loops.pop();
        self.emit(format!("br label %{head}"));
        self.block(&end);
        if ty == TypeKind::Never {
            return CVal::Never;
        }
        self.load_result(slot, &ty)
    }

    /// `break` / `continue` (ADR-0049 §4, §5): the temporaries of the
    /// regions inside the loop and the locals the move check left owned on
    /// this exit are dropped, then control goes to the loop's end or head.
    fn jump(&mut self, e: &Expr, label: &Option<Ident>, value: Option<&Expr>, brk: bool) -> CVal {
        let k = match label {
            Some(l) => self
                .loops
                .iter()
                .rposition(|c| c.label.as_deref() == Some(l.name.as_str()))
                .expect("checked label"),
            None => self.loops.len() - 1,
        };
        let v = match value {
            Some(x) => self.expr(x),
            None => CVal::Unit,
        };
        let v = self.detach(v);
        if let (Some(slot), true) = (self.loops[k].slot.clone(), brk) {
            let ty = self.loops[k].ty.clone();
            self.store_result(&Some(slot), v, &ty);
        }
        let inside: Vec<(String, TypeKind)> = self.pending[self.loops[k].mark..].to_vec();
        for (h, t) in inside {
            self.drop_at(&h, &t);
        }
        let exits = self
            .t
            .moves
            .get(&self.base)
            .and_then(|m| m.exits.get(&(e.span.file, e.span.lo, e.span.hi)))
            .cloned()
            .unwrap_or_default();
        for p in exits {
            self.drop_path(&p);
        }
        let target = if brk {
            self.loops[k].end.clone()
        } else {
            self.loops[k].head.clone()
        };
        self.emit(format!("br label %{target}"));
        let after = self.l("after_jump");
        self.block(&after);
        CVal::Never
    }

    // ---------- expressions ----------

    fn expr(&mut self, e: &Expr) -> CVal {
        match &e.kind {
            ExprKind::Return(x) => self.return_expr(e, x.as_deref()),
            ExprKind::Try(x) => self.try_expr(e, x),
            ExprKind::Loop { label, body, .. } => self.loop_expr(e, label, body),
            ExprKind::Break { label, value } => self.jump(e, label, value.as_deref(), true),
            ExprKind::Continue { label } => self.jump(e, label, None, false),
            ExprKind::Int(v) => CVal::Imm(v.to_string()),
            ExprKind::ByteLit(v) => CVal::Imm(v.to_string()),
            ExprKind::ByteStr(v) => {
                let name = format!("@\"{}.bytes.{}\"", self.path, self.globals.lines().count());
                let body: String = v.iter().map(|b| format!("\\{b:02X}")).collect();
                let _ = writeln!(
                    self.globals,
                    "{name} = private unnamed_addr constant [{} x i8] c\"{body}\", align 1",
                    v.len()
                );
                CVal::Arr {
                    ptr: name,
                    len: v.len().to_string(),
                }
            }
            ExprKind::Bool(b) => CVal::Imm(b.to_string()),
            ExprKind::Unit => CVal::Unit,
            ExprKind::Var(id) if id.name == "None" && !self.slots.contains_key("None") => {
                let t = self.ty(e);
                self.variant(&t, "None", &[])
            }
            ExprKind::Var(id) => match self.slots[&id.name].clone() {
                Slot::Scalar { ptr, ty } => {
                    let r = self.t();
                    self.emit(format!("{r} = load {ty}, ptr {ptr}"));
                    CVal::Imm(r)
                }
                Slot::Arr { ptr, len, .. } => CVal::Arr { ptr, len },
                Slot::Data { ptr, ty } => CVal::Ptr { ptr, ty },
            },
            ExprKind::Unary(UnOp::Neg, x) => {
                if let ExprKind::Int(v) = x.kind {
                    return CVal::Imm(format!("-{v}"));
                }
                let v = self.expr(x).imm();
                self.arith(e, "sub", "0", &v, true)
            }
            ExprKind::Unary(UnOp::Not, x) => {
                let v = self.expr(x).imm();
                let r = self.t();
                self.emit(format!("{r} = xor i1 {v}, true"));
                CVal::Imm(r)
            }
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or), l, r) => {
                let slot = self.alloca("i1");
                let lv = self.expr(l).imm();
                self.emit(format!("store i1 {lv}, ptr {slot}"));
                let rhs = self.l("rhs");
                let end = self.l("sc");
                if *op == BinOp::And {
                    self.emit(format!("br i1 {lv}, label %{rhs}, label %{end}"));
                } else {
                    self.emit(format!("br i1 {lv}, label %{end}, label %{rhs}"));
                }
                self.block(&rhs);
                let rv = self.region(|g| g.expr(r)).imm();
                self.emit(format!("store i1 {rv}, ptr {slot}"));
                self.emit(format!("br label %{end}"));
                self.block(&end);
                let out = self.t();
                self.emit(format!("{out} = load i1, ptr {slot}"));
                CVal::Imm(out)
            }
            ExprKind::Binary(op, l, r) => {
                let lt = self.ty(l);
                let signed = lt.is_signed();
                let a = self.expr(l).imm();
                let b = self.expr(r).imm();
                let ty = llty(&lt);
                let cmp = |o: BinOp| -> Option<&'static str> {
                    Some(match (o, signed) {
                        (BinOp::Eq, _) => "eq",
                        (BinOp::Ne, _) => "ne",
                        (BinOp::Lt, true) => "slt",
                        (BinOp::Lt, false) => "ult",
                        (BinOp::Le, true) => "sle",
                        (BinOp::Le, false) => "ule",
                        (BinOp::Gt, true) => "sgt",
                        (BinOp::Gt, false) => "ugt",
                        (BinOp::Ge, true) => "sge",
                        (BinOp::Ge, false) => "uge",
                        _ => return None,
                    })
                };
                if let Some(c) = cmp(*op) {
                    let res = self.t();
                    self.emit(format!("{res} = icmp {c} {ty} {a}, {b}"));
                    return CVal::Imm(res);
                }
                match op {
                    // A byte's sum or product fits in 64 bits: compute it
                    // there, then the byte range is the obligation.
                    BinOp::Add | BinOp::Mul if lt == TypeKind::Int(IntTy::U8) => {
                        let inst = if *op == BinOp::Add { "add" } else { "mul" };
                        let r = self.t();
                        self.emit(format!("{r} = {inst} nuw i64 {a}, {b}"));
                        if self.proved("E0803", e) {
                            self.emit("; proved E0803");
                        } else {
                            let c = self.t();
                            self.emit(format!("{c} = icmp ule i64 {r}, 255"));
                            self.check(&c, &format!("`{inst}` may overflow"));
                        }
                        CVal::Imm(r)
                    }
                    BinOp::Add => self.arith(e, "add", &a, &b, signed),
                    BinOp::Sub => self.arith(e, "sub", &a, &b, signed),
                    BinOp::Mul => self.arith(e, "mul", &a, &b, signed),
                    BinOp::Div | BinOp::Rem => {
                        if !self.proved("E0804", r) {
                            let c = self.t();
                            self.emit(format!("{c} = icmp ne i64 {b}, 0"));
                            self.check(&c, "divisor may be zero");
                        }
                        if signed && !self.proved("E0803", e) {
                            let m = self.t();
                            let n = self.t();
                            let both = self.t();
                            let ok = self.t();
                            self.emit(format!("{m} = icmp eq i64 {a}, -9223372036854775808"));
                            self.emit(format!("{n} = icmp eq i64 {b}, -1"));
                            self.emit(format!("{both} = and i1 {m}, {n}"));
                            self.emit(format!("{ok} = xor i1 {both}, true"));
                            self.check(&ok, "i64::MIN / -1");
                        }
                        let inst = match (op, signed) {
                            (BinOp::Div, true) => "sdiv",
                            (BinOp::Div, false) => "udiv",
                            (_, true) => "srem",
                            _ => "urem",
                        };
                        let res = self.t();
                        let note = if self.proved("E0804", r) {
                            " ; proved E0804"
                        } else {
                            ""
                        };
                        self.emit(format!("{res} = {inst} i64 {a}, {b}{note}"));
                        CVal::Imm(res)
                    }
                    _ => unreachable!(),
                }
            }
            ExprKind::Index { base, index } if is_mem_elem(&self.ty(e)) => {
                let ety = self.ty(e);
                let CVal::Arr { ptr, len } = self.array_of(base) else {
                    unreachable!()
                };
                // A temporary array of owned elements, read here.
                let bt = self.ty(base);
                if matches!(bt, TypeKind::Array(..))
                    && Self::fresh(base)
                    && crate::check::is_move(&bt, &self.t.adts)
                {
                    self.pending.push((ptr.clone(), bt));
                }
                let i = self.expr(index).imm();
                let p = self.data_elem(e, &ptr, &len, &i, &ety);
                self.read(&p, &ety)
            }
            ExprKind::Index { base, index } => {
                let (ptr, ty) = self.elem_ptr(e, base, index);
                CVal::Imm(self.load_elem(ty, &ptr))
            }
            // A conversion keeps the 64-bit value: the range check is the
            // sign bit (`i64` -> `u64`/`usize` and back), dropped when proved.
            ExprKind::Method { recv, name, .. } if crate::check::conv_call(recv).is_some() => {
                let (target, args) = crate::check::conv_call(recv).unwrap();
                let st = self.ty(&args[0]);
                let v = self.expr(&args[0]).imm();
                let needs = crate::check::conv_obligation(&st, &target);
                // To `u8`: `0 <= v <= 255` is one unsigned comparison.
                let in_range = |g: &mut Self| {
                    let c = g.t();
                    if target == TypeKind::Int(IntTy::U8) {
                        g.emit(format!("{c} = icmp ule i64 {v}, 255"));
                    } else {
                        g.emit(format!("{c} = icmp sge i64 {v}, 0"));
                    }
                    c
                };
                if name.name == "unwrap" {
                    if needs && self.proved("E0815", e) {
                        self.emit("; proved E0815");
                    } else if needs {
                        let c = in_range(self);
                        self.check(&c, "conversion may be out of range");
                    }
                    return CVal::Imm(v);
                }
                let ok = if needs {
                    in_range(self)
                } else {
                    "true".to_string()
                };
                self.option(e, &ok, &v)
            }
            ExprKind::Method { recv, name, args } if is_vec(&self.ty(recv)) => {
                self.vec_method(e, recv, &name.name, args)
            }
            ExprKind::Method { recv, name, args } => {
                let rv = self.expr(recv);
                match name.name.as_str() {
                    "len" => match rv {
                        CVal::Arr { len, .. } => CVal::Imm(len),
                        _ => unreachable!(),
                    },
                    op => {
                        let b = self.expr(&args[0]).imm();
                        let rt = self.ty(recv);
                        let r = self.int_method(e, op, &rv.imm(), &b, rt.is_signed());
                        if rt == TypeKind::Int(IntTy::U8) {
                            self.byte_method(e, op, r)
                        } else {
                            r
                        }
                    }
                }
            }
            ExprKind::Call { path, args, .. }
                if path.len() == 2
                    && path[0].name == "Vec"
                    && matches!(path[1].name.as_str(), "new" | "with_capacity") =>
            {
                self.vec_new(e, args)
            }
            ExprKind::Call { path, args, .. } if crate::check::is_box_new(path) => {
                self.box_new(e, args)
            }
            ExprKind::Call { path, args, .. } if crate::check::mem_op(path).is_some() => {
                self.mem_call(crate::check::mem_op(path).expect("checked"), args)
            }
            ExprKind::Call { path, args, .. } if self.is_variant(path) => {
                let t = self.ty(e);
                self.variant(&t, &path.last().unwrap().name, args)
            }
            ExprKind::Call { path, args, .. } => self.call(path, args, e),
            ExprKind::If { cond, then, els } if matches!(self.ty(e), TypeKind::Array(..)) => {
                // An array-valued `if`: each branch copies its array into
                // one buffer (backlog 1B-25).
                let TypeKind::Array(el, n) = self.ty(e) else {
                    unreachable!()
                };
                let elem = elem_kind(&el.kind);
                let buf = self.alloca(&format!("[{n} x {}]", mem(elem)));
                let bytes = n * width(elem);
                let c = self.expr(cond).imm();
                let lt = self.l("then");
                let le = self.l("else");
                let end = self.l("endif");
                self.emit(format!("br i1 {c}, label %{lt}, label %{le}"));
                for (label, branch) in [(lt, Some(then)), (le, None)] {
                    self.block(&label);
                    let v = match branch {
                        Some(b) => self.block_expr(b),
                        None => self.expr(els.as_ref().expect("array `if` has an else")),
                    };
                    if let CVal::Arr { ptr: src, .. } = v {
                        self.emit(format!(
                            "call void @llvm.memcpy.p0.p0.i64(ptr {buf}, ptr {src}, i64 {bytes}, i1 false)"
                        ));
                    }
                    self.emit(format!("br label %{end}"));
                }
                self.block(&end);
                CVal::Arr {
                    ptr: buf,
                    len: n.to_string(),
                }
            }
            ExprKind::If { cond, then, els } => {
                let ty = self.ty(e);
                let slot = self.result_slot(&ty);
                let c = self.expr(cond).imm();
                let lt = self.l("then");
                let le = self.l("else");
                let end = self.l("endif");
                self.emit(format!("br i1 {c}, label %{lt}, label %{le}"));
                self.block(&lt);
                let tv = self.block_expr(then);
                self.store_result(&slot, tv, &ty);
                self.emit(format!("br label %{end}"));
                self.block(&le);
                if let Some(x) = els {
                    let ev = self.expr(x);
                    self.store_result(&slot, ev, &ty);
                }
                self.emit(format!("br label %{end}"));
                self.block(&end);
                self.load_result(slot, &ty)
            }
            ExprKind::Block(b) => self.block_expr(b),
            // `&f()`: the temporary lives to the end of the region.
            ExprKind::Ref { expr, .. } => {
                let v = self.expr(expr);
                let t = self.ty(expr);
                self.read_temp(expr, &v, value_ty(&t));
                v
            }
            ExprKind::Array(_) | ExprKind::Repeat { .. } if matches!(&self.ty(e), TypeKind::Array(el, _) if is_mem_elem(&el.kind)) =>
            {
                let ty = self.ty(e);
                let ptr = self.temp(&ty);
                self.init_data_array(e, &ptr, &ty);
                let TypeKind::Array(_, n) = ty else {
                    unreachable!()
                };
                CVal::Arr {
                    ptr,
                    len: n.to_string(),
                }
            }
            ExprKind::Array(_) | ExprKind::Repeat { .. } => {
                let ty = self.ty(e);
                let TypeKind::Array(el, n) = &ty else {
                    unreachable!()
                };
                let elem = elem_kind(&el.kind);
                let ptr = self.alloca(&format!("[{n} x {}]", mem(elem)));
                self.init_array(e, &ptr, elem);
                CVal::Arr {
                    ptr,
                    len: n.to_string(),
                }
            }
            ExprKind::Old(_) | ExprKind::Forall { .. } | ExprKind::Trigger(_) | ExprKind::Error => {
                unreachable!("specification construct in executable code")
            }
            // Fields are evaluated and copied in the written order.
            ExprKind::StructLit { fields, .. } => {
                let t = self.ty(e);
                let tmp = self.temp(&t);
                for (n, x) in fields {
                    let v = self.expr(x);
                    let (k, ft) = self.field_of(&t, &n.name);
                    let p = self.field_ptr(&tmp, &t, k);
                    self.put(v, &p, &ft);
                }
                CVal::Ptr { ptr: tmp, ty: t }
            }
            // Fields project through `Box`es (ADR-0047 §1).
            ExprKind::Field { base, name } => {
                let bv = self.expr(base);
                let CVal::Ptr { mut ptr, mut ty } = bv.clone() else {
                    unreachable!("field of a struct value")
                };
                let temp = self.read_temp(base, &bv, &ty);
                while let TypeKind::Box(inner) = ty {
                    ptr = self.unbox(&ptr);
                    ty = inner.kind;
                }
                let (k, ft) = self.field_of(&ty, &name.name);
                let p = self.field_ptr(&ptr, &ty, k);
                let v = self.read(&p, &ft);
                if temp { self.keep(v, &ft) } else { v }
            }
            // `*b` of a `Box`: the value at its address.
            ExprKind::Deref(x) if matches!(self.ty(x), TypeKind::Box(_)) => {
                let bv = self.expr(x);
                let CVal::Ptr { ptr, ty } = bv.clone() else {
                    unreachable!("a Box value")
                };
                let temp = self.read_temp(x, &bv, &ty);
                let TypeKind::Box(inner) = ty else {
                    unreachable!("a Box value")
                };
                let h = self.unbox(&ptr);
                let v = self.read(&h, &inner.kind);
                if temp { self.keep(v, &inner.kind) } else { v }
            }
            // A reference denotes its value (references are second-class).
            ExprKind::Deref(x) => self.expr(x),
            ExprKind::Path(segs) if crate::check::int_const(segs).is_some() => CVal::Imm(
                crate::check::int_const(segs)
                    .expect("checked")
                    .1
                    .to_string(),
            ),
            ExprKind::Path(segs) => {
                let t = self.ty(e);
                self.variant(&t, &segs.last().unwrap().name, &[])
            }
            ExprKind::Match { scrut, arms } => self.match_expr(e, scrut, arms),
        }
    }

    // ---------- Vec (ADR-0031 §4, backlog 2G-04) ----------

    /// The buffer and length of the `Vec` header at `hdr`, read now (a
    /// `push` may move the buffer).
    fn vec_view(&mut self, hdr: &str) -> (String, String) {
        let buf = self.t();
        self.emit(format!("{buf} = load ptr, ptr {hdr}"));
        let lp = self.vec_field(hdr, 1);
        let len = self.t();
        self.emit(format!("{len} = load i64, ptr {lp}"));
        (buf, len)
    }

    fn vec_field(&mut self, hdr: &str, k: u32) -> String {
        let p = self.t();
        self.emit(format!(
            "{p} = getelementptr inbounds %bmb.vec, ptr {hdr}, i32 0, i32 {k}"
        ));
        p
    }

    /// An array or slice value, or the elements of a `Vec` (a temporary
    /// `Vec` is freed when the region ends).
    fn array_of(&mut self, base: &Expr) -> CVal {
        if !is_vec(&self.ty(base)) && !self.vec_var(base) {
            return self.expr(base);
        }
        let hdr = self.vec_hdr(base);
        let (ptr, len) = self.vec_view(&hdr);
        CVal::Arr { ptr, len }
    }

    /// Whether `e` names a `Vec` local or parameter (assignment targets have
    /// no recorded type).
    fn vec_var(&self, e: &Expr) -> bool {
        matches!(&e.kind, ExprKind::Var(id)
            if matches!(self.slots.get(&id.name), Some(Slot::Data { ty, .. }) if is_vec(ty)))
    }

    /// The header of a `Vec` expression that is read, not moved: a place's
    /// own header, or a temporary freed when the region ends.
    fn vec_hdr(&mut self, e: &Expr) -> String {
        let CVal::Ptr { ptr, .. } = self.expr(e) else {
            unreachable!("a Vec value")
        };
        if Self::fresh(e) {
            let t = self.ty(e);
            self.pending.push((ptr.clone(), value_ty(&t).clone()));
        }
        ptr
    }

    /// Whether `e` produces a value of its own (a call, a literal, a block,
    /// `Box::new(..)`), not a place or a part of one.
    fn fresh(e: &Expr) -> bool {
        !matches!(
            e.kind,
            ExprKind::Var(_)
                | ExprKind::Field { .. }
                | ExprKind::Deref(_)
                | ExprKind::Index { .. }
                | ExprKind::Ref { .. }
        )
    }

    /// A temporary `v` of type `t` from expression `e` that is read, not
    /// moved: dropped when the region ends, if it owns heap memory.
    fn read_temp(&mut self, e: &Expr, v: &CVal, t: &TypeKind) -> bool {
        if let CVal::Ptr { ptr, .. } = v
            && Self::fresh(e)
            && crate::check::is_move(t, &self.t.adts)
        {
            self.pending.push((ptr.clone(), t.clone()));
            return true;
        }
        false
    }

    /// Drops the value of type `t` at `ptr` if it owns heap memory.
    fn drop_at(&mut self, ptr: &str, t: &TypeKind) {
        let t = value_ty(t);
        if crate::check::is_move(t, &self.t.adts) {
            self.emit(format!("call void {}(ptr {ptr})", glue(t)));
        }
    }

    /// The address of the value in the `Box` whose cell is at `cell`.
    fn unbox(&mut self, cell: &str) -> String {
        let h = self.t();
        self.emit(format!("{h} = load ptr, ptr {cell}"));
        h
    }

    /// `v` read from memory that may be dropped before it is used: an
    /// array is copied out now (data values are copied by `detach`).
    fn keep(&mut self, v: CVal, t: &TypeKind) -> CVal {
        match v {
            CVal::Arr { ptr, len } => {
                let tmp = self.temp(t);
                self.copy(&tmp, &ptr, t);
                let _ = ptr;
                CVal::Arr { ptr: tmp, len }
            }
            v => v,
        }
    }

    /// A place of any type: its address, its type, and the register type
    /// when it is a scalar slot (a scalar local or reference parameter,
    /// held in register form; other scalars are in memory form).
    fn any_place(&mut self, e: &Expr) -> (String, TypeKind, Option<&'static str>) {
        let var = match &e.kind {
            ExprKind::Var(id) => Some(id),
            ExprKind::Deref(x) => match &x.kind {
                ExprKind::Var(id) if matches!(self.slots[&id.name], Slot::Scalar { .. }) => {
                    Some(id)
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(id) = var {
            let t = self
                .local_ty(&id.name)
                .map(|t| value_ty(&t).clone())
                .unwrap_or(TypeKind::Error);
            match self.slots[&id.name].clone() {
                Slot::Scalar { ptr, ty } => return (ptr, t, Some(ty)),
                Slot::Arr { ptr, .. } => return (ptr, t, None),
                Slot::Data { ptr, ty } => return (ptr, ty, None),
            }
        }
        let (p, t) = self.place(e);
        (p, t, None)
    }

    /// The value at a place from `any_place`.
    fn read_any(&mut self, p: &str, t: &TypeKind, reg: Option<&'static str>) -> CVal {
        match reg {
            Some(lt) => {
                let r = self.t();
                self.emit(format!("{r} = load {lt}, ptr {p}"));
                CVal::Imm(r)
            }
            None => self.read(p, t),
        }
    }

    /// Stores `v` at a place from `any_place` (an aggregate is copied).
    fn write_any(&mut self, p: &str, t: &TypeKind, reg: Option<&'static str>, v: CVal) {
        match reg {
            Some(lt) => self.emit(format!("store {lt} {}, ptr {p}", v.imm())),
            None => self.put(v, p, t),
        }
    }

    /// `std::mem::replace(&mut p, v)` and `std::mem::swap(&mut a, &mut b)`
    /// (ADR-0047 §2): values move between the places, so nothing is dropped.
    fn mem_call(&mut self, op: &str, args: &[Expr]) -> CVal {
        let inner = |a: &Expr| match &a.kind {
            ExprKind::Ref { expr, .. } => (**expr).clone(),
            _ => a.clone(),
        };
        let (p0, t, r0) = self.any_place(&inner(&args[0]));
        // The old value, in its own memory when it is an aggregate.
        let old = match r0 {
            Some(_) => self.read_any(&p0, &t, r0),
            None if is_scalar_ty(value_ty(&t)) => self.read(&p0, &t),
            None => {
                let tmp = self.temp(&t);
                self.copy(&tmp, &p0, &t);
                self.read(&tmp, &t)
            }
        };
        if op == "replace" {
            let v = self.expr(&args[1]);
            self.write_any(&p0, &t, r0, v);
            return old;
        }
        let (p1, t1, r1) = self.any_place(&inner(&args[1]));
        let v1 = self.read_any(&p1, &t1, r1);
        self.write_any(&p0, &t, r0, v1);
        self.write_any(&p1, &t1, r1, old);
        CVal::Unit
    }

    /// `Box::new(v)` (ADR-0047 §4): `v` is evaluated, then copied into
    /// memory from `malloc`; the value is the cell holding its address.
    fn box_new(&mut self, e: &Expr, args: &[Expr]) -> CVal {
        let t = self.ty(e);
        let TypeKind::Box(inner) = &t else {
            unreachable!("Box::new has a Box type")
        };
        let inner = inner.kind.clone();
        let v = self.expr(&args[0]);
        let h = self.t();
        self.emit(format!(
            "{h} = call ptr @bmb_box_alloc(i64 {})",
            size_of(&inner)
        ));
        self.put(v, &h, &inner);
        let cell = self.temp(&t);
        self.emit(format!("store ptr {h}, ptr {cell}"));
        CVal::Ptr { ptr: cell, ty: t }
    }

    /// Runs `f` as a region: temporaries it leaves are freed at its end. A
    /// data value pointing into a freed buffer is copied out first.
    fn region(&mut self, f: impl FnOnce(&mut Self) -> CVal) -> CVal {
        let mark = self.pending.len();
        let v = f(self);
        self.release(mark, v)
    }

    fn release(&mut self, mark: usize, v: CVal) -> CVal {
        if self.pending.len() == mark {
            return v;
        }
        let v = self.detach(v);
        for (h, t) in self.pending.split_off(mark) {
            self.drop_at(&h, &t);
        }
        v
    }

    /// A struct or enum value copied into its own memory.
    fn detach(&mut self, v: CVal) -> CVal {
        match v {
            CVal::Ptr { ptr, ty } if is_data(&ty) => {
                let tmp = self.temp(&ty);
                self.copy(&tmp, &ptr, &ty);
                CVal::Ptr { ptr: tmp, ty }
            }
            v => v,
        }
    }

    fn vec_new(&mut self, e: &Expr, args: &[Expr]) -> CVal {
        let t = self.ty(e);
        let el = vec_elem(&t);
        let hdr = self.temp(&t);
        let n = match args.first() {
            Some(a) => {
                let n = self.expr(a).imm();
                if self.proved("E0816", e) {
                    self.emit("; proved E0816");
                } else {
                    let c = self.t();
                    self.emit(format!("{c} = icmp ule i64 {n}, {}", vec_len_max(&el)));
                    self.check(&c, "the capacity may exceed the maximum length");
                }
                n
            }
            None => "0".to_string(),
        };
        self.emit(format!(
            "call void @bmb_vec_alloc(ptr {hdr}, i64 {n}, i64 {})",
            size_of(&el)
        ));
        CVal::Ptr { ptr: hdr, ty: t }
    }

    fn vec_method(&mut self, e: &Expr, recv: &Expr, m: &str, args: &[Expr]) -> CVal {
        let vt = self.ty(recv);
        let el = vec_elem(&vt);
        let hdr = self.vec_hdr(recv);
        match m {
            "len" => {
                let lp = self.vec_field(&hdr, 1);
                let r = self.t();
                self.emit(format!("{r} = load i64, ptr {lp}"));
                CVal::Imm(r)
            }
            "as_slice" | "as_mut_slice" => {
                let (ptr, len) = self.vec_view(&hdr);
                CVal::Arr { ptr, len }
            }
            "clone" => {
                let t = value_ty(&vt).clone();
                let dst = self.temp(&t);
                self.emit(format!(
                    "call void @bmb_vec_clone(ptr {dst}, ptr {hdr}, i64 {})",
                    size_of(&el)
                ));
                CVal::Ptr { ptr: dst, ty: t }
            }
            "truncate" => {
                let n = self.expr(&args[0]).imm();
                let lp = self.vec_field(&hdr, 1);
                let (len, r) = (self.t(), self.t());
                self.emit(format!("{len} = load i64, ptr {lp}"));
                self.emit(format!("{r} = call i64 @llvm.umin.i64(i64 {len}, i64 {n})"));
                if crate::check::is_move(&el, &self.t.adts) {
                    // The elements cut off are dropped, in order.
                    let buf = self.t();
                    self.emit(format!("{buf} = load ptr, ptr {hdr}"));
                    let i = self.alloca("i64");
                    self.emit(format!("store i64 {r}, ptr {i}"));
                    let (head, body, end) = (self.l("cut"), self.l("cutb"), self.l("cutd"));
                    self.emit(format!("br label %{head}"));
                    self.block(&head);
                    let (k, c) = (self.t(), self.t());
                    self.emit(format!("{k} = load i64, ptr {i}"));
                    self.emit(format!("{c} = icmp ult i64 {k}, {len}"));
                    self.emit(format!("br i1 {c}, label %{body}, label %{end}"));
                    self.block(&body);
                    let ep = self.t();
                    self.emit(format!(
                        "{ep} = getelementptr inbounds {}, ptr {buf}, i64 {k}",
                        mem_ty(&el)
                    ));
                    self.drop_at(&ep, &el);
                    let k2 = self.t();
                    self.emit(format!("{k2} = add nuw i64 {k}, 1"));
                    self.emit(format!("store i64 {k2}, ptr {i}"));
                    self.emit(format!("br label %{head}"));
                    self.block(&end);
                }
                self.emit(format!("store i64 {r}, ptr {lp}"));
                CVal::Unit
            }
            "push" => {
                // The value first: it may be an element of this `Vec`,
                // whose buffer the growth may move.
                let x = self.expr(&args[0]);
                let x = self.detach(x);
                let lp = self.vec_field(&hdr, 1);
                let cp = self.vec_field(&hdr, 2);
                let (len, cap, full) = (self.t(), self.t(), self.t());
                self.emit(format!("{len} = load i64, ptr {lp}"));
                if self.proved("E0816", e) {
                    self.emit("; proved E0816");
                } else {
                    let c = self.t();
                    self.emit(format!("{c} = icmp ult i64 {len}, {}", vec_len_max(&el)));
                    self.check(&c, "`push` may exceed the maximum length");
                }
                self.emit(format!("{cap} = load i64, ptr {cp}"));
                self.emit(format!("{full} = icmp eq i64 {len}, {cap}"));
                let grow = self.l("grow");
                let put = self.l("put");
                self.emit(format!("br i1 {full}, label %{grow}, label %{put}"));
                self.block(&grow);
                self.emit(format!(
                    "call void @bmb_vec_grow(ptr {hdr}, i64 {})",
                    size_of(&el)
                ));
                self.emit(format!("br label %{put}"));
                self.block(&put);
                let buf = self.t();
                self.emit(format!("{buf} = load ptr, ptr {hdr}"));
                let p = self.t();
                self.emit(format!(
                    "{p} = getelementptr inbounds {}, ptr {buf}, i64 {len}",
                    mem_ty(&el)
                ));
                self.put(x, &p, &el);
                let n = self.t();
                self.emit(format!("{n} = add nuw i64 {len}, 1"));
                self.emit(format!("store i64 {n}, ptr {lp}"));
                CVal::Unit
            }
            "pop" => {
                let ot = self.ty(e);
                let out = self.temp(&ot);
                let lp = self.vec_field(&hdr, 1);
                let (len, some) = (self.t(), self.t());
                self.emit(format!("{len} = load i64, ptr {lp}"));
                self.emit(format!("{some} = icmp ne i64 {len}, 0"));
                let tag = self.field_ptr(&out, &ot, 0);
                let k = self.t();
                self.emit(format!("{k} = zext i1 {some} to i64"));
                self.emit(format!("store i64 {k}, ptr {tag}"));
                let take = self.l("pop");
                let done = self.l("popped");
                self.emit(format!("br i1 {some}, label %{take}, label %{done}"));
                self.block(&take);
                let last = self.t();
                self.emit(format!("{last} = sub nuw i64 {len}, 1"));
                self.emit(format!("store i64 {last}, ptr {lp}"));
                let buf = self.t();
                self.emit(format!("{buf} = load ptr, ptr {hdr}"));
                let src = self.t();
                self.emit(format!(
                    "{src} = getelementptr inbounds {}, ptr {buf}, i64 {last}",
                    mem_ty(&el)
                ));
                let dst = self.t();
                self.emit(format!(
                    "{dst} = getelementptr inbounds {}, ptr {out}, i32 0, i32 1",
                    view_ty(&ot, "Some")
                ));
                self.copy(&dst, &src, &el);
                self.emit(format!("br label %{done}"));
                self.block(&done);
                CVal::Ptr { ptr: out, ty: ot }
            }
            _ => unreachable!("checked Vec method {m}"),
        }
    }

    /// Frees the `Vec` locals the move check left owned at the end of `b`.
    fn drop_block(&mut self, b: &Block) {
        let Some(names) = self
            .t
            .moves
            .get(&self.base)
            .and_then(|m| m.drops.get(&(b.span.file, b.span.lo, b.span.hi)))
            .cloned()
        else {
            return;
        };
        for n in names {
            self.drop_path(&n);
        }
    }

    /// Drops the local or field path `p` (`s`, `s.buf`, backlog 2O-03).
    fn drop_path(&mut self, p: &str) {
        let mut parts = p.split('.');
        let root = parts.next().unwrap_or(p);
        let (mut ptr, mut ty) = match self.slots.get(root).cloned() {
            Some(Slot::Data { ptr, ty }) => (ptr, ty),
            // An array of owned elements (backlog 2O-03c).
            Some(Slot::Arr { ptr, .. }) => match self.local_ty(root) {
                Some(t) => (ptr, value_ty(&t).clone()),
                None => return,
            },
            _ => return,
        };
        for f in parts {
            let (k, ft) = self.field_of(&ty, f);
            ptr = self.field_ptr(&ptr, &ty, k);
            ty = ft;
        }
        self.drop_at(&ptr, &ty);
    }

    /// The field paths still owned that an assignment at `span` drops
    /// (a partially moved place, backlog 2O-03).
    fn partial_replaces(&self, span: Span) -> Vec<String> {
        self.t
            .moves
            .get(&self.base)
            .and_then(|m| m.partial_replaces.get(&(span.file, span.lo, span.hi)))
            .cloned()
            .unwrap_or_default()
    }

    fn replaces(&self, span: Span) -> bool {
        self.t
            .moves
            .get(&self.base)
            .is_some_and(|m| m.replaces.contains(&(span.file, span.lo, span.hi)))
    }

    /// `Some(v)` when `ok` (an `i1`), else `None`, of `e`'s `Option` type.
    fn option(&mut self, e: &Expr, ok: &str, v: &str) -> CVal {
        let t = self.ty(e);
        let tmp = self.temp(&t);
        let tag = self.field_ptr(&tmp, &t, 0);
        let k = self.t();
        self.emit(format!("{k} = zext i1 {ok} to i64"));
        self.emit(format!("store i64 {k}, ptr {tag}"));
        let p = self.t();
        self.emit(format!(
            "{p} = getelementptr inbounds {}, ptr {tmp}, i32 0, i32 1",
            view_ty(&t, "Some")
        ));
        self.emit(format!("store i64 {v}, ptr {p}"));
        CVal::Ptr { ptr: tmp, ty: t }
    }

    /// A built-in integer method `a.op(b)` (ADR-0028 §6): `wrapping_*` is the
    /// plain instruction, `saturating_*` LLVM's saturating intrinsics (`mul`:
    /// the overflow flag selects the bound), `checked_*` the overflow flag or
    /// the divisor test as the `Option` tag. A division whose divisor fails
    /// the test divides by 1 instead, so it never traps.
    fn int_method(&mut self, e: &Expr, op: &str, a: &str, b: &str, signed: bool) -> CVal {
        let (kind, base) = op.split_once('_').expect("integer method");
        let s = if signed { "s" } else { "u" };
        let r = self.t();
        match (kind, base) {
            ("wrapping", _) => {
                self.emit(format!("{r} = {base} i64 {a}, {b}"));
                CVal::Imm(r)
            }
            ("saturating", "add" | "sub") => {
                self.emit(format!(
                    "{r} = call i64 @llvm.{s}{base}.sat.i64(i64 {a}, i64 {b})"
                ));
                CVal::Imm(r)
            }
            ("saturating", _) => {
                let (v, ovf) = self.with_overflow(s, base, a, b);
                let bound = if signed {
                    // The product is negative exactly when the signs differ.
                    let x = self.t();
                    let neg = self.t();
                    let m = self.t();
                    self.emit(format!("{x} = xor i64 {a}, {b}"));
                    self.emit(format!("{neg} = icmp slt i64 {x}, 0"));
                    self.emit(format!(
                        "{m} = select i1 {neg}, i64 -9223372036854775808, i64 9223372036854775807"
                    ));
                    m
                } else {
                    "-1".to_string()
                };
                self.emit(format!("{r} = select i1 {ovf}, i64 {bound}, i64 {v}"));
                CVal::Imm(r)
            }
            (_, "add" | "sub" | "mul") => {
                let (v, ovf) = self.with_overflow(s, base, a, b);
                self.emit(format!("{r} = xor i1 {ovf}, true"));
                self.option(e, &r, &v)
            }
            _ => {
                self.emit(format!("{r} = icmp ne i64 {b}, 0"));
                let ok = if signed {
                    // `i64::MIN / -1` overflows; `i64::MIN % -1` too, in Rust.
                    let (m, n, both, fine, ok) = (self.t(), self.t(), self.t(), self.t(), self.t());
                    self.emit(format!("{m} = icmp eq i64 {a}, -9223372036854775808"));
                    self.emit(format!("{n} = icmp eq i64 {b}, -1"));
                    self.emit(format!("{both} = and i1 {m}, {n}"));
                    self.emit(format!("{fine} = xor i1 {both}, true"));
                    self.emit(format!("{ok} = and i1 {r}, {fine}"));
                    ok
                } else {
                    r
                };
                let d = self.t();
                let q = self.t();
                let inst = if base == "div" { "div" } else { "rem" };
                self.emit(format!("{d} = select i1 {ok}, i64 {b}, i64 1"));
                self.emit(format!("{q} = {s}{inst} i64 {a}, {d}"));
                self.option(e, &ok, &q)
            }
        }
    }

    /// A `u8` method: the 64-bit result of `int_method` brought into the
    /// byte range (ADR-0044 §1). Operands are bytes, so a sum or product
    /// never leaves 64 bits, and a saturating or checked `sub` already
    /// stops at zero; wrapping keeps the low byte.
    fn byte_method(&mut self, e: &Expr, op: &str, r: CVal) -> CVal {
        let (kind, base) = op.split_once('_').expect("integer method");
        if kind != "wrapping" && matches!(base, "sub" | "div" | "rem") {
            return r;
        }
        match kind {
            "wrapping" => {
                let w = self.t();
                self.emit(format!("{w} = and i64 {}, 255", r.imm()));
                CVal::Imm(w)
            }
            "saturating" => {
                let w = self.t();
                self.emit(format!(
                    "{w} = call i64 @llvm.umin.i64(i64 {}, i64 255)",
                    r.imm()
                ));
                CVal::Imm(w)
            }
            // `checked_add`/`checked_mul`: `Some` exactly when the value is a byte.
            _ => {
                let CVal::Ptr { ptr, ty } = r else {
                    unreachable!("checked_* is an Option")
                };
                let p = self.t();
                self.emit(format!(
                    "{p} = getelementptr inbounds {}, ptr {ptr}, i32 0, i32 1",
                    view_ty(&ty, "Some")
                ));
                let (v, fits, tagp, tag, ok, k) =
                    (self.t(), self.t(), self.t(), self.t(), self.t(), self.t());
                self.emit(format!("{v} = load i64, ptr {p}"));
                self.emit(format!("{fits} = icmp ule i64 {v}, 255"));
                self.emit(format!(
                    "{tagp} = getelementptr inbounds {}, ptr {ptr}, i32 0, i32 0",
                    data_ty(&ty)
                ));
                self.emit(format!("{tag} = load i64, ptr {tagp}"));
                self.emit(format!("{ok} = icmp ne i64 {tag}, 0"));
                let both = self.t();
                self.emit(format!("{both} = and i1 {ok}, {fits}"));
                self.emit(format!("{k} = zext i1 {both} to i64"));
                self.emit(format!("store i64 {k}, ptr {tagp}"));
                let _ = e;
                CVal::Ptr { ptr, ty }
            }
        }
    }

    /// LLVM's `{s,u}op.with.overflow`: (result, overflow flag).
    fn with_overflow(&mut self, s: &str, op: &str, a: &str, b: &str) -> (String, String) {
        let (pair, v, ovf) = (self.t(), self.t(), self.t());
        self.emit(format!(
            "{pair} = call {{i64, i1}} @llvm.{s}{op}.with.overflow.i64(i64 {a}, i64 {b})"
        ));
        self.emit(format!("{v} = extractvalue {{i64, i1}} {pair}, 0"));
        self.emit(format!("{ovf} = extractvalue {{i64, i1}} {pair}, 1"));
        (v, ovf)
    }

    /// `add`/`sub`/`mul` with `nsw`/`nuw` when proved, else overflow-checked.
    fn arith(&mut self, e: &Expr, inst: &str, a: &str, b: &str, signed: bool) -> CVal {
        let r = self.t();
        if self.proved("E0803", e) {
            let flag = if signed { "nsw" } else { "nuw" };
            self.emit(format!("{r} = {inst} {flag} i64 {a}, {b} ; proved E0803"));
        } else {
            let s = if signed { "s" } else { "u" };
            let pair = self.t();
            let ovf = self.t();
            let ok = self.t();
            self.emit(format!(
                "{pair} = call {{i64, i1}} @llvm.{s}{inst}.with.overflow.i64(i64 {a}, i64 {b})"
            ));
            self.emit(format!("{ovf} = extractvalue {{i64, i1}} {pair}, 1"));
            self.emit(format!("{ok} = xor i1 {ovf}, true"));
            self.check(&ok, &format!("`{inst}` may overflow"));
            self.emit(format!("{r} = extractvalue {{i64, i1}} {pair}, 0"));
        }
        CVal::Imm(r)
    }

    /// Address of `base[index]` (bounds-checked unless proved) and the element type.
    fn elem_ptr(&mut self, e: &Expr, base: &Expr, index: &Expr) -> (String, &'static str) {
        // Assignment targets have no recorded type; the variable's slot knows it.
        let elem = match &base.kind {
            ExprKind::Var(id) => match &self.slots[&id.name] {
                Slot::Arr { elem, .. } => elem,
                Slot::Data { ty, .. } => elem_kind(&vec_elem(ty)),
                Slot::Scalar { .. } => unreachable!("indexing a scalar"),
            },
            _ => elem_kind(&elem_of(&self.ty(base))),
        };
        let CVal::Arr { ptr, len } = self.array_of(base) else {
            unreachable!()
        };
        let i = self.expr(index).imm();
        let note = if self.proved("E0805", e) {
            " ; proved E0805"
        } else {
            let c = self.t();
            self.emit(format!("{c} = icmp ult i64 {i}, {len}"));
            self.check(&c, "index may be out of bounds");
            ""
        };
        let p = self.t();
        self.emit(format!(
            "{p} = getelementptr inbounds {}, ptr {ptr}, i64 {i}{note}",
            mem(elem)
        ));
        (p, elem)
    }

    /// Fills the array of structs or enums of type `t` at `ptr` from an
    /// array literal or a repeat (2B-14). Every element is evaluated and
    /// copied out before the first store, as for scalar elements.
    fn init_data_array(&mut self, e: &Expr, ptr: &str, t: &TypeKind) {
        let TypeKind::Array(el, _) = t else {
            unreachable!("an array type")
        };
        let et = el.kind.clone();
        let at = |g: &mut Self, k: &str| {
            let p = g.t();
            g.emit(format!(
                "{p} = getelementptr inbounds {}, ptr {ptr}, i64 {k}",
                mem_ty(&et)
            ));
            p
        };
        match &e.kind {
            ExprKind::Array(items) => {
                let vals: Vec<CVal> = items
                    .iter()
                    .map(|it| {
                        let v = self.expr(it);
                        self.detach(v)
                    })
                    .collect();
                for (k, v) in vals.into_iter().enumerate() {
                    let p = at(self, &k.to_string());
                    self.put(v, &p, &et);
                }
            }
            ExprKind::Repeat { value, count, .. } => {
                let v = self.expr(value);
                let v = self.detach(v);
                let i = self.alloca("i64");
                self.emit(format!("store i64 0, ptr {i}"));
                let head = self.l("fill");
                let body = self.l("fillb");
                let end = self.l("filled");
                self.emit(format!("br label %{head}"));
                self.block(&head);
                let k = self.t();
                let c = self.t();
                self.emit(format!("{k} = load i64, ptr {i}"));
                self.emit(format!("{c} = icmp ult i64 {k}, {count}"));
                self.emit(format!("br i1 {c}, label %{body}, label %{end}"));
                self.block(&body);
                let p = at(self, &k);
                self.put(v, &p, &et);
                let k2 = self.t();
                self.emit(format!("{k2} = add nuw i64 {k}, 1"));
                self.emit(format!("store i64 {k2}, ptr {i}"));
                self.emit(format!("br label %{head}"));
                self.block(&end);
            }
            // Any other array value is copied whole.
            _ => {
                let v = self.expr(e);
                self.put(v, ptr, t);
            }
        }
    }

    fn init_array(&mut self, e: &Expr, ptr: &str, elem: &str) {
        match &e.kind {
            ExprKind::Array(items) => {
                // Evaluate every element before the first store: in
                // `a = [a[1], a[0]]` the right side reads the old `a`.
                let vals: Vec<String> = items.iter().map(|it| self.expr(it).imm()).collect();
                for (k, v) in vals.into_iter().enumerate() {
                    let p = self.t();
                    self.emit(format!(
                        "{p} = getelementptr inbounds {}, ptr {ptr}, i64 {k}",
                        mem(elem)
                    ));
                    self.store_elem(elem, &v, &p);
                }
            }
            ExprKind::Repeat { value, count, .. } => {
                let v = self.expr(value).imm();
                let i = self.alloca("i64");
                self.emit(format!("store i64 0, ptr {i}"));
                let head = self.l("fill");
                let body = self.l("fillb");
                let end = self.l("filled");
                self.emit(format!("br label %{head}"));
                self.block(&head);
                let k = self.t();
                let c = self.t();
                self.emit(format!("{k} = load i64, ptr {i}"));
                self.emit(format!("{c} = icmp ult i64 {k}, {count}"));
                self.emit(format!("br i1 {c}, label %{body}, label %{end}"));
                self.block(&body);
                let p = self.t();
                let k2 = self.t();
                self.emit(format!(
                    "{p} = getelementptr inbounds {}, ptr {ptr}, i64 {k}",
                    mem(elem)
                ));
                self.store_elem(elem, &v, &p);
                self.emit(format!("{k2} = add nuw i64 {k}, 1"));
                self.emit(format!("store i64 {k2}, ptr {i}"));
                self.emit(format!("br label %{head}"));
                self.block(&end);
            }
            _ => {
                // Copy from another array value.
                let CVal::Arr { ptr: src, len } = self.expr(e) else {
                    unreachable!()
                };
                let bytes = len.parse::<u64>().unwrap_or(0) * width(elem);
                self.emit(format!(
                    "call void @llvm.memcpy.p0.p0.i64(ptr {ptr}, ptr {src}, i64 {bytes}, i1 false)"
                ));
            }
        }
    }

    fn call(&mut self, path: &[Ident], args: &[Expr], e: &Expr) -> CVal {
        let mut target = self.resolve(path);
        // A generic callee: the instance for the call's type arguments.
        let mut csub: BTreeMap<String, TypeKind> = match (
            self.t.fns.get(&target),
            self.t.targs.get(&(e.span.file, e.span.lo, e.span.hi)),
        ) {
            (Some(sig), Some(targs)) => sig
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(targs.iter().map(|a| crate::check::subst(a, &self.tsub)))
                .collect(),
            _ => BTreeMap::new(),
        };
        // A trait method runs its impl's function for `Self` (ADR-0046 §5).
        if let Some(sig) = self.t.fns.get(&target)
            && sig.trait_of.is_some()
        {
            let targs: Vec<TypeKind> = sig.generics.iter().map(|g| csub[&g.name].clone()).collect();
            let (t, a) = crate::check::dispatch(&self.t.fns, &self.t.trait_impls, &target, &targs)
                .expect("a lowered trait call has an impl");
            csub = self.t.fns[&t]
                .generics
                .iter()
                .map(|g| g.name.clone())
                .zip(a)
                .collect();
            target = t;
        }
        let param_tys: Vec<TypeKind> = self
            .t
            .fns
            .get(&target)
            .map(|f| {
                f.params
                    .iter()
                    .map(|p| crate::check::subst(p, &csub))
                    .collect()
            })
            .unwrap_or_default();
        let mut flat = Vec::new();
        let mut write_back = Vec::new();
        for (k, a) in args.iter().enumerate() {
            let ty = self.ty(a);
            let by_value = matches!(param_tys.get(k), Some(TypeKind::Array(..)));
            if let Some(TypeKind::Ref { mutable, inner }) = param_tys.get(k)
                && is_scalar_ty(&inner.kind)
            {
                let ptr = self.scalar_ref(a, &inner.kind, *mutable, &mut write_back);
                flat.push(format!("ptr {ptr}"));
                continue;
            }
            match self.expr(a) {
                // A data value is passed as a private copy; a reference as
                // the address of its place.
                CVal::Ptr { ptr, ty }
                    if matches!(
                        param_tys.get(k),
                        Some(TypeKind::Named { .. } | TypeKind::Vec(_) | TypeKind::Box(_))
                    ) =>
                {
                    let tmp = self.temp(&ty);
                    self.copy(&tmp, &ptr, &ty);
                    flat.push(format!("ptr {tmp}"));
                }
                CVal::Ptr { ptr, .. } => flat.push(format!("ptr {ptr}")),
                CVal::Arr { ptr, .. } if by_value => flat.push(format!("ptr {ptr}")),
                CVal::Arr { ptr, len } => {
                    flat.push(format!("ptr {ptr}"));
                    flat.push(format!("i64 {len}"));
                }
                CVal::Imm(v) => flat.push(format!("{} {v}", llty(&ty))),
                CVal::Unit => {}
                // An argument that returns: the call is unreachable.
                CVal::Never => match param_tys.get(k) {
                    Some(t) if is_mem(t) || is_array(t) || matches!(t, TypeKind::Array(..)) => {
                        flat.push("ptr undef".into());
                        if is_array(t) && !by_value {
                            flat.push("i64 undef".into());
                        }
                    }
                    _ if llty(&ty) != "void" => flat.push(format!("{} undef", llty(&ty))),
                    _ => {}
                },
            }
        }
        let (callee, ret) = if matches!(target.as_str(), "print_i64" | "print_u64" | "print_bytes")
        {
            (format!("@bmb_{target}"), TypeKind::Unit)
        } else if csub.is_empty() {
            (fn_sym(&target), self.t.fns[&target].ret.clone())
        } else {
            let sig = &self.t.fns[&target];
            let args: Vec<TypeKind> = sig.generics.iter().map(|g| csub[&g.name].clone()).collect();
            let key = crate::verify::inst_key(&target, &args);
            let ret = crate::check::subst(&sig.ret, &csub);
            self.used.insert(ty_str(&ret), ret.clone());
            self.wants.push((key.clone(), target.clone(), csub));
            (fn_sym(&key), ret)
        };
        let v = if is_mem(&ret) || matches!(ret, TypeKind::Array(..)) {
            // A data or array result comes back through the caller's memory.
            let out = self.temp(&ret);
            flat.insert(0, format!("ptr {out}"));
            self.emit(format!("call void {callee}({})", flat.join(", ")));
            self.read(&out, &ret)
        } else if ret == TypeKind::Unit {
            self.emit(format!("call void {callee}({})", flat.join(", ")));
            CVal::Unit
        } else {
            let r = self.t();
            self.emit(format!(
                "{r} = call {} {callee}({})",
                llty(&ret),
                flat.join(", ")
            ));
            CVal::Imm(r)
        };
        for (tmp, back, t) in write_back {
            let x = self.t();
            self.emit(format!("{x} = load {}, ptr {tmp}", llty(&t)));
            match back {
                Back::Mem(place) => self.put(CVal::Imm(x), &place, &t),
                Back::Elem(elem, p) => self.store_elem(elem, &x, &p),
            }
        }
        v
    }

    /// The address passed for a scalar reference argument `a` (backlog
    /// 2T-08): a local's or a reference parameter's slot itself; for a
    /// field or an element (stored in memory form), a temporary holding
    /// its value, copied back after the call when `mutable` (references
    /// are second-class, so nothing else sees the place meanwhile).
    fn scalar_ref(
        &mut self,
        a: &Expr,
        t: &TypeKind,
        mutable: bool,
        write_back: &mut Vec<(String, Back, TypeKind)>,
    ) -> String {
        let inner = match &a.kind {
            ExprKind::Ref { expr, .. } => expr.as_ref(),
            _ => a,
        };
        if let ExprKind::Var(id) = &inner.kind
            && let Slot::Scalar { ptr, .. } = self.slots[&id.name].clone()
        {
            return ptr;
        }
        let tmp = self.alloca(llty(t));
        let place = match &inner.kind {
            ExprKind::Index { base, index }
                if matches!(base.kind, ExprKind::Var(_)) && !is_mem_elem(t) =>
            {
                let (p, ety) = self.elem_ptr(inner, base, index);
                let v = self.load_elem(ety, &p);
                self.emit(format!("store {} {v}, ptr {tmp}", llty(t)));
                if mutable {
                    write_back.push((tmp.clone(), Back::Elem(ety, p), t.clone()));
                }
                return tmp;
            }
            _ => self.place(inner).0,
        };
        let v = self.read(&place, t).imm();
        self.emit(format!("store {} {v}, ptr {tmp}", llty(t)));
        if mutable {
            write_back.push((tmp.clone(), Back::Mem(place), t.clone()));
        }
        tmp
    }

    // ---------- statements ----------

    fn block_expr(&mut self, b: &Block) -> CVal {
        for s in &b.stmts {
            let mark = self.pending.len();
            self.stmt(s);
            self.release(mark, CVal::Unit);
        }
        let v = match &b.tail {
            Some(t) => self.region(|g| g.expr(t)),
            None => CVal::Unit,
        };
        let freed = self
            .t
            .moves
            .get(&self.base)
            .is_some_and(|m| m.drops.contains_key(&(b.span.file, b.span.lo, b.span.hi)));
        if !freed {
            return v;
        }
        // The value may point into a buffer freed here.
        let v = self.detach(v);
        self.drop_block(b);
        v
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::LetFields { .. } => unreachable!("rejected by the checker (E0230)"),
            Stmt::Let { name, init, .. }
                if self.local_ty(&name.name).is_some_and(|t| is_mem(&t)) =>
            {
                let t = self.local_ty(&name.name).unwrap();
                let ptr = self.temp(&t);
                let v = self.expr(init);
                self.put(v, &ptr, &t);
                self.slots
                    .insert(name.name.clone(), Slot::Data { ptr, ty: t });
            }
            Stmt::Let { name, init, .. }
                if matches!(self.local_ty(&name.name),
                    Some(TypeKind::Array(el, _)) if is_mem_elem(&el.kind)) =>
            {
                let t = self.local_ty(&name.name).unwrap();
                let TypeKind::Array(el, n) = &t else {
                    unreachable!()
                };
                let ptr = self.temp(&t);
                self.init_data_array(init, &ptr, &t);
                self.slots.insert(
                    name.name.clone(),
                    Slot::Arr {
                        ptr,
                        len: n.to_string(),
                        elem: elem_kind(&el.kind),
                        ety: el.kind.clone(),
                    },
                );
            }
            Stmt::Let { name, ty, init, .. } => {
                match &self.local_ty(&name.name).unwrap_or(ty.kind.clone()) {
                    TypeKind::Array(el, n) => {
                        let elem = elem_kind(&el.kind);
                        let ptr = self.alloca(&format!("[{n} x {}]", mem(elem)));
                        self.init_array(init, &ptr, elem);
                        self.slots.insert(
                            name.name.clone(),
                            Slot::Arr {
                                ptr,
                                len: n.to_string(),
                                elem,
                                ety: el.kind.clone(),
                            },
                        );
                    }
                    t => {
                        let lt = llty(t);
                        let ptr = self.alloca(lt);
                        let v = self.expr(init).imm();
                        self.emit(format!("store {lt} {v}, ptr {ptr}"));
                        self.slots
                            .insert(name.name.clone(), Slot::Scalar { ptr, ty: lt });
                    }
                }
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => match &target.kind {
                // `*x = v` through a scalar reference parameter (2T-08).
                ExprKind::Deref(x) if matches!(&x.kind, ExprKind::Var(id) if matches!(self.slots[&id.name], Slot::Scalar { .. })) =>
                {
                    let ExprKind::Var(id) = &x.kind else {
                        unreachable!()
                    };
                    let Slot::Scalar { ptr, ty } = self.slots[&id.name].clone() else {
                        unreachable!()
                    };
                    let v = self.expr(value).imm();
                    self.emit(format!("store {ty} {v}, ptr {ptr}"));
                }
                ExprKind::Var(id) => match self.slots[&id.name].clone() {
                    Slot::Scalar { ptr, ty } => {
                        let v = self.expr(value).imm();
                        self.emit(format!("store {ty} {v}, ptr {ptr}"));
                    }
                    Slot::Arr { ptr, ety, .. } if is_mem_elem(&ety) => {
                        let t = self.local_ty(&id.name).unwrap();
                        self.init_data_array(value, &ptr, &t);
                    }
                    Slot::Arr { ptr, elem, .. } => self.init_array(value, &ptr, elem),
                    Slot::Data { ptr, ty } => {
                        let v = self.expr(value);
                        if self.replaces(*span) {
                            self.drop_at(&ptr, &ty);
                        }
                        for q in self.partial_replaces(*span) {
                            self.drop_path(&q);
                        }
                        self.put(v, &ptr, &ty);
                    }
                },
                ExprKind::Index { base, .. }
                    if match &base.kind {
                        ExprKind::Var(id) => match &self.slots[&id.name] {
                            Slot::Arr { ety, .. } => is_mem_elem(ety),
                            Slot::Data { ty, .. } => is_vec(ty) && is_mem_elem(&vec_elem(ty)),
                            Slot::Scalar { .. } => false,
                        },
                        // An array field (2B-14, 2B-15).
                        // An element of an array field or of an inner array.
                        _ => true,
                    } =>
                {
                    // Rust evaluates the value, then the place; an owned
                    // element replaced is dropped.
                    let v = self.expr(value);
                    let (p, t) = self.place(target);
                    if self.replaces(*span) {
                        self.drop_at(&p, &t);
                    }
                    self.put(v, &p, &t);
                }
                ExprKind::Index { base, index } => {
                    let (ptr, ty) = self.elem_ptr(target, base, index);
                    let v = self.expr(value).imm();
                    self.store_elem(ty, &v, &ptr);
                }
                _ => {
                    let v = self.expr(value);
                    let (p, t) = self.place(target);
                    if self.replaces(*span) {
                        self.drop_at(&p, &t);
                    }
                    for q in self.partial_replaces(*span) {
                        self.drop_path(&q);
                    }
                    self.put(v, &p, &t);
                }
            },
            Stmt::While {
                label, cond, body, ..
            } => {
                let head = self.l("while");
                let lb = self.l("body");
                let end = self.l("endwhile");
                self.emit(format!("br label %{head}"));
                self.block(&head);
                let c = self.region(|g| g.expr(cond)).imm();
                self.emit(format!("br i1 {c}, label %{lb}, label %{end}"));
                self.block(&lb);
                self.loops.push(GLoop {
                    label: label.as_ref().map(|l| l.name.clone()),
                    head: head.clone(),
                    end: end.clone(),
                    mark: self.pending.len(),
                    slot: None,
                    ty: TypeKind::Unit,
                });
                self.block_expr(body);
                self.loops.pop();
                self.emit(format!("br label %{head}"));
                self.block(&end);
            }
            // The bounds once, then `i` from `lo` while `i < hi` (`<=`);
            // `continue` goes to the step, which for `lo..=hi` leaves at
            // `i == hi` so that `i + 1` never passes the type's maximum.
            Stmt::For {
                label,
                var,
                lo,
                hi,
                inclusive,
                body,
                ..
            } => {
                let t = self.local_ty(&var.name).expect("checked `for` variable");
                let lt = llty(&t);
                let (lt_op, le_op) = if t.is_signed() {
                    ("slt", "sle")
                } else {
                    ("ult", "ule")
                };
                let ptr = self.alloca(lt);
                let a = self.region(|g| g.expr(lo)).imm();
                self.emit(format!("store {lt} {a}, ptr {ptr}"));
                let b = self.region(|g| g.expr(hi)).imm();
                self.slots.insert(
                    var.name.clone(),
                    Slot::Scalar {
                        ptr: ptr.clone(),
                        ty: lt,
                    },
                );
                let head = self.l("for");
                let lb = self.l("body");
                let next = self.l("next");
                let end = self.l("endfor");
                self.emit(format!("br label %{head}"));
                self.block(&head);
                let i = self.t();
                self.emit(format!("{i} = load {lt}, ptr {ptr}"));
                let c = self.t();
                let op = if *inclusive { le_op } else { lt_op };
                self.emit(format!("{c} = icmp {op} {lt} {i}, {b}"));
                self.emit(format!("br i1 {c}, label %{lb}, label %{end}"));
                self.block(&lb);
                self.loops.push(GLoop {
                    label: label.as_ref().map(|l| l.name.clone()),
                    head: next.clone(),
                    end: end.clone(),
                    mark: self.pending.len(),
                    slot: None,
                    ty: TypeKind::Unit,
                });
                self.block_expr(body);
                self.loops.pop();
                self.emit(format!("br label %{next}"));
                self.block(&next);
                let i = self.t();
                self.emit(format!("{i} = load {lt}, ptr {ptr}"));
                if *inclusive {
                    let at_end = self.t();
                    let step = self.l("step");
                    self.emit(format!("{at_end} = icmp eq {lt} {i}, {b}"));
                    self.emit(format!("br i1 {at_end}, label %{end}, label %{step}"));
                    self.block(&step);
                }
                let n = self.t();
                self.emit(format!("{n} = add {lt} {i}, 1"));
                self.emit(format!("store {lt} {n}, ptr {ptr}"));
                self.emit(format!("br label %{head}"));
                self.block(&end);
            }
            // Proof-only statement: no code.
            Stmt::Assert { .. } => {}
            // A discarded value is dropped (a local named here is moved).
            Stmt::Expr(e) => {
                let v = self.expr(e);
                if let CVal::Ptr { ptr, ty } = v {
                    self.drop_at(&ptr, &ty);
                }
            }
        }
    }
}

/// A function to lower: its symbol path, the definition, its unit, its
/// syntax, and the type arguments of an instance (ADR-0046 §5).
type Lowering<'p> = (
    String,
    String,
    String,
    &'p FnDef,
    BTreeMap<String, TypeKind>,
);

pub struct Module {
    pub ir: String,
    /// Runtime checks left in the IR because their obligation is unproved.
    pub checks: u32,
    pub main: Option<String>,
}

/// Emits one LLVM module for the whole program. The module is the whole
/// program (closed world, backlog 2D-07): every function, including the BMB
/// `main` and the print helper, has `internal` linkage, so LLVM may inline
/// and drop them freely. Only the C entry point `@main` is external.
pub fn emit(p: &Program, t: &Tables, proven: &BTreeSet<ObKey>) -> Module {
    emit_with(p, t, proven, None)
}

/// As `emit`, taking the IR of each definition from the incremental cache
/// when its key (source, callee signatures, usable proofs inside it) is
/// unchanged, and storing what it lowers (backlog 2D-10). Each definition's
/// IR depends only on those, so the module is the same either way.
pub fn emit_with(
    p: &Program,
    t: &Tables,
    proven: &BTreeSet<ObKey>,
    mut incr: Option<&mut crate::incremental::Session>,
) -> Module {
    let mut defs = crate::check::defs_of(p, &t.fns);
    defs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut ir = String::from(
        "; generated by bmb (ADR-0017 semantics; checks only where proofs are missing)\n\
         @bmb.fmt = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n\
         declare i32 @printf(ptr, ...)\n\
         declare void @llvm.trap() noreturn nounwind\n\
         declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)\n\
         declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)\n",
    );
    let types_at = ir.len();
    let uses_vec = all_types(t, &[]).into_iter().any(|x| holds_vec(x, t));
    let owns = |x: &TypeKind| crate::check::is_move(value_ty(x), &t.adts);
    let uses_heap =
        uses_vec || t.types.values().any(owns) || t.locals.values().any(|m| m.values().any(owns));
    if uses_heap {
        ir.push_str(HEAP_RUNTIME);
    }
    if uses_vec {
        ir.push_str(VEC_RUNTIME);
    }
    ir.push_str("declare i64 @llvm.umin.i64(i64, i64)\n");
    for s in ["s", "u"] {
        for op in ["add", "sub", "mul"] {
            let _ = writeln!(
                ir,
                "declare {{i64, i1}} @llvm.{s}{op}.with.overflow.i64(i64, i64)"
            );
        }
        for op in ["add", "sub"] {
            let _ = writeln!(ir, "declare i64 @llvm.{s}{op}.sat.i64(i64, i64)");
        }
    }
    ir.push_str(
        "define internal void @bmb_print_i64(i64 %x) {\n  call i32 (ptr, ...) @printf(ptr @bmb.fmt, i64 %x)\n  ret void\n}\n\
         @bmb.fmtu = private unnamed_addr constant [6 x i8] c\"%llu\\0A\\00\"\n\
         define internal void @bmb_print_u64(i64 %x) {\n  call i32 (ptr, ...) @printf(ptr @bmb.fmtu, i64 %x)\n  ret void\n}\n\
         @stdout = external global ptr\n\
         declare i64 @fwrite(ptr, i64, i64, ptr)\n\
         define internal void @bmb_print_bytes(ptr %p, i64 %n) {\n  %f = load ptr, ptr @stdout\n  call i64 @fwrite(ptr %p, i64 1, i64 %n, ptr %f)\n  ret void\n}\n",
    );
    let mut checks = 0;
    let mut main = None;
    // Generic functions are lowered per instance a lowered function calls
    // (ADR-0046 §5), after the definitions, in the order they are reached.
    let by_path: BTreeMap<String, (String, &FnDef)> = defs
        .iter()
        .map(|(p, m, f)| (p.clone(), (m.clone(), *f)))
        .collect();
    let mut queue: std::collections::VecDeque<Lowering> = defs
        .iter()
        .filter(|(p, _, f)| f.kind == FnKind::Exec && t.fns[p].generics.is_empty())
        .map(|(p, m, f)| (p.clone(), p.clone(), m.clone(), *f, BTreeMap::new()))
        .collect();
    let mut lowered: BTreeSet<String> = queue.iter().map(|q| q.0.clone()).collect();
    let mut used_types: BTreeMap<String, TypeKind> = BTreeMap::new();
    while let Some((path, base, mod_path, f, tsub)) = queue.pop_front() {
        let instance = !tsub.is_empty();
        // The entry point is a unit's `main`, not an `impl` function.
        if f.name.name == "main" && path == format!("{mod_path}::main") {
            main = Some(path.clone());
        }
        let usable: Vec<ObKey> = if incr.is_some() {
            proven
                .iter()
                .filter(|(_, file, lo, hi)| {
                    *file == f.span.file && f.span.lo <= *lo && *hi <= f.span.hi
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        // An instance is stored under its own key (backlog 2T-04), with
        // the instances it calls and the types it uses.
        let targs: Vec<TypeKind> = t.fns[&base]
            .generics
            .iter()
            .map(|g| tsub.get(&g.name).cloned().unwrap_or(TypeKind::Error))
            .collect();
        let inst: crate::incremental::Instance = instance.then_some((path.as_str(), &targs[..]));
        let hit = incr
            .as_ref()
            .and_then(|s| s.ir_lookup(&base, inst, f.span, &usable));
        if let Some(h) = &hit
            && !incr.as_ref().is_some_and(|s| s.audit())
        {
            checks += h.checks;
            ir.push_str(&h.ir);
            if let Some(s) = incr.as_mut() {
                s.stats.ir_reused.push(path.clone());
            }
            used_types.extend(h.deps.used.iter().map(|t| (ty_str(t), t.clone())));
            for (key, callee, csub) in h.deps.wants.clone() {
                if lowered.insert(key.clone()) {
                    let (m, cf) = &by_path[&callee];
                    queue.push_back((key, callee, m.clone(), *cf, csub));
                }
            }
            continue;
        }
        let mut g = Gen {
            t,
            proven,
            path: path.clone(),
            base: base.clone(),
            tsub,
            wants: Vec::new(),
            used: BTreeMap::new(),
            mod_path,
            out: String::new(),
            allocas: String::new(),
            tmp: 0,
            label: 0,
            slots: BTreeMap::new(),
            checks: 0,
            pending: Vec::new(),
            globals: String::new(),
            ret: TypeKind::Unit,
            loops: Vec::new(),
        };
        let mut sig = t.fns[&base].clone();
        sig.params = sig.params.iter().map(|p| g.inst(p)).collect();
        sig.ret = g.inst(&sig.ret);
        g.ret = sig.ret.clone();
        let mut params = Vec::new();
        for (prm, pty) in f.params.iter().zip(&sig.params) {
            let n = &prm.name.name;
            match pty {
                // `&T` / `&mut T` of a struct, an enum or a `Vec`: its address.
                TypeKind::Ref { mutable, .. } if is_mem(pty) && !is_array(pty) => {
                    let attr = if *mutable { "noalias " } else { "readonly " };
                    params.push(format!("ptr {attr}%\"{n}.ptr\""));
                    g.slots.insert(
                        n.clone(),
                        Slot::Data {
                            ptr: format!("%\"{n}.ptr\""),
                            ty: value_ty(pty).clone(),
                        },
                    );
                }
                // `&T` / `&mut T` of a scalar (backlog 2T-08): the
                // caller's slot, read and written through as the local's.
                TypeKind::Ref { mutable, inner } if is_scalar_ty(&inner.kind) => {
                    let attr = if *mutable { "noalias " } else { "readonly " };
                    params.push(format!("ptr {attr}%\"{n}.ptr\""));
                    g.slots.insert(
                        n.clone(),
                        Slot::Scalar {
                            ptr: format!("%\"{n}.ptr\""),
                            ty: llty(&inner.kind),
                        },
                    );
                }
                TypeKind::Ref { mutable, .. } => {
                    let attr = if *mutable { "noalias " } else { "readonly " };
                    params.push(format!("ptr {attr}%\"{n}.ptr\""));
                    params.push(format!("i64 %\"{n}.len\""));
                    let elem = elem_kind(&elem_of(pty));
                    g.slots.insert(
                        n.clone(),
                        Slot::Arr {
                            ptr: format!("%\"{n}.ptr\""),
                            len: format!("%\"{n}.len\""),
                            elem,
                            ety: elem_of(pty),
                        },
                    );
                }
                // A data value: the caller passes a private copy (of a
                // `Vec`, its header: the callee owns the buffer).
                _ if is_mem(pty) => {
                    params.push(format!("ptr readonly %\"{n}.in\""));
                    g.slots.insert(
                        n.clone(),
                        Slot::Data {
                            ptr: format!("%\"{n}.in\""),
                            ty: pty.clone(),
                        },
                    );
                }
                TypeKind::Array(el, len) => {
                    // Arrays by value: copied into a local.
                    let elem = elem_kind(&el.kind);
                    params.push(format!("ptr %\"{n}.in\""));
                    let ptr = g.temp(pty);
                    g.copy(&ptr, &format!("%\"{n}.in\""), pty);
                    g.slots.insert(
                        n.clone(),
                        Slot::Arr {
                            ptr,
                            len: len.to_string(),
                            elem,
                            ety: el.kind.clone(),
                        },
                    );
                }
                ty => {
                    let lt = llty(ty);
                    params.push(format!("{lt} %\"{n}\""));
                    let ptr = g.alloca(lt);
                    g.emit(format!("store {lt} %\"{n}\", ptr {ptr}"));
                    g.slots.insert(n.clone(), Slot::Scalar { ptr, ty: lt });
                }
            }
        }
        let rv = g.block_expr(&f.body);
        g.emit_return(rv);
        let ret = if g.ret_in_memory() {
            // A data or array result is written to the caller's memory.
            params.insert(0, "ptr noalias %\"ret.out\"".into());
            "void"
        } else {
            llty(&sig.ret)
        };
        checks += g.checks;
        let frag = format!(
            "\n{}define internal {ret} {}({}) {{\nentry:\n{}{}}}\n",
            g.globals,
            fn_sym(&path),
            params.join(", "),
            g.allocas,
            g.out
        );
        let deps = crate::incremental::IrDeps {
            wants: g.wants.clone(),
            used: g.used.values().cloned().collect(),
        };
        if let Some(s) = incr.as_mut() {
            s.stats.lowered.push(path.clone());
            if let Some(h) = &hit
                && (h.ir != frag
                    || h.checks != g.checks
                    || h.deps.wants != deps.wants
                    || h.deps.used != deps.used)
            {
                s.stats.audit_mismatches.push(path.clone());
            }
            s.ir_record(&base, inst, f.span, &usable, &frag, g.checks, &deps);
        }
        ir.push_str(&frag);
        used_types.extend(g.used);
        for (key, callee, csub) in g.wants {
            if lowered.insert(key.clone()) {
                let (m, cf) = &by_path[&callee];
                queue.push_back((key, callee, m.clone(), *cf, csub));
            }
        }
    }
    let extra: Vec<TypeKind> = used_types.into_values().collect();
    let glue = drop_glue(t, &extra);
    // The `Vec` header first: data types and glue use it.
    let header = if uses_vec {
        "%bmb.vec = type { ptr, i64, i64 }\n"
    } else {
        ""
    };
    ir.insert_str(types_at, &format!("{header}{}{glue}", type_defs(t, &extra)));
    if let Some(m) = &main {
        let _ = write!(
            ir,
            "\ndefine i32 @main() {{\n  call void {}()\n  ret i32 0\n}}\n",
            fn_sym(m)
        );
    }
    Module { ir, checks, main }
}
