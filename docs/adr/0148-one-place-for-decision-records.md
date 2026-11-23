# ADR-0148: One place for decision records, `docs/adr`; nothing of the process stays in this repository

- Status: Accepted (maintainer instruction 2026-11-23: "Remove CLAUDE.md and bring those instructions to bmb-works; the working sessions move to bmb-works. Move all of dev-docs to bmb-works. docs keeps refined, always current documents of OSS quality. So that there is no confusion, progress records, decision records, experiments and logs all belong to bmb-works", then on the recommendation that followed: "recommendation accepted; remove dev-docs and keep the one path docs/")
- Date: 2026-11-23
- Related: ADR-0147 (two repositories; this record amends its decisions 1, 2 and 8)

## Context

- ADR-0147 split the product (`lang-bmb`) from the workspace (`bmb-works`) and planned the move as the last of five steps. Until then the work rules, the backlog, the reports and the measurement documents stayed here beside the public documents, in `dev-docs/`, and the working sessions were opened here.
- Two directories of documents in one repository made it unclear which one a reader should trust, and a session opened here wrote process records into the product.
- The decision records had two homes in prospect: the Korean originals in `dev-docs/adr/` and the English ones in `docs/decisions/`.

## Decision

1. **This repository has one directory of documents, `docs/`.** `dev-docs/`, `CLAUDE.md` and the session settings are removed here and live in `bmb-works`, with the history of `dev-docs/`. Working sessions are opened in `bmb-works`, which has this repository as a submodule.
2. **`docs/` is kept current.** A change of the design or of the state changes the documents in the same piece of work. Records of progress, of the way to a decision, of experiments and of sessions are not written here.
3. **Decision records are in `docs/adr/`, and only there.** The directory `docs/decisions/` is renamed: `adr` is the name most projects use and the name the code cites (`ADR-NNNN`). A new record is written here, in English, from the start. The Korean originals of records 0001 to 0147 are an archive in `bmb-works` and are not edited.
4. **The product gate reads nothing of the process.** The checks of the backlog, the health ratchet, the progress report and the utility report are the process gate of `bmb-works`. What the product gate needs moved with it: the surface fingerprint is `tests/surface.json` and the component pins are `tests/port/components.json`.
5. **The measurement reports go to `bmb-works` with the rest of `dev-docs/`** (ADR-0147 had them go to `bmb-bench`). What is published of them is written for `bmb-bench` from there.

## Rationale

| Option | Verdict |
| --- | --- |
| **Decision records public, in `docs/adr` (accepted)** | A decision record answers "why is it this way" and belongs beside the code it explains; the citations in code and documents stay alive. One home removes the question of which copy is current |
| Decision records to `bmb-works` with the rest | Every citation in this repository would point at a private repository |
| Keep the name `docs/decisions` | It works, but it is the less common name and does not match how the records are cited |
| Keep `dev-docs/` here until the move of the benches | The confusion this record removes would last as long as the longest measurement |

## Consequences

- Still to do from ADR-0147: the benches move to `bmb-bench` (their self-checks leave the product gate with them), and the backlog IDs that remain in comments here are replaced by the reason in place, with a check in the gate.
- A session that needs the work rules, the backlog or a measurement report opens `bmb-works`.
