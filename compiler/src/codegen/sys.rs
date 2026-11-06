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
  %ec = call i64 @bmb_errno()
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
  %ec = call i64 @bmb_errno()
  ret i64 %ec
done:
  %len3 = load i64, ptr %lp
  %n = sub nuw i64 %len3, %start
  ret i64 %n
}
";

/// Files (ADR-0091 §2, slice (b)): `bmb_rt_fs_read` reads the file at the
/// path into the `Vec<u8>` at `out`, `bmb_rt_fs_read_to_string` also checks
/// UTF-8 (Rust's `fs::read_to_string`), `bmb_rt_fs_write` creates or truncates
/// the file (mode 0o666 under the umask, as Rust's `File::create`) and
/// writes every byte. Reads and writes retry on `EINTR` as Rust's do. A
/// path is copied with a NUL at its end; a path that holds a NUL byte is
/// Rust's `InvalidInput` error (code 1000 here, 0 in the `io::Error`). Each
/// returns 0, or `-1 - code`. Needs the heap and `Vec` runtimes and
/// `bmb_utf8_valid`.
const FS: &str = "declare i32 @open(ptr, i32, ...)
declare i64 @read(i32, ptr, i64)
declare i64 @write(i32, ptr, i64)
declare i32 @close(i32)
define internal i64 @bmb_fd_read_all(i32 %fd, ptr %h) {
entry:
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
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
  %got = call i64 @read(i32 %fd, ptr %at, i64 %room)
  %fail = icmp slt i64 %got, 0
  br i1 %fail, label %error, label %some
error:
  %intr = call i1 @bmb_interrupted()
  br i1 %intr, label %next, label %os
os:
  %r = call i64 @bmb_errno()
  ret i64 %r
some:
  %none = icmp eq i64 %got, 0
  br i1 %none, label %done, label %more
more:
  %len2 = add nuw i64 %len1, %got
  store i64 %len2, ptr %lp
  br label %next
done:
  ret i64 0
}
define internal i64 @bmb_rt_fs_read(ptr %out, ptr %p, i64 %n) {
entry:
  call void @bmb_vec_alloc(ptr %out, i64 0, i64 1)
  %c = call ptr @bmb_cpath(ptr %p, i64 %n)
  %nul = icmp eq ptr %c, null
  br i1 %nul, label %badpath, label %open
badpath:
  ret i64 -1001
open:
  %fd = call i32 (ptr, i32, ...) @open(ptr %c, i32 524288)
  %fail = icmp slt i32 %fd, 0
  br i1 %fail, label %oserr, label %go
oserr:
  %e = call i64 @bmb_errno()
  call void @free(ptr %c)
  ret i64 %e
go:
  call void @free(ptr %c)
  %r = call i64 @bmb_fd_read_all(i32 %fd, ptr %out)
  call i32 @close(i32 %fd)
  %bad = icmp slt i64 %r, 0
  br i1 %bad, label %drop, label %ok
drop:
  %b = load ptr, ptr %out
  call void @free(ptr %b)
  ret i64 %r
ok:
  ret i64 0
}
define internal i64 @bmb_rt_fs_read_to_string(ptr %out, ptr %p, i64 %n) {
entry:
  %r = call i64 @bmb_rt_fs_read(ptr %out, ptr %p, i64 %n)
  %bad = icmp slt i64 %r, 0
  br i1 %bad, label %fail, label %check
fail:
  ret i64 %r
check:
  %buf = load ptr, ptr %out
  %lp = getelementptr inbounds %bmb.vec, ptr %out, i32 0, i32 1
  %len = load i64, ptr %lp
  %valid = call i1 @bmb_utf8_valid(ptr %buf, i64 %len)
  br i1 %valid, label %ok, label %invalid
invalid:
  call void @free(ptr %buf)
  ret i64 -1
ok:
  ret i64 0
}
define internal i64 @bmb_rt_fs_write(ptr %p, i64 %n, ptr %d, i64 %dn) {
entry:
  %c = call ptr @bmb_cpath(ptr %p, i64 %n)
  %nul = icmp eq ptr %c, null
  br i1 %nul, label %badpath, label %open
badpath:
  ret i64 -1001
open:
  %fd = call i32 (ptr, i32, ...) @open(ptr %c, i32 524865, i32 438)
  %fail = icmp slt i32 %fd, 0
  br i1 %fail, label %oserr, label %go
oserr:
  %e = call i64 @bmb_errno()
  call void @free(ptr %c)
  ret i64 %e
go:
  call void @free(ptr %c)
  br label %next
next:
  %off = phi i64 [0, %go], [%off, %again], [%off1, %wrote]
  %left = sub nuw i64 %dn, %off
  %more = icmp ugt i64 %left, 0
  br i1 %more, label %put, label %done
put:
  %at = getelementptr inbounds i8, ptr %d, i64 %off
  %w = call i64 @write(i32 %fd, ptr %at, i64 %left)
  %wfail = icmp slt i64 %w, 0
  br i1 %wfail, label %error, label %wrote
error:
  %intr = call i1 @bmb_interrupted()
  br i1 %intr, label %again, label %os
again:
  br label %next
os:
  %r = call i64 @bmb_errno()
  call i32 @close(i32 %fd)
  ret i64 %r
wrote:
  %off1 = add nuw i64 %off, %w
  br label %next
done:
  call i32 @close(i32 %fd)
  ret i64 0
}
";

/// Directories (ADR-0091 §2, slice (c)): `rename`, `remove_file`,
/// `create_dir_all` (Rust's algorithm: make the directory, on `ENOENT`
/// make its parent first; an existing directory is success) and `read_dir`
/// (the names but `.` and `..`, sorted by their bytes for determinism, §6;
/// a name that is not UTF-8 is the UTF-8 error). Whether a path is a
/// directory is `open(O_PATH | O_DIRECTORY)`, which needs no read right.
/// Linux x86-64: `struct dirent` keeps the name at byte 19.
const DIRS: &str = "declare i32 @rename(ptr, ptr)
declare i32 @unlink(ptr)
declare i32 @mkdir(ptr, i32)
declare ptr @opendir(ptr)
declare ptr @readdir(ptr)
declare i32 @closedir(ptr)
declare void @qsort(ptr, i64, i64, ptr)
define internal i64 @bmb_rt_fs_rename(ptr %p, i64 %n, ptr %q, i64 %m) {
entry:
  %a = call ptr @bmb_cpath(ptr %p, i64 %n)
  %b = call ptr @bmb_cpath(ptr %q, i64 %m)
  %an = icmp eq ptr %a, null
  %bn = icmp eq ptr %b, null
  %either = or i1 %an, %bn
  br i1 %either, label %badpath, label %go
badpath:
  call void @free(ptr %a)
  call void @free(ptr %b)
  ret i64 -1001
go:
  %r = call i32 @rename(ptr %a, ptr %b)
  %fail = icmp ne i32 %r, 0
  br i1 %fail, label %os, label %ok
os:
  %e = call i64 @bmb_errno()
  call void @free(ptr %a)
  call void @free(ptr %b)
  ret i64 %e
ok:
  call void @free(ptr %a)
  call void @free(ptr %b)
  ret i64 0
}
define internal i64 @bmb_rt_fs_remove_file(ptr %p, i64 %n) {
entry:
  %c = call ptr @bmb_cpath(ptr %p, i64 %n)
  %nul = icmp eq ptr %c, null
  br i1 %nul, label %badpath, label %go
badpath:
  ret i64 -1001
go:
  %r = call i32 @unlink(ptr %c)
  %fail = icmp ne i32 %r, 0
  br i1 %fail, label %os, label %ok
os:
  %e = call i64 @bmb_errno()
  call void @free(ptr %c)
  ret i64 %e
ok:
  call void @free(ptr %c)
  ret i64 0
}
define internal i1 @bmb_is_dir(ptr %c) {
entry:
  %fd = call i32 (ptr, i32, ...) @open(ptr %c, i32 2686976)
  %ok = icmp sge i32 %fd, 0
  br i1 %ok, label %yes, label %no
yes:
  call i32 @close(i32 %fd)
  ret i1 true
no:
  ret i1 false
}
define internal i64 @bmb_mkdir_once(ptr %c, i64 %n) {
entry:
  %endp = getelementptr inbounds i8, ptr %c, i64 %n
  %saved = load i8, ptr %endp
  store i8 0, ptr %endp
  %r = call i32 @mkdir(ptr %c, i32 511)
  %fail = icmp ne i32 %r, 0
  br i1 %fail, label %error, label %ok
error:
  %e = call i64 @bmb_errno()
  %dir = call i1 @bmb_is_dir(ptr %c)
  store i8 %saved, ptr %endp
  %res = select i1 %dir, i64 0, i64 %e
  ret i64 %res
ok:
  store i8 %saved, ptr %endp
  ret i64 0
}
define internal i64 @bmb_parent_len(ptr %c, i64 %n) {
entry:
  br label %strip
strip:
  %m0 = phi i64 [%n, %entry], [%m0d, %stripmore]
  %s0 = icmp ugt i64 %m0, 0
  br i1 %s0, label %stripchk, label %back
stripchk:
  %m0m = sub nuw i64 %m0, 1
  %sp = getelementptr inbounds i8, ptr %c, i64 %m0m
  %sb = load i8, ptr %sp
  %slash = icmp eq i8 %sb, 47
  br i1 %slash, label %stripmore, label %back
stripmore:
  %m0d = sub nuw i64 %m0, 1
  br label %strip
back:
  %m1 = phi i64 [%m0, %strip], [%m0, %stripchk], [%m1d, %backmore]
  %s1 = icmp ugt i64 %m1, 0
  br i1 %s1, label %backchk, label %trim
backchk:
  %m1m = sub nuw i64 %m1, 1
  %bp = getelementptr inbounds i8, ptr %c, i64 %m1m
  %bb = load i8, ptr %bp
  %isslash = icmp eq i8 %bb, 47
  br i1 %isslash, label %trim, label %backmore
backmore:
  %m1d = sub nuw i64 %m1, 1
  br label %back
trim:
  %m2 = phi i64 [%m1, %back], [%m1, %backchk], [%m2d, %trimmore]
  %s2 = icmp ugt i64 %m2, 1
  br i1 %s2, label %trimchk, label %done
trimchk:
  %m2m = sub nuw i64 %m2, 1
  %tp = getelementptr inbounds i8, ptr %c, i64 %m2m
  %tb = load i8, ptr %tp
  %tslash = icmp eq i8 %tb, 47
  br i1 %tslash, label %trimmore, label %done
trimmore:
  %m2d = sub nuw i64 %m2, 1
  br label %trim
done:
  %m = phi i64 [%m2, %trim], [%m2, %trimchk]
  ret i64 %m
}
define internal i64 @bmb_mkdir_all(ptr %c, i64 %n) {
entry:
  %empty = icmp eq i64 %n, 0
  br i1 %empty, label %ok, label %try
try:
  %endp = getelementptr inbounds i8, ptr %c, i64 %n
  %saved = load i8, ptr %endp
  store i8 0, ptr %endp
  %r = call i32 @mkdir(ptr %c, i32 511)
  %fail = icmp ne i32 %r, 0
  br i1 %fail, label %error, label %made
made:
  store i8 %saved, ptr %endp
  br label %ok
error:
  %e = call i64 @bmb_errno()
  %dir = call i1 @bmb_is_dir(ptr %c)
  store i8 %saved, ptr %endp
  br i1 %dir, label %ok, label %why
why:
  %enoent = icmp eq i64 %e, -3
  br i1 %enoent, label %parent, label %fail2
fail2:
  ret i64 %e
parent:
  %m = call i64 @bmb_parent_len(ptr %c, i64 %n)
  %pr = call i64 @bmb_mkdir_all(ptr %c, i64 %m)
  %pbad = icmp slt i64 %pr, 0
  br i1 %pbad, label %pfail, label %again
pfail:
  ret i64 %pr
again:
  %r2 = call i64 @bmb_mkdir_once(ptr %c, i64 %n)
  ret i64 %r2
ok:
  ret i64 0
}
define internal i64 @bmb_rt_fs_create_dir_all(ptr %p, i64 %n) {
entry:
  %c = call ptr @bmb_cpath(ptr %p, i64 %n)
  %nul = icmp eq ptr %c, null
  br i1 %nul, label %badpath, label %go
badpath:
  ret i64 -1001
go:
  %r = call i64 @bmb_mkdir_all(ptr %c, i64 %n)
  call void @free(ptr %c)
  ret i64 %r
}
define internal i32 @bmb_name_cmp(ptr %a, ptr %b) {
entry:
  %ap = load ptr, ptr %a
  %alp = getelementptr inbounds %bmb.vec, ptr %a, i32 0, i32 1
  %al = load i64, ptr %alp
  %bp = load ptr, ptr %b
  %blp = getelementptr inbounds %bmb.vec, ptr %b, i32 0, i32 1
  %bl = load i64, ptr %blp
  %r = call i32 @bmb_str_cmp(ptr %ap, i64 %al, ptr %bp, i64 %bl)
  ret i32 %r
}
define internal void @bmb_names_free(ptr %h) {
entry:
  %buf = load ptr, ptr %h
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %len = load i64, ptr %lp
  br label %head
head:
  %i = phi i64 [0, %entry], [%i1, %body]
  %more = icmp ult i64 %i, %len
  br i1 %more, label %body, label %done
body:
  %e = getelementptr inbounds %bmb.vec, ptr %buf, i64 %i
  %eb = load ptr, ptr %e
  call void @free(ptr %eb)
  %i1 = add nuw i64 %i, 1
  br label %head
done:
  call void @free(ptr %buf)
  ret void
}
define internal i64 @bmb_rt_fs_read_dir(ptr %out, ptr %p, i64 %n) {
entry:
  call void @bmb_vec_alloc(ptr %out, i64 0, i64 24)
  %c = call ptr @bmb_cpath(ptr %p, i64 %n)
  %nul = icmp eq ptr %c, null
  br i1 %nul, label %badpath, label %open
badpath:
  ret i64 -1001
open:
  %d = call ptr @opendir(ptr %c)
  %fail = icmp eq ptr %d, null
  br i1 %fail, label %oserr, label %go
oserr:
  %e = call i64 @bmb_errno()
  call void @free(ptr %c)
  ret i64 %e
go:
  call void @free(ptr %c)
  %lp = getelementptr inbounds %bmb.vec, ptr %out, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %out, i32 0, i32 2
  br label %next
next:
  %ep = call ptr @__errno_location()
  store i32 0, ptr %ep
  %ent = call ptr @readdir(ptr %d)
  %end = icmp eq ptr %ent, null
  br i1 %end, label %ended, label %got
got:
  %name = getelementptr inbounds i8, ptr %ent, i64 19
  %nlen = call i64 @strlen(ptr %name)
  %c0 = load i8, ptr %name
  %dot = icmp eq i8 %c0, 46
  %short = icmp ule i64 %nlen, 2
  %maybe = and i1 %dot, %short
  br i1 %maybe, label %dots, label %keep
dots:
  %one = icmp eq i64 %nlen, 1
  %c1p = getelementptr inbounds i8, ptr %name, i64 1
  %c1 = load i8, ptr %c1p
  %dot2 = icmp eq i8 %c1, 46
  %skip = or i1 %one, %dot2
  br i1 %skip, label %next, label %keep
keep:
  %valid = call i1 @bmb_utf8_valid(ptr %name, i64 %nlen)
  br i1 %valid, label %push, label %notutf
notutf:
  call i32 @closedir(ptr %d)
  call void @bmb_names_free(ptr %out)
  ret i64 -1
push:
  %len = load i64, ptr %lp
  %cap = load i64, ptr %cp
  %full = icmp eq i64 %len, %cap
  br i1 %full, label %grow, label %put
grow:
  call void @bmb_vec_grow(ptr %out, i64 24)
  br label %put
put:
  %buf = load ptr, ptr %out
  %slot = getelementptr inbounds %bmb.vec, ptr %buf, i64 %len
  call void @bmb_vec_alloc(ptr %slot, i64 %nlen, i64 1)
  %sb = load ptr, ptr %slot
  call void @llvm.memcpy.p0.p0.i64(ptr %sb, ptr %name, i64 %nlen, i1 false)
  %slp = getelementptr inbounds %bmb.vec, ptr %slot, i32 0, i32 1
  store i64 %nlen, ptr %slp
  %len1 = add nuw i64 %len, 1
  store i64 %len1, ptr %lp
  br label %next
ended:
  %ep2 = call ptr @__errno_location()
  %err = load i32, ptr %ep2
  %bad = icmp ne i32 %err, 0
  br i1 %bad, label %rderr, label %sort
rderr:
  %re = call i64 @bmb_errno()
  call i32 @closedir(ptr %d)
  call void @bmb_names_free(ptr %out)
  ret i64 %re
sort:
  call i32 @closedir(ptr %d)
  %all = load i64, ptr %lp
  %base = load ptr, ptr %out
  %some = icmp ugt i64 %all, 1
  br i1 %some, label %qs, label %done
qs:
  call void @qsort(ptr %base, i64 %all, i64 24, ptr @bmb_name_cmp)
  br label %done
done:
  ret i64 0
}
";

/// A path or name as libc takes it: a copy with a NUL at its end, or null
/// when it holds a NUL byte (Rust refuses those too). Needs the heap
/// runtime.
const CPATH: &str = "declare ptr @memchr(ptr, i32, i64)
define internal ptr @bmb_cpath(ptr %p, i64 %n) {
entry:
  %empty = icmp eq i64 %n, 0
  br i1 %empty, label %copy, label %scan
scan:
  %nul = call ptr @memchr(ptr %p, i32 0, i64 %n)
  %has = icmp ne ptr %nul, null
  br i1 %has, label %bad, label %copy
bad:
  ret ptr null
copy:
  %sz = add nuw i64 %n, 1
  %c = call ptr @bmb_box_alloc(i64 %sz)
  call void @llvm.memcpy.p0.p0.i64(ptr %c, ptr %p, i64 %n, i1 false)
  %end = getelementptr inbounds i8, ptr %c, i64 %n
  store i8 0, ptr %end
  ret ptr %c
}
";

/// The environment and the clock (ADR-0091 §2, slice (d)):
/// `bmb_monotonic_ms` is `CLOCK_MONOTONIC` in milliseconds (an arbitrary
/// start, never earlier than an earlier reading). Linux x86-64: a
/// `timespec` is two `i64`s.
const CLOCK: &str = "declare i32 @clock_gettime(i32, ptr)
define internal i64 @bmb_monotonic_ms() {
  %ts = alloca [2 x i64]
  call i32 @clock_gettime(i32 1, ptr %ts)
  %s = load i64, ptr %ts
  %np = getelementptr inbounds [2 x i64], ptr %ts, i32 0, i32 1
  %ns = load i64, ptr %np
  %ms = mul i64 %s, 1000
  %frac = udiv i64 %ns, 1000000
  %r = add i64 %ms, %frac
  ret i64 %r
}
";

/// `std::env::var` (with the program's own LLVM names of `Option<String>`):
/// `Some` of the value, `None` when it is not set, not UTF-8, or the name
/// holds a NUL byte (where Rust's `env::var` returns an `Err`, ADR-0091 §2
/// gives an `Option`).
pub(super) fn env_runtime(ir: &str, tb: &Tables) -> String {
    let mut out = String::new();
    if ir.contains("@bmb_monotonic_ms(") {
        out.push_str(CLOCK);
    }
    if !ir.contains("@bmb_env_var(") {
        return out;
    }
    let result = &tb.fns["std::env::var"].ret;
    let vs = enum_variants(result, tb).expect("`Option<String>`");
    let tag = |v: &str| vs.iter().position(|(n, _)| n == v).expect("variant");
    let _ = write!(
        out,
        "declare ptr @getenv(ptr)
define internal void @bmb_env_var(ptr %out, ptr %p, i64 %n) {{
entry:
  %tag = getelementptr inbounds {res}, ptr %out, i32 0, i32 0
  %c = call ptr @bmb_cpath(ptr %p, i64 %n)
  %nul = icmp eq ptr %c, null
  br i1 %nul, label %none, label %look
look:
  %v = call ptr @getenv(ptr %c)
  call void @free(ptr %c)
  %unset = icmp eq ptr %v, null
  br i1 %unset, label %none, label %check
check:
  %vn = call i64 @strlen(ptr %v)
  %valid = call i1 @bmb_utf8_valid(ptr %v, i64 %vn)
  br i1 %valid, label %some, label %none
some:
  store i64 {some}, ptr %tag
  %s = getelementptr inbounds {somev}, ptr %out, i32 0, i32 1
  call void @bmb_vec_alloc(ptr %s, i64 %vn, i64 1)
  %b = load ptr, ptr %s
  call void @llvm.memcpy.p0.p0.i64(ptr %b, ptr %v, i64 %vn, i1 false)
  %lp = getelementptr inbounds %bmb.vec, ptr %s, i32 0, i32 1
  store i64 %vn, ptr %lp
  ret void
none:
  store i64 {none}, ptr %tag
  ret void
}}
",
        res = data_ty(result),
        some = tag("Some"),
        somev = view_ty(result, "Some"),
        none = tag("None"),
    );
    out
}

/// The helpers `text` (the runtime parts taken so far) calls, each once:
/// the path copy and the declaration of `strlen`.
pub(super) fn helpers(text: &str) -> String {
    let mut out = String::new();
    if text.contains("@bmb_cpath(") {
        out.push_str(CPATH);
    }
    if text.contains("@strlen(") {
        out.push_str("declare i64 @strlen(ptr)\n");
    }
    out
}

/// Processes (ADR-0091 §2, slice (e)): `bmb_rt_process_run` runs `prog`
/// (looked up in `PATH` as Rust's `Command` does) with `args`, writes
/// `input` to its standard input while reading its standard output and
/// error (a `poll` loop, so neither side blocks the other; writes are at
/// most `PIPE_BUF` bytes), and waits. `SIGPIPE` is ignored as in Rust's
/// runtime: a child that stops reading ends the writing, not this process.
/// The status is the exit code, or 128 + the signal that ended the child.
/// An argument with a NUL byte is the `InvalidInput` error, a program
/// that cannot start is the spawn error (`ENOENT`, ..). Linux x86-64
/// layouts: a `pollfd` is 8 bytes, the spawn file actions fit in 128.
const RUN: &str = "declare i32 @pipe2(ptr, i32)
declare i32 @posix_spawn_file_actions_init(ptr)
declare i32 @posix_spawn_file_actions_destroy(ptr)
declare i32 @posix_spawn_file_actions_adddup2(ptr, i32, i32)
declare i32 @posix_spawnp(ptr, ptr, ptr, ptr, ptr, ptr)
declare i32 @poll(ptr, i64, i32)
declare i32 @waitpid(i32, ptr, i32)
declare ptr @signal(i32, ptr)
@environ = external global ptr
define internal i64 @bmb_fd_read_some(i32 %fd, ptr %h) {
entry:
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  %cp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 2
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
  %got = call i64 @read(i32 %fd, ptr %at, i64 %room)
  %fail = icmp slt i64 %got, 0
  br i1 %fail, label %error, label %some
error:
  %intr = call i1 @bmb_interrupted()
  %again = select i1 %intr, i64 1, i64 -1
  ret i64 %again
some:
  %len2 = add nuw i64 %len1, %got
  store i64 %len2, ptr %lp
  ret i64 %got
}
define internal void @bmb_argv_free(ptr %argv, i64 %n) {
entry:
  br label %head
head:
  %i = phi i64 [0, %entry], [%i1, %body]
  %more = icmp ult i64 %i, %n
  br i1 %more, label %body, label %done
body:
  %ap = getelementptr inbounds ptr, ptr %argv, i64 %i
  %a = load ptr, ptr %ap
  call void @free(ptr %a)
  %i1 = add nuw i64 %i, 1
  br label %head
done:
  call void @free(ptr %argv)
  ret void
}
define internal i64 @bmb_rt_process_run(ptr %out, ptr %p, i64 %pn, ptr %args, i64 %an, ptr %in, i64 %inn) {
entry:
  %cprog = call ptr @bmb_cpath(ptr %p, i64 %pn)
  %pnul = icmp eq ptr %cprog, null
  br i1 %pnul, label %badprog, label %mkargv
badprog:
  ret i64 -1001
mkargv:
  %cnt = add nuw i64 %an, 2
  %bytes = mul nuw i64 %cnt, 8
  %argv = call ptr @bmb_box_alloc(i64 %bytes)
  store ptr %cprog, ptr %argv
  br label %ahead
ahead:
  %i = phi i64 [0, %mkargv], [%i1, %astore]
  %amore = icmp ult i64 %i, %an
  br i1 %amore, label %abody, label %aend
abody:
  %eh = getelementptr inbounds %bmb.vec, ptr %args, i64 %i
  %ep = load ptr, ptr %eh
  %elp = getelementptr inbounds %bmb.vec, ptr %eh, i32 0, i32 1
  %el = load i64, ptr %elp
  %ca = call ptr @bmb_cpath(ptr %ep, i64 %el)
  %anul = icmp eq ptr %ca, null
  br i1 %anul, label %badarg, label %astore
badarg:
  %filled = add nuw i64 %i, 1
  call void @bmb_argv_free(ptr %argv, i64 %filled)
  ret i64 -1001
astore:
  %slot = add nuw i64 %i, 1
  %sp = getelementptr inbounds ptr, ptr %argv, i64 %slot
  store ptr %ca, ptr %sp
  %i1 = add nuw i64 %i, 1
  br label %ahead
aend:
  %last = add nuw i64 %an, 1
  %lastp = getelementptr inbounds ptr, ptr %argv, i64 %last
  store ptr null, ptr %lastp
  %fds = alloca [6 x i32]
  %f_in = getelementptr inbounds [6 x i32], ptr %fds, i32 0, i32 0
  %f_out = getelementptr inbounds [6 x i32], ptr %fds, i32 0, i32 2
  %f_err = getelementptr inbounds [6 x i32], ptr %fds, i32 0, i32 4
  %p1 = call i32 @pipe2(ptr %f_in, i32 524288)
  %p2 = call i32 @pipe2(ptr %f_out, i32 524288)
  %p3 = call i32 @pipe2(ptr %f_err, i32 524288)
  %p12 = or i32 %p1, %p2
  %p123 = or i32 %p12, %p3
  %pipefail = icmp ne i32 %p123, 0
  br i1 %pipefail, label %nopipe, label %spawn
nopipe:
  %pe = call i64 @bmb_errno()
  call void @bmb_argv_free(ptr %argv, i64 %last)
  ret i64 %pe
spawn:
  %in_r = load i32, ptr %f_in
  %in_wp = getelementptr inbounds [6 x i32], ptr %fds, i32 0, i32 1
  %in_w = load i32, ptr %in_wp
  %out_r = load i32, ptr %f_out
  %out_wp = getelementptr inbounds [6 x i32], ptr %fds, i32 0, i32 3
  %out_w = load i32, ptr %out_wp
  %err_r = load i32, ptr %f_err
  %err_wp = getelementptr inbounds [6 x i32], ptr %fds, i32 0, i32 5
  %err_w = load i32, ptr %err_wp
  %fa = alloca [128 x i8]
  call i32 @posix_spawn_file_actions_init(ptr %fa)
  call i32 @posix_spawn_file_actions_adddup2(ptr %fa, i32 %in_r, i32 0)
  call i32 @posix_spawn_file_actions_adddup2(ptr %fa, i32 %out_w, i32 1)
  call i32 @posix_spawn_file_actions_adddup2(ptr %fa, i32 %err_w, i32 2)
  %pid = alloca i32
  %env = load ptr, ptr @environ
  %rc = call i32 @posix_spawnp(ptr %pid, ptr %cprog, ptr %fa, ptr null, ptr %argv, ptr %env)
  call i32 @posix_spawn_file_actions_destroy(ptr %fa)
  call void @bmb_argv_free(ptr %argv, i64 %last)
  call i32 @close(i32 %in_r)
  call i32 @close(i32 %out_w)
  call i32 @close(i32 %err_w)
  %spawnfail = icmp ne i32 %rc, 0
  br i1 %spawnfail, label %nospawn, label %started
nospawn:
  call i32 @close(i32 %in_w)
  call i32 @close(i32 %out_r)
  call i32 @close(i32 %err_r)
  %rc64 = sext i32 %rc to i64
  %se = sub i64 -1, %rc64
  ret i64 %se
started:
  call ptr @signal(i32 13, ptr inttoptr (i64 1 to ptr))
  %so = getelementptr inbounds %\"std::process::Output\", ptr %out, i32 0, i32 1
  %sr = getelementptr inbounds %\"std::process::Output\", ptr %out, i32 0, i32 2
  call void @bmb_vec_alloc(ptr %so, i64 0, i64 1)
  call void @bmb_vec_alloc(ptr %sr, i64 0, i64 1)
  %win = alloca i32
  %rout = alloca i32
  %rerr = alloca i32
  %off = alloca i64
  store i32 %in_w, ptr %win
  store i32 %out_r, ptr %rout
  store i32 %err_r, ptr %rerr
  store i64 0, ptr %off
  %pfd = alloca [24 x i8]
  %noinput = icmp eq i64 %inn, 0
  br i1 %noinput, label %closein0, label %loop
closein0:
  call i32 @close(i32 %in_w)
  store i32 -1, ptr %win
  br label %loop
loop:
  %w0 = load i32, ptr %win
  %o0 = load i32, ptr %rout
  %e0 = load i32, ptr %rerr
  %wo = and i32 %w0, %o0
  %woe = and i32 %wo, %e0
  %allshut = icmp eq i32 %woe, -1
  br i1 %allshut, label %wait, label %poll
poll:
  store i32 %w0, ptr %pfd
  %ev0 = getelementptr inbounds i8, ptr %pfd, i64 4
  store i16 4, ptr %ev0
  %re0 = getelementptr inbounds i8, ptr %pfd, i64 6
  store i16 0, ptr %re0
  %fd1 = getelementptr inbounds i8, ptr %pfd, i64 8
  store i32 %o0, ptr %fd1
  %ev1 = getelementptr inbounds i8, ptr %pfd, i64 12
  store i16 1, ptr %ev1
  %re1 = getelementptr inbounds i8, ptr %pfd, i64 14
  store i16 0, ptr %re1
  %fd2 = getelementptr inbounds i8, ptr %pfd, i64 16
  store i32 %e0, ptr %fd2
  %ev2 = getelementptr inbounds i8, ptr %pfd, i64 20
  store i16 1, ptr %ev2
  %re2 = getelementptr inbounds i8, ptr %pfd, i64 22
  store i16 0, ptr %re2
  %pr = call i32 @poll(ptr %pfd, i64 3, i32 -1)
  %pbad = icmp slt i32 %pr, 0
  br i1 %pbad, label %loop, label %step_in
step_in:
  %r0 = load i16, ptr %re0
  %has0 = icmp ne i16 %r0, 0
  %live0 = icmp sge i32 %w0, 0
  %go0 = and i1 %has0, %live0
  br i1 %go0, label %write, label %step_out
write:
  %o = load i64, ptr %off
  %left = sub nuw i64 %inn, %o
  %big = icmp ugt i64 %left, 4096
  %chunk = select i1 %big, i64 4096, i64 %left
  %at = getelementptr inbounds i8, ptr %in, i64 %o
  %wr = call i64 @write(i32 %w0, ptr %at, i64 %chunk)
  %wfail = icmp slt i64 %wr, 0
  br i1 %wfail, label %werr, label %wrote
werr:
  %wintr = call i1 @bmb_interrupted()
  br i1 %wintr, label %step_out, label %closein
wrote:
  %o1 = add nuw i64 %o, %wr
  store i64 %o1, ptr %off
  %alldone = icmp eq i64 %o1, %inn
  br i1 %alldone, label %closein, label %step_out
closein:
  call i32 @close(i32 %w0)
  store i32 -1, ptr %win
  br label %step_out
step_out:
  %r1 = load i16, ptr %re1
  %has1 = icmp ne i16 %r1, 0
  %live1 = icmp sge i32 %o0, 0
  %go1 = and i1 %has1, %live1
  br i1 %go1, label %readout, label %step_err
readout:
  %g1 = call i64 @bmb_fd_read_some(i32 %o0, ptr %so)
  %end1 = icmp sle i64 %g1, 0
  br i1 %end1, label %closeout, label %step_err
closeout:
  call i32 @close(i32 %o0)
  store i32 -1, ptr %rout
  br label %step_err
step_err:
  %r2 = load i16, ptr %re2
  %has2 = icmp ne i16 %r2, 0
  %live2 = icmp sge i32 %e0, 0
  %go2 = and i1 %has2, %live2
  br i1 %go2, label %readerr, label %loop
readerr:
  %g2 = call i64 @bmb_fd_read_some(i32 %e0, ptr %sr)
  %end2 = icmp sle i64 %g2, 0
  br i1 %end2, label %closeerr, label %loop
closeerr:
  call i32 @close(i32 %e0)
  store i32 -1, ptr %rerr
  br label %loop
wait:
  %st = alloca i32
  store i32 0, ptr %st
  %pidv = load i32, ptr %pid
  br label %waiting
waiting:
  %wp = call i32 @waitpid(i32 %pidv, ptr %st, i32 0)
  %wbad = icmp slt i32 %wp, 0
  br i1 %wbad, label %waitretry, label %reaped
waitretry:
  %wi = call i1 @bmb_interrupted()
  br i1 %wi, label %waiting, label %reaped
reaped:
  %s = load i32, ptr %st
  %sig = and i32 %s, 127
  %exited = icmp eq i32 %sig, 0
  %code = lshr i32 %s, 8
  %code8 = and i32 %code, 255
  %bysig = add nuw i32 %sig, 128
  %status = select i1 %exited, i32 %code8, i32 %bysig
  %stp = getelementptr inbounds %\"std::process::Output\", ptr %out, i32 0, i32 0
  store i32 %status, ptr %stp
  ret i64 0
}
";

/// What every input and output built-in shares: `errno` as a result code,
/// whether the last failure was `EINTR`, and the message of an
/// `std::io::Error` (Rust's `Display`): `strerror` and ` (os error N)` for
/// an OS error, Rust's own text for invalid UTF-8 (code 0) and for a path
/// with a NUL byte (code 1000).
const IO_COMMON: &str = "declare ptr @__errno_location()
define internal i64 @bmb_errno() {
  %ep = call ptr @__errno_location()
  %e = load i32, ptr %ep
  %w = sext i32 %e to i64
  %neg = sub i64 -1, %w
  ret i64 %neg
}
define internal i1 @bmb_interrupted() {
  %ep = call ptr @__errno_location()
  %e = load i32, ptr %ep
  %i = icmp eq i32 %e, 4
  ret i1 %i
}
declare ptr @strerror(i32)
declare i32 @snprintf(ptr, i64, ptr, ...)
@bmb.os_error = private unnamed_addr constant [17 x i8] c\"%s (os error %d)\\00\"
@bmb.not_utf8 = private unnamed_addr constant [34 x i8] c\"stream did not contain valid UTF-8\"
@bmb.nul_path = private unnamed_addr constant [42 x i8] c\"file name contained an unexpected NUL byte\"
define internal void @bmb_io_text(ptr %h, ptr %t, i64 %n) {
  call void @bmb_vec_alloc(ptr %h, i64 %n, i64 1)
  %b = load ptr, ptr %h
  call void @llvm.memcpy.p0.p0.i64(ptr %b, ptr %t, i64 %n, i1 false)
  %lp = getelementptr inbounds %bmb.vec, ptr %h, i32 0, i32 1
  store i64 %n, ptr %lp
  ret void
}
define internal void @bmb_io_message(ptr %h, i64 %code) {
entry:
  %tmp = alloca [160 x i8]
  %z = icmp eq i64 %code, 0
  br i1 %z, label %utf, label %notutf
utf:
  call void @bmb_io_text(ptr %h, ptr @bmb.not_utf8, i64 34)
  ret void
notutf:
  %np = icmp eq i64 %code, 1000
  br i1 %np, label %nul, label %os
nul:
  call void @bmb_io_text(ptr %h, ptr @bmb.nul_path, i64 42)
  ret void
os:
  %c32 = trunc i64 %code to i32
  %s = call ptr @strerror(i32 %c32)
  %w = call i32 (ptr, i64, ptr, ...) @snprintf(ptr %tmp, i64 160, ptr @bmb.os_error, ptr %s, i32 %c32)
  %w64 = sext i32 %w to i64
  %over = icmp ugt i64 %w64, 159
  %n = select i1 %over, i64 159, i64 %w64
  call void @bmb_io_text(ptr %h, ptr %tmp, i64 %n)
  ret void
}
";

/// What an input or output built-in's runtime gives on success.
#[derive(Clone, Copy, PartialEq)]
enum Payload {
    /// The returned count is the value (`usize`).
    Count,
    /// The runtime writes the value (a `Vec<u8>` or `String` header)
    /// through its first argument.
    Value,
    /// `()`.
    Unit,
}

/// The built-ins whose result is `Result<T, std::io::Error>`: path, the
/// runtime function, its flattened parameters (as the call passes the
/// built-in's arguments) and what it gives on success.
const IO_BUILTINS: [(&str, &str, &[&str], Payload); 10] = [
    (
        "std::io::read_line",
        "bmb_stdin_line",
        &["ptr"],
        Payload::Count,
    ),
    (
        "std::io::read_to_end",
        "bmb_stdin_all",
        &["ptr"],
        Payload::Count,
    ),
    (
        "std::fs::read",
        "bmb_rt_fs_read",
        &["ptr", "i64"],
        Payload::Value,
    ),
    (
        "std::fs::read_to_string",
        "bmb_rt_fs_read_to_string",
        &["ptr", "i64"],
        Payload::Value,
    ),
    (
        "std::fs::write",
        "bmb_rt_fs_write",
        &["ptr", "i64", "ptr", "i64"],
        Payload::Unit,
    ),
    (
        "std::fs::rename",
        "bmb_rt_fs_rename",
        &["ptr", "i64", "ptr", "i64"],
        Payload::Unit,
    ),
    (
        "std::fs::remove_file",
        "bmb_rt_fs_remove_file",
        &["ptr", "i64"],
        Payload::Unit,
    ),
    (
        "std::fs::create_dir_all",
        "bmb_rt_fs_create_dir_all",
        &["ptr", "i64"],
        Payload::Unit,
    ),
    (
        "std::fs::read_dir",
        "bmb_rt_fs_read_dir",
        &["ptr", "i64"],
        Payload::Value,
    ),
    (
        "std::process::run",
        "bmb_rt_process_run",
        &["ptr", "i64", "ptr", "i64", "ptr", "i64"],
        Payload::Value,
    ),
];

/// Whether `path` is an input or output built-in (its call goes through
/// `wrapper`).
pub(super) fn is_io(path: &str) -> bool {
    IO_BUILTINS.iter().any(|(p, ..)| *p == path)
}

/// The function a call of the input or output built-in `path` calls:
/// `@bmb_io_read_line`, `@bmb_fs_read`, ..
pub(super) fn wrapper(path: &str) -> String {
    format!(
        "@bmb_{}",
        path.trim_start_matches("std::").replace("::", "_")
    )
}

/// The input and output runtime `ir` calls: each wrapper builds its
/// `Result` (with the program's own LLVM names of the result and error
/// types) from what its runtime returned (`n`, or `-1 - code`).
pub(super) fn io_runtime(ir: &str, tb: &Tables) -> String {
    let used: Vec<_> = IO_BUILTINS
        .iter()
        .filter(|(p, ..)| ir.contains(&format!("{}(", wrapper(p))))
        .collect();
    if used.is_empty() {
        return String::new();
    }
    let err_ty = data_ty(&crate::check::io_error());
    let mut out = format!(
        "define internal void @bmb_io_error(ptr %e, i64 %code) {{
  %os = icmp ult i64 %code, 1000
  %kept = select i1 %os, i64 %code, i64 0
  %c32 = trunc i64 %kept to i32
  %cp = getelementptr inbounds {err_ty}, ptr %e, i32 0, i32 0
  store i32 %c32, ptr %cp
  %mp = getelementptr inbounds {err_ty}, ptr %e, i32 0, i32 1
  call void @bmb_io_message(ptr %mp, i64 %code)
  ret void
}}
"
    );
    for (path, runtime, params, payload) in &used {
        let result = &tb.fns[*path].ret;
        let vs = enum_variants(result, tb).expect("`Result` of an input or output built-in");
        let tag = |v: &str| vs.iter().position(|(n, _)| n == v).expect("variant");
        let ok_ty = vs[tag("Ok")].1[0].clone();
        let decl: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{t} %a{i}"))
            .collect();
        let mut args = decl.clone();
        let (pay, store) = match payload {
            Payload::Value => {
                args.insert(0, "ptr %pay".into());
                let m = mem_ty(&ok_ty);
                (
                    format!("  %pay = alloca {m}\n"),
                    format!(
                        "  %v = getelementptr inbounds {okv}, ptr %out, i32 0, i32 1\n  %x = load {m}, ptr %pay\n  store {m} %x, ptr %v\n",
                        okv = view_ty(result, "Ok")
                    ),
                )
            }
            Payload::Count => (
                String::new(),
                format!(
                    "  %v = getelementptr inbounds {okv}, ptr %out, i32 0, i32 1\n  store i64 %r, ptr %v\n",
                    okv = view_ty(result, "Ok")
                ),
            ),
            Payload::Unit => (String::new(), String::new()),
        };
        let _ = write!(
            out,
            "define internal void {w}(ptr %out, {decl}) {{
entry:
{pay}  %r = call i64 @{runtime}({args})
  %tag = getelementptr inbounds {res}, ptr %out, i32 0, i32 0
  %neg = icmp slt i64 %r, 0
  br i1 %neg, label %err, label %ok
ok:
  store i64 {ok}, ptr %tag
{store}  ret void
err:
  store i64 {err}, ptr %tag
  %e = getelementptr inbounds {errv}, ptr %out, i32 0, i32 1
  %code = sub i64 -1, %r
  call void @bmb_io_error(ptr %e, i64 %code)
  ret void
}}
",
            w = wrapper(path),
            decl = decl.join(", "),
            args = args.join(", "),
            res = data_ty(result),
            ok = tag("Ok"),
            err = tag("Err"),
            errv = view_ty(result, "Err"),
        );
    }
    if used.iter().any(|(p, ..)| p.starts_with("std::io::")) {
        out.push_str(STDIN);
    }
    let runs = used.iter().any(|(p, ..)| *p == "std::process::run");
    if runs || used.iter().any(|(p, ..)| p.starts_with("std::fs::")) {
        out.push_str(FS);
    }
    if runs {
        out.push_str(RUN);
    }
    let dirs = [
        "std::fs::rename",
        "std::fs::remove_file",
        "std::fs::create_dir_all",
        "std::fs::read_dir",
    ];
    if used.iter().any(|(p, ..)| dirs.contains(p)) {
        out.push_str(DIRS);
    }
    out.push_str(IO_COMMON);
    out
}

/// The stack of a program with recursion (ADR-0091 §5, slice (f)): its
/// `main` runs on a thread with 256 MiB of stack (stage0's `STACK_BYTES`,
/// for nesting 256 deep in the compiler's recursive passes), which `main`
/// joins; the memory is only reserved until it is touched. Should the
/// thread not start, `main` runs on the process's own stack.
pub(super) fn big_stack(main: &str) -> String {
    format!(
        "declare i32 @pthread_attr_init(ptr)
declare i32 @pthread_attr_setstacksize(ptr, i64)
declare i32 @pthread_create(ptr, ptr, ptr, ptr)
declare i32 @pthread_join(i64, ptr)
define internal ptr @bmb_main_thread(ptr %u) {{
  call void {main}()
  ret ptr null
}}
define internal void @bmb_big_stack() {{
entry:
  %attr = alloca [64 x i8]
  call i32 @pthread_attr_init(ptr %attr)
  call i32 @pthread_attr_setstacksize(ptr %attr, i64 {STACK_BYTES})
  %t = alloca i64
  %rc = call i32 @pthread_create(ptr %t, ptr %attr, ptr @bmb_main_thread, ptr null)
  %fail = icmp ne i32 %rc, 0
  br i1 %fail, label %direct, label %join
direct:
  call void {main}()
  ret void
join:
  %tv = load i64, ptr %t
  call i32 @pthread_join(i64 %tv, ptr null)
  ret void
}}
"
    )
}

/// The stack of `big_stack`'s thread.
pub(super) const STACK_BYTES: u64 = 256 << 20;
