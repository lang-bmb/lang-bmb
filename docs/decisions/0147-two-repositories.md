# ADR-0147: Two repositories: the refined `lang-bmb` and the work-only `bmb-works`

- Status: Accepted (maintainer instruction 2026-11-22: "Only refined files and documents in lang-bmb; bmb-works is a work-only umbrella that has lang-bmb as a submodule. Restructure completely, manage at the level of an external OSS project", and "Ok, go ahead" to the recommendation of the review). Of decision 7, the language was answered on 2026-11-22 ("the official language is written in English, works allows both"). The same day "recommendation accepted, go ahead": the benches go whole to `bmb-works` (decision 1), the decision records are all rewritten in English (decision 2), the commit messages of `lang-bmb` are in English from after the move (decision 7). The licence and whether `bmb-works` is public have no answer yet
- Date: 2026-11-22
- Related: CLAUDE.md (document locations, commit rules), `dev-docs/AUTONOMY.md` (work loop), `dev-docs/HEALTH.md` (`scripts/ship.sh`), `scripts/check.sh`, ADR-0021 (frozen sets), ADR-0058 (hypothesis registry)

## Context

- One repository has two readers. The outside reader looks at the current language and documents; workers and agents look at the records of decisions, process and measurement. The tracked files are `bench` 4,073 (2,313 results), `tests` 1,662, `dev-docs` 340, `compiler` 108, `docs` 13.
- The two are entangled. The gate (`scripts/check.sh`) reads the backlog, the health baseline, the progress report and the utility report, and `docs/ROADMAP.md` holds the hypothesis registry and backlog IDs. Code and documents write ADR numbers about 2,900 times, and `compiler` writes backlog IDs 716 times. There is no `LICENSE`·`CONTRIBUTING`·`CHANGELOG`.

## Decision

1. **Two repositories.** `lang-bmb` is the product: compiler, standard library, the port, tests, public documents, the product gate. The benches (`bench/`: harnesses, frozen sets, result rows) go whole to `bmb-works` (corrected 2026-11-22: at first the harnesses and frozen sets were to stay. The text inside the frozen sets is Korean data, and fixing it changes the sets). `bmb-works` is the workspace: `lang-bmb` as a submodule (`lang-bmb/`), the work rules (`CLAUDE.md`), process documents (backlog, judgement criteria, autonomy rules, health, measurements, experiments, reviews, investigations, reports), the benches, process scripts and the process gate.
2. **The decision records (ADRs) stay on the public side** (`lang-bmb/docs/decisions/`), all rewritten in English (numbers and file names unchanged, the Korean originals are kept in `bmb-works`). The ADR citations of code and documents stand as they are. Backlog IDs are removed from `lang-bmb`: comments state the reason in place.
3. **`lang-bmb` stands alone.** It does not point to paths of `bmb-works` or to backlog IDs, and the product gate (build, tests, proofs, differential tests, fixed point) passes with `lang-bmb` alone. The gate checks that (dead reference check).
4. **History is not rewritten.** The history of `dev-docs` is copied to `bmb-works` (the history of the subdirectory is split off and moved), and in `lang-bmb` it is deleted in a commit after that. The rule of no force push to main stays.
5. **Order of commits.** After a commit of `lang-bmb` passes the product gate, one commit of `bmb-works` raises the submodule pointer together with the records of that work (backlog, measurements). The `ship` of `bmb-works` runs the two gates in turn. Measurement documents point to code by the commit hash of `lang-bmb`, as now.
6. **The public documents are rewritten.** `docs/ROADMAP.md` holds only the current state and plan, and the hypothesis registry is kept on the public side as "claims and evidence" (including what was refuted). Detailed measurement reports are in `bmb-works`, and the public documents give the figures and commits.
7. **Language.** The text of `lang-bmb` is English (maintainer decision 2026-11-22): documents, guides, decision records, the summaries and comments of the source, the descriptions of tests. `bmb-works` uses both Korean and English. The commit messages of `lang-bmb` are in English from after the move. The text inside frozen bench sets is data, not documentation: fixing it changes the sets (ADR-0021). **Not yet decided (the maintainer):** the licence, whether `bmb-works` is public. Thanks to decision 3, neither blocks the earlier steps of the restructuring.
8. **Order and timing.** (1) the skeleton of `bmb-works` and the list of what to move, (2) the gate into two, (3) the public documents, (4) the basic OSS files, (5) moving, deleting from `lang-bmb`, and the dead reference check. The move (5) is done in one go after the running measurement (`3P-05b` of gate B) finishes: before that nothing is deleted from `lang-bmb` (the same document is not kept in two places).

## Rationale

| Option | Verdict |
| --- | --- |
| **Split in two, with the ADRs public (accepted)** | The readers are separated. Decision records are a public asset in OSS (Rust's RFCs, Python's PEPs) and the 2,900 citations do not die. The backlog, measurements, experiments and session rules are process |
| The ADRs to `bmb-works` too | All the citations in `lang-bmb` would have to be removed, and removing them makes the "why" disappear from the code |
| Split one repository into `docs/` and `dev-docs/` (now) | The navigation cost for the outside reader stays the same. The majority of the tracked files are bench result rows and process documents |
| Rewrite the history too | It changes the history of main. All that is gained is the tidiness of past commits |

## Consequences

- Backlog `INF-44`~`INF-48` (the five steps of decision 8).
- Rules that change after acceptance: CLAUDE.md goes to `bmb-works`, and a contribution guide (`CONTRIBUTING.md`) stands in `lang-bmb`. "When the design changes, DESIGN.md is fixed together" and "decisions are recorded as ADRs" stay, and their place is `lang-bmb`. The place of "update the backlog when work starts or completes" is `bmb-works`.
