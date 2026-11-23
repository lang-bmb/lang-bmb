# ADR-0100: stage0 feature freeze

- Status: Accepted, freeze declared 2026-11-14 (approved by the maintainer "1/2 approved": premises 14/14 `dev-docs/health/freeze-readiness.md`, `frozen: true` in this commit, compiler version `0.1.0`) · earlier approval (approved by the maintainer 2026-11-09 "ADR-0097·0098·0099, 2V-18, and after them ADR-0100 approved"). Adoption and declaration (`frozen: true`) come after the implementation of ADR-0052·0053·0097·0098·0099 (backlog 4D-01) · the pinning of ported components is supplemented by ADR-0113
- Date: 2026-11-09
- Related: ADR-0037 §3 (stage0 feature freeze during the port), `dev-docs/measurements/selfhost-port-map.md` (port map, §6 cleanup before the freeze), ADR-0095·0096 (syntax decisions), ADR-0097~0099 (open surface decisions), ROADMAP ⑤·⑥

## Context

- ADR-0037 decided that the features of stage0 (the Rust compiler) are frozen when the port starts, and that stage0 changes after that are only bug fixes and the minimum changes bootstrapping needs. The port proceeds per component with differential tests that produce the same bytes as stage0 (P0~P11). If the language of stage0 moves, the ported side has to keep following (the dual maintenance cost of v0).
- For the freeze to be measurable, a machine has to know that "the language changed". There is no such check now.
- Four open surface decisions remain: ADR-0097 (`impl` blocks with different bounds), ADR-0098 (specifications read moved values), ADR-0099 (specification integer widening), 2V-18 (ADR-0052 constants, ADR-0053 closures). After the freeze these can enter only on the BMB compiler side (ADR-0037 consequences).

## Decision (proposed)

1. **What is frozen is the accepted language.** The syntax, the type, move and effect checks, the table of diagnostic codes, the normal form (`fmt`), the identity hash (`hash`), the meaning of proof obligations. The cost of the prover (rlimit) and tools, benches and documents are not covered.
2. **Detection is a surface fingerprint (`scripts/surface.py`, `dev-docs/health/surface.json`).** For every corpus program (one per file of `tests/check`·`tests/verify`·`tests/run`, one for `lib/std`) it keeps a digest of the source and digests of the output of `bmb check` (diagnostics, exit code), `fmt` and `hash`, and it keeps the list of diagnostic codes. The standard library is input to every program, so it goes into each source digest (changing the library is a source change). An entry whose result changed while its source is unchanged, and an added or removed code, are a "surface change". The gate step `surface (4D-01)` runs it.
3. **The fingerprint keeps up before the freeze too.** While `"frozen": false`, a surface change needs no reason, but the gate fails if the fingerprint is not updated with `scripts/surface.py record` in the same commit (so that the freeze does not start from a stale fingerprint). Once it is changed to `true` at the declaration (approval), an exception with a reason is needed.
4. **Exceptions are recorded with a reason.** Only bug fixes (soundness, hangs, wrong diagnostics) and changes bootstrapping requires. For each entry `{"kind", "name", "reason"}` (a backlog ID in the reason) is written in `exceptions`, and `scripts/surface.py record` moves it to `history`. The same change is also applied to the ported side in the same batch of commits (port map §5 risks).
5. **Premises of the declaration (approval is sought only when all of these are closed).** (a) ADR-0097·0098·0099·0052·0053 are adopted or rejected and the adopted ones implemented, (b) 2R-05 (A/B of the changed surface) and 2H-02 (stage 3 scope), (c) the cleanup before the freeze of port map §6 (4A-03 done).

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **Fingerprint of corpus results, comparing only entries with the same source (recommended)** | Editing or adding to the corpus is not a change, and it catches only the compiler accepting the same source differently. 3 seconds (219 programs, 178 codes). Sensitivity check: run on the compiler before ADR-0095 (`99138a9d`), it reports a difference of 5 codes and 68 programs with a changed result |
| Only a tag and no detection | The freeze is merely a declaration, not a measurable criterion (genesis §3.5) |
| Detecting changes to the parser and checker source files | It cannot tell bug fixes and refactoring from language changes. It also blocks changes with the same result |
| Proof verdicts in the fingerprint too | Proof cost and seeds are not covered by the freeze (decision 1), and the verdicts are already pinned by the existing gates. The fingerprint gets slower |

## Consequences

- `scripts/surface.py`, `scripts/test_surface.py`, `dev-docs/health/surface.json` (`frozen: false` now), the `surface (4D-01)` step of the gate.
- A commit that changes the language before the freeze updates the fingerprint with `scripts/surface.py record` (`check` reports the changed entries, and the gate fails if it is not updated).
- The freeze declaration is the adoption of this ADR and one `frozen: true` commit.
