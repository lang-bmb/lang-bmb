//! The run-time support of the system interface (ADR-0091 §4, backlog
//! 4C-02): IR functions over libc, put into a module only when it calls them
//! (as `print_f64` and the text runtime are).

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
