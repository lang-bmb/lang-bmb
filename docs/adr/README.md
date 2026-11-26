# Decision records (ADRs)

Each design decision of BMB is one file here. The current design is in [`../DESIGN.md`](../DESIGN.md); a decision
record says *why* the design is the way it is, one decision at a time.

## Rules

- File name: `NNNN-short-slug.md`. Four digits, increasing, never reused.
- One decision per file, short enough to read on one screen.
- An accepted record is not edited. When a decision changes, a new record is written and the old one's status
  becomes `Superseded by ADR-NNNN`. Typos and links are the exception.
- A decision that changes `docs/DESIGN.md` changes it in the same commit.
- A claim is written only when a code path or a measurement backs it.

## Status values

| Status | Meaning |
| --- | --- |
| Proposed | Under review. Not binding yet |
| Accepted | In force |
| Superseded by ADR-NNNN | Replaced by another record |
| Rejected | Not adopted |

## Shape

```markdown
# ADR-NNNN: Title

- Status: Accepted
- Date: YYYY-MM-DD
- Related: DESIGN.md §n, ADR-NNNN

## Context
What was at issue.

## Decision
What was decided.

## Rationale
Why. The alternatives considered, when there were any.
```

## About these files

Records 0001 to 0147 were written in Korean and translated to English when the repository was split
(ADR-0147). The translation keeps the structure and every identifier of the original. They are historical
records: they name backlog tasks (`T1-12`, `4D-05`) and working documents (`dev-docs/...`) of the project's
workspace, which are not part of this repository. An option that the original labelled with a Korean letter
appears as (a), (b), (c), or as (i), (ii), (iii) where the record already used Latin letters.

## Index

| No. | Title | Status |
| --- | --- | --- |
| [0001](0001-ai-friendliness-as-definition.md) | AI-friendliness is a definition, not a goal | Accepted |
| [0002](0002-correctness-constraint-performance-objective.md) | Correctness is the constraint, performance is the objective function | Accepted |
| [0003](0003-minimize-test-surface.md) | Compute and minimize the area that needs tests | Accepted |
| [0004](0004-design-doc-derived-not-written.md) | The design document is derived, not written | Accepted |
| [0005](0005-language-scope-code-and-maps.md) | The language is responsible for code and maps, no further | Accepted |
| [0006](0006-rationale-bound-to-node-hash.md) | Rationale lives in the code, bound to a node hash | Accepted |
| [0007](0007-design-doc-tool-separate-repo.md) | The design document tool lives in a separate repo, Markdown first | Accepted |
| [0008](0008-standard-maps-l0-l2.md) | Standard maps L0–L2 are fixed, additions are backward compatible | Accepted |
| [0009](0009-transactional-writes.md) | Writes are transactions, proof is a build gate | Accepted |
| [0010](0010-extension-maps-in-pure-bmb.md) | Extension map generators are pure BMB functions | Accepted |
| [0011](0011-storage-format-text.md) | The storage format is text, the tool decides the layout | Accepted |
| [0012](0012-independent-compiler-to-llvm.md) | An independent compiler without rustc, straight to LLVM | Accepted |
| [0013](0013-inherit-surface-strengthen-semantics.md) | The surface is inherited, progress comes from giving up conveniences | Accepted |
| [0014](0014-deterministic-tools-ai-judges.md) | Deterministic tools fill the gap, the AI only judges | Accepted |
| [0015](0015-second-class-references.md) | Ownership and moves + second-class references | Accepted |
| [0016](0016-milestone-1-scope.md) | Scope of the first milestone: language subset and tool queries | Accepted |
| [0017](0017-integer-semantics-and-smt-encoding.md) | Integer semantics and SMT encoding | Accepted |
| [0018](0018-obligation-isolation-and-division-facts.md) | Proof obligation isolation and auxiliary division facts | Accepted |
| [0019](0019-normal-form-and-definition-hash.md) | Normal form and definition hash | Accepted |
| [0020](0020-map-schema.md) | Map JSON schema L0–L2 | Accepted |
| [0021](0021-measurement-procedure.md) | M1 measurement procedure | Accepted |
| [0022](0022-mixed-integer-types-in-specs.md) | Mixing integer types is allowed in specifications | Accepted |
| [0023](0023-ground-instance-retry.md) | Ground-instance retry for undecidable obligations | Accepted |
| [0024](0024-checker-relaxations.md) | Three checker relaxations | Accepted |
| [0025](0025-task-set-m1b.md) | The second task set `m1b` | Accepted |
| [0026](0026-data-types-surface.md) | Data type surface: structs, enums, `match`, `impl` | Accepted |
| [0027](0027-data-types-encoding.md) | Proof encoding of data types | Accepted |
| [0028](0028-integer-conversions.md) | Integer conversions and checked·saturating arithmetic | Accepted |
| [0029](0029-escape-hatches-and-rationale.md) | Escape hatches (`assume`·`external_body`) and structured rationale | Accepted |
| [0030](0030-effects-alloc-panic-nondet.md) | Effect extension (`alloc`·`panic`·`nondet`) and the area that needs testing | Accepted |
| [0031](0031-vec-and-ownership.md) | Heap `Vec<T>` and the ownership·move check | Accepted |
| [0032](0032-incremental-build-key.md) | The key of the contract firewall incremental build | Accepted (autonomous) |
| [0033](0033-per-query-spec-declarations.md) | Send only the `spec fn` declarations each query reaches | Accepted (autonomous) |
| [0034](0034-mission-goals-and-benchmarks.md) | Mission goals and the benchmark system | Accepted |
| [0035](0035-spec-strength-kill-check.md) | The kill definition of the spec strength metric (BSB) | Accepted |
| [0036](0036-syntax-evaluation-scale.md) | Syntax evaluation scale | Accepted |
| [0037](0037-bootstrap-and-language-maturity.md) | Mandatory self-hosting bootstrap and language maturity phases | Accepted |
| [0038](0038-bvb-porting-rules.md) | BVB (porting public Verus tasks) rules | Accepted |
| [0039](0039-continuous-health-management.md) | Continuous management system (multi-axis metrics, ratchet, self-hosting ladder, practicality rules) | Accepted |
| [0040](0040-premises-revised-by-pain-research.md) | Premises revised by the pain point research, and new hypotheses | Accepted |
| [0041](0041-tradeoff-ledger.md) | Trade-off ledger (costs accepted, axes maximized, rules) | Accepted |
| [0042](0042-bcb-design.md) | Design and pre-registration of BCB (accumulated verification cost benchmark) | Accepted (autonomous) |
| [0043](0043-caller-key-without-uses.md) | The caller key does not contain the callee's `#[uses]` | Accepted |
| [0044](0044-bytes-and-byte-strings.md) | Byte `u8`, byte string literals, byte output and the bytes library | Accepted |
| [0045](0045-judgment-criteria.md) | Judgement criteria — fix at the layer of the cause and choose the standard solution; cost is not a criterion | Accepted |
| [0046](0046-generics-and-traits.md) | Generics and traits | Accepted |
| [0047](0047-owned-types.md) | Generalizing owned types: `Box<T>`, recursive types, types that hold a `Vec` | Accepted |
| [0048](0048-integer-types-and-bit-operations.md) | Complete integer types, bit operations, integer literal formats | Accepted |
| [0049](0049-early-exit-and-for.md) | Early exit (`return`·`break`·`continue`·`?`) and `for`·`loop` | Accepted |
| [0050](0050-strings.md) | `String`·`&str`·`char` | Accepted |
| [0051](0051-method-call-syntax.md) | Method call syntax as the canonical form | Accepted |
| [0052](0052-const-items.md) | `const` items | Accepted |
| [0053](0053-closures-and-function-values.md) | Closures and function values | Accepted |
| [0054](0054-scalar-reference-parameters.md) | Scalar reference parameters `&T`·`&mut T` | Accepted |
| [0055](0055-match-on-references.md) | `match` on references and reference bindings | Accepted |
| [0056](0056-recursive-struct-encoding.md) | Proof encoding of recursion through structs only | Accepted |
| [0057](0057-canonical-counting-loops.md) | The normal-form fix for countable `while` is a lint warning plus `bmb fix` | Accepted |
| [0058](0058-hypothesis-first-tracks.md) | Hypothesis-first track reset (T0–T5) and demand-driven language phases | Accepted |
| [0059](0059-hash-references-names-as-metadata.md) | Hash references and names as metadata | Accepted |
| [0060](0060-type-invariants.md) | Type invariants: data invariants proved at boundaries instead of visibility | Accepted |
| [0061](0061-floating-point-f64.md) | Floating point `f64`: IEEE 754 meaning, opaque encoding by default and bit-precise proof inside `by (bit_vector)` | Accepted |
| [0062](0062-proof-functions-and-induction.md) | Proof functions (`proof fn`) and induction: lemmas written by the user | Accepted |
| [0063](0063-canonical-query-symbols.md) | Canonical names for query symbols: position numbers inside the query instead of identity hashes | Accepted |
| [0064](0064-non-recursive-spec-fn-macros.md) | The definition of a leaf `spec fn` that binds an array is written as a macro instead of a quantified axiom | Accepted |
| [0065](0065-commutation-relation.md) | The commutation relation of edits: write set, read set, re-proof of overlapping definitions | Accepted |
| [0066](0066-measurement-model-environment.md) | Measurement model environment: the default is `qwen3.8-27b`, the frontier cross-comparison is `claude-sonnet-5-5` | Accepted |
| [0067](0067-macro-probe-for-obligations.md) | A proof obligation first solves a probe that writes every leaf definition as a macro, with 1% of the limit | Accepted |
| [0068](0068-compact-agent-view.md) | Compact agent projection (`bmb mcp --compact`) | Accepted |
| [0069](0069-probe-for-lint-checks.md) | The vacuity check and the precondition satisfiability check also solve the probe first | Accepted |
| [0070](0070-measurement-condition.md) | The same measurement condition is defined not by the commit but by what the model sees and the verdict (supplements ADR-0066 §3) | Accepted |
| [0071](0071-lean-fixed-context.md) | Fixed context: tool documentation in one place, the tool descriptions; prompt language rules only for what cannot be learned from diagnostics | Accepted |
| [0072](0072-satisfiability-probe-unfolds-non-recursive.md) | The probe of the satisfiability checks writes every definition outside recursion as a macro | Accepted |
| [0073](0073-edit-surfaces-file-diff-and-json-ast.md) | The comparison surfaces of H-EDIT: the file diff is `git apply --recount`, the JSON AST is the syntax tree + JSON Patch | Accepted |
| [0074](0074-transaction-writes-every-item.md) | The `put`·`remove` of a transaction write every item of a unit by address (types, `impl` functions, traits, trait impls) | Accepted |
| [0075](0075-specialized-instances-under-quantifiers.md) | The ground instance retry adds specialized instances for applications under quantifiers, and completeness is judged per application | Accepted |
| [0076](0076-visible-output-limit-and-length-continuation.md) | The session output limit counts only visible output, and a response cut off at the output limit is continued once in the same turn | Accepted |
| [0077](0077-repeated-runs-and-target-rule.md) | Repeated measurement and the rule for judging against targets | Accepted |
| [0078](0078-supplementary-hypotheses.md) | Registration of six supplementary hypotheses (H-EFFECT, H-INTENT, H-REVIEW, H-SMALL, H-REPAIR, H-COMPAT) | Accepted |
| [0079](0079-exists-in-specifications.md) | `exists` in specifications | Accepted |
| [0080](0080-struct-literal-field-order.md) | The fields of a struct literal are written in declaration order (the checker requires it and fixes it) | Accepted |
| [0081](0081-static-drop-at-branch-joins.md) | A value moved in only one branch is dropped statically at the end of the other branches | Accepted |
| [0082](0082-equal-types-share-an-identity.md) | Two types with the same fields have one identity (structural identity is kept) | Accepted |
| [0083](0083-in-place-moves-under-type-invariants.md) | A type invariant broken between iterations: the loop rule stays, in-place shifts use `copy_within` | Accepted |
| [0084](0084-effect-attributes-sound-subset.md) | LLVM attributes from effects are only the sound subset, and implementation waits for measured evidence | Accepted |
| [0085](0085-truncating-division-from-one-div-mod.md) | Truncating division toward 0 from one `div`·`mod` pair of the same operands | Accepted |
| [0086](0086-repeats-for-bvb-and-brb.md) | Repeated measurement of BVB·BRB and a minimum difference per set | Accepted |
| [0088](0088-gateway-capacity-failures.md) | A gateway capacity failure is not a session result | Accepted |
| [0089](0089-early-ground-instance-retry.md) | An obligation the probe could not decide is first solved by the ground instance retry at the probe's share | Accepted |
| [0090](0090-bit-operands-two-complement-image.md) | In specifications the operand of a bit operation is the two's complement image at the width of its type | Accepted |
| [0091](0091-system-interface-and-runtime.md) | System interface and runtime | Accepted |
| [0092](0092-standard-library-core.md) | Standard library core | Accepted |
| [0093](0093-prompt-caching-in-measurement-requests.md) | Prompt caching in measurement requests | Accepted |
| [0094](0094-input-driven-loops.md) | Loops that run until the input ends | Accepted |
| [0095](0095-syntax-decision-from-measurement.md) | Syntax decision from measurement (2R-06) | Accepted |
| [0096](0096-default-proof-context-without-closure.md) | The default proof context does not close over bodies (r2) | Accepted |
| [0097](0097-inherent-impl-blocks-by-bounds.md) | Inherent `impl` blocks with different bounds | Accepted |
| [0098](0098-specifications-read-moved-values.md) | Specifications read moved values | Accepted |
| [0099](0099-spec-integer-widening.md) | Integer widening in specifications: branches and results | Accepted |
| [0100](0100-stage0-feature-freeze.md) | stage0 feature freeze | Accepted |
| [0101](0101-official-write-surface.md) | The write surface of formal measurement is the single write per definition (`write`) | Accepted |
| [0102](0102-lifecycle-unit-and-test-replacement-scope.md) | The unit of judgement for utility (lifecycle unit G5-L) and the scope of principle 4 | Accepted |
| [0103](0103-evidence-stage-before-freeze.md) | The evidence stage before the freeze (2H-01) and the power of the measurements | Accepted |
| [0104](0104-large-scale-tasks-on-the-port-twin.md) | The large-scale task axis (comprehension, refactoring, task completion) and its measurement ground: the port twin | Accepted |
| [0105](0105-stage3-scope.md) | Stage 3 scope: the verdict is on the port twin, tool phases open on measured need | Accepted |
| [0106](0106-twin-documentation-parity.md) | The documentation condition of the port twin: the Rust arm without comments, with a control condition with comments registered alongside | Accepted |
| [0107](0107-map-function-values.md) | Function values in the map: `passes` edges, the position of generated definitions, `display` | Accepted |
| [0108](0108-undecided-obligations-carry-a-verdict.md) | Undecided obligations carry verdict information: bounded counterexample search, `uses` search | Accepted |
| [0109](0109-probe-candidate-invariants.md) | A batch probe that does not commit: a read tool that filters candidate invariants | Accepted |
| [0110](0110-stateless-repair-condition.md) | Stateless repair turns: a measurement condition in which every request is a new conversation | Accepted |
| [0111](0111-decision-gates-and-hypothesis-tiers.md) | Decision gates of the roadmap and hypothesis tiers | Accepted |
| [0112](0112-ab-verdict-by-the-interval-of-the-difference.md) | A/B comparisons are judged by the interval of the difference | Accepted |
| [0113](0113-ported-components-are-fixed-in-stage0.md) | Ported components are fixed in stage0 | Accepted |
| [0114](0114-closure-proof-context.md) | The proof context of a closure is that of the function that wrote it | Accepted |
| [0115](0115-mission-targets-confirmed.md) | Mission target values confirmed, and the judgement rules | Accepted |
| [0116](0116-cost-checkpoint-b0-and-stage0-shrink.md) | The porting cost checkpoint gate B0 and the stage0 shrink survey before the freeze | Accepted |
| [0117](0117-definition-summaries.md) | The definition summary `#[summary("..")]` and its input form `///` | Accepted |
| [0118](0118-twin-main-documentation-condition.md) | The main documentation condition of the twin measurement is definition summaries versus `///` | Accepted |
| [0119](0119-oracle-strength-by-mutants.md) | The mutant edition of H-ORACLE (oracle strength) | Accepted |
| [0120](0120-tool-surfaces-follow-their-hypotheses.md) | Candidate tool surfaces are cleaned up following the hypothesis verdicts | Accepted |
| [0121](0121-port-mirrors-stage0-data-shapes.md) | The port follows the data shapes of stage0 | Accepted |
| [0122](0122-port-ledger-measures.md) | Definitions of the port ledger metrics | Accepted |
| [0123](0123-solver-memory-limit.md) | The memory limit of the solver | Accepted |
| [0124](0124-push-length-is-a-runtime-fact.md) | The length of `push` is not a proof obligation but a fact the runtime maintains | Accepted |
| [0125](0125-parent-agent-with-subagents.md) | A way of working in which a parent agent manages subagents | Accepted |
| [0126](0126-advisory-solver-memory-limit.md) | The advisory queries of `lint` run under a separate memory limit | Accepted |
| [0127](0127-bounded-responses.md) | A response at the agent boundary is a window inside a byte budget | Accepted |
| [0128](0128-review-change-set.md) | Freezing the H-REVIEW change set, and its limits | Accepted |
| [0129](0129-memory-verdict-in-a-fresh-process.md) | An undecided result caused by memory is judged only on the first query of a fresh process | Accepted |
| [0130](0130-intent-item-set.md) | Freezing the H-INTENT item set, and its limits | Accepted |
| [0131](0131-advisory-rlimit.md) | Advisory queries run under their own `rlimit` | Accepted |
| [0132](0132-token-criteria-of-review-and-intent.md) | Handling the token criteria of H-REVIEW and H-INTENT | Accepted |
| [0133](0133-recursive-functions-in-satisfiability-checks.md) | Recursive definitions as recursive functions in satisfiability checks | Accepted |
| [0134](0134-examples-of-recursive-and-unrevealed-contracts.md) | The examples tool answers recursive, quantified and unrevealed contracts; `bench/intent` re-frozen | Accepted |
| [0135](0135-induction-lemma-drafts.md) | Induction lemma drafts come from `bmb fix --induction`, and only those the gate proves | Accepted |
| [0136](0136-ablation-sets.md) | Freezing the ablation editions of H-UNDECIDE, H-PROBE and H-HISTORY | Accepted |
| [0137](0137-gate-a-evidence-split.md) | The evidence premise of gate A is narrowed to "what can change the accepted language" | Accepted |
| [0138](0138-stability-and-versions.md) | Stability and version policy | Accepted |
| [0139](0139-python-in-measurement-tools.md) | Python in the measurement tools and gate tools | Accepted |
| [0140](0140-port-walk-measure.md) | Syntax tree walks in the port use the remaining depth as the measure | Accepted |
| [0141](0141-clone-of-owned-values.md) | `clone` of values that own memory | Accepted |
| [0142](0142-modeled-on-the-agent.md) | The language is modelled on how the agent works | Accepted |
| [0143](0143-succession-is-a-stopgap.md) | The surface succession is a supplementary means below the model | Accepted |
| [0144](0144-first-twin-measurement-conditions.md) | Conditions of the first measurement of the porting twins | Accepted |
| [0145](0145-practices-are-payback-targets.md) | Development practices are also targets of payback | Accepted |
| [0146](0146-text-search-and-bounded-twin-arm.md) | Text search of bodies `query text`; the BMB arm of the porting twins turns on the response window | Accepted |
| [0147](0147-two-repositories.md) | Two repositories: the refined `lang-bmb` and the work-only `bmb-works` | Accepted |
| [0148](0148-one-place-for-decision-records.md) | One place for decision records, `docs/adr`; nothing of the process stays in this repository | Accepted |
| [0149](0149-gate-b-verdict-carried-over.md) | Gate B (economics) cannot be judged yet and is carried over to gate D | Accepted |
| [0150](0150-compiler-identity-in-the-cache-header-only.md) | The compiler identity guards the incremental cache in its header only | Accepted (autonomous) |
| [0151](0151-a-line-limit-on-solver-answers.md) | A line limit on the solver's answers | Accepted |
| [0152](0152-no-units-for-a-query-the-backstop-ended.md) | A query the wall-clock safeguard ended spends no units | Accepted (autonomous) |
| [0153](0153-what-self-proof-means-and-the-oracle-after-retirement.md) | What self-proof means, and the oracle after the Rust compiler retires | Accepted |
| [0154](0154-gate-d-outcomes-and-the-north-star-measures.md) | The outcomes registered for gate D, and the measures that judge the economic premise | Accepted |
| [0155](0155-the-anchor-and-the-roads-kept-open.md) | The anchor of the design, and the roads kept open | Accepted |
| [0156](0156-packages-as-working-material-before-gate-e.md) | Packages are built as working material before gate E | Accepted |
| [0157](0157-byte-round-trips-in-the-proof-encoder.md) | Byte round trips in the proof encoder | Accepted (autonomous) |
| [0158](0158-the-standard-library-is-in-the-normal-form.md) | The standard library is in the normal form | Accepted |
| [0159](0159-a-package-calls-a-package-through-the-package-manager.md) | A package calls a package through the package manager | Accepted |
