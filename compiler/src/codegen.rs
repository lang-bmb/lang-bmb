//! LLVM text IR generation (backlog 1C-01, 1C-03, 1C-04).
//!
//! Every implicit proof obligation (overflow, division, bounds) has a runtime
//! check here unless the proof gate proved that very obligation. Proved
//! arithmetic carries `nsw`/`nuw`, proved indexing uses `inbounds` without a
//! check, and every `&mut` parameter is `noalias` (ADR-0015). Unproved
//! obligations keep a check that traps; `bmb build` refuses such programs
//! anyway, so this matters only for `bmb emit-llvm` on partial proofs.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use crate::ast::*;
use crate::verify::{ObKey, Tables};

#[derive(Clone)]
enum CVal {
    Imm(String),
    Arr { ptr: String, len: String },
    Unit,
}

impl CVal {
    fn imm(&self) -> String {
        match self {
            CVal::Imm(s) => s.clone(),
            CVal::Unit => "0".into(),
            CVal::Arr { .. } => panic!("array used as scalar"),
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
    },
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

fn fn_sym(path: &str) -> String {
    format!("@\"{path}\"")
}

struct Gen<'p> {
    t: &'p Tables,
    proven: &'p BTreeSet<ObKey>,
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

    fn resolve(&self, path: &[Ident]) -> String {
        if path.len() == 1 {
            let local = format!("{}::{}", self.mod_path, path[0].name);
            if self.t.fns.contains_key(&local) {
                return local;
            }
            return path[0].name.clone();
        }
        path.iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::")
    }

    // ---------- expressions ----------

    fn expr(&mut self, e: &Expr) -> CVal {
        match &e.kind {
            ExprKind::Int(v) => CVal::Imm(v.to_string()),
            ExprKind::Bool(b) => CVal::Imm(b.to_string()),
            ExprKind::Unit => CVal::Unit,
            ExprKind::Var(id) => match self.slots[&id.name].clone() {
                Slot::Scalar { ptr, ty } => {
                    let r = self.t();
                    self.emit(format!("{r} = load {ty}, ptr {ptr}"));
                    CVal::Imm(r)
                }
                Slot::Arr { ptr, len, .. } => CVal::Arr { ptr, len },
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
            ExprKind::Index { base, index } => {
                let (ptr, ty) = self.elem_ptr(e, base, index);
                let r = self.t();
                self.emit(format!("{r} = load {ty}, ptr {ptr}"));
                CVal::Imm(r)
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
            ExprKind::Call { path, args } => self.call(path, args, e),
            ExprKind::If { cond, then, els } => {
                let ty = self.ty(e);
                let slot = if matches!(ty, TypeKind::Unit | TypeKind::Error) {
                    None
                } else {
                    Some(self.alloca(llty(&ty)))
                };
                let c = self.expr(cond).imm();
                let lt = self.l("then");
                let le = self.l("else");
                let end = self.l("endif");
                self.emit(format!("br i1 {c}, label %{lt}, label %{le}"));
                self.block(&lt);
                let tv = self.block_expr(then);
                if let Some(s) = &slot {
                    self.emit(format!("store {} {}, ptr {s}", llty(&ty), tv.imm()));
                }
                self.emit(format!("br label %{end}"));
                self.block(&le);
                if let Some(x) = els {
                    let ev = self.expr(x);
                    if let Some(s) = &slot {
                        self.emit(format!("store {} {}, ptr {s}", llty(&ty), ev.imm()));
                    }
                }
                self.emit(format!("br label %{end}"));
                self.block(&end);
                match slot {
                    Some(s) => {
                        let r = self.t();
                        self.emit(format!("{r} = load {}, ptr {s}", llty(&ty)));
                        CVal::Imm(r)
                    }
                    None => CVal::Unit,
                }
            }
            ExprKind::Block(b) => self.block_expr(b),
            ExprKind::Ref { expr, .. } => self.expr(expr),
            ExprKind::Array(_) | ExprKind::Repeat { .. } => {
                let ty = self.ty(e);
                let TypeKind::Array(el, n) = &ty else {
                    unreachable!()
                };
                let elem = llty(&el.kind);
                let ptr = self.alloca(&format!("[{n} x {elem}]"));
                self.init_array(e, &ptr, elem);
                CVal::Arr {
                    ptr,
                    len: n.to_string(),
                }
            }
            ExprKind::Old(_) | ExprKind::Forall { .. } | ExprKind::Trigger(_) | ExprKind::Error => {
                unreachable!("specification construct in executable code")
            }
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
                Slot::Scalar { .. } => unreachable!("indexing a scalar"),
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
            "{p} = getelementptr inbounds {elem}, ptr {ptr}, i64 {i}{note}"
        ));
        (p, elem)
    }

    fn init_array(&mut self, e: &Expr, ptr: &str, elem: &str) {
        match &e.kind {
            ExprKind::Array(items) => {
                for (k, it) in items.iter().enumerate() {
                    let v = self.expr(it).imm();
                    let p = self.t();
                    self.emit(format!(
                        "{p} = getelementptr inbounds {elem}, ptr {ptr}, i64 {k}"
                    ));
                    self.emit(format!("store {elem} {v}, ptr {p}"));
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
                    "{p} = getelementptr inbounds {elem}, ptr {ptr}, i64 {k}"
                ));
                self.emit(format!("store {elem} {v}, ptr {p}"));
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
            Stmt::Let { name, ty, init, .. } => match &ty.kind {
                TypeKind::Array(el, n) => {
                    let elem = llty(&el.kind);
                    let ptr = self.alloca(&format!("[{n} x {elem}]"));
                    self.init_array(init, &ptr, elem);
                    self.slots.insert(
                        name.name.clone(),
                        Slot::Arr {
                            ptr,
                            len: n.to_string(),
                            elem,
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
                },
                ExprKind::Index { base, index } => {
                    let (ptr, ty) = self.elem_ptr(target, base, index);
                    let v = self.expr(value).imm();
                    self.emit(format!("store {ty} {v}, ptr {ptr}"));
                }
                _ => unreachable!(),
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
}

pub struct Module {
    pub ir: String,
    /// Runtime checks left in the IR because their obligation is unproved.
    pub checks: u32,
    pub main: Option<String>,
}

/// Emits one LLVM module for the whole program.
pub fn emit(p: &Program, t: &Tables, proven: &BTreeSet<ObKey>) -> Module {
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
         declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)\n",
    );
    for s in ["s", "u"] {
        for op in ["add", "sub", "mul"] {
            let _ = writeln!(
                ir,
                "declare {{i64, i1}} @llvm.{s}{op}.with.overflow.i64(i64, i64)"
            );
        }
    }
    ir.push_str(
        "define void @bmb_print_i64(i64 %x) {\n  call i32 (ptr, ...) @printf(ptr @bmb.fmt, i64 %x)\n  ret void\n}\n",
    );
    let mut checks = 0;
    let mut main = None;
    for (path, mod_path, f) in defs {
        if f.kind != FnKind::Exec {
            continue;
        }
        if f.name.name == "main" {
            main = Some(path.clone());
        }
        let mut g = Gen {
            t,
            proven,
            mod_path,
            out: String::new(),
            allocas: String::new(),
            tmp: 0,
            label: 0,
            slots: BTreeMap::new(),
            checks: 0,
        };
        let mut params = Vec::new();
        for prm in &f.params {
            let n = &prm.name.name;
            match &prm.ty.kind {
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
                        },
                    );
                }
                TypeKind::Array(el, len) => {
                    // Arrays by value: copied into a local.
                    let elem = llty(&el.kind);
                    params.push(format!("ptr %\"{n}.in\""));
                    let ptr = g.alloca(&format!("[{len} x {elem}]"));
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
        let ret = llty(&f.ret.ty.kind);
        if ret == "void" {
            g.emit("ret void");
        } else {
            g.emit(format!("ret {ret} {}", rv.imm()));
        }
        checks += g.checks;
        let _ = write!(
            ir,
            "\ndefine {ret} {}({}) {{\nentry:\n{}{}}}\n",
            fn_sym(&path),
            params.join(", "),
            g.allocas,
            g.out
        );
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
