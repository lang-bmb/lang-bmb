# ADR-0139: Python in the measurement tools and gate tools

- Status: Accepted (approved by the maintainer 2026-11-14 "all recommendations accepted", backlog `5D-03`)
- Date: 2026-11-14
- Related: ADR-0021 (bench freezing), ADR-0037 (bootstrap), ADR-0039 (continuous maintenance, §3 trusted base), `dev-docs/HEALTH.md` §4 (self-hosting ladder)

## Context

- ADR-0039 left the Python of the bench harness as a "measurement tool outside the language" to be decided again after stage0 retirement (4G). That decision is drafted in advance (bringing forward the time spent waiting for approval).
- The Python in the repository now is 129 files, 41,827 lines (`scripts/` and `bench/`). It has three uses.
  - **Gate tools** (`scripts/`): backlog, health indicators, fingerprint, components, conformance, differential tests, progress report.
  - **Measurement harness** (`bench/`): model API clients, session records, judges, result tables.
  - **Generators and judges of frozen sets**: the MANIFEST holds the SHA-256 of those files (ADR-0021). If one byte changes, the set has to be re-frozen and the results before and after cannot be compared.
- All of it uses only the standard library (checked on 2026-11-14 by counting `import`: 0 external packages). The Python version is not pinned (it is not in `scripts/toolchain.env`).
- Self-hosting ladder L4 is "the gate and CI run without cargo". It says nothing about Python.

## Decision

1. **Python stays as a declared trusted base.** Like Z3, clang and libc it is a tool outside the language. Moving it to BMB is not made a goal. The claims of self-hosting (L0~L4) are about the verdict path of the compiler and its gate, and the measurement tools are outside it.
2. **Three conditions are raised to rules.** The gate checks what is being kept now.
   - Only the standard library is used (0 external packages).
   - The version is pinned in `scripts/toolchain.env` and CI runs with that version.
   - Every script has tests and the gate runs them (the current `test_*.py`).
3. **The files of frozen sets are not moved.** Rewriting a generator, judge or harness in another language is a re-freeze under ADR-0021, and it is not done on sets that have official results.
4. **Tools on the verdict path are looked at again one by one after 4G.** The tools that judge the correctness of the compiler (differential test `difftest.py`, fingerprint `surface.py`, component pinning `components.py`) are part of "a gate that runs without Rust" once stage0 retires. Whether to move them to BMB is looked at then as program candidates of the practicality check (HEALTH §5): the reason to move them is not self-hosting but evidence of practical programs written in BMB.
5. **The compiler does not execute Python.** S2 (external processes the compiler executes) stays clang and z3.

## Rationale

| Option | Verdict |
| --- | --- |
| **Declare as trusted base and raise the conditions to rules (proposed)** | Measurement tools are a means of measuring hypotheses, not part of the language. Moving them yields no evidence (it measures no hypothesis) and breaks the frozen sets. The three conditions are already kept, so raising them to rules costs little |
| Move everything to BMB | 40,000 lines multiplied by the porting factor (porting map §7.3). The model API clients need TLS and HTTP, so the closed builtin list of the runtime (ADR-0091) would have to be widened. The MANIFEST of every frozen set changes |
| Move only the gate tools, all right after 4G | That adds Python to the definition of L4 (without cargo). It widens the claim of self-hosting, but no measurement or user demands that claim. Decision 4 leaves a way to open it one by one |
| Do not decide | As ADR-0039 postponed it until after 4G. Meanwhile no rule prevents external packages coming in or versions diverging |

## Consequences

- Follow-up: the external package check of the gate and pinning the Python version (backlog `5D-04`).
- In the table of ADR-0039 §3, "decided again after 4G (5D-03)" is replaced by this ADR.
- Open question: whether the Python versions of the measurement machine and CI are the same now has not been checked (looked at in the pinning task).
