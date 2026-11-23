# Claims and evidence

BMB makes claims about what a language built for agents is worth. Each claim here is a hypothesis with a refutation condition registered before its measurement. This page gives the state of each one, including the ones that were refuted.

Benchmarks, harnesses, frozen task sets, result rows and the full reports are in [`bmb-bench`](https://github.com/lang-bmb/bmb-bench); a report points to the compiler by commit.

## How to read this

| State | Meaning |
| --- | --- |
| Confirmed | Measured with models on a frozen set; the criterion holds outside run-to-run variation |
| Refuted | Measured; the registered refutation condition was met |
| Deterministic evidence | Measured without a model (replays, mutations, proofs); supports the claim as far as it reaches |
| Measuring | A formal measurement is under way or holds only at part of its range |
| Pilot | Early runs only; not used for claims |
| Not measured | No data. No estimate is given |

- **Models.** Claims are judged with `claude-sonnet-5-5` ("sonnet") and reported alongside `qwen3.8-27b` ("qwen", repeated three times where the difference is inside variation).
- **Arms.** "BMB" is the BMB program with the compiler's tools (map, definition-level read and write, proof gate). "Rust" is the same program in Rust with file tools, `cargo build` and `cargo test`. "Verus" is the original Verus task.
- **A counted success** is a first attempt judged by a hidden judge after the session; there are no retries.
- **Learned against unlearned.** Models were trained on Rust and not on BMB. A difference against Rust is the sum of the cost of the design and the cost of being unlearned, and the two are not yet separated.

## Summary

| | Count | Claims |
| --- | --- | --- |
| Confirmed | 0 | |
| Refuted | 4 | H-SCALE (at the boundary), H-REFACTOR, H-EFFECT, H-REPAIR |
| Deterministic evidence | 7 | H-STABLE, H-MAINT, H-ID, H-PAR, H-VIEW, H-COMMUTE, H-COMPAT |
| Measuring | 5 | H-AMORT, H-GAME, H-ENCAP, H-COMPREHEND, H-TASK |
| Pilot | 5 | H-SPEC, H-ORACLE, H-FAMILIAR, H-NAME, H-GRAIN |
| Not measured | 14 | the rest |

What this adds up to: the results the language produces are shown (speed, size, a gate that was not gamed, edits that stay local). The economic claim, that an agent reaches the same result in BMB at no more total cost than in Rust with tests, is not shown: at the size of one task BMB costs more tokens, and the life-cycle measurement that could offset it is not finished.

## Goals

| Goal | State |
| --- | --- |
| G1 Trust in the verdict | 0 known unfixed violations. Nine soundness defects found and fixed so far (three by fuzzers, five by use, one by audit); 17 soundness fuzzers run in the gate |
| G2 First-attempt correctness | Depends on the model. Ported Verus tasks: sonnet BMB 44/45, Verus 43/45; qwen BMB 14/45, Verus 15/45 |
| G3 Specification quality | qwen 126/186 hidden mutants rejected (67.7%), below the 90% target |
| G4 Navigation at scale | Met with sonnet at 10,000 definitions: 0.087 times the tokens of file tools at equal accuracy (27/27). Not met with qwen (1.39 to 3.07 times: responses overflow its context) |
| G5 Proof beats tests | Defects: cannot be judged (neither arm let a hidden defect through on four task sets). Tokens: not met at the size of one task |
| G6 Performance | Met: 7 of 7 baseline benchmarks at 0.962 to 1.041 times safe Rust |
| G7 Cumulative verification cost | Measuring |
| G8 Least test code | Direction agrees; too early to judge |
| G9 Least binary size | Met: seven benchmarks strip to about 14.5 KB each against 342 KB for safe Rust |

## Claims the design stands on

If one of these is refuted, a premise or a principle of the design is rewritten.

| Claim | State | Evidence |
| --- | --- | --- |
| **H-ORACLE** A passed proof lets fewer hidden defects through than passed tests | Pilot; cannot be judged | Four attempts did not discriminate: sessions that passed and still failed the hidden judge were 0 in both arms (sonnet 0/21 against 0/24). On 96 mutants, kill rate BMB 0.958 against Rust 0.885, difference +0.073 [−0.012, +0.158], p = 0.15 |
| **H-AMORT** Cumulative verification cost over a commit history is below Rust with test selection and caching | Measuring | Replay of 1,000-commit histories over 1,000 and 10,000 definitions; harness and histories frozen, no result yet |
| **H-TASK** Feature additions and defect repairs replayed from real history succeed as often as in Rust with tests, at no more total tokens over the life cycle | Measuring. With text search: neither refuted nor confirmed (sonnet, one run; the token interval contains 1). Without text search: refuted on tokens | On the compiler twins, 22 replayed commits and 4 histories. With text search and bounded responses, sonnet: success 21/22 against 22/22, API tokens 1.30 times (median of per-task ratios, interval 0.71 to 2.14); histories 4/4 against 3/4 at 1.07 times. Without text search, sonnet: 21/22 against 21/22 at 3.66 times (2.14 to 5.46); histories 4/4 against 3/4 at 2.5 times. qwen: without text search most sessions fail in both arms (two thirds overflow the context); with text search the BMB arm did not finish, so there is no comparison |
| **H-GAME** With escape hatches forbidden and audited, the gate is not bypassed | Measuring; not refuted | 13 impossible tasks, each with a proved certificate of impossibility, and 6 possible controls. Gate passes on impossible tasks: qwen 0/13, sonnet 0/13; contract changes 0/38. Weakening is detected by comparing specifications (2,005 edited definitions judged as the generator's ground truth says) |
| **H-STABLE** Proofs are stable under meaning-preserving rewrites and solver seeds | Deterministic evidence | 718 obligations under 16 rewrites: 0% of verdicts change (criterion < 2.6%) |
| **H-SCALE** Success and tokens do not depend on codebase size | Refuted at the boundary (sonnet) | 24 local tasks stay 24/24 from 1,000 to 10,000 definitions; tokens read grow 3.05 times (refuted at 3; Rust 7.33 times). Two tasks using one unbounded query carry the growth; without them 1.14 times |
| **H-COMPREHEND** On a large codebase, questions about callers, impact, diagnostic paths and where state changes are answered at least as accurately as in Rust with file tools, at a third of the tokens read or less | Measuring; meets the criterion at the size of the front half | 24 questions on the compiler twins, BMB arm with text search and bounded responses. sonnet: 24/24 against 22/24, tokens read 0.07 times (0.05 to 0.09). qwen (three runs): 21, 22, 21 against 16, 18, 18; 0.07 times. Without text search the same questions refuted the claim (sonnet 22/24 against 23/24, tokens read 7.4 times in total). Full size is not measured |
| **H-REVIEW** Reviewing the specification diff alone catches as many defects as reviewing the code diff | Not measured | 143 frozen items. Measured without a model, the specification diff is 1.75 times the tokens of the code diff on these single-function items, so this set can judge detection only |
| **H-INTENT** Examples and counterexamples generated from a specification expose a specification that misses the intent better than review does | Not measured | 90 frozen items; the token criterion is out of reach on this set (1.66 times) |

## Claims that distinguish BMB

If one of these is refuted, a decision record is revised.

| Claim | State | Evidence |
| --- | --- | --- |
| **H-MAINT** Re-proof stays inside the impact set of an edit | Deterministic evidence | In replayed commit histories every re-proved definition is in the set the impact query reports |
| **H-ID** With identity as a hash and names as metadata, a rename invalidates nothing else | Deterministic evidence (partial) | 424 renames: no other definition's identity, body hash or contract hash changes (87% changed some before). 502 renames: incremental re-encodings 1,005 → 0, proof cache misses 4,297 → 0 |
| **H-PAR** Definition-level transactions conflict less than file-level text merge | Deterministic evidence | Same replay as H-COMMUTE: with re-proof of overlapped definitions, false conflicts 0 against 3 for `git merge-file` |
| **H-COMMUTE** The commutation relation (one side's writes against the other's reads) is sound and gives fewer false conflicts than text merge | Deterministic evidence for soundness; the judgement alone gave more false conflicts, and re-proving overlapped definitions removes them | 192 pairs with semantic conflicts and 113 history pairs: 0 violations. All 58 merges that fail to prove are judged conflicts; text merge passes 57 of them cleanly. False conflicts 9 against 3 by judgement alone, 0 against 3 with re-proof, at 0.8 to 3.3% of the CPU of proving the whole merge |
| **H-COMPAT** Compatibility of a contract change is decided by proof and the broken call sites are named exactly | Deterministic evidence | 483 contract edits: recall of call sites 100% (673, none missed, none extra), compatibility verdicts 483/483 |
| **H-ENCAP** Proved type invariants hold without visibility restrictions | Measuring; cannot be judged, not refuted | Four data-structure tasks with a hidden judge on raw fields. Passed the gate and violated the invariant: 0/4 for both models. Rust also 0/4. The weak solution passes its own tests in Rust 4/4 and is rejected by the gate 4/4 |
| **H-EFFECT** Effects separate a proved core from a shell that needs tests, and the remaining test area is small | Refuted (sonnet) | Both arms 4/4; remaining test area 4 of 6 definitions (criterion ≤ 20%), test lines 169 against 41. The area comes from how agents decompose: the shell itself is area. The principle is now stated for the pure core only |
| **H-REFACTOR** Behaviour-preserving refactorings succeed as often and cost no more tokens | Refuted on tokens (sonnet, one run) | 20 tasks on the compiler twins (10 renames, 6 parameter swaps, 2 inlines, 2 extractions). sonnet: 20/20 against 20/20, API tokens 3.52 times (1.90 to 5.14). qwen: 14, 11, 13 of 20 against 20, 20, 20. The measured BMB surface did not expose the rename operation |
| **H-SPEC** With tools, written specifications reject more hidden mutants than in Verus | Pilot | Specification strength on ported Verus tasks: BMB 42/45, Verus 44/45; the three BMB rejections were undecided comparisons of equivalent specifications |
| **H-FAMILIAR** The cost of BMB being unlearned is below Verus's cost of writing everything twice (code and specification language) | Pilot | Same success level as Verus; BMB cost about twice in the pilot |
| **H-NAME** Contracts carry meaning that names otherwise carry | Pilot (BMB arm only) | qwen: writing 23/25 with original names, 17/25 with opaque names, 21/25 with misleading names, all inside variation; no Rust arm with the same renaming yet |
| **H-REUSE** A deterministic map reduces duplicate code and out-of-scope edits | Not measured | |
| **H-SLICE** What must be read to change one definition does not grow with the codebase | Not measured | |
| **H-SPLIT** Work divided after fixing contracts integrates with no failures | Not measured | |
| **H-TRUNK** Concurrent definition-level commits to one trunk cost less to integrate than branches, with no integration failure | Not measured | In the port's history, 88% of re-proofs come from changes to type declarations |
| **H-LIVE** A replacement proved to refine its contract can be applied without restart | Not measured | Upper bound from the port's history: 184 of 222 edited definitions keep their contract (82.9%) |
| **H-RACE** Choosing by measurement among proved competing implementations beats writing one | Not measured | |

## Claims that set defaults

If one of these is refuted, a default of the tools changes.

| Claim | State | Evidence |
| --- | --- | --- |
| **H-VIEW** The normal-form, Rust-shaped projection gives the most success per token | Deterministic evidence (partial) | Minimal whitespace saves 13 to 15% of tokens for the same program; a projection without bodies is 0.44 to 0.59 times the code. Success not measured |
| **H-GRAIN** Editing by definition costs the fewest total tokens | Pilot (partial) | One write per definition against multi-call transactions, same success: cost −22% to −31%, turns −40% |
| **H-EDIT** A structural address with text content beats file diffs and JSON syntax-tree patches | Not measured | Three surfaces frozen over the same tasks |
| **H-UNIT** A derived table of contents serves navigation as well as declared units | Not measured | |
| **H-REPAIR** Counterexamples and fix suggestions in diagnostics reduce repair iterations | Refuted (qwen, one run) | Rejected commits and failed proofs per successful session: 1.26 with them, 0.88 without; the difference sits in one task |
| **H-SMALL** Better tools and diagnostics narrow the gap between weak and strong models | Not measured | |
| **H-UNDECIDE** Adding decision information to an undecided response reduces tokens per success | Not measured | In one set, 88% of sonnet's tokens went to nine sessions that received an undecided response |
| **H-PROBE** A batch probe of candidate invariants that commits nothing reduces tokens per success | Not measured | |
| **H-HISTORY** Starting each request as a new conversation with a one-line history keeps success and reduces tokens | Not measured | Estimated upper bound 0.34 times |

## What changed a verdict

- **A missing tool decided a claim.** Understanding went from refuted to meeting its criterion when the BMB arm got one tool (text search in definition bodies): tokens read 0.83 times → 0.07 times on the same questions. An unfavourable result is checked first for whether both arms could do the same work. Refactoring and task replay are under the same suspicion.
- **A judge with a hole.** The refactoring judge counted a session that changed nothing as a success. It now requires the requested change, with a do-nothing control that must fail.
- **Carried over once.** Three verdicts were moved to the compiler twins: token cost per task (to the life-cycle measure), the remaining test area (to the effect distribution of real code), and size independence.
