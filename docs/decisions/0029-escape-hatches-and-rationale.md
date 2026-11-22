# ADR-0029: Escape hatches (`assume`·`external_body`) and structured rationale

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved"). Implementation order is after effects (ADR-0030, 2F) (2E-02~2E-04)
- Date: 2026-10-24
- Related: DESIGN.md §4.1 (escape hatches, comments)·principles 3·4·9·§7.2 (unproved area), ADR-0003, ADR-0006, ADR-0016, ADR-0019 §2, ADR-0020, ADR-0030 (proposed), ROADMAP 2E, backlog 2E-01

## Context
M1 has no escape hatch. Without a proof there is no build, and the unproved area is only the `io` boundary (`print_i64`) (ADR-0016). This strictness blocks the following.

- **Non-linear arithmetic.** One obligation of `pow` (base ≤ 10, exponent ≤ 15) used 4.1 million of the 5 million `rlimit` and was removed from the corpus (`dev-docs/measurements/m1.md` §6). When the solver cannot prove a mathematically true fact, there is no way around.
- **Solver limits.** A predicate that keeps a quantifier can end in `unknown` (E0811) (ADR-0023 consequences). The verdict is correctly a failure, but the agent cannot proceed.
- **Boundaries.** The bodies of future external boundaries (input, time, FFI) cannot be proved.

DESIGN.md §4.1 set escape hatches as "rationale declaration required, counted in the map as unproved area", and ADR-0006 set that the rationale is bound to a node hash and marked "unconfirmed" when it changes. The syntax, format and code generation rules were not decided. Principle 9 is "an unverified contract is not used for optimization".

## Decision (proposed)

### 1. Surface
- `assume(p(args));`: a statement in the body of an executable function. The argument is one call of a named predicate, as in a contract clause (the same rule as E0501, a new code because the position differs). `p(args)` is assumed for proofs after that point. It is the same name as Verus's `assume`.
- `#[external_body]`: an attribute of an executable function. The body is only checked (names, types, effects, ownership) and generates no proof obligation. Callers prove `requires` and assume `ensures` as usual. It cannot be used on `spec fn` or `main`.
- **Rationale required:** both escape hatches must have `#[rationale(kind = K, hash = "..", "description")]` right before them. Without it, E1001.
  - `kind` is a closed list: `nonlinear` (non-linear arithmetic), `solver_limit` (solver limits such as E0811), `boundary` (external boundary), `performance` (benefit against proof cost). The map uses it to count the trust base by reason.
  - `hash` is the hash of the node the rationale is bound to (§3). **The tool fills it in** (principle 6): when a rationale is first written or confirmed, `bmb fix`/a transaction operation writes the current hash. The agent writes only the description and `kind`.
- A structured rationale can also be attached outside escape hatches (`fn`, `spec fn`, `mod`) (optional). `//` comments are kept in M2 as well (open question 4).

### 2. Proof gate: separating trusted facts
The trusted fact set `T` = the `assume` facts in the definition ∪ the `ensures` of `external_body` functions ∪ the `ensures` of callees whose trust level is not `proved`.

1. The obligations of the definition are first solved without `T` (pass 1). If all are proved, the trust level is `proved`. In that case the `assume`s of the definition are unnecessary, so `bmb lint` warns (W1002).
2. If an obligation fails in pass 1, `T` is added and only that obligation is solved again (pass 2). If all are proved, the trust level is `assumed`, otherwise it fails as now (`failed`, build refused).
3. An `external_body` definition has no obligation and its trust level is `external`.
4. Trust levels are computed as a fixed point along the call graph. If a callee is not `proved`, its `ensures` enters the caller's `T`, so a caller that relies on it also becomes `assumed` (per SCC, monotone so it terminates).
- The pass 1 result is independent of `T`. So adding or removing an escape hatch does not change the verdict of `proved` definitions (fuzzer property 2).
- The trust level of callees is added to the proof cache key (the starting point of ADR-0019 §2). Even with the same contract, a changed level means solving again.

### 3. The node hash of a rationale and "unconfirmed"
- Rationale hash = SHA-256(the qualified normal form of the bound node ‖ the body hashes of the predicates that node calls by name). The bound node is **the body of the enclosing definition** for `assume`, and **that whole definition** for `external_body`. If the definition of the assumed predicate changes, the rationale comes to say something else, so the predicate hashes are included.
- The `#[rationale(..)]` attribute is excluded from the input of every hash (`body`·`iface` of ADR-0019, the rationale hash). Editing the rationale text does not disturb the proof cache or duplicate detection, and there is no cycle of a hash containing itself. This complements ADR-0019 §2 ("attributes … whole").
- If the stored `hash` differs from the current value, that rationale is **unconfirmed**. Unconfirmed does not block the build (open question 2). It is shown in `status`·L1 and `bmb lint` reports it as W1001.
- The transaction `tx.put` does not rewrite the rationale hash. Rewriting automatically would hide the unconfirmed state. Confirmation is an explicit operation (`tx.confirm_rationale(def)`, CLI `bmb fix --confirm`). Confirmation is a judgement, so it is the agent's job.

### 4. Code generation: trusted facts are not grounds for optimization
- Only `proved` definitions drop checks by proved obligations (as now).
- `assumed`·`external` definitions are compiled in **full check mode**. That is, overflow, division by zero, index and conversion checks all remain as `llvm.trap`. It is implemented by the current code generator removing that definition's obligations from the set of proved obligations (`proven` of `codegen::emit`). It is the same path as the "full check build" the fuzzer already uses.
- **Downward propagation:** the callees called by a full check mode definition are also compiled in full check mode. The `requires` of that call was proved on top of a hypothesis that may be false (the callee's check removal relies on the precondition). At first only one copy per definition is made, and if measurement shows the cost is large, an optimization that keeps a separate check mode copy is considered.
- `assume` itself generates no code (a predicate is not executable in general). Safety is left to the remaining checks. The result of a false `assume` is a trap, not undefined behavior.
- A definition with remaining checks has the `panic` effect (ADR-0030 proposal). So the influence of an escape hatch is visible to callers as an effect.
- `noalias`·`readonly` come from the type rules (ADR-0015), so they are kept regardless of trust level.

### 5. Map and `status` (ADR-0020 backward compatible: key additions only)
- Keys added to L1 rows and `status` rows: `trust` (`"proved"|"assumed"|"external"|"failed"`), `escapes` (number of `assume`s in the definition + whether it is `external_body`), `assumed_obligations` (number of obligations proved only in pass 2), `checked` (whether in full check mode), `rationale` (`"none"|"confirmed"|"unconfirmed"`), `rationale_kinds`.
- `proved`, `assumed`, `external`, `escapes`, `unconfirmed` are added to the `status` summary row. The values and meanings of the existing `status` keys are unchanged (`verified` = passed the build gate). The measurement harness judges success by `trust` (§6).
- The unproved area view (DESIGN.md §7.2) emits the definitions with `trust != "proved"` and their rationales, and the cause edges of definitions whose level dropped through `T` (because of which callee's `ensures`).

### 6. Relation to measurement
- With escape hatches, an agent can "pass" a task with `assume`. The `m1b`/`m2` harness adds to the success condition of authoring tasks that **every newly written definition has `trust = "proved"`**. Tasks that allow escape hatches are marked separately. The frozen 1E tasks and prompts are not changed.
- The unproved area (number of definitions, sum of `assumed_obligations`, distribution by `kind`) is a measurement axis of DESIGN.md §9 R4.

### 7. Rust·Verus habits to reject, and diagnostics (numbers are proposals)

| Habit | Handling | Code |
| --- | --- | --- |
| `unsafe { .. }`, `unsafe fn` | Rejected. A body that cannot be proved goes under `#[external_body]` | E0215 |
| `#[verifier::external_body]`, `#[verifier(external_body)]` (Verus) | Mechanically replaced with `#[external_body]` | E0216 |
| `admit()` (Verus), `assume!`/`debug_assert!` | Hint towards `assume(p(..))` + rationale (macros are the existing E0202) | E0217 |
| An escape hatch after a `// SAFETY: ..` comment | Fix suggestion that moves the comment text into the description of `#[rationale(..)]` | `fix` of E1001 |
| An escape hatch without a rationale | Error | E1001 |
| Rationale format error (value outside `kind`, empty description, before a statement that is not an escape hatch) | Error | E1002 |
| The argument of `assume` is not a predicate call, `assume` in a `spec fn` or spec context | Error | E1003 |
| `#[external_body]` attached to `spec fn`·`main` | Error | E1004 |
| Rationale unconfirmed | Warning (`bmb lint`) | W1001 |
| An `assume` that is unnecessary because it is already proved in pass 1 | Warning (`bmb lint`) | W1002 |

E10xx·W10xx are a new category "escape hatches and rationale". They are not mixed with W09xx (contract adjustment lints).

### 8. Tests and fuzzer
- Corpus: `tests/check/err` (E1001~E1004, E0215~E0217), `tests/verify` (`proved`/`assumed`/`external`/level propagation cases), `tests/run` (cases where a false `assume` ends in a trap).
- IR check: `assumed`·`external` definitions and the definitions they propagate down to have no `; proved` comment and no unchecked operation.
- Soundness fuzzer (new `soundness_escape.rs`, reusing the existing generator): randomly mixes true and false `assume`s and `external_body` contracts that may be false. Properties:
  1. A `proved` definition, even when `T` is false, has no error on inputs that satisfy the precondition and satisfies the postcondition (the interpreter ignores `assume` and executes).
  2. The `proved` set is the same as for the program with every `assume` removed.
  3. The output and exit status of the executable equal those of the interpreter with the meaning "trap at a remaining check". That is, a false trusted fact does not produce silent misbehavior (wrong output without a trap, out-of-bounds access).
- Rationale hash: tests that changing the bound node or the assumed predicate gives unconfirmed, and that changing only the rationale text or `#[rationale]` leaves the confirmation state and the definition hash unchanged. Determinism (same bytes on two runs) and placement independence (ADR-0020) are in the same tests.

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **Separating trusted facts + full check mode (proposed for acceptance)** | Keeps principle 9 as is. The worst case of a false escape hatch is a trap. The implementation is the existing "full check build" path applied per definition |
| Use trusted facts exactly like proved facts, as Verus does | One false `assume` becomes undefined behavior through `inbounds`·`nsw`. It is the same kind of defect as v0's "emit `llvm.assume` for every contract regardless of verification" (genesis §3.2 principle 9 row). Rejected |
| Keep checks only per obligation (check only the obligations proved in pass 2) | Insufficient. An obligation proved in pass 1 can also use, as hypotheses, invariants·`assert`s·callee preconditions proved only in pass 2, so if those hypotheses are false the removed check is wrong. A refinement limited to the case where all spec-level obligations of the definition are proved in pass 1 is left as an optimization after measurement |
| Compile `assume(P)` into a runtime check (like `assert!`) | Predicates are `spec fn`·`forall`·unbounded integers, so they are not executable in general. Checking only the executable subset makes the meaning of the escape hatch depend on the shape of the predicate. Rejected (full check mode gives the same safety) |
| Only proof hints (`assert .. by(nonlinear_arith)` etc.), no escape hatch | Hints are the better direction, but the scope is large and they do not resolve every limit. The escape hatch stays as a last resort, and hints are proposed separately on the grounds of the `kind = nonlinear` counts |
| Bind the rationale only to the `assume` statement node, not to the definition | Not detected when the surrounding code changes and the reason for the assumption disappears. The default is the enclosing definition (open question 3) |
| Fail the build on an unconfirmed rationale | Unrelated to correctness (a fact already trusted), and one confirmation operation gets past it, so it has little effect. Start with a warning + counting |

## Consequences
- The proof gate solves an obligation at most twice (only those that fail pass 1). The cost is proportional to the number of escape hatches.
- Because of trust level propagation, one escape hatch can lower the level and the performance (full check mode) of many callers. This is intended, honest accounting, and the influence is visible in `impact`/the unproved area view.
- Excluding `#[rationale]` from hash input complements ADR-0019 §2. On acceptance, "trusted facts are not used for optimization" is added to the escape hatch row of DESIGN.md §4.1.

## Open questions (decided by measurement 1E-07 or by the maintainer)
1. **Timing of 2E:** priority needs the frequency of actually being blocked for lack of an escape hatch (E0811 and non-linear failures in 1E-10). The current evidence is the single `pow` of the author corpus.
2. **Force of unconfirmed:** warning (proposed) versus `build` refusal. The maintainer decides.
3. **Scope of the rationale hash:** the whole enclosing definition (proposed) produces unconfirmed on unrelated edits too. Decided by measuring in `m2` the ratio of rationales that are really stale to the number of unconfirmed occurrences.
4. **Handling of `//` comments:** DESIGN.md §4.1 said free comments are replaced by structured rationale. The maintainer decides whether to remove `//` in M2 (an increase in Rust habit errors is expected).
5. **The `kind` list:** whether four values are enough. Whether to have `todo` (temporary) — with it there is a risk of abuse, without it there is no way to express a work-in-progress state.
6. **Effects of `external_body`:** effects are confirmed by checking the body, so effects are not trusted (proposed). Decided again when external boundaries (FFI) arrive (after the phase 4 ecosystem).

## Implementation (2026-11-02, backlog 2E-02)

- Surface: `assume(p(..));` (`Stmt::Assume`, `assume` is a reserved word), `#[external_body]`, `#[rationale(kind = K, hash = "..", "description")]` (attribute arguments `key = value`, `AttrArg::Pair`). An attribute before a statement is accepted only as `#[rationale]` before `assume`, and the rest are E1002 (`#[trigger]` is read by the expression). The normal form writes the rationale, but the qualified text and the identity print do not (§3: editing the rationale text leaves hashes and incremental keys unchanged). The JSON AST has `external_body`·`rationale` on a function and the statement `{"kind":"assume","cond","rationale"}`.
- **The `panic` effect (ADR-0030) is decided by syntax:** it is the own effect of a definition that contains `assume` or is `#[external_body]`. The trust level is known only after proving, but the effect check is before proving, so the source is "a definition that has an escape hatch" instead of "a definition in full check mode" (§4). So a definition that is proved without the escape hatch (a definition that does not need its `assume`) also declares `panic` (W1002 makes it delete that `assume`). The trap of a false trusted fact occurs only within the call scope of a definition that has an escape hatch, and its callers have `panic` by following the declaration.
- Proof (§2): `verify::verify_trusting`. A program without escape hatches (`Tables::may_untrusted` is empty) runs once as before. Otherwise pass A (no trusted facts: `assume` does nothing, and the `ensures` of untrusted callees are not assumed) runs on every definition, and pass B (all facts) only on definitions that fail A and can reach an escape hatch. "Only the failed obligation again" of §2 was done per definition (B proves the whole definition again, and the diagnostics and result of that definition are B's). A definition that became `assumed` enters the untrusted set, and the definitions that were `proved` and can reach an escape hatch are proved again with A (the set only grows, so it ends). A trait method is untrusted if even one impl function is untrusted.
- Incremental cache: a definition that can reach an escape hatch is neither looked up nor stored. So the trust level of callees need not be put in the key ("the level in the proof cache key" of §2 is already separated by the query text: a pass A query has no trusted facts).
- Code generation (§4): `Outcome::checked` = the definitions that are not `proved` or failed, and every definition they call (the checker's call graph `Tables::calls`). The obligations of those definitions are not put in `proven`. Those checks are not missing proofs, so `build` is not blocked (`codegen::Module::checks_by` counts per definition and excludes those definitions).
- Status line: if not `proved`, `"trust":"assumed"`·`"external"` is appended (a key addition, the line of a proved definition is unchanged).
- §5 (backlog 2E-03): `trust`·`escapes`·`assumed_obligations`·`checked`·`rationale`·`rationale_kinds` of L1·`status` rows, and `proved`·`assumed`·`external`·`escapes`·`unconfirmed` of the `status` summary. `assumed_obligations`, "obligations proved only in pass 2", was measured as "the number of obligations that fail without trusted facts", to fit the per-definition pass B. `panic` (`eff*`) and `trust` (`assumed`·`external`) were added to the test reasons (ADR-0030 §4). The cause edges of the unproved area view (because of which callee) do not exist yet.
- §3 (backlog 2E-04): the rationale hash `escape::Hashes` (the qualified normal form of the definition ‖ the qualified text of the `spec fn`s it calls, the first 16 digits of SHA-256). The rationales of both `assume` and `#[external_body]` are bound to the enclosing definition (the recommendation of open question 3). Unconfirmed is a warning (W1001). Confirmation is the CLI `bmb fix --confirm`, and a missing hash is written by `bmb fix` (§1 "the tool fills it in"). The transaction operation `tx.confirm_rationale` does not exist yet (an MCP agent does not call `fix`, so it is added when demand appears). W1002 is per `assume` in a `proved` definition of the first verification.
