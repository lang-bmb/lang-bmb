//! LLVM text IR generation (backlog 1C-01, 1C-03, 1C-04).
//!
//! Every implicit proof obligation (overflow, division, bounds) has a runtime
//! check here unless the proof gate proved that very obligation. Proved
//! arithmetic carries `nsw`/`nuw`, proved indexing uses `inbounds` without a
//! check, and every `&mut` parameter is `noalias` (ADR-0015). Structs and
//! enums (ADR-0026) live in memory as named LLVM types and are copied where
//! Rust copies them (backlog 2B-06). Unproved
//! obligations keep a check that traps; `bmb build` refuses such programs
//! anyway, so this matters only for `bmb emit-llvm` on partial proofs.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::ast::*;
use crate::check::ty_str;
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
}

impl CVal {
    fn imm(&self) -> String {
        match self {
            CVal::Imm(s) => s.clone(),
            CVal::Unit => "0".into(),
            CVal::Arr { .. } | CVal::Ptr { .. } => panic!("aggregate used as scalar"),
        }
    }
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
        TypeKind::Bool => "i8".into(),
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
        TypeKind::Array(e, n) => {
            let (s, a) = layout(&e.kind, tb);
            (s * n, a)
        }
        TypeKind::Named { .. } => {
            if let Some(fs) = struct_fields(t, tb) {
                return fields(fs.iter().map(|(_, ft)| ft), tb);
            }
            let vs = enum_variants(t, tb).unwrap_or_default();
            let tag = TypeKind::I64;
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
fn type_defs(tb: &Tables) -> String {
    fn visit(t: &TypeKind, tb: &Tables, seen: &mut BTreeSet<String>, out: &mut String) {
        match t {
            TypeKind::Array(e, _) | TypeKind::Slice(e) => return visit(&e.kind, tb, seen, out),
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
            for (_, ft) in fs {
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
    let mut all: Vec<&TypeKind> = tb.types.values().collect();
    all.extend(tb.locals.values().flat_map(|m| m.values()));
    for sig in tb.fns.values() {
        all.extend(sig.params.iter());
        all.push(&sig.ret);
    }
    let mut seen = BTreeSet::new();
    let mut out = String::new();
    for t in all {
        visit(t, tb, &mut seen, &mut out);
    }
    out
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

/// In-memory type of an array element: `bool` is a byte, as in Rust and C,
/// so LLVM recognizes fills and vectorizes loads (backlog 1E-09).
fn mem(elem: &str) -> &str {
    if elem == "i1" { "i8" } else { elem }
}

/// ABI alignment of an array element in memory (`&[i64]` is 8-aligned, as in Rust).
fn align(elem: &str) -> u32 {
    if elem == "i1" { 1 } else { 8 }
}

fn fn_sym(path: &str) -> String {
    format!("@\"{path}\"")
}

struct Gen<'p> {
    t: &'p Tables,
    proven: &'p BTreeSet<ObKey>,
    /// The definition being lowered (its canonical local types).
    path: String,
    mod_path: String,
    out: String,
    allocas: String,
    tmp: u32,
    label: u32,
    slots: BTreeMap<String, Slot>,
    /// Runtime checks emitted (unproved obligations), for reporting.
    checks: u32,
}

impl<'p> Gen<'p> {
    fn ty(&self, e: &Expr) -> TypeKind {
        self.t
            .types
            .get(&(e.span.file, e.span.lo, e.span.hi))
            .cloned()
            .unwrap_or(TypeKind::Error)
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
        if elem != "i1" {
            return r;
        }
        let b = self.t();
        self.emit(format!("{b} = trunc i8 {r} to i1"));
        b
    }

    fn store_elem(&mut self, elem: &str, v: &str, ptr: &str) {
        if elem == "i1" {
            let z = self.t();
            self.emit(format!("{z} = zext i1 {v} to i8"));
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
            Some(sig) if path.len() == 1 || (path.len() == 2 && sig.in_impl) => local,
            _ => name,
        }
    }

    /// Whether a call builds a variant (`Some(x)`, `E::V(..)`): it names no
    /// function.
    fn is_variant(&self, path: &[Ident]) -> bool {
        let target = self.resolve(path);
        target != "print_i64" && !self.t.fns.contains_key(&target)
    }

    // ---------- data values in memory ----------

    fn temp(&mut self, t: &TypeKind) -> String {
        self.alloca(&mem_ty(t))
    }

    /// Loads a scalar of type `t` from data memory.
    fn load_mem(&mut self, ptr: &str, t: &TypeKind) -> String {
        let r = self.t();
        if *t == TypeKind::Bool {
            self.emit(format!("{r} = load i8, ptr {ptr}"));
            let b = self.t();
            self.emit(format!("{b} = trunc i8 {r} to i1"));
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
            CVal::Imm(x) if *value_ty(t) == TypeKind::Bool => {
                let z = self.t();
                self.emit(format!("{z} = zext i1 {x} to i8"));
                self.emit(format!("store i8 {z}, ptr {ptr}"));
            }
            CVal::Imm(x) => self.emit(format!("store i64 {x}, ptr {ptr}")),
            CVal::Ptr { ptr: src, .. } | CVal::Arr { ptr: src, .. } => self.copy(ptr, &src, t),
            CVal::Unit => {}
        }
    }

    /// The value of type `t` at `ptr`: a scalar is loaded, an array or a
    /// data value is its address.
    fn read(&mut self, ptr: &str, t: &TypeKind) -> CVal {
        match value_ty(t) {
            TypeKind::Named { .. } => CVal::Ptr {
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
            ExprKind::Deref(x) | ExprKind::Ref { expr: x, .. } => self.place(x),
            ExprKind::Field { base, name } => {
                let (bp, bt) = self.place(base);
                let (k, ft) = self.field_of(&bt, &name.name);
                (self.field_ptr(&bp, &bt, k), ft)
            }
            ExprKind::Index { base, index } => {
                let ExprKind::Var(id) = &base.kind else {
                    unreachable!("indexing a named slice")
                };
                let Slot::Arr { ptr, len, ety, .. } = self.slots[&id.name].clone() else {
                    unreachable!()
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
            data_ty(ety)
        ));
        p
    }

    /// A `match`: the tag selects the arm; each binding is a copy of its
    /// payload position; the arms' values meet in one slot.
    fn match_expr(&mut self, e: &Expr, scrut: &Expr, arms: &[Arm]) -> CVal {
        let rt = self.ty(e);
        let slot = self.result_slot(&rt);
        let CVal::Ptr { ptr: sp, ty: st } = self.expr(scrut) else {
            unreachable!("match on an enum value")
        };
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
            for (j, (b, pt)) in a.pat.binds.iter().zip(&ps).enumerate() {
                let Some(id) = b else { continue };
                let fp = self.t();
                self.emit(format!(
                    "{fp} = getelementptr inbounds {}, ptr {sp}, i32 0, i32 {}",
                    view_ty(&st, &v),
                    j + 1
                ));
                self.bind(&id.name, &fp, pt);
            }
            let v = self.expr(&a.body);
            self.store_result(&slot, v, &rt);
            self.emit(format!("br label %{end}"));
        }
        self.block(&end);
        self.load_result(slot, &rt)
    }

    /// A new local `name` holding a copy of the value of type `t` at `src`.
    fn bind(&mut self, name: &str, src: &str, t: &TypeKind) {
        let slot = match value_ty(t) {
            TypeKind::Named { .. } => {
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
                    elem: llty(&e.kind),
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
            t if is_data(t) => Some(self.temp(t)),
            t => Some(self.alloca(llty(t))),
        }
    }

    fn store_result(&mut self, slot: &Option<String>, v: CVal, t: &TypeKind) {
        match slot {
            Some(s) if is_data(t) => self.put(v, s, t),
            Some(s) => self.emit(format!("store {} {}, ptr {s}", llty(t), v.imm())),
            None => {}
        }
    }

    fn load_result(&mut self, slot: Option<String>, t: &TypeKind) -> CVal {
        match slot {
            Some(s) if is_data(t) => CVal::Ptr {
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

    // ---------- expressions ----------

    fn expr(&mut self, e: &Expr) -> CVal {
        match &e.kind {
            ExprKind::Int(v) => CVal::Imm(v.to_string()),
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
                let rv = self.expr(r).imm();
                self.emit(format!("store i1 {rv}, ptr {slot}"));
                self.emit(format!("br label %{end}"));
                self.block(&end);
                let out = self.t();
                self.emit(format!("{out} = load i1, ptr {slot}"));
                CVal::Imm(out)
            }
            ExprKind::Binary(op, l, r) => {
                let lt = self.ty(l);
                let signed = lt == TypeKind::I64;
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
            ExprKind::Index { base, index } if is_data(&self.ty(e)) => {
                let ety = self.ty(e);
                let CVal::Arr { ptr, len } = self.expr(base) else {
                    unreachable!()
                };
                let i = self.expr(index).imm();
                let p = self.data_elem(e, &ptr, &len, &i, &ety);
                CVal::Ptr { ptr: p, ty: ety }
            }
            ExprKind::Index { base, index } => {
                let (ptr, ty) = self.elem_ptr(e, base, index);
                CVal::Imm(self.load_elem(ty, &ptr))
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
                        let inst = match op {
                            "wrapping_add" => "add",
                            "wrapping_sub" => "sub",
                            _ => "mul",
                        };
                        let r = self.t();
                        self.emit(format!("{r} = {inst} i64 {}, {b}", rv.imm()));
                        CVal::Imm(r)
                    }
                }
            }
            ExprKind::Call { path, args } if self.is_variant(path) => {
                let t = self.ty(e);
                self.variant(&t, &path.last().unwrap().name, args)
            }
            ExprKind::Call { path, args } => self.call(path, args, e),
            ExprKind::If { cond, then, els } if matches!(self.ty(e), TypeKind::Array(..)) => {
                // An array-valued `if`: each branch copies its array into
                // one buffer (backlog 1B-25).
                let TypeKind::Array(el, n) = self.ty(e) else {
                    unreachable!()
                };
                let elem = llty(&el.kind);
                let buf = self.alloca(&format!("[{n} x {}]", mem(elem)));
                let bytes = if elem == "i1" { n } else { n * 8 };
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
                    let CVal::Arr { ptr: src, .. } = v else {
                        unreachable!()
                    };
                    self.emit(format!(
                        "call void @llvm.memcpy.p0.p0.i64(ptr {buf}, ptr {src}, i64 {bytes}, i1 false)"
                    ));
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
            ExprKind::Ref { expr, .. } => self.expr(expr),
            ExprKind::Array(_) | ExprKind::Repeat { .. } => {
                let ty = self.ty(e);
                let TypeKind::Array(el, n) = &ty else {
                    unreachable!()
                };
                let elem = llty(&el.kind);
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
            ExprKind::Field { base, name } => {
                let CVal::Ptr { ptr, ty } = self.expr(base) else {
                    unreachable!("field of a struct value")
                };
                let (k, ft) = self.field_of(&ty, &name.name);
                let p = self.field_ptr(&ptr, &ty, k);
                self.read(&p, &ft)
            }
            // A reference denotes its value (references are second-class).
            ExprKind::Deref(x) => self.expr(x),
            ExprKind::Path(segs) => {
                let t = self.ty(e);
                self.variant(&t, &segs.last().unwrap().name, &[])
            }
            ExprKind::Match { scrut, arms } => self.match_expr(e, scrut, arms),
        }
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
                Slot::Scalar { .. } | Slot::Data { .. } => unreachable!("indexing a scalar"),
            },
            _ => llty(&elem_of(&self.ty(base))),
        };
        let CVal::Arr { ptr, len } = self.expr(base) else {
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
                let bytes = if elem == "i1" {
                    len
                } else {
                    format!("{}", len.parse::<u64>().unwrap_or(0) * 8)
                };
                self.emit(format!(
                    "call void @llvm.memcpy.p0.p0.i64(ptr {ptr}, ptr {src}, i64 {bytes}, i1 false)"
                ));
            }
        }
    }

    fn call(&mut self, path: &[Ident], args: &[Expr], e: &Expr) -> CVal {
        let target = self.resolve(path);
        let param_tys = self
            .t
            .fns
            .get(&target)
            .map(|f| f.params.clone())
            .unwrap_or_default();
        let mut flat = Vec::new();
        for (k, a) in args.iter().enumerate() {
            let ty = self.ty(a);
            let by_value = matches!(param_tys.get(k), Some(TypeKind::Array(..)));
            match self.expr(a) {
                // A data value is passed as a private copy; a reference as
                // the address of its place.
                CVal::Ptr { ptr, ty }
                    if matches!(param_tys.get(k), Some(TypeKind::Named { .. })) =>
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
            }
        }
        let (callee, ret) = if target == "print_i64" {
            ("@bmb_print_i64".to_string(), TypeKind::Unit)
        } else {
            (fn_sym(&target), self.t.fns[&target].ret.clone())
        };
        let _ = e;
        if is_data(&ret) {
            // A data result comes back through the caller's memory.
            let out = self.temp(&ret);
            flat.insert(0, format!("ptr {out}"));
            self.emit(format!("call void {callee}({})", flat.join(", ")));
            return CVal::Ptr { ptr: out, ty: ret };
        }
        if ret == TypeKind::Unit {
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
        }
    }

    // ---------- statements ----------

    fn block_expr(&mut self, b: &Block) -> CVal {
        for s in &b.stmts {
            self.stmt(s);
        }
        match &b.tail {
            Some(t) => self.expr(t),
            None => CVal::Unit,
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::LetFields { .. } => unreachable!("rejected by the checker (E0230)"),
            Stmt::Let { name, init, .. }
                if self
                    .t
                    .locals
                    .get(&self.path)
                    .and_then(|m| m.get(&name.name))
                    .is_some_and(is_data) =>
            {
                let t = self.t.locals[&self.path][&name.name].clone();
                let ptr = self.temp(&t);
                let v = self.expr(init);
                self.put(v, &ptr, &t);
                self.slots
                    .insert(name.name.clone(), Slot::Data { ptr, ty: t });
            }
            Stmt::Let { name, ty, init, .. } => match &ty.kind {
                TypeKind::Array(el, n) => {
                    let elem = llty(&el.kind);
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
            },
            Stmt::Assign { target, value, .. } => match &target.kind {
                ExprKind::Var(id) => match self.slots[&id.name].clone() {
                    Slot::Scalar { ptr, ty } => {
                        let v = self.expr(value).imm();
                        self.emit(format!("store {ty} {v}, ptr {ptr}"));
                    }
                    Slot::Arr { ptr, elem, .. } => self.init_array(value, &ptr, elem),
                    Slot::Data { ptr, ty } => {
                        let v = self.expr(value);
                        self.put(v, &ptr, &ty);
                    }
                },
                ExprKind::Index { base, .. }
                    if matches!(&base.kind, ExprKind::Var(id)
                        if matches!(&self.slots[&id.name], Slot::Arr { ety, .. } if is_data(ety))) =>
                {
                    // Rust evaluates the value, then the place.
                    let v = self.expr(value);
                    let (p, t) = self.place(target);
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
                    self.put(v, &p, &t);
                }
            },
            Stmt::While { cond, body, .. } => {
                let head = self.l("while");
                let lb = self.l("body");
                let end = self.l("endwhile");
                self.emit(format!("br label %{head}"));
                self.block(&head);
                let c = self.expr(cond).imm();
                self.emit(format!("br i1 {c}, label %{lb}, label %{end}"));
                self.block(&lb);
                self.block_expr(body);
                self.emit(format!("br label %{head}"));
                self.block(&end);
            }
            // Proof-only statement: no code.
            Stmt::Assert { .. } => {}
            Stmt::Expr(e) => {
                self.expr(e);
            }
        }
    }
}

fn collect<'p>(m: &'p Mod, prefix: &str, out: &mut Vec<(String, String, &'p FnDef)>) {
    let mp = format!("{prefix}{}", m.name.name);
    for s in &m.mods {
        collect(s, &format!("{mp}::"), out);
    }
    for f in &m.fns {
        out.push((format!("{mp}::{}", f.name.name), mp.clone(), f));
    }
    for i in &m.impls {
        for f in &i.fns {
            out.push((
                format!("{mp}::{}::{}", i.ty.name, f.name.name),
                mp.clone(),
                f,
            ));
        }
    }
}

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
    let mut defs = Vec::new();
    for m in &p.mods {
        collect(m, "", &mut defs);
    }
    defs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut ir = String::from(
        "; generated by bmb (ADR-0017 semantics; checks only where proofs are missing)\n\
         @bmb.fmt = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n\
         declare i32 @printf(ptr, ...)\n\
         declare void @llvm.trap() noreturn nounwind\n\
         declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)\n\
         declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)\n",
    );
    ir.push_str(&type_defs(t));
    for s in ["s", "u"] {
        for op in ["add", "sub", "mul"] {
            let _ = writeln!(
                ir,
                "declare {{i64, i1}} @llvm.{s}{op}.with.overflow.i64(i64, i64)"
            );
        }
    }
    ir.push_str(
        "define internal void @bmb_print_i64(i64 %x) {\n  call i32 (ptr, ...) @printf(ptr @bmb.fmt, i64 %x)\n  ret void\n}\n",
    );
    let mut checks = 0;
    let mut main = None;
    for (path, mod_path, f) in defs {
        if f.kind != FnKind::Exec {
            continue;
        }
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
        let hit = incr
            .as_ref()
            .and_then(|s| s.ir_lookup(&path, f.span, &usable));
        if let Some((frag, c)) = &hit
            && !incr.as_ref().is_some_and(|s| s.audit())
        {
            checks += c;
            ir.push_str(frag);
            if let Some(s) = incr.as_mut() {
                s.stats.ir_reused.push(path.clone());
            }
            continue;
        }
        let mut g = Gen {
            t,
            proven,
            path: path.clone(),
            mod_path,
            out: String::new(),
            allocas: String::new(),
            tmp: 0,
            label: 0,
            slots: BTreeMap::new(),
            checks: 0,
        };
        let sig = t.fns[&path].clone();
        let mut params = Vec::new();
        for (prm, pty) in f.params.iter().zip(&sig.params) {
            let n = &prm.name.name;
            match &prm.ty.kind {
                // `&T` / `&mut T` of a struct or enum: its address.
                TypeKind::Ref { mutable, .. } if is_data(pty) && !is_array(pty) => {
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
                TypeKind::Ref { mutable, .. } => {
                    let attr = if *mutable { "noalias " } else { "readonly " };
                    params.push(format!("ptr {attr}%\"{n}.ptr\""));
                    params.push(format!("i64 %\"{n}.len\""));
                    let elem = llty(&elem_of(&prm.ty.kind));
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
                // A data value: the caller passes a private copy.
                _ if is_data(pty) => {
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
                    let elem = llty(&el.kind);
                    params.push(format!("ptr %\"{n}.in\""));
                    let ptr = g.alloca(&format!("[{len} x {}]", mem(elem)));
                    let bytes = if elem == "i1" { *len } else { len * 8 };
                    g.emit(format!(
                        "call void @llvm.memcpy.p0.p0.i64(ptr {ptr}, ptr %\"{n}.in\", i64 {bytes}, i1 false)"
                    ));
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
        let ret = if is_data(&sig.ret) {
            // A data result is written to the caller's memory.
            params.insert(0, "ptr noalias %\"ret.out\"".into());
            g.put(rv, "%\"ret.out\"", &sig.ret);
            g.emit("ret void");
            "void"
        } else {
            let ret = llty(&f.ret.ty.kind);
            if ret == "void" {
                g.emit("ret void");
            } else {
                g.emit(format!("ret {ret} {}", rv.imm()));
            }
            ret
        };
        checks += g.checks;
        let frag = format!(
            "\ndefine internal {ret} {}({}) {{\nentry:\n{}{}}}\n",
            fn_sym(&path),
            params.join(", "),
            g.allocas,
            g.out
        );
        if let Some(s) = incr.as_mut() {
            s.stats.lowered.push(path.clone());
            if let Some((old, c)) = &hit
                && (old != &frag || *c != g.checks)
            {
                s.stats.audit_mismatches.push(path.clone());
            }
            s.ir_record(&path, f.span, &usable, &frag, g.checks);
        }
        ir.push_str(&frag);
    }
    if let Some(m) = &main {
        let _ = write!(
            ir,
            "\ndefine i32 @main() {{\n  call void {}()\n  ret i32 0\n}}\n",
            fn_sym(m)
        );
    }
    Module { ir, checks, main }
}
