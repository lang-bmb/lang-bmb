//! The run-time support of the system interface (ADR-0091 §4, backlog
//! 4C-02): IR functions over libc, put into a module only when it calls them
//! (as `print_f64` and the text runtime are).

use super::*;

/// `eprint_str`: the bytes on standard error, after flushing standard
/// output, so that the two streams interleave in the order of the calls
/// whether `stdout` is a terminal or a pipe (Rust's `stdout` is line
/// buffered and its output lines end in newlines, which gives the same
/// order; libc buffers a pipe fully).
const EPRINT: &str = "@stderr = external global ptr
define internal void @bmb_eprint_bytes(ptr %p, i64 %n) {
  %o = load ptr, ptr @stdout
  call i32 @fflush(ptr %o)
  %f = load ptr, ptr @stderr
  call i64 @fwrite(ptr %p, i64 1, i64 %n, ptr %f)
  ret void
}
";

/// `std::process::exit`: libc's `exit`, which flushes the standard streams
/// (Rust's `exit` flushes its `stdout` too) and runs no destructor.
const EXIT: &str = "declare void @exit(i32) noreturn
define internal void @bmb_exit(i64 %c) cold noreturn {
  %w = trunc i64 %c to i32
  call void @exit(i32 %w)
  unreachable
}
";

/// `std::env::args`: `main` keeps `argc` and `argv` (`codegen::emit`);
/// each argument is copied into a `String` after Rust's UTF-8 check. An
/// argument that is not UTF-8 ends the process with a message and code 101,
/// what Rust's `env::args` panic ends in (no unwinding here). Needs the
/// `Vec` runtime (the result is a `Vec<String>`) and `bmb_utf8_valid`.
const ARGS: &str = "@bmb.argc = internal global i32 0
@bmb.argv = internal global ptr null
declare i64 @strlen(ptr)
@bmb.args_bad = private unnamed_addr constant [__N__ x i8] c\"__MSG__\"
define internal void @bmb_args(ptr %out) {
entry:
  %c32 = load i32, ptr @bmb.argc
  %argc = sext i32 %c32 to i64
  %argv = load ptr, ptr @bmb.argv
  call void @bmb_vec_alloc(ptr %out, i64 %argc, i64 24)
  %buf = load ptr, ptr %out
  br label %head
head:
  %i = phi i64 [0, %entry], [%i1, %ok]
  %more = icmp ult i64 %i, %argc
  br i1 %more, label %body, label %done
body:
  %ap = getelementptr inbounds ptr, ptr %argv, i64 %i
  %a = load ptr, ptr %ap
  %n = call i64 @strlen(ptr %a)
  %valid = call i1 @bmb_utf8_valid(ptr %a, i64 %n)
  br i1 %valid, label %ok, label %bad
bad:
  call void @bmb_eprint_bytes(ptr @bmb.args_bad, i64 __N__)
  call void @bmb_exit(i64 101)
  unreachable
ok:
  %h = getelementptr inbounds %bmb.vec, ptr %buf, i64 %i
  call void @bmb_vec_alloc(ptr %h, i64 %n, i64 1)
  %d = load ptr, ptr %h
  call void @llvm.memcpy.p0.p0.i64(ptr %d, ptr %a, i64 %n, i1 false)
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  store i64 %n, ptr %lp
  %i1 = add nuw i64 %i, 1
  br label %head
done:
  %olp = getelementptr inbounds %bmb.vec, ptr %out, i32 0, i32 1
  store i64 %argc, ptr %olp
  ret void
}
";

/// The parts of the runtime that `ir` calls, and the parts they call.
pub fn runtime(ir: &str) -> String {
    let args = ARGS
        .replace("__N__", &ARGS_BAD.len().to_string())
        .replace("__MSG__", &ARGS_BAD.replace('\n', "\\0A"));
    let parts = [
        ("@bmb_args(", args.as_str()),
        ("@bmb_eprint_bytes(", EPRINT),
        ("@bmb_exit(", EXIT),
    ];
    let mut out = String::new();
    let mut taken = [false; 3];
    loop {
        let before = taken;
        for (k, (sym, text)) in parts.iter().enumerate() {
            if !taken[k] && (ir.contains(sym) || out.contains(sym)) {
                taken[k] = true;
                out.push_str(text);
            }
        }
        if taken == before {
            return out;
        }
    }
}

/// The message of a command-line argument that is not UTF-8.
const ARGS_BAD: &str = "bmb: a command-line argument is not valid UTF-8\n";

/// Standard input (ADR-0091 §2, slice (a)): `bmb_stdin_line` appends the
/// next line (its `\n` included) to the `String` at `h` and
/// `bmb_stdin_all` the rest of the input to the `Vec<u8>` at `h`, both
/// through libc's buffered `stdin`, so the two may be mixed. A closed
/// standard input (`EBADF`) reads as empty, as Rust's `Stdin` does. Each returns
/// the number of bytes appended, or `-1 - errno` (errno 0: the line is not
/// UTF-8). As Rust's `read_line`: a line that is not UTF-8 is taken back
/// out of the buffer; on an OS error the bytes read stay when they are
/// UTF-8. As Rust's `read_to_end`: the bytes read stay. Needs the `Vec`
/// runtime and `bmb_utf8_valid`.
const STDIN: &str = "@stdin = external global ptr
declare i32 @fgetc(ptr)
declare i32 @ferror(ptr)
declare ptr @__errno_location()
define internal i64 @bmb_stdin_errno() {
  %ep = call ptr @__errno_location()
  %e = load i32, ptr %ep
  %w = sext i32 %e to i64
  %neg = sub i64 -1, %w
  ret i64 %neg
}
define internal i1 @bmb_stdin_closed() {
  %ep = call ptr @__errno_location()
  %e = load i32, ptr %ep
  %badf = icmp eq i32 %e, 9
  ret i1 %badf
}
define internal i64 @bmb_stdin_line(ptr %h) {
entry:
  %f = load ptr, ptr @stdin
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
  %start = load i64, ptr %lp
  br label %next
next:
  %c = call i32 @fgetc(ptr %f)
  %eof = icmp slt i32 %c, 0
  br i1 %eof, label %end, label %byte
byte:
  %len = load i64, ptr %lp
  %cap = load i64, ptr %cp
  %full = icmp eq i64 %len, %cap
  br i1 %full, label %grow, label %put
grow:
  call void @bmb_vec_grow(ptr %h, i64 1)
  br label %put
put:
  %buf = load ptr, ptr %h
  %at = getelementptr inbounds i8, ptr %buf, i64 %len
  %b = trunc i32 %c to i8
  store i8 %b, ptr %at
  %len1 = add nuw i64 %len, 1
  store i64 %len1, ptr %lp
  %nl = icmp eq i32 %c, 10
  br i1 %nl, label %check, label %next
end:
  %err = call i32 @ferror(ptr %f)
  %bad = icmp ne i32 %err, 0
  br i1 %bad, label %which, label %check
which:
  %closed = call i1 @bmb_stdin_closed()
  br i1 %closed, label %check, label %os
os:
  %ec = call i64 @bmb_stdin_errno()
  %olen = load i64, ptr %lp
  %on = sub nuw i64 %olen, %start
  %obuf = load ptr, ptr %h
  %ofrom = getelementptr inbounds i8, ptr %obuf, i64 %start
  %okeep = call i1 @bmb_utf8_valid(ptr %ofrom, i64 %on)
  br i1 %okeep, label %osdone, label %osdrop
osdrop:
  store i64 %start, ptr %lp
  br label %osdone
osdone:
  ret i64 %ec
check:
  %len2 = load i64, ptr %lp
  %n = sub nuw i64 %len2, %start
  %buf2 = load ptr, ptr %h
  %from = getelementptr inbounds i8, ptr %buf2, i64 %start
  %valid = call i1 @bmb_utf8_valid(ptr %from, i64 %n)
  br i1 %valid, label %done, label %invalid
invalid:
  store i64 %start, ptr %lp
  ret i64 -1
done:
  ret i64 %n
}
declare i64 @fread(ptr, i64, i64, ptr)
define internal i64 @bmb_stdin_all(ptr %h) {
entry:
  %f = load ptr, ptr @stdin
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
  %start = load i64, ptr %lp
  br label %next
next:
  %len = load i64, ptr %lp
  %cap = load i64, ptr %cp
  %full = icmp eq i64 %len, %cap
  br i1 %full, label %grow, label %read
grow:
  call void @bmb_vec_grow(ptr %h, i64 1)
  br label %read
read:
  %len1 = load i64, ptr %lp
  %cap1 = load i64, ptr %cp
  %room = sub nuw i64 %cap1, %len1
  %buf = load ptr, ptr %h
  %at = getelementptr inbounds i8, ptr %buf, i64 %len1
  %got = call i64 @fread(ptr %at, i64 1, i64 %room, ptr %f)
  %len2 = add nuw i64 %len1, %got
  store i64 %len2, ptr %lp
  %none = icmp eq i64 %got, 0
  br i1 %none, label %end, label %next
end:
  %err = call i32 @ferror(ptr %f)
  %bad = icmp ne i32 %err, 0
  br i1 %bad, label %which, label %done
which:
  %closed = call i1 @bmb_stdin_closed()
  br i1 %closed, label %done, label %os
os:
  %ec = call i64 @bmb_stdin_errno()
  ret i64 %ec
done:
  %len3 = load i64, ptr %lp
  %n = sub nuw i64 %len3, %start
  ret i64 %n
}
";

/// The message of an `std::io::Error` (Rust's `Display`): for an OS error
/// `strerror` and ` (os error N)`, for a line that is not UTF-8 Rust's own
/// text. Written into the `String` header at `h`.
const IO_MESSAGE: &str = "declare ptr @strerror(i32)
declare i32 @snprintf(ptr, i64, ptr, ...)
@bmb.os_error = private unnamed_addr constant [17 x i8] c\"%s (os error %d)\\00\"
@bmb.not_utf8 = private unnamed_addr constant [34 x i8] c\"stream did not contain valid UTF-8\"
define internal void @bmb_io_message(ptr %h, i64 %code) {
entry:
  %tmp = alloca [160 x i8]
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %z = icmp eq i64 %code, 0
  br i1 %z, label %utf, label %os
utf:
  call void @bmb_vec_alloc(ptr %h, i64 34, i64 1)
  %ub = load ptr, ptr %h
  call void @llvm.memcpy.p0.p0.i64(ptr %ub, ptr @bmb.not_utf8, i64 34, i1 false)
  store i64 34, ptr %lp
  ret void
os:
  %c32 = trunc i64 %code to i32
  %s = call ptr @strerror(i32 %c32)
  %w = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %tmp, i64 160, ptr @bmb.os_error, ptr %s, i32 %c32)
  %w64 = sext i32 %w to i64
  %over = icmp ugt i64 %w64, 159
  %n = select i1 %over, i64 159, i64 %w64
  call void @bmb_vec_alloc(ptr %h, i64 %n, i64 1)
  %ob = load ptr, ptr %h
  call void @llvm.memcpy.p0.p0.i64(ptr %ob, ptr %tmp, i64 %n, i1 false)
  store i64 %n, ptr %lp
  ret void
}
";

/// The `Result<usize, std::io::Error>` of an input built-in, from what its
/// runtime returned (`n`, or `-1 - errno`): the wrappers `bmb_read_line`
/// and `bmb_read_to_end` the program calls, with the program's own LLVM
/// names of the result and error types.
pub(super) fn io_wrappers(result: &TypeKind, tb: &Tables) -> String {
    let vs = enum_variants(result, tb).expect("`Result` of an input built-in");
    let tag = |v: &str| vs.iter().position(|(n, _)| n == v).expect("variant");
    let err_ty = data_ty(&crate::check::io_error());
    let mut out = format!(
        "define internal void @bmb_io_result(ptr %out, i64 %r) {{
entry:
  %tag = getelementptr inbounds {res}, ptr %out, i32 0, i32 0
  %neg = icmp slt i64 %r, 0
  br i1 %neg, label %err, label %ok
ok:
  store i64 {ok}, ptr %tag
  %v = getelementptr inbounds {okv}, ptr %out, i32 0, i32 1
  store i64 %r, ptr %v
  ret void
err:
  store i64 {err}, ptr %tag
  %e = getelementptr inbounds {errv}, ptr %out, i32 0, i32 1
  %code = sub i64 -1, %r
  %c32 = trunc i64 %code to i32
  %cp = getelementptr inbounds {err_ty}, ptr %e, i32 0, i32 0
  store i32 %c32, ptr %cp
  %mp = getelementptr inbounds {err_ty}, ptr %e, i32 0, i32 1
  call void @bmb_io_message(ptr %mp, i64 %code)
  ret void
}}
",
        res = data_ty(result),
        ok = tag("Ok"),
        okv = view_ty(result, "Ok"),
        err = tag("Err"),
        errv = view_ty(result, "Err"),
    );
    for (name, inner) in [("read_line", "line"), ("read_to_end", "all")] {
        let _ = write!(
            out,
            "define internal void @bmb_{name}(ptr %out, ptr %h) {{
  %r = call i64 @bmb_stdin_{inner}(ptr %h)
  call void @bmb_io_result(ptr %out, i64 %r)
  ret void
}}
"
        );
    }
    out.push_str(STDIN);
    out.push_str(IO_MESSAGE);
    out
}
