# ADR-0138: Stability and version policy

- Status: Accepted (approved by the maintainer 2026-11-14 "1/2 approved", backlog `5B-01`)
- Date: 2026-11-14
- Related: ADR-0019 (normal form), ADR-0037 (bootstrap), ADR-0059 (identity hash), ADR-0095 (input aliases), ADR-0100 (stage0 feature freeze), ADR-0113 (pinning of ported components), `docs/ROADMAP.md` 5B, `docs/reference/`

## Context

- The version of the compiler has never changed from `0.0.1` (`compiler/Cargo.toml`). What change changes the version has never been defined.
- Deterministic tools that judge compatibility already exist. The accepted language is watched by the fingerprint (`scripts/surface.py`, 248 programs and 191 diagnostic codes) and the 402 conformance cases (`scripts/conformance.py`), contract compatibility by `bmb weaken` (H-COMPAT), and the identity of a definition by the hash (ADR-0059). Diagnostic codes are retired but not reused (`diag::RETIRED`).
- There is a precedent of carrying a breaking change by machine: the syntax change of ADR-0095 accepted the old form as an input alias, and `bmb fix` and the normal form moved it to the new form (corpus 128/128).
- The freeze (ADR-0100) binds the accepted language during porting. There is no rule for after it (after stage0 retirement).
- During 0.x, refactoring is always allowed (JUDGMENT §6). This ADR does not change that: what it defines is what is announced and how things are migrated.

## Decision

1. **There are four objects of compatibility, and each is judged by a tool that exists now.**

   | Object | What broken means | Judgement |
   | --- | --- | --- |
   | Accepted language | A program that was accepted is rejected, or the meaning of the same program (output, proof result) changes | Fingerprint and conformance cases |
   | Diagnostic codes | The meaning of a code changes or a number is reused | `docs/diagnostics.md`, `RETIRED` |
   | Contracts of the standard library | The contract of a `pub` function changes incompatibly | The verdict of `bmb weaken` |
   | Output shape of the tools | A JSONL field of the CLI or an argument or response field of an MCP tool disappears or changes meaning | Differential test cases (`tests/port`) and CLI tests |

   A change in the computation of the identity hash counts as a breaking change (caches and `expect` values become invalid).
2. **Versions before 1.0 are `0.MINOR.PATCH`.** If one of the four above breaks, MINOR is raised; otherwise PATCH. A commit in which an entry appears in the `history` of the fingerprint must raise MINOR (the gate checks it; a task after approval).
3. **Milestone versions.** The freeze declaration (`4D-01`) is `0.1.0`, and the full fixed point (gate D, self-hosting ladder L3) is `0.2.0`. The conditions of `1.0.0` are stage0 retirement (`4G-01`), the verdicts of gates B and D being committed, and every section of the reference corresponding to conformance cases (5A).
4. **There are no editions.** One compiler does not accept two dialects. A breaking surface change is made only in the following order.
   1. The new form is accepted as canonical, and the old form is accepted as an input alias with a warning (W code) and a machine fix (`bmb fix`) attached. The normal form emits only the new form.
   2. In the next MINOR the old form is rejected (E code, the fix stays attached).
   - A breaking change without a machine fix has its migration procedure written in an ADR before 1.0, and is MAJOR after 1.0.
5. **Pass criterion of a migration.** A commit that raises MINOR shows that the corpus of the previous MINOR (the `tests/`, `lib/std` and bench seeds of the tag) passes the gate after `bmb fix --write`.
6. **After 1.0 it is semantic versioning.** MAJOR is a breaking change without machine migration, MINOR is additive changes and breaking changes with machine migration, PATCH is the rest.
7. **During the freeze period the rules of ADR-0100 take precedence.** From the freeze to stage0 retirement, changes to the accepted language are only the exceptions of that ADR, and each exception raises MINOR.

## Rationale

| Option | Verdict |
| --- | --- |
| **Four objects, no editions, migration through aliases (proposed)** | The judgement of compatibility rests on deterministic tools that already exist (claims by code path). It fits one compiler and one normal form (ADR-0019), and ADR-0095 went through this procedure once |
| Rust-style editions | Accepting an old dialect forever leaves two surfaces in the compiler and its port (amount to port, amount to prove). In a language written by agents the cost of machine migration is far lower than for people, so editions buy little |
| No policy until 1.0 | The way so far. The freeze declaration is the first version event, yet the version cannot say it. The condition notation of measurement results (ADR-0070) also leans only on the commit hash |
| Calendar versions | They say nothing about compatibility |

## Consequences

- The version was raised to `0.1.0` in the freeze declaration commit (`compiler/Cargo.toml`). The version goes into `serverInfo` of MCP and the compiler identity of the incremental cache.
- Follow-up: the gate compares the `history` of the fingerprint with the version (backlog `5B-02`). `5C-04` (release procedure) uses this version in tags and the change log.
- Open question: whether to put the `pub` surface of the standard library into the fingerprint (now it looks only at the results of programs). It can be solved by a gate job that compares `bmb weaken` against the `lib/std` of the previous MINOR (looked at together in `5B-02`).
