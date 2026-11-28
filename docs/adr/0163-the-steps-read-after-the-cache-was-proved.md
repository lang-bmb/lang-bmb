# ADR-0163: The steps read after the reuse of a cached proof was proved

- Status: Accepted (approved by the maintainer on 2026-11-28)
- Date: 2026-11-28
- Related: ADR-0162 decisions 1 and 3 (P3 is a criterion of layer 9; the closed list of trusted steps, which this record extends), ADR-0153 (what self-proof means), ADR-0032 and ADR-0150 (the caches and their keys), ROADMAP.md layer 9

## Context

- ADR-0162 decision 1 made P3 a criterion of layer 9: a cached proof is reused only under an equal key, and the key is a function of what was read.
- P3 is now stated and proved in the compiler written in BMB as a chain of contracts, for the proof cache and the per-definition cache:
  - a proof or a stored result is given only under a key equal to the one asked (the search of the table, the lookups of both caches and of the prover carry it as postconditions);
  - a proof is stored only for a proved verdict (a precondition of the one function that stores);
  - the text the proof cache's key is the digest of is the solver's version, the memory limit and the query, with no byte more, and it reads back one way (two proof functions).
  It cost 196 lines net in five files, 57 obligations and 0.70% of the resource units of the whole proof. No output changed. Fourteen mutations of the bodies and of one contract were each rejected.
- Decision 1 as worded could not be written as a contract. A specification cannot name the result of an executable function, so "the key is the digest of this text" and "equal inputs give equal keys" are not statements a contract makes; the proof stops at the text before it is hashed. That a definition's key covers everything its encoding reads is a property of the whole program, measured by `bmb verify --audit`, not a contract of one function.
- So the chain of P3 has steps between its links that are read, and none of them is on the closed list of ADR-0162 decision 3. That list takes a step only by a record.

## Decision

1. **P3 is met as proved above, with the steps of decision 2 read.** Layer 9's condition "the named properties proved" (ADR-0153 decision 6) reads P3 this way.
2. **Five steps join the closed list of ADR-0162 decision 3.**
   - `prove::obligation` and `prove::sub_obligation` count an obligation as proved without an answer of its own only in the arm where `prove::lookup` gives a proof, under the key of the text they would send; `prove::definition` takes a stored status only from a hit of `incremental::lookup`.
   - The key of the proof cache is the SHA-256 of the framed text, and two texts do not share a digest.
   - `cache::insert` is called by `prove::remember` only, and the files that `cache::load` and `incremental::load` read are trusted as the source tree is.
   - The per-definition key covers everything the encoding of the definition reads (measured by the audit), and `incremental::record` stores only definitions without a failure.
   - The pinned solver gives the same answer to the same text, and the canonical form of a query keeps its meaning.
3. **The list is closed again at ten steps.** Layer 9 closes with these or fewer.
4. **Two of them are to be shortened, not kept.** The second step falls to a contract once a specification can name the result of a pure executable function; the first once the hit arm is the only path that the type of a proved verdict allows without an answer. Each is taken up as its own change and removes its step by a record.

## Rationale

| Option | Verdict |
| --- | --- |
| **Extend the closed list by a record (chosen)** | It says what is proved and what is read, in the one place that bounds the unproved area. The list grows from five to ten, and two of the new steps name the change that removes them |
| Hold P3 open until every step is a contract | It needs a language change (naming an executable function's result in a specification) and a model of the table; layer 9 would wait on both while the proved part already rejects the mutations that matter |
| Count P3 as proved without naming the steps | The chain would have links nothing bounds, which is what the closed list exists to prevent |

## Consequences

- The roadmap's layer 9 sentence already separates what is proved, read and measured for the cache; it now points here for the list.
- The table of the caches has no model: "what was inserted is found" is not stated. It is not needed for P3 (a hit under an unequal key is what matters) and stays outside the list.
- The gate's check of the text (`scripts/trusted_steps.py steps`) now also fails when a proof is put into the proof cache anywhere but in `prove::remember`. The other four steps are read; this record is where they stand.
