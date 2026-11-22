# ADR-0031: Heap `Vec<T>` and the ownership·move check

- Status: Accepted (the length obligation of `v.push(x)` is superseded by ADR-0124: a fact the runtime keeps. 2026-10-25. Following the approval to start 2G and the instruction to adopt the recommendation, backlog 2G-07. "Updates at acceptance" below is where it differs from the proposal). The exclusion of structs holding a `Vec` and of `Vec<Vec<T>>` in §1 and update at acceptance 2 is superseded by ADR-0047. The `if`·`else` and `match` join points of the §2 join point rule are superseded by ADR-0081 (static free at the end of the arm that did not move)
- Date: 2026-10-24 (proposed), 2026-10-25 (accepted)
- Related: DESIGN.md §4.1 (lifetimes, failure)·§5·§8.1 (absence of aliasing), ADR-0015, ADR-0016, ADR-0017 §3, ADR-0024, ADR-0028·0029·0030 (proposed), ROADMAP 2G, backlog 2G-01

## Context
The values of M1 are only integers, `bool` and fixed-length arrays `[T; N]`, and references are only `&[T]`·`&mut [T]` in parameter position (ADR-0016). There is no data whose size is fixed at run time, so tasks are limited to fixed-size array algorithms (measurement report `dev-docs/measurements/m1.md` §2 bias 1). There is no type for which a move has meaning either, so the ownership check is only the name and aliasing rules (E0410, ADR-0024).

DESIGN.md §5 set ownership and moves + second-class references. References cannot be stored or returned, so there are no lifetimes, and structures that hold references (iterators, views) are replaced by handles and indices. The first heap type for that is `Vec`.

Related evidence:
- The slice length upper bound fact (ADR-0017 §3, `isize::MAX / size_of(T)`) is already in the encoding (`LEN_MAX_WORD` in `compiler/src/verify.rs`). The same fact fits `Vec`.
- M1 friction: the element sum loop was not proved without a length upper bound (`a.len() <= N`) (m1 §6). `Vec` needs the same condition.
- The speed gain from dropping checks by proof was not observed in the baseline benchmark (1E-08, m1 §4). No gain is claimed in advance for strengthening `noalias` on `Vec` either.

## Updates at acceptance (2026-10-25, on top of the 2B surface)

After the proposal, 2B (ADR-0026·0027) and 2C (ADR-0028) arrived. Decided at acceptance:

1. **2B dependency resolved (open question 1):** `pop` returns `Option<T>` (the built-in `Option` of ADR-0026). The handle newtype is a struct with a named field, `struct Id { i: usize }` (a tuple struct is E0233).
2. **Element types (open question 2):** `Copy` scalars and `Copy` structs and enums, but data types with an array inside the element are excluded (the same proof gate boundary as slices of data types, E0814). Structs holding a `Vec` and `Vec<Vec<T>>` are out of scope (E0402).
3. **"Maybe moved" (open question 3):** starts as E0414, as proposed. The frequency is measured in the `m2` tasks.
4. **`with_capacity`·`drop` (open question 4):** as proposed, `with_capacity` allowed, `drop` rejected (E0219).
5. **Buffer `noalias` metadata (open question 5):** not emitted before measurement. Decided after measuring with the `Vec` version of the baseline benchmark.
6. **`resize`·`extend_from_slice` (open question 6):** outside this scope. Added separately after measuring encoding stability with `bmb stability`.
7. **`alloc` effect:** ADR-0030 (2F) is decided after 1E-07. Until then `Vec` operations have no effect (the current effect set `pure`·`io` as is, and a `pure` function can use `Vec`). When ADR-0030 is accepted, the effect rule of §4 applies as is, and functions with an insufficient declaration are fixed by the mechanical fix of the effect check diagnostic. 2G-04 goes only as far as the free points and the runtime.
8. **`vec!` (§7):** the existing E0202 replacement `vec![a, b]` → `[a, b]` changes meaning once `Vec` arrives. The replacement is removed and only the hint is kept (2G-06).
9. **Address check build (§8):** the container may not have the AddressSanitizer runtime. The fuzzer's third build is ASan (including LeakSanitizer) if ASan is present, valgrind memcheck (`--leak-check=full`, fail on error) if not, and fails if neither is present (it does not skip silently). The nightly runs with ASan.
10. **Diagnostic numbers:** E0218·E0219·E0413·E0414·E0514·E0816 were confirmed free in `compiler/src/diag.rs`.

## Decision

### 1. Scope
- Element types are `Copy` scalars (`i64`·`u64`·`usize`·`bool`) and, after 2B, `Copy` structs and enums. `Vec<Vec<T>>`, structs holding a `Vec`, and `Box`·`Rc`·`String`·`HashMap` are outside M2.
- **API (one normal form):** `Vec::new()`, `Vec::with_capacity(n)`, `v.push(x)`, `v.pop()`, `v.len()`, `v[i]` (read and assign), `v.truncate(n)`, `v.clone()`, `v.as_slice()`, `v.as_mut_slice()`. `pop` returns `Option<T>`, so it is **after 2B** (2G is after 2B, the same as the ROADMAP default order).
- Parameter types: `Vec<T>` (move), `&Vec<T>`, `&mut Vec<T>` (the length can change). To a slice parameter, `v.as_slice()`·`v.as_mut_slice()` is passed in argument position (second-class references, so allowed only in argument position). Return is only a `Vec<T>` value.

### 2. Ownership·move check (M2 scope)
- Using a `Vec` value on the right of a `let` or assignment, as a value argument, or as a tail expression is a move. Using or referencing a moved variable is E0413. Assigning to a moved variable again initializes it.
- **Join point rule:** at a branch join point and at a loop head, a variable must be either "definitely initialized" or "definitely moved". "Maybe moved" is E0414. This is stricter than Rust, but the free point is fixed statically and there is no runtime drop flag. Moving an outer variable inside a loop and assigning it again in the same iteration (`acc = f(acc);`) is allowed.
- `&mut v`·`v.as_mut_slice()` and another use of `v` (move, reference) in the same call is E0410 (an extension of the existing aliasing rule). Scalar reads computed as values (`v.len()`, `v[i]`) are allowed per the ADR-0024 rule.
- References are not stored or returned (ADR-0015), so the borrow period is one call. "Moved while borrowed" arises only through the rule above, and no lifetime analysis is needed.
- Freeing: a definitely initialized `Vec` variable is freed at the end of its block. A moved variable is not freed. There is no explicit `drop(v)` (open question 4).

### 3. Proof encoding
- A `Vec<T>` value is the same pair as a slice, `(Array Int Int, Int)` (elements, length) (ADR-0017 §3). It is a value, so assignment is a copy of the pair, and moves are handled only in the checker. Capacity is not in the model (it cannot be observed).
- For the length, the range fact `0 ≤ len ≤ LEN_MAX(T)` is asserted at the point of use (the existing `len_max`).

| Operation | Obligation | Result |
| --- | --- | --- |
| `Vec::new()`, `Vec::with_capacity(n)` | `with_capacity`: `n ≤ LEN_MAX(T)` (Rust panics above it) → E0816 | Length 0, elements unspecified |
| `v.push(x)` | `len < LEN_MAX(T)` → E0816 | `store(e, len, x)`, `len + 1` |
| `v.pop()` (2B) | None | `None` if `len = 0`, otherwise `Some(e[len−1])` and `len − 1` |
| `v[i]`, `v[i] = x` | `i < len` → the existing E0805 | `select`/`store` |
| `v.truncate(n)` | None | `len = min(len, n)` |
| `v.clone()` | None | The same pair |
| `as_slice`/`as_mut_slice` | None | The same pair. A callee given `&mut [T]` cannot change the length, so length preservation is a free frame fact |

- After a call with a `&mut Vec<T>` argument, both elements and length become fresh constants and only the callee's `ensures` is assumed (the same as existing `&mut` slices, with the length added). `old(v)` is allowed on `&mut Vec<T>` parameters (extension of the E0508 rule).
- **`Vec` equality in specs:** elements past the length remain in the array after `pop`·`truncate`, so array equality is not `Vec` equality. `==` between `Vec`s in a spec is rejected with E0514, and length and elements are compared with predicates.
- In a spec, a `Vec` is passed to the `&[T]` parameter of a `spec fn` as `v.as_slice()` (one form, the same as executable code).
- Out of memory is not a proof target. It shows as the `alloc` effect (ADR-0030).
- There is no new auxiliary fact. If one is added, a machine proof goes in `lemmas.rs` per ADR-0018 §2.

### 4. Code generation and runtime
- The header `{ptr, len, cap}` is on the stack. `push` does `realloc` if `len == cap` (capacity doubled, minimum 4), and an allocation failure is `abort`. The capacity computation does not overflow because `len ≤ LEN_MAX(T)` is proved (clamped to `cap ≤ LEN_MAX(T)`).
- A proved index is an unchecked `getelementptr inbounds` as now (`; proved E0805`). Freeing is `free` at the static points of §2.
- **Aliasing:** a `&mut Vec<T>` parameter has `noalias` on the header pointer, and `&Vec<T>` has `readonly` (the same rationale as ADR-0015). The element buffer is a pointer read from the header, so the parameter `noalias` does not cover it. The buffer is always owned by that one `Vec` (clone is a deep copy, there is no shared type) and references are second-class, so per-parameter `!alias.scope`/`!noalias` metadata can be attached to buffer accesses. This fact comes from the type rules. A speed gain is claimed only after measurement (open question 5).
- An `as_mut_slice()` argument uses the existing `&mut [T]` lowering (buffer pointer + length, `noalias`) as is.
- Effects: `Vec::new`·`with_capacity`·`push`·`clone` are `alloc` (ADR-0030). `Vec::new` does not really allocate, but the rule is kept simple. Freeing·`pop`·`truncate`·indexing have no effect.

### 5. Handle·index pattern (DESIGN.md §5)
- Instead of a structure that holds references, use a `Vec<T>` store + `usize` indices (after 2B, a newtype such as `struct Id(usize)`). Handle validity is written in contracts with a predicate such as `spec fn valid(h: usize, s: &[T]) -> bool { h < s.len() }`.
- An API whose store only grows writes `old(s).len() <= s.len()` and preservation of existing elements in `ensures`. If this frame condition is left out, the caller's handles become invalid and the proof fails. It is seen as recurring friction and kept as a candidate for automatic filling (DESIGN.md §8.2 "frame condition candidates").
- Advanced patterns such as generation-numbered handles and free lists are outside M2.

### 6. Map and hashes
- `Vec` operations are built-ins, so they create no call edge. `Vec<T>`·`&Vec<T>`·`&mut Vec<T>` appear in L1 signatures. `&mut Vec` parameters are marked as sources of the mutation and data flow view (DESIGN.md §7.2) (used in 3B).
- The normal form and hash rules (ADR-0019) do not change. The type text enters the normal form as is.

### 7. Rust habits to reject, and diagnostics (numbers are proposals)

| Habit | Handling | Code |
| --- | --- | --- |
| `vec![..]`, `vec![x; n]` | The existing E0202. Hint: `Vec::with_capacity(n)` + a `push` loop | E0202 (existing) |
| Passing `f(&v)` to a `&[T]` parameter (Deref coercion) | Mechanically replaced with `v.as_slice()`/`v.as_mut_slice()` | E0218 |
| `drop(v)`, `std::mem::take/replace/swap` | Rejected. Freeing is at the end of the block | E0219 |
| `v.iter()`, `for x in &v` | The existing E0404·E0204 | Existing |
| `v.get(i)`, `first()`, `last()`, `is_empty()` | The existing E0404. Hint: proved `v[i]`, `v.len() == 0` | E0404 (existing) |
| `Box`·`Rc`·`RefCell`·`String`·`HashMap`·`Vec<Vec<T>>` | The existing E0402. Hint per type | E0402 (existing) |
| Use after move | Error + the move point shown | E0413 |
| "Maybe moved" at a join point | Error. Hint to move in both arms or assign again | E0414 |
| `Vec ==` in a spec | Error | E0514 |
| `push`·`with_capacity` can exceed the maximum length | Proof failure + counterexample | E0816 |

### 8. Tests and fuzzer
- Corpus: the move check (`tests/check/err/moves.bmb`: use after move, maybe moved, `&mut` and move in the same call), `tests/verify` (pass and fail for push, pop, index, frame, handle validity), `tests/run` (growth, shrinking, cloning, return).
- Soundness fuzzer (new `soundness_vec.rs`): randomly generates push, pop, truncate, index, clone and move on `Vec` locals and `&mut Vec` parameters. The interpreter models `Vec` as a value. The properties are as before (a proved function has no error and satisfies the postcondition, a confirmed counterexample really breaks it, the two builds give the same output as the interpreter). In addition, **a third build is made with AddressSanitizer (including LeakSanitizer)** to check the free points (double free, use after free, leak). 0 violations over 60 batches is 2G pass criterion 2.
- Effects: the interpreter records allocations and checks "dynamic effects ⊆ static effects" (ADR-0030 §8).

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **Value pair model + move check in the checker (proposed for acceptance)** | Uses the existing slice encoding and the length upper bound fact as is. The prover has no heap or pointers. Thanks to second-class references, a move check alone is enough, without a borrow checker |
| Heap model (pointers, ownership predicates, separation logic) | Second-class references and unique ownership already remove aliasing, so nothing is gained, and the solver burden is large. Rejected |
| Verus-style `v@` (mathematical sequence view) and the `Seq` theory | Specs are cleaner to express, but a new spec type and a bundle of axioms enter the trust base. The current `(Array, Int)` pair can answer the same questions. Reviewed again if spec `==` becomes necessary |
| Allow "maybe moved" like Rust + drop flags | Fits the Rust habit better, but code generation gets runtime flags and the fuzzer has more states to cover. Start strict and review by the E0414 frequency (open question 3) |
| `pop` without 2B, returning `T` with `requires len > 0` | A meaning different from Rust's `pop() -> Option<T>` attaches to the same name (principle 8, ADR-0013). Rejected. Before 2B the same job is done with `v[v.len() - 1]` + `truncate` |
| Allow `&Vec<T>` directly for a `&[T]` parameter | This is Rust's Deref coercion itself. The call resolution row of DESIGN.md §4.1 forbids it. Rejected |

## Consequences
- Free code and a runtime (`malloc`/`realloc`/`free`/`abort`) appear for the first time. The executable depends on the libc allocator.
- It must go in together with the `alloc` effect (ADR-0030). It depends on 2B (`Option`).
- Where the move rule is stricter than Rust (maybe moved, `drop`), the cost is measured by the Rust habit error rate.

## Open questions (decided by measurement 1E-07 or by the maintainer)
1. **Priority of 2G and the 2B dependency:** `pop` and the handle newtype come after 2B. Whether to close 2B first, or to add `Vec` without `pop` first.
2. **Element type scope:** `Copy` elements only (proposed) versus up to structs holding a `Vec` and `Vec<Vec<T>>`. The latter needs partial moves and nested freeing.
3. **Ban on "maybe moved" (E0414):** if the E0414 frequency is high in the `m2` tasks, drop flags are introduced to match Rust.
4. **`drop(v)`·`Vec::with_capacity`:** explicit early freeing and a capacity hint are performance tools. The maintainer decides whether to follow the one-normal-form principle or the performance objective (proposal: `with_capacity` allowed, `drop` rejected).
5. **Gain of the buffer `noalias` metadata:** measured by adding a `Vec` version to the baseline benchmark. If there is no gain, the metadata is not emitted (code generation simplicity).
6. **`resize`·`extend_from_slice`:** needed as a replacement for the `vec![x; n]` habit, but `resize` needs constant arrays or a `lambda` encoding. Added after measuring stability with `bmb stability` on the pinned Z3 version (4.15.4).
