# ADR-0030: Effect extension (`alloc`·`panic`·`nondet`) and the area that needs testing

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved"). Open questions are closed in the implementation (2F-02) as this ADR recommends, and anything to change goes to a new ADR. §6 (code generation attributes) is superseded by ADR-0084
- Date: 2026-10-24
- Related: DESIGN.md §4.1 (effects)·principle 4·§7.2 (effect view)·§8.1 (effect-based optimization), ADR-0003, ADR-0016, ADR-0020, ADR-0028·0029·0031 (proposed), ROADMAP 2F, backlog 2F-01·2D-11

## Context
The effects of M1 are two, `pure` and `io` (ADR-0016). Every executable function must have `#[effects(..)]`, and the check is on direct calls only: a `pure` function calling an `io` function is E0603 (the call check in `compiler/src/check.rs`). Since every function declares its effects, the direct rule alone gives transitivity. `status` counts the number of `io_defs` (`compiler/src/map.rs`).

Why two effects are not enough in M2:
- **Allocation.** `Vec` (ADR-0031 proposal) allocates, and aborts when out of memory. Whether a function allocates affects pure-region optimization and the area that needs testing.
- **Abort.** In M1 every obligation must be proved for a build, so there is no trap at run time. Once escape hatches (ADR-0029 proposal) arrive, some definitions keep checks. That fact must be visible to callers.
- **Non-determinism.** Time, random numbers and input break the premise that the same input gives the same result. There is no source yet.

ADR-0003 decided to derive the area that needs testing from effect declarations. The current derivation is only "the number of `io` definitions". There is no measurement evidence yet for this effect extension (the 1E tasks use only `pure`/`io`).

## Decision (proposed)

### 1. Effect set (lattice)
- There are four atomic effects. An effect is a **set** of them and the lattice is the subset order. `pure` is the name of the empty set.

| Effect | Meaning | Source |
| --- | --- | --- |
| `alloc` | Heap allocation. Aborts when out of memory (the same as Rust's `handle_alloc_error`) | `Vec` creation, growth, cloning (ADR-0031). Freeing is not an effect (it does not fail and is deterministic) |
| `io` | Interaction with the outside. The order is observed | `print_i64`, `print_u64` (ADR-0028) |
| `nondet` | The result depends on something outside the arguments | Introduced together with the first source (input, time, random built-ins) (open question 2) |
| `panic` | May end in a trap at run time | Definitions compiled in full check mode (ADR-0029 §4) |

- `io` does not imply the other effects (orthogonal). An input built-in has both `io` and `nondet`.
- The out-of-memory abort belongs to `alloc`, not `panic`. Otherwise every allocating function would be `panic` and `panic` would lose its information.

### 2. Surface
- `#[effects(pure)]` or `#[effects(e1, e2, ..)]`. `pure` cannot be written together with other effects (E0607). The normal form is in name order (`alloc, io, nondet, panic`) and a duplicate is an error (E0607).
- M1 code stays valid as is. `#[effects(io)]` is `{io}`.
- `main` must include `io`. It can add other effects (the condition of E0701 is widened to "includes `io`". Only the message is updated).

### 3. Check and propagation
- **Required effects** `req(f)` = the own effects of the body (built-in calls, `Vec` operations, `panic` if in full check mode) ∪ the **declared** effects of executable callees. The declaration `decl(f)` must include `req(f)`. Otherwise E0606 (per missing effect, with a `fix.replacement` that fixes `#[effects(..)]`). The existing E0603 keeps its meaning as the code for a `pure` function calling `io`.
- A declaration wider than required is allowed. The declaration is an interface. If the callee body changes and its effects grow, the error is at the callee, not at the caller (the same principle as the contract firewall). An unused declared effect is reported by a `bmb lint` warning (the next free number after W09xx) so it gets narrowed.
- The **transitive effects** for the map, `eff*(f)` = `req(f)` collected as a fixed point over the call graph (per SCC, deterministic order). Since the declaration rule holds at every direct call, `eff*(f) ⊆ decl(f)` is always true.
- `spec fn` and spec context have no effects (E0604 kept). The body of `external_body` (ADR-0029) is checked by the same rule. Effects are not trusted.

### 4. The area that needs testing (ADR-0003 made concrete)
`test_reasons` is computed per definition. If it is not empty, the definition needs testing.

| Reason | Condition | Why |
| --- | --- | --- |
| `io` | `io ∈ eff*(f)` | The contract says nothing about the order and content of external interaction |
| `nondet` | `nondet ∈ eff*(f)` | A different result is possible on the same input |
| `panic` | `panic ∈ eff*(f)` | It is not proved that no trap occurs |
| `trust` | The trust level is not `proved` (ADR-0029) | The contract itself is not proved |

- A definition with only `alloc` does not need testing. Out of memory is an environment condition, and the correctness of values is proved. This judgement states explicitly that `alloc` is not a reason (open question 3).
- The area is given in two units: the number of definitions, and the number of source sites (the number of locations of effect source calls and operations). Minimizing means fewer source sites. Tests only need to aim at source sites.
- Minimization tools: the lint for effects declared wider than required, and a split suggestion that divides out small definitions wrapping the source sites (a case of decomposition and splitting in DESIGN.md §8.2).

### 5. Queries and map (ADR-0020 backward compatible: key additions only)
- L1 rows: the existing `effects` stays the declaration text string (it can now be a comma list like `"alloc, io"`). Added keys `effects_required` (the `eff*` array), `test_reasons` (array).
- `effects_required`·`test_reasons` are added to `status` rows, and the summary row gains the number of definitions per effect (`alloc_defs`, `panic_defs`, `nondet_defs`) and `test_needed_defs`, `test_sites`. The existing `io_defs` keeps its meaning.
- `query effects DEF` (extension of 2D-11): for each effect of `eff*(DEF)`, one shortest call path to a source (ties broken by lexicographic path order) and the source site.
- `query effect E`: the list of definitions that declare E and definitions that require E (source kind: own/callee). This is the 2F pass criterion "list of definitions per effect".

### 6. Code generation
- LLVM function attributes are emitted from effects. `nounwind` on every function (BMB has no unwinding).
- `willreturn`·`mustprogress` only on functions with `panic ∉ eff*` and trust level `proved`. Termination comes from the `decreases` proof, and normal return from the absence of traps. They are not emitted for full check mode functions (a trap is not a return).
- If the effects are empty and there is no `&mut` argument, `memory(argmem: read)` (`memory(none)` if there is no reference argument). With `alloc`, `inaccessiblemem: readwrite` is added. No memory attribute is emitted for `io`·`nondet` functions.
- These attributes are guaranteed by the effect check (facts from the type rules, the same standing as `noalias`). Speed gains are claimed only by measurement, as in 1E-08. Larger effect-based optimizations (memoization, parallelization, DESIGN.md §8.1) get a separate ADR after measurement.

### 7. Rust habits to reject, and diagnostics (numbers are proposals)

| Habit | Handling | Code |
| --- | --- | --- |
| Omitting the effect attribute | The existing E0601. The fix suggestion carries the minimal effect set computed by `bmb fix` | E0601 (existing) |
| Not declaring a required effect | A replacement that adds the missing effect | E0606 |
| `#[effects(pure, io)]`, duplicate effect | Error | E0607 |
| Unknown effect (`#[effects(unsafe)]`, `async`) | The existing E0602. The hint text lists the new effects | E0602 (existing) |
| `panic!`·`unreachable!`·`todo!`·`unimplemented!`·`assert!` | The existing E0202. Hint per macro: unreachability goes into a precondition or proof, `assert!` becomes `assert(p(..))` (spec) or `requires` | E0202 (existing) |
| `std::process::exit/abort`, `std::time`, `rand::` paths | The existing E0304. When a built-in with the effect arrives, the hint names it | E0304 (existing) |
| `#[effects(io)]` on every function (over-declaration) | Warning (`bmb lint`) | W09xx |

### 8. Tests and fuzzer
- Diagnostic corpus: E0606·E0607 and SCC (mutual recursion) cases in `tests/check/err/effects.bmb`.
- Property test: generate random call graphs with random declared and own effects, and compare whether the checker's verdict equals "`decl ⊇ req` at every definition" and whether the map's `effects_required` equals a simple fixed point reference implementation.
- Soundness fuzzer: the interpreter records per definition the effects that really occurred at run time (allocation, trap, output). Property: **dynamic effects ⊆ static `eff*`**. In particular a definition without `panic` does not trap on any allowed input. Only the recording is added to the interpreters of the 6 existing fuzzers.
- IR test: `willreturn` is only on functions that are `proved` and have no `panic`. Determinism (same bytes on two runs).

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **A set of orthogonal atomic effects (proposed for acceptance)** | Queries ("every path that reaches `alloc`") and the area that needs testing come out per atom. The declaration check is one set inclusion |
| Chain lattice (`pure < alloc < panic < io`) | Simple, but it cannot tell a definition that is `io` and does not allocate from one that only allocates, and the effect view (DESIGN.md §7.2) becomes coarse. Rejected |
| `io` as top (implies every effect) | M1 code fits at once and the `main` declaration is short, but everything under an `io` definition needs testing, which makes the area derivation meaningless. Rejected |
| Only infer effects, no declaration | Fits the principle that the tool fills in explicitness (principle 6), but the declaration loses its interface role (keeping callee changes from spreading to callers). The declaration is required, and `bmb fix` fills it in |
| The declaration must equal the requirement exactly | The declaration becomes fully mechanical and interface stability disappears. Excess is a warning |
| Out of memory as `panic` | Rejected for the reason below the §1 table |
| Include freeing in `alloc` | Every function that takes a `Vec` by value would be `alloc`, but freeing does not fail and is not observed. Rejected (open question 3) |

## Consequences
- The syntax of the `#[effects]` attribute widens. M1 code does not need to change.
- Adding one effect to a callee requires fixing the declarations of all callers (the same cost as the propagation of `async`·`Result` in Rust). `bmb fix` fills it in mechanically, but the change impact (`impact`) widens. It is measured together with the Rust habit error rate.
- The `panic` effect gets a source only when ADR-0029 is accepted. Approving the two together is natural.

## Open questions (decided by measurement 1E-07 or by the maintainer)
1. **Timing of 2F:** the sources of the effect extension are made by other phases (`alloc` by 2G, `panic` by 2E, `nondet` none yet). Whether to run 2F bundled with those phases, or to add only the lattice first.
2. **The first source of `nondet`:** whether to add an input built-in (`read_i64`) or a time built-in to M2. An effect without a source cannot be verified, so it is introduced only together with a source (CLAUDE.md "a feature that is not verified is not put in the language").
3. **Whether `alloc` needs testing, and the effect of freeing:** the proposal is neither. Reviewed again when a task appears in which a memory bound matters.
4. **Unit of the area that needs testing:** which of the number of definitions and the number of source sites is the representative value of the measurement axis (DESIGN.md §9 R4). Both values are reported in `m2` and then it is decided.
5. **Gain of the LLVM attributes:** whether `memory`·`willreturn` make a measurable difference in the baseline benchmark. If not, the attributes are kept but no performance claim is made.

## Implementation (2026-11-02, backlog 2F-02)

- The effect set `check::Effects` (a bit set, inclusion order). Only the atoms that have a source, `alloc`·`io`, are accepted. `panic` (full check mode, 2E) and `nondet` (input built-ins) are introduced together with their sources (the recommendation of open questions 1·2: an effect without a source cannot be verified).
- Sources of `alloc`: `Vec::new`, `Vec::with_capacity`, `push`, `clone` of `Vec`, `Box::new` (an allocation ADR-0047 introduced after this ADR). `std::mem::take` leaves an empty `Vec` but does not allocate, so it is not a source (matched to `bmb_vec_alloc`·`grow`·`clone`·`bmb_box_alloc` of code generation).
- E0606 is one per definition ("per missing effect" of §3 is in the wording: the first source is written for each missing effect). The fix is an attribute that lists every effect the body requires, so one `bmb fix` is enough. When a `pure` function calls an `io` function, E0603 stays on that call, and if that is the only thing missing, E0606 is not added.
- An unused declared effect is W0909 (`bmb lint`). Trait methods, functions of a trait impl, and the `io` of `main` are not reported.
- A function of a trait impl has a subset of the trait method's effects (E0316), and `bmb weaken` compares effects by inclusion order (wider is weaker, incomparable is `incomparable`).
- Corpus: `alloc` was added to 66 allocating definitions (libraries, the run and verify corpora, Benchmarks Game programs) (only the E0606 replacement applied up to the fixed point). The seeds and reference solutions of the frozen set have no allocation and did not change.
- §5 (backlog 2F-03): `effects_required`·`test_reasons` of L1·`status` rows, `alloc_defs`·`test_needed_defs` of the `status` summary, the `alloc` row of `query effects`, `query effect E`. `panic_defs`·`nondet_defs` of the summary and `test_sites` (present in the `tests` query) are added when the sources appear. `query effects DEF` is not one path per effect but, as now, one row per definition that has the effect (it keeps the shape of 2D-11 so consumers read it as is).
- Remaining: the LLVM attributes of §6 and the dynamic effect fuzzer of §8 (backlog 2F-04).
