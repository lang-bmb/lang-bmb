# ADR-0070: The same measurement condition is defined not by the commit but by what the model sees and the verdict (supplements ADR-0066 §3)

- Status: Accepted (approved by the maintainer 2026-10-31: "recommendation accepted". Approval item of the 2026-10-31 execution plan)
- Date: 2026-10-31
- Related: ADR-0021 (measurement procedure, freeze), ADR-0066 §3·§5 (cross-comparison at the same commit, one run per (set, model, commit)), backlog INF-17·INF-18·T0-11, JUDGMENT §10

## Context

- ADR-0066 §3 fixed the cross-comparison as "the same harness and the same commit". The commit is a proxy for the condition. If what the model sees (the compiler's diagnostics, proofs and tool descriptions, the frozen tasks, the prompt, the requests the adapter builds) and the verdict are the same, it is the same condition, but the commit makes even fixes unrelated to measurement a different condition.
- T0-10 (qwen) left only verdict fields in the result rows. The harness deletes the workspace and keeps no call record, so failure causes cannot be seen (10 of 13 BVB failures are "a commit that is proved, signature differs"). Adding session records (INF-17) changes the commit. Then, because of ADR-0066 §3, T0-11 (sonnet) would have to run at the old commit without records. The measurement machine is also bound by gateway generation at about 10 minutes per session, and parallel sessions (INF-18) cannot be used for the same reason.

## Decision

1. **Definition of the same condition.** Two runs are the same condition when, per set, all of the following are the same. (a) The `compiler/` tree from which `bmb` was built. (b) The frozen task inputs: the MANIFEST of the set (including the MANIFEST of hidden mutants). The instrument code that the BRB MANIFEST pins along with it (`harness.py`·`judge.py`·`rust_tools.py`) is excluded. (c) The harness: when the two harnesses are run with the same scripted model, the whole request bodies the model receives and the verdict fields of the result rows are byte-identical. Fields added by the later harness and `*_seconds` times are not compared.
2. **Checking tool.** `bench/condition.py same --base COMMIT --sets ...` checks (b)·(c) and writes the report `bench/conditions/BASE-HEAD.json`. There are two scripts. `oracle` stops after the reference solution, and `stress` is, after the reference solution, a non-existent tool, broken arguments and exceeding the call limit. There are two providers (`openai`, `anthropic`), and a fake transport is attached to the real adapters. The report is committed and kept as evidence.
3. **Running.** `run_models.py --same-as COMMIT` keeps (a) by construction. It uses a `bmb` built with the compiler of COMMIT (`condition.py build`, provenance `provenance.json`). It starts only if there is a report for every set, and refuses if the harness in `bench/` changed after the report. The result header line and the summary record `condition` (base commit, compiler tree, binary hash, report).
4. **The "same commit" of ADR-0066 §3 is replaced by this.** The cross-comparison needs only the same condition. "One run per (set, model, commit)" of §5 stays, and a `--same-as` run is the one run of its own commit.
5. **Where (c) does not reach.** Adapter paths that only a real model takes (`length` stop, API errors) are not passed by the scripts. The user of the report checks that the code of those paths is the same as at the base commit. What changed in `model_api.py`·`anthropic_agent.py` after `f83cd47d` is only the request record (`requests`, not in rows) and the optional record (`BMB_CONTEXT_LOG`, 2D-25, off in measurement).

## Rationale

- **Cause and layer (JUDGMENT §3):** the cause of attribution being blocked after measurement is a process rule. The condition was defined by a proxy, so improving the instrument breaks comparability. It is not a problem of the language or compiler layer.
- **Canonical way (S1):** when an experiment ties treatment and measuring instrument to a version, a change of the instrument is shown equivalent by calibration (same output for the same input). Here the request bodies and verdicts for the same scripted input are the calibration criterion. Comparing code hashes is too strict because it says "different" even when one recording feature is added. Looking at nothing is too loose.
- **Alternatives:** (i) Keep the commit. T0-11 runs serially without records, and the failures of the cross-comparison cannot be attributed either. (ii) Put the harness file hashes in the condition. The recording feature itself is a harness change, so it is the same as (i). (iii) Record with a wrapper outside the harness. It does not change frozen files, but misses the result of the last call that hit the limit and relies on monkey patching. (iv) This decision. The criterion that decided it is the definition that the condition is "what the model sees and the verdict".
- **What was checked (2026-10-31):** the harnesses of `f83cd47d`·`8d569693` were the same as the current harness on m1 (two scripts, two providers). Changing one character of the harness's error prefix is caught as a request body difference, and changing or deleting a verdict field is caught as a row difference (`bench/test_condition.py`).

## Consequences

- From T0-11 on, session records (INF-17) and parallel sessions (INF-18) are used. For each set the commit of T0-10 is given as `--same-as`. m1·m1b are `8d569693` and the other four are `f83cd47d`. That compiler is built and the report committed before the run.
- When the harness is changed and there is an earlier measurement that needs comparison, `condition.py same` is run. If it comes out different, the change is a change of condition (a new measurement, ADR-0066 §5).
- The BRB MANIFEST pinning the instrument code along with it stays as it is. When the instrument code is changed the MANIFEST is rewritten, and (c) shows the sameness.
- Cost: one check per set (eight sessions per task: two scripts, two providers, two trees), one compiler build per base commit.
