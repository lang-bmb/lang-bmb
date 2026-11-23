# ADR-0091: System interface and runtime

- Status: Accepted (2026-11-06, 4C-01 completed by approval of the maintainer. The draft is 4C-03)
- Date: 2026-11-05
- Related: ADR-0029 (escape hatches and trust levels), ADR-0030 (effects `io`·`nondet`), ADR-0037 (self-hosting), ADR-0050 (strings), `dev-docs/measurements/selfhost-gap.md` §3.4·§5.3, `selfhost-port-map.md` §4, ROADMAP "full sequence of the language" ③

## Context

- Generated binaries only output (`print_i64`·`print_u64`·`print_bytes`·`print_str`·`print_f64`, libc's `printf`·`fwrite`·`fflush`, the heap is `malloc`·`realloc`·`free`). **There is no input.** Every run program carries its data in the source.
- The external contacts the compiler (stage0) uses are all in §3.4 of the gap analysis: reading sources (directory listing, UTF-8 validation), atomic writes (temporary file + rename), Z3 with interactive pipes, clang with captured output and `--version`, command-line arguments and exit codes 0·1·2, one line and the whole of standard input, standard error, one environment variable (`BMB_Z3_RLIMIT`), a monotonic clock (the Z3 reuse window), a 256MiB stack, and the `io::Error` text that goes into output bytes (`strerror` + ` (os error N)`). The 4D port (P0) and bootstrap stage 1 cannot start without these.

## Decision

1. **A closed list of builtins, no FFI.** System contact is only the builtin functions the compiler knows (the same way as the current `print_*`). There are no user-declared `extern` functions (bindings are 5E, after principles 4·6 are demonstrated). The implementation of the builtins is a runtime that goes into the generated IR (LLVM IR text, on top of libc): no C file and no separate library (S3 "0 runtime outside libc" is kept).
2. **Rust's names, signatures that fit BMB.** Paths and names follow the Rust standard library (S1, the same principle as ADR-0050 §2). Only where Rust uses iterators, builders or trait objects is the shape changed to a BMB one, and the difference is explained by habit diagnostics.

   | Area | Builtins | Difference from Rust |
   | --- | --- | --- |
   | Files | `std::fs::read(p: &str) -> Result<Vec<u8>, io::Error>`, `read_to_string`, `write(p, data: &[u8])`, `rename`, `remove_file`, `create_dir_all`, `read_dir(p) -> Result<Vec<String>, io::Error>` (names, sorted in byte order) | `read_dir` gives sorted names instead of an iterator (determinism), paths are `&str` |
   | Standard input/output | `std::io::read_line(buf: &mut String) -> Result<usize, io::Error>`, `std::io::read_to_end(buf: &mut Vec<u8>)`, `eprint_str(s: &str)`, `flush()` | functions instead of the `stdin().read_line` chain |
   | Processes | `std::process::exit(code: i32) -> !`, `std::env::args() -> Vec<String>`, `std::env::var(name: &str) -> Option<String>`, `std::process::run(prog, args: &[String], input: &[u8]) -> Result<Output, io::Error>` (until it ends, `Output { status, stdout, stderr }`), `std::process::spawn(prog, args) -> Result<Child, io::Error>` and `write_all`·`read_line`·`wait` of `Child` (a move type, released by `wait`) | two functions instead of the `Command` builder, `args` is `Vec<String>` instead of an iterator |
   | Time | `std::time::monotonic_ms() -> u64` | a number instead of `Instant` |
   | Errors | the builtin struct `io::Error { code: i32, message: String }`, `message` is the same bytes as Rust's `Display` (`strerror` text + ` (os error N)`) | fields instead of a trait |

3. **Effects and trust.** Every system builtin has `io`, and those whose value can differ per run (`env::var`·`args`·the clock·the contents of files and processes) also have `nondet` (ADR-0030). In proofs the results have only type facts (a length bound; UTF-8 is the construction rule of ADR-0050: `read_to_string` is `Ok` after validation): it is the outside world without contracts. The trust level is a builtin like `print_*` (TCB; the list of runtime functions is added to the table of `dev-docs/HEALTH.md` §3), and it is not counted in the unproved area (ADR-0029). In the self-proof of 4F, the 15 to 25 runtime boundary definitions (port map §5.4) become this list.
4. **Runtime implementation.** IR functions that wrap libc's POSIX functions (`open`·`read`·`write`·`close`·`rename`·`unlink`·`mkdir`·`opendir`/`readdir`·`posix_spawnp`·`pipe`·`waitpid`·`getenv`·`clock_gettime`·`exit`). Only what is used goes into the module (the same rule as the current `print_f64` and text runtime). A failure carries `errno` into `io::Error`. Allocation failure is `abort`, as now.
5. **Stack.** The runtime runs the BMB `main` on a thread with a large stack (`pthread_create`, default 256MiB: stage0's `STACK_BYTES`, recursion nesting 256). The value is set by measurement when this ADR is adopted.
6. **Determinism.** Everything that goes into tool output is in a fixed order: `read_dir` is sorted, the environment is only read by name, the clock is not used in output (the same rule as stage0: in differential tests, time-dependent paths are not used for the verdict, port map §4).

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **Closed list of builtins + IR runtime (adopted)** | One compiler, tool determinism, a small TCB. It covers all the contacts self-hosting needs (§3.4) and fits a stage without bindings |
| General FFI (`extern "C"` + `external_body`) | A surface of pointers, lifetimes and ABI enters the language. It is a decision of the ecosystem stage (5E), and the demand now is a fixed list |
| A runtime library written in C | A second implementation language and a build step arise (S3). With IR text it links with the same single `clang` |
| Rust signatures as is (`Command`, `impl Iterator`) | The language has no builders, iterators or trait objects, so only the surface would be similar and it could not be used. Only the names are followed and the shape is changed |

## Consequences

- The implementation (4C-02) in pieces: (a) `args`·`exit`·standard error·standard input, (b) files, (c) directories and atomic writes, (d) environment and clock, (e) piped processes, (f) the stack. For each piece: a run program (runs in a temporary directory), error path tests, and the `io::Error` text checked against Rust.
- The runtime functions enter the TCB table of HEALTH §3, and S3 stays `[]`.
- 4D-02 (P0) and bootstrap stage 1 (4E-02) are unblocked.

## Confirmed during implementation (2026-11-07, the rest of 4C-02 (e), accepted autonomously: an implementation decision within the scope of this ADR)

- Open question 2 (release of `Child`): Rust's `Drop` is followed. A `Child` that leaves scope without `wait` closes both pipes and does not wait (the child sees the end of input). It is not rejected (E04xx) in order to keep it the same as the Rust semantics and not add a new rule (must be consumed) to move checking. `wait` has a value receiver (`self: Child`), so it can be called only once.
- The operations of `Child` are `impl` functions of the system type `std::process::Child` (`std::process::Child::write_all` and so on), so both the method syntax of ADR-0051 (`c.write_all(..)`) and path calls work. Instead of Rust's `child.stdin`·`child.stdout` (option fields and `BufReader`) they are methods of `Child`: only standard input and output are pipes, and standard error is inherited (the default of Rust's `spawn`).
- The fields (`pid` and the runtime's two `FILE`s) cannot be constructed or read by hand (E0436). `Child` is a move type even though its fields are only integers (`check::is_move`).
- Open question 1 (`io::Error`) stays a struct with fields: method syntax now exists, but there is no place yet that uses `raw_os_error()`·`kind()` (`code`·`message` are enough). Methods are added when the demand appears.
- Measurement: a BMB program that sends three queries in turn to a pinned Z3 and reads the answers (`a_program_talks_to_z3` in `tests/system.rs`), 0 leaks under valgrind.

## Confirmed during implementation (2026-11-07, 4C-06, accepted autonomously: the remaining contacts of stage0 found by the 4C closing audit)

- Four are added to the closed list (the table of §2): `std::fs::metadata`·`symlink_metadata` (Rust names; the result is the struct with fields `std::fs::Metadata { is_dir, is_file, is_symlink, len }`: fields instead of Rust's methods, the same shape as `io::Error`), `std::fs::write_new(path, data)` (Rust's `File::create_new(path)?.write_all(data)`: the function shape of BMB, which has no file type; only the name differs), the third argument `std::process::Stdio` of `std::process::spawn` (`Inherit`·`Null`: Rust's `Stdio::inherit()`·`Stdio::null()`, an argument instead of a builder).
- `SIGPIPE`: a generated program ignores it at startup (the Rust runtime), and children are started with the default (Rust's `Command`). If a write or flush of standard output fails with `EPIPE`, the exit code is 141 (stage0's `CLOSED_STDOUT`: the same number as the shell status of a process killed by the signal, but it is a normal exit so the parent reads the code). Rust's `println!` panics (101) here, but the criterion is replacing the behaviour of stage0, so 141 was chosen.

## Open questions

1. Whether `io::Error` is a struct with fields or has methods like Rust (`raw_os_error()`, `kind()`): decided together with method syntax (2M).
2. Release of `Child` (end of block without `wait`): whether to wait (Rust's `Drop` does not wait) or reject (E04xx).
3. Whether paths are `&str` or bytes (`&[u8]`; Unix paths may not be UTF-8): stage0 uses only `&str`.
4. Other platforms (5C): this list is POSIX.
