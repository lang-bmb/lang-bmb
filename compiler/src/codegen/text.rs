//! Text in the code generator (ADR-0050 §4, backlog 2U-03).
//!
//! A `String` is a `Vec<u8>` header and a `&str` a (pointer, length) pair
//! (`Tables::lowered`); a `char` is its scalar value in an `i64` register.
//! The runtime below appends bytes (also bytes of the string itself, whose
//! buffer the growth may move), encodes a `char` as UTF-8, checks UTF-8 as
//! Rust's `str::from_utf8` does, and compares bytes as Rust's `Ord` for
//! `str` does (`memcmp` on the common length, then the lengths).

use super::*;

/// `bmb_str_cmp`: `-1`, `0` or `1` as `a` sorts before, with or after `b`.
pub(super) const TEXT_CMP: &str = "declare i32 @memcmp(ptr, ptr, i64)
define internal i32 @bmb_str_cmp(ptr %a, i64 %la, ptr %b, i64 %lb) {
entry:
  %short = icmp ult i64 %la, %lb
  %m = select i1 %short, i64 %la, i64 %lb
  %z = icmp eq i64 %m, 0
  br i1 %z, label %len, label %cmp
cmp:
  %c = call i32 @memcmp(ptr %a, ptr %b, i64 %m)
  %ne = icmp ne i32 %c, 0
  br i1 %ne, label %bytes, label %len
bytes:
  %neg = icmp slt i32 %c, 0
  %r0 = select i1 %neg, i32 -1, i32 1
  ret i32 %r0
len:
  %long = icmp ugt i64 %la, %lb
  %r1 = select i1 %short, i32 -1, i32 0
  %r = select i1 %long, i32 1, i32 %r1
  ret i32 %r
}
";

/// `bmb_str_append(h, src, n)` (needs `VEC_RUNTIME`): the `n` bytes at
/// `src` after the bytes of the `String` at `h`. The length bound is the
/// caller's (proved or checked, E0816). `bmb_str_push_char(h, c)`: the
/// UTF-8 bytes of scalar value `c`.
pub(super) const TEXT_GROW: &str =
    "define internal void @bmb_str_append(ptr %h, ptr %src, i64 %n) {
entry:
  %z = icmp eq i64 %n, 0
  br i1 %z, label %done, label %go
go:
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
  %len = load i64, ptr %lp
  %cap = load i64, ptr %cp
  %buf0 = load ptr, ptr %h
  %b0 = ptrtoint ptr %buf0 to i64
  %s0 = ptrtoint ptr %src to i64
  %off = sub i64 %s0, %b0
  %inside = icmp ult i64 %off, %cap
  %need = add nuw i64 %len, %n
  %fits = icmp ule i64 %need, %cap
  br i1 %fits, label %copy, label %grow
grow:
  %d = shl i64 %cap, 1
  %big = icmp ugt i64 %need, %d
  %w = select i1 %big, i64 %need, i64 %d
  %small = icmp ult i64 %w, 8
  %new = select i1 %small, i64 8, i64 %w
  %p = call ptr @realloc(ptr %buf0, i64 %new)
  %null = icmp eq ptr %p, null
  br i1 %null, label %fail, label %grown
fail:
  call void @abort()
  unreachable
grown:
  store ptr %p, ptr %h
  store i64 %new, ptr %cp
  br label %copy
copy:
  %buf = load ptr, ptr %h
  %moved = getelementptr i8, ptr %buf, i64 %off
  %from = select i1 %inside, ptr %moved, ptr %src
  %dst = getelementptr inbounds i8, ptr %buf, i64 %len
  call void @llvm.memcpy.p0.p0.i64(ptr %dst, ptr %from, i64 %n, i1 false)
  store i64 %need, ptr %lp
  br label %done
done:
  ret void
}
define internal void @bmb_str_push_char(ptr %h, i64 %c) {
entry:
  %e = alloca [4 x i8]
  %e1 = getelementptr inbounds [4 x i8], ptr %e, i64 0, i64 1
  %e2 = getelementptr inbounds [4 x i8], ptr %e, i64 0, i64 2
  %e3 = getelementptr inbounds [4 x i8], ptr %e, i64 0, i64 3
  %x6 = lshr i64 %c, 6
  %x12 = lshr i64 %c, 12
  %x18 = lshr i64 %c, 18
  %m0 = and i64 %c, 63
  %m6 = and i64 %x6, 63
  %m12 = and i64 %x12, 63
  %k0 = or i64 %m0, 128
  %k6 = or i64 %m6, 128
  %k12 = or i64 %m12, 128
  %t0 = trunc i64 %k0 to i8
  %t6 = trunc i64 %k6 to i8
  %t12 = trunc i64 %k12 to i8
  %c1 = icmp ult i64 %c, 128
  br i1 %c1, label %one, label %t2
one:
  %a = trunc i64 %c to i8
  store i8 %a, ptr %e
  br label %put
t2:
  %c2 = icmp ult i64 %c, 2048
  br i1 %c2, label %two, label %t3
two:
  %h2 = or i64 %x6, 192
  %h2b = trunc i64 %h2 to i8
  store i8 %h2b, ptr %e
  store i8 %t0, ptr %e1
  br label %put
t3:
  %c3 = icmp ult i64 %c, 65536
  br i1 %c3, label %three, label %four
three:
  %h3 = or i64 %x12, 224
  %h3b = trunc i64 %h3 to i8
  store i8 %h3b, ptr %e
  store i8 %t6, ptr %e1
  store i8 %t0, ptr %e2
  br label %put
four:
  %h4 = or i64 %x18, 240
  %h4b = trunc i64 %h4 to i8
  store i8 %h4b, ptr %e
  store i8 %t12, ptr %e1
  store i8 %t6, ptr %e2
  store i8 %t0, ptr %e3
  br label %put
put:
  %n = phi i64 [1, %one], [2, %two], [3, %three], [4, %four]
  call void @bmb_str_append(ptr %h, ptr %e, i64 %n)
  ret void
}
";

/// `bmb_utf8_valid(p, n)`: Rust's `str::from_utf8` rules (no overlong
/// forms, no surrogates, nothing above `0x10FFFF`).
pub(super) const TEXT_UTF8: &str = "define internal i1 @bmb_utf8_valid(ptr %p, i64 %n) {
entry:
  br label %head
head:
  %i = phi i64 [0, %entry], [%i1, %ascii], [%end, %next]
  %fin = icmp uge i64 %i, %n
  br i1 %fin, label %yes, label %lead
lead:
  %q0 = getelementptr inbounds i8, ptr %p, i64 %i
  %b8 = load i8, ptr %q0
  %b = zext i8 %b8 to i32
  %i1 = add nuw i64 %i, 1
  %isa = icmp ult i32 %b, 128
  br i1 %isa, label %ascii, label %multi
ascii:
  br label %head
multi:
  %ge2 = icmp uge i32 %b, 194
  %le2 = icmp ule i32 %b, 223
  %l2 = and i1 %ge2, %le2
  %ge3 = icmp uge i32 %b, 224
  %le3 = icmp ule i32 %b, 239
  %l3 = and i1 %ge3, %le3
  %ge4 = icmp uge i32 %b, 240
  %le4 = icmp ule i32 %b, 244
  %l4 = and i1 %ge4, %le4
  %s4 = select i1 %l4, i64 4, i64 0
  %s3 = select i1 %l3, i64 3, i64 %s4
  %len = select i1 %l2, i64 2, i64 %s3
  %bad = icmp eq i64 %len, 0
  br i1 %bad, label %no, label %fit
fit:
  %end = add nuw i64 %i, %len
  %over = icmp ugt i64 %end, %n
  br i1 %over, label %no, label %second
second:
  %e0 = icmp eq i32 %b, 224
  %f0 = icmp eq i32 %b, 240
  %ed = icmp eq i32 %b, 237
  %f4 = icmp eq i32 %b, 244
  %lo1 = select i1 %f0, i32 144, i32 128
  %lo = select i1 %e0, i32 160, i32 %lo1
  %hi1 = select i1 %f4, i32 143, i32 191
  %hi = select i1 %ed, i32 159, i32 %hi1
  %q1 = getelementptr inbounds i8, ptr %p, i64 %i1
  %c18 = load i8, ptr %q1
  %c1 = zext i8 %c18 to i32
  %okl = icmp uge i32 %c1, %lo
  %okh = icmp ule i32 %c1, %hi
  %ok1 = and i1 %okl, %okh
  br i1 %ok1, label %third, label %no
third:
  %need3 = icmp uge i64 %len, 3
  br i1 %need3, label %t3, label %next
t3:
  %i2 = add nuw i64 %i, 2
  %q2 = getelementptr inbounds i8, ptr %p, i64 %i2
  %c28 = load i8, ptr %q2
  %c2 = and i8 %c28, -64
  %ok2 = icmp eq i8 %c2, -128
  br i1 %ok2, label %fourth, label %no
fourth:
  %need4 = icmp eq i64 %len, 4
  br i1 %need4, label %t4, label %next
t4:
  %i3 = add nuw i64 %i, 3
  %q3 = getelementptr inbounds i8, ptr %p, i64 %i3
  %c38 = load i8, ptr %q3
  %c3 = and i8 %c38, -64
  %ok3 = icmp eq i8 %c3, -128
  br i1 %ok3, label %next, label %no
next:
  br label %head
yes:
  ret i1 true
no:
  ret i1 false
}
";

/// Whether `t` is a text value or a reference to one: `String`
/// (lowered, a `Vec<u8>`) or `&str` (a `&[u8]`).
impl Gen<'_> {
    pub(super) fn is_text_expr(&self, e: &Expr) -> bool {
        self.t.text.contains(&(e.span.file, e.span.lo, e.span.hi)) || self.text_targ(e)
    }

    /// Whether `e` has a type parameter's type that this instance gives a
    /// text type argument (`String`, lowered, as the only text type that is
    /// `Ord`, backlog 4B-04): its comparisons are text's.
    fn text_targ(&self, e: &Expr) -> bool {
        if self.tsub.is_empty() {
            return false;
        }
        let Some(TypeKind::Param(n)) = self.t.types.get(&(e.span.file, e.span.lo, e.span.hi))
        else {
            return false;
        };
        self.tsub.get(n).is_some_and(|t| {
            crate::check::is_text(t)
                || matches!(t, TypeKind::Vec(x) if x.kind == TypeKind::Int(IntTy::U8))
        })
    }

    /// The bytes of text expression `e`: pointer and length.
    pub(super) fn text_view(&mut self, e: &Expr) -> (String, String) {
        if is_vec(&self.ty(e)) || self.vec_var(e) {
            let hdr = self.vec_hdr(e);
            return self.vec_view(&hdr);
        }
        match self.expr(e) {
            CVal::Arr { ptr, len } => (ptr, len),
            _ => unreachable!("text is bytes"),
        }
    }

    /// `l op r` on text: Rust's byte-lexicographic order.
    pub(super) fn text_cmp(&mut self, op: BinOp, l: &Expr, r: &Expr) -> CVal {
        let (a, la) = self.text_view(l);
        let (b, lb) = self.text_view(r);
        let c = self.t();
        self.emit(format!(
            "{c} = call i32 @bmb_str_cmp(ptr {a}, i64 {la}, ptr {b}, i64 {lb})"
        ));
        let pred = match op {
            BinOp::Eq => "eq",
            BinOp::Ne => "ne",
            BinOp::Lt => "slt",
            BinOp::Le => "sle",
            BinOp::Gt => "sgt",
            BinOp::Ge => "sge",
            _ => unreachable!("a comparison"),
        };
        let r = self.t();
        self.emit(format!("{r} = icmp {pred} i32 {c}, 0"));
        CVal::Imm(r)
    }

    /// Rust's `is_char_boundary(i)` of the `n` bytes at `p`: an `i1`. The
    /// byte is read only below `n`.
    fn boundary(&mut self, p: &str, n: &str, i: &str) -> String {
        let (inside, read, done) = (self.l("bnd"), self.l("bndr"), self.l("bndd"));
        let from = self.l("bnde");
        let lt = self.t();
        self.emit(format!("br label %{from}"));
        self.block(&from);
        self.emit(format!("{lt} = icmp ult i64 {i}, {n}"));
        self.emit(format!("br i1 {lt}, label %{read}, label %{inside}"));
        self.block(&read);
        let q = self.t();
        self.emit(format!("{q} = getelementptr inbounds i8, ptr {p}, i64 {i}"));
        let b = self.t();
        self.emit(format!("{b} = load i8, ptr {q}"));
        let start = self.t();
        self.emit(format!("{start} = icmp sge i8 {b}, -64"));
        self.emit(format!("br label %{done}"));
        self.block(&inside);
        let end = self.t();
        self.emit(format!("{end} = icmp eq i64 {i}, {n}"));
        self.emit(format!("br label %{done}"));
        self.block(&done);
        let r = self.t();
        self.emit(format!(
            "{r} = phi i1 [{start}, %{read}], [{end}, %{inside}]"
        ));
        r
    }

    /// `&s[lo..hi]` (`idx` is the indexing): the bytes between, after the
    /// boundary check the proof did not discharge (E0820).
    pub(super) fn str_slice(&mut self, idx: &Expr, base: &Expr, lo: &Expr, hi: &Expr) -> CVal {
        let (p, n) = self.text_view(base);
        let lo = self.expr(lo).imm();
        let hi = self.expr(hi).imm();
        if self.proved("E0820", idx) {
            self.emit("; proved E0820");
        } else {
            let ord = self.t();
            self.emit(format!("{ord} = icmp ule i64 {lo}, {hi}"));
            let a = self.boundary(&p, &n, &lo);
            let b = self.boundary(&p, &n, &hi);
            let (ab, c) = (self.t(), self.t());
            self.emit(format!("{ab} = and i1 {a}, {b}"));
            self.emit(format!("{c} = and i1 {ab}, {ord}"));
            self.check(&c, "the range may not lie on character boundaries");
        }
        let (ptr, len) = (self.t(), self.t());
        self.emit(format!("{ptr} = getelementptr i8, ptr {p}, i64 {lo}"));
        self.emit(format!("{len} = sub nuw i64 {hi}, {lo}"));
        CVal::Arr { ptr, len }
    }

    /// `&v[lo..hi]` of an array, `Vec` or slice of integers (backlog
    /// 4C-05): the elements from `lo`, after the bounds check the proof
    /// did not discharge (E0805).
    pub(super) fn elem_slice(&mut self, idx: &Expr, base: &Expr, lo: &Expr, hi: &Expr) -> CVal {
        let bt = self.ty(base);
        let whole = match value_ty(&bt) {
            TypeKind::Ref { inner, .. } => inner.kind.clone(),
            t => t.clone(),
        };
        let et = match whole {
            TypeKind::Array(e, _) | TypeKind::Vec(e) | TypeKind::Slice(e) => e.kind,
            other => unreachable!("not a slice: {other:?}"),
        };
        let CVal::Arr { ptr: p, len: n } = self.array_of(base) else {
            unreachable!("an array value")
        };
        let lo = self.expr(lo).imm();
        let hi = self.expr(hi).imm();
        if self.proved("E0805", idx) {
            self.emit("; proved E0805");
        } else {
            let (ord, end, c) = (self.t(), self.t(), self.t());
            self.emit(format!("{ord} = icmp ule i64 {lo}, {hi}"));
            self.emit(format!("{end} = icmp ule i64 {hi}, {n}"));
            self.emit(format!("{c} = and i1 {ord}, {end}"));
            self.check(&c, "the range may be out of bounds");
        }
        let (ptr, len) = (self.t(), self.t());
        self.emit(format!(
            "{ptr} = getelementptr {}, ptr {p}, i64 {lo}",
            mem_ty(&et)
        ));
        self.emit(format!("{len} = sub nuw i64 {hi}, {lo}"));
        CVal::Arr { ptr, len }
    }

    /// The UTF-8 length of the scalar value in register `c`.
    fn utf8_len(&mut self, c: &str) -> String {
        let (a, b, d) = (self.t(), self.t(), self.t());
        self.emit(format!("{a} = icmp ult i64 {c}, 128"));
        self.emit(format!("{b} = icmp ult i64 {c}, 2048"));
        self.emit(format!("{d} = icmp ult i64 {c}, 65536"));
        let (x, y, z) = (self.t(), self.t(), self.t());
        self.emit(format!("{x} = select i1 {d}, i64 3, i64 4"));
        self.emit(format!("{y} = select i1 {b}, i64 2, i64 {x}"));
        self.emit(format!("{z} = select i1 {a}, i64 1, i64 {y}"));
        z
    }

    /// The length check of growing a `String` at `hdr` by `n` bytes
    /// (E0816), unless proved.
    fn grow_check(&mut self, e: &Expr, hdr: &str, n: &str, what: &str) {
        if self.proved("E0816", e) {
            self.emit("; proved E0816");
            return;
        }
        let lp = self.vec_field(hdr, 1);
        let (len, room, c) = (self.t(), self.t(), self.t());
        self.emit(format!("{len} = load i64, ptr {lp}"));
        self.emit(format!(
            "{room} = sub i64 {}, {len}",
            vec_len_max(&TypeKind::Int(IntTy::U8))
        ));
        self.emit(format!("{c} = icmp ule i64 {n}, {room}"));
        self.check(&c, what);
    }

    /// A method of text (ADR-0050 §2) other than `len` and `clone` (those
    /// are the `Vec` ones): `push_str`, `push`, `as_str`, `as_bytes`,
    /// `is_char_boundary`.
    pub(super) fn text_method(&mut self, e: &Expr, recv: &Expr, m: &str, args: &[Expr]) -> CVal {
        match m {
            "push_str" => {
                let (src, n) = self.text_view(&args[0]);
                let hdr = self.vec_hdr(recv);
                self.grow_check(e, &hdr, &n, "`push_str` may exceed the maximum length");
                self.emit(format!(
                    "call void @bmb_str_append(ptr {hdr}, ptr {src}, i64 {n})"
                ));
                CVal::Unit
            }
            "push" => {
                let c = self.expr(&args[0]).imm();
                let hdr = self.vec_hdr(recv);
                let n = self.utf8_len(&c);
                self.grow_check(e, &hdr, &n, "`push` may exceed the maximum length");
                self.emit(format!("call void @bmb_str_push_char(ptr {hdr}, i64 {c})"));
                CVal::Unit
            }
            "as_str" | "as_bytes" => {
                let (ptr, len) = self.text_view(recv);
                CVal::Arr { ptr, len }
            }
            "is_char_boundary" => {
                let (p, n) = self.text_view(recv);
                let i = self.expr(&args[0]).imm();
                CVal::Imm(self.boundary(&p, &n, &i))
            }
            _ => unreachable!("checked text method {m}"),
        }
    }

    /// `c.len_utf8()`.
    pub(super) fn len_utf8(&mut self, recv: &Expr) -> CVal {
        let c = self.expr(recv).imm();
        CVal::Imm(self.utf8_len(&c))
    }

    /// `String::from(t)`: a new buffer of `t`'s length holding its bytes.
    pub(super) fn string_from(&mut self, e: &Expr, arg: &Expr) -> CVal {
        let (src, n) = self.text_view(arg);
        let t = self.ty(e);
        let hdr = self.temp(&t);
        self.emit(format!(
            "call void @bmb_vec_alloc(ptr {hdr}, i64 {n}, i64 1)"
        ));
        self.emit(format!(
            "call void @bmb_str_append(ptr {hdr}, ptr {src}, i64 {n})"
        ));
        CVal::Ptr { ptr: hdr, ty: t }
    }

    /// `String::from_utf8(v)`: `Ok` of the bytes when they are UTF-8, else
    /// `Err` of them; the buffer moves into the result either way.
    pub(super) fn string_from_utf8(&mut self, e: &Expr, arg: &Expr) -> CVal {
        let CVal::Ptr { ptr: v, .. } = self.expr(arg) else {
            unreachable!("a Vec value")
        };
        let (p, n) = self.vec_view(&v);
        let ok = self.t();
        self.emit(format!("{ok} = call i1 @bmb_utf8_valid(ptr {p}, i64 {n})"));
        let t = self.ty(e);
        let out = self.temp(&t);
        let tag = self.field_ptr(&out, &t, 0);
        // `Ok` is variant 0 of `Result`, `Err` variant 1.
        let k = self.t();
        let bad = self.t();
        self.emit(format!("{bad} = xor i1 {ok}, true"));
        self.emit(format!("{k} = zext i1 {bad} to i64"));
        self.emit(format!("store i64 {k}, ptr {tag}"));
        let dst = self.t();
        self.emit(format!(
            "{dst} = getelementptr inbounds {}, ptr {out}, i32 0, i32 1",
            view_ty(&t, "Ok")
        ));
        let vt = self.ty(arg);
        self.copy(&dst, &v, value_ty(&vt));
        CVal::Ptr { ptr: out, ty: t }
    }

    /// `char::from_u32(x)`: `Some(x)` for a scalar value.
    pub(super) fn char_from_u32(&mut self, e: &Expr, arg: &Expr) -> CVal {
        let x = self.expr(arg).imm();
        let (a, b, c, d) = (self.t(), self.t(), self.t(), self.t());
        self.emit(format!("{a} = icmp ult i64 {x}, 55296"));
        self.emit(format!("{b} = icmp uge i64 {x}, 57344"));
        self.emit(format!("{c} = icmp ult i64 {x}, 1114112"));
        self.emit(format!("{d} = and i1 {b}, {c}"));
        let ok = self.t();
        self.emit(format!("{ok} = or i1 {a}, {d}"));
        self.option(e, &ok, &x)
    }
}
