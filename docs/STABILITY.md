# BMB Language Stability

> Version: v0.86 (Alpha)
> Status: Language Freeze

This document defines the stability guarantees for the BMB programming language starting from v0.86.

---

## Stability Tiers

### Tier 1: Stable (Frozen)

Features in this tier are **frozen** and will not change in backward-incompatible ways.

| Category | Features |
|----------|----------|
| **Primitive Types** | `i64`, `f64`, `bool`, `char`, `String`, `()` |
| **Composite Types** | `struct`, `enum`, `Array<T>`, `Vec<T>`, `Box<T>`, `Tuple` |
| **Concurrency Types** | `Thread`, `Mutex<T>`, `RwLock<T>`, `Atomic<T>`, `Channel<T>`, `Sender<T>`, `Receiver<T>`, `Future<T>`, `Barrier`, `Condvar`, `ThreadPool`, `Scope` |
| **Async Types** | `AsyncFile`, `AsyncSocket` |
| **Control Flow** | `if`/`else`, `match`, `while`, `for`/`in`, `loop`, `break`, `continue`, `return` |
| **Functions** | `fn`, `async fn`, parameter patterns, generics, where clauses |
| **Contracts** | `pre`, `post`, `invariant`, `assert` |
| **Operators** | Arithmetic, comparison, logical, bitwise |
| **Ownership** | `own`, `&`, `&mut`, move semantics |

### Tier 2: Stable API

Runtime functions and methods that are guaranteed to be available:

#### Core Functions
```bmb
// Output
fn print(s: String) -> ()
fn println(s: String) -> ()

// Assertions
fn assert(cond: bool) -> ()
fn todo -> !

// Conversion
fn to_string<T>(value: T) -> String
fn parse_int(s: String) -> i64
fn parse_float(s: String) -> f64
```

#### Thread Functions
```bmb
fn thread_spawn(task: fn() -> ()) -> Thread
fn thread_join(thread: Thread) -> ()
fn thread_sleep(ms: i64) -> ()
fn thread_yield() -> ()
fn thread_pool_new(size: i64) -> ThreadPool
fn thread_scope() -> Scope
```

#### Synchronization
```bmb
fn mutex_new<T>(value: T) -> Mutex<T>
fn rwlock_new<T>(value: T) -> RwLock<T>
fn atomic_new<T>(value: T) -> Atomic<T>
fn barrier_new(count: i64) -> Barrier
fn condvar_new() -> Condvar
fn channel<T>() -> (Sender<T>, Receiver<T>)
fn channel_bounded<T>(capacity: i64) -> (Sender<T>, Receiver<T>)
```

#### Async Functions
```bmb
fn block_on<T>(future: Future<T>) -> T
fn async_open(path: String) -> Future<AsyncFile>
fn tcp_connect(host: String, port: i64) -> Future<AsyncSocket>
```

### Tier 3: Experimental

Features that may change or be removed:

| Feature | Status | Notes |
|---------|--------|-------|
| `select` macro | Experimental | Basic polling implemented, full I/O multiplexing planned |
| Platform-specific I/O | Experimental | epoll/IOCP integration pending |

---

## Backward Compatibility Policy

Starting from v0.86:

### Guaranteed
- Code that compiles with v0.86 will compile with all future v0.8x, v0.9x versions
- Runtime behavior of Tier 1 features will not change
- Tier 2 API signatures will not change

### Not Guaranteed
- Compile times may vary
- Optimization behavior may change (performance may improve)
- Error message format may change
- Internal compiler representation (MIR, etc.)

---

## Deprecation Policy

When a feature needs to be deprecated:

1. **v0.N**: Feature marked deprecated with compiler warning
2. **v0.N+2**: Feature removed

Minimum deprecation period: 2 minor versions

---

## Version Numbering

```
v0.MAJOR.MINOR

Examples:
- v0.86.0  - Language Freeze (Alpha)
- v0.90.0  - Beta release
- v0.98.0  - Release Candidate 1
- v1.0.0   - Stable release
```

---

## Frozen Keywords

The following keywords are frozen and will not change meaning:

```
// Types
i64 f64 bool char String
struct enum fn type

// Control Flow
if else match while for in loop break continue return

// Ownership
own mut let const

// Contracts
pre post invariant assert

// Concurrency
async await Future Thread Mutex RwLock Atomic Channel
Sender Receiver Barrier Condvar ThreadPool Scope
AsyncFile AsyncSocket

// Async
select

// Other
pub use extern impl trait where self Self
true false todo
```

---

## Frozen Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `%` |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` |
| Logical | `&&`, `\|\|`, `!` |
| Bitwise | `&`, `\|`, `^`, `<<`, `>>` |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=` |
| Access | `.`, `::`, `->` |
| Range | `..`, `..=` |

---

## Platform Support

### Tier 1 Platforms (Full Support)
- Windows x64 (MSVC, MinGW)
- Linux x64 (glibc)

### Tier 2 Platforms (Best Effort)
- Linux ARM64
- macOS x64/ARM64

### Tier 3 Platforms (Community)
- WebAssembly
- Other architectures

---

## Reporting Stability Issues

If you find behavior that contradicts this stability document, please report it as a bug. Stability violations are treated as high-priority issues.

Issue tracker: https://github.com/iyulab/lang-bmb/issues
