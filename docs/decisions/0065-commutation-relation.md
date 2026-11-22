# ADR-0065: The commutation relation of edits: write set, read set, re-proof of overlapping definitions

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation accepted, approval items approved": both approval items 1·2. Decision 3 (refinement exemption) is not part of the relation, and decision 4 (re-proof of overlapping definitions) is the default verdict). Drafted 2026-10-29, decisions 1·3 refuted by T1-27 fixed the same day, T1-31 recommendation applied 2026-10-30
- Date: 2026-10-29
- Related: ADR-0009 (transactions), ADR-0032·0043 (incremental key = read set), ADR-0059 (identity hash, `expect`), ADR-0063 (query symbols, proposed), ADR-0064, backlog 2D-19 (`bmb weaken`), T1-12 (H-COMMUTE), T1-21, T1-27, T1-28, T1-29, T1-30, T1-31, `dev-docs/reviews/collaboration-2026-10-28.md` §3·§4

## Context

- Observation of the collaboration discussion (2026-10-28): the problem of several agents editing together is not storage but **deciding whether two edits are independent**. In a general language what a change relies on (its read set) has no boundary, so it can be decided only by testing the whole merged result again.
- In BMB what the proof of a definition relies on is already computed. That is the incremental build key (ADR-0032·0043), and the audit test (`with_audit` in `tests/incremental.rs`) checks that the key contains everything the encoding reads.
- H-COMMUTE replay (T1-12, `dev-docs/experiments/commute/results.md`):
  - The definition-level merges of the 77 pairs judged independent were all proved (0 soundness violations).
  - But the verdict was conservative. Of the 37 pairs seen as conflicts, 18 were proved when merged. The causes were treating every contract change as a conflict and taking the read set as all L2 edges.
- Specification weakening detection (2D-19, `bmb weaken`) judges the direction of a contract change. With `same`·`refined`·`compatible` the new contract keeps every promise of the old contract. The draft used this to exempt contract changes from conflict (the previous draft of decision 3 below). When T1-29 added the re-proof of overlapping definitions, the exemption did not change the verdict and only added cost. T1-31 judged that relation (rationale 5).

## Decision

1. **Write set.** The write set of one edit (one transaction, ADR-0009) is the definitions whose own text the edit changed, the definitions it added, and the definitions it removed. The own text is the `self` row of the incremental key: the qualified normal form that writes other definitions by path (`bmb reads`, the same criterion as `changed` of a commit).
   - **Draft fix (T1-27 measurement):** the draft said the definitions whose identity (`id`, ADR-0059) changed. The measurement refuted it in both directions.
     - `id` is transitive. A body edit of a callee changes the `id` of every caller, so two edits that fixed different definitions become a write conflict (53 of the 192 pairs of the semantic conflict set; the BCB pairs are in `results.md`).
     - `id` has no names. An edit that changed a contract to another `spec fn` with the same body (`pos(x)` → `alt(x)`) does not change the `id` and so drops out of the write set. But a proof that does not unfold that predicate sees them as two symbols. Running the verdict rule of the first replay (write set by the hash of `bmb hash`) on the current compiler (the `id` of ADR-0059) judged 3 such pairs commutable, and the merged result was not proved.
   - Taking renames and moves out of the write set comes after query symbols are named without names (ADR-0063, T1-10e). Then the own-text row also becomes a nameless row. The value `expect` compares (T1-21) follows the same criterion.
2. **Read set.** The read set of definition d is what the incremental proof key of d contains: its own text, the contracts of the executable functions it calls (caller key, ADR-0043), the bodies of the `spec fn`s it unfolds with `#[uses]`, the definitions of the types it reaches (including invariants, ADR-0060). The definitions edit X has to prove again are those whose key rows changed between the base version O and X. The read set of X is the union of the read sets of those definitions.
3. **The refinement exemption is not part of the relation (T1-31 fix).** A proof that reads a contract change is an overlapping read even if that change is a refinement. The verdict is settled by decision 4.
   - The refinement verdict (`bmb weaken` of 2D-19, the reader-side comparison `--readers`) remains as a tool the agent uses: it asks whether a change to a contract my proof reads is a refinement for me. The commutation and merge verdict and the incremental key do not use that answer.
   - **Previous draft (up to T1-27):** when edit A changed the contract of definition f, if that change was a refinement for every proof that reads f, the read was said not to be a conflict. Refinement is the forward direction of 2D-19 (old `requires` ⇒ new `requires`, old `requires` ∧ new `ensures` ⇒ old `ensures`, effects only `io` → `pure`, not a refinement if the signature changes), the verdict was an SMT query with fixed rlimit, and on `unknown` there was no exemption. The two soundness conditions the measurement found must be kept wherever the refinement verdict is used again.
     - **Judge by the predicates the reader sees.** Which `spec fn`s the proof of caller c unfolds is fixed by the `#[uses]` of c. A refinement that holds only with f's own `#[uses]` breaks the proof of a caller that does not unfold that predicate (found in 2D-19, 10 violations in the semantic conflict set).
     - **What the query reads is also a read set.** The query answers with the predicate bodies of O. If the reader's edit changed the body of that predicate, the answer is not right (`refine_vs_spec`, 6 violations). If the other side wrote a predicate the query unfolded (`revealed` of `--readers`), there is no exemption.
4. **Commutation and mergeability.** For two edits A, B:
   - **Commutation:** the write sets do not overlap, and the write set of A ∩ the read set of B and the write set of B ∩ the read set of A are both empty. Two commutable edits give the same program in either order of application, and every definition of that program is proved without being proved again: for each definition the key rows are the same as in one of the versions.
   - **Mergeable (default verdict):** the write sets do not overlap, and in the definition-level merge M (the definition operations of A and B on top of the base, built deterministically from O·A·B) **the definitions with overlapping reads**, namely the definitions to prove again that read a row the other side changed and that remain in M, are all proved. The other definitions have key rows the same as in one of the versions, so their proofs are kept. If M does not pass the check or the re-proof fails or is `unknown` (fixed rlimit, DESIGN §8.2), it cannot be merged (conflict).
   - If the write sets overlap there is no merge, so it is a conflict.
   - A pair that does not commute is always settled by re-proof. The re-proof uses the same incremental cache as the build gate. Then the gate of the merged version (ADR-0009) reuses those proofs, and the re-proof is an advance on work the gate would do anyway.
5. **Concurrency control.** At commit the tool checks with `expect` (ADR-0059 §5) that the read set is the same as at the start. A mismatch where a read row changed does not have to end in rejection: in the version to commit (= merge M) the definitions that read that row are proved again by decision 4. The refinement exemption is not needed on this path (rationale 5). What `expect` compares (transitive `id`, own text, key of the read set) is decided in T1-21 to fit this relation. This ADR does not decide a remote or multi-session tool server (collaboration discussion Q2, out of scope).

## Rationale

1. **Problem:** when several agents edit the same codebase, text merge is wrong in both directions (false conflicts, false merges). Proving the whole merged result again is right, but the cost follows the size of the codebase.
2. **Cause and layer:** there is no criterion for the independence verdict. The layer is the language specification. What relies on what (the read set) is fixed by the language's modular verification rules (contract boundary, `#[uses]`, no global mutable state), so the commutation relation that results is also defined by the language.
3. **Canonical way:** the serializability verdict of databases (write set and read set conflicts, optimistic concurrency control) and semantic merge. Row = definition, integrity constraint = proof, transaction = tool write (table in collaboration discussion §3).
4. **Candidates:**

| Candidate | Soundness | False conflicts | Cost |
| --- | --- | --- | --- |
| (a) Text merge (git) | No (false merges possible) | Many (near the same lines) | None |
| (b) Definition-level write conflicts only | No (a contract change can break callers) | Few | None |
| (c) Write ∩ read, every contract change is a conflict | Yes (0 violations in the 77 H-COMMUTE pairs) | Conservative (18 of 37 conflict pairs are mergeable) | Key comparison |
| (d) (c) + reader-side refinement exemption (previous recommendation) | Yes (exemption only for proved implications. The draft missed the predicates the query read, 6 violations; 0 violations after the fix of decision 3, T1-27) | Fewer than (c) (BCB 9, semantic conflict set 56) | An SMT query per definition whose contract changed |
| (e) (d) + re-proof of overlapping definitions (the configuration of T1-29) | Yes (0 violations) | 0 | Queries + re-proof |
| **(f) (c) + re-proof of overlapping definitions (recommended, T1-31)** | Yes (0 violations, T1-29·T1-31) | 0 | Re-proof (shared with the gate) |

5. **The relation of decisions 3·4 (T1-31, JUDGMENT §10 form).**
   1. **Problem:** once the re-proof stage was attached in T1-29, the presence or absence of the exemption did not change the quality of the verdict (both verdicts 0 violations, 0 false conflicts). The exemption queries cost more than the re-proof they save (BCB 272.4 seconds vs 1.8 seconds, semantic conflict set 26.8 seconds vs 2.4 seconds). The remaining question is whether there is a case where the exemption wins (a verdict that does not build the merge, the `expect` commit check, edits with large overlapping reads).
   2. **Cause and layer:** the language specification (the definition of the relation). The refinement exemption is a sufficient condition for "this read is proved without being proved again". Decision 4 answers the same question by the proof itself. The two mechanisms answer the same question, so the issue is whether to keep both in the specification.
   3. **Canonical solution and sources:** optimistic concurrency control re-validates or rolls back a transaction that failed validation of its read set (Kung·Robinson 1981). Semantic commutativity (Weihl 1988) and behavioural subtyping (Liskov·Wing 1994) are optimizations layered on top. Incremental re-verification in verifiers verifies again the definitions for which the checksum of a specification they depend on changed, and does not block invalidation by refinement (the verification result cache of Dafny·Boogie, Leino·Wüstholz 2015). BMB's incremental key (ADR-0032·0043) works the same way. The sources are cited from memory and the originals were not checked in this environment.
   4. **Comparison of candidates:**

| Candidate | C1 Soundness | C2 Determinism | S1 Canonical | S2 Root cause | S3 Economy of concepts | S4 Goal (measurement) |
| --- | --- | --- | --- | --- | --- | --- |
| A. Decision 3 → 4: exemption first, re-prove only the remaining overlap (previous draft) | Yes. Relies on two conditions (two drafts had 6·10 violations) | Yes | An optimization on top of the canonical way | A sufficient-condition special case. Falls through to 4 when it cannot answer | Adds refinement, revealed sets and the world of the queries to the language specification | Queries cost more than re-proof: BCB 272.4 vs 1.8 seconds, semantic conflict set 26.8 vs 2.4 seconds, the set with large overlapping reads 50.5 vs 7.0 seconds. The gate proves exempted callers again anyway |
| **B. Decision 4 only: re-prove overlapping definitions with no exemption (recommended)** | Yes. The re-proof is the proof (0 violations) | Yes | OCC validation + checksum incremental re-verification | The verdict is the same as the proof of the merge | No new concept (incremental key and gate) | Cheapest. With the re-proof on the gate's cache, the definitions the gate re-encodes go 356 → 17 |
| C. B as the default, exemption as a cost-based pre-filter of the tool | Yes | Yes | — | Special-case branch | Two verdict paths | The query wins in no measured cell (N ≤ 64). Even if it wins, the gate proves that definition |
| D. Put the exemption in the incremental key (the gate also skips exempted callers) | Risk: the gate comes to trust the answer of an SMT query. A defect in the refinement verdict is at once a false proof (the 6·10 violations of T1-27 are such defects) | Yes | No precedent (verifier caches are checksums) | An implication instead of a proof | Adds a semantic verdict to the incremental key | The only candidate that can reduce the gate's re-encoding. But the query cost is larger than that re-proof (above) |
| E. Ask the exemption in the reader's version (T1-30) | Yes | Yes | — | Builds the same version as B (reader's version + the other side's contract change = merge) and gives a weaker answer (sufficient condition) | Adds a concept | Adds query cost to the re-proof input of B |

   5. **Recommendation and decisive criterion:** B. The criteria that decided it are S2·S3. The exemption of A·C·E is a sufficient condition for a question B answers exactly from the same three versions (O, A, B), and when it cannot answer it falls through to B in the end. So there is no basis for adding a separate concept to the language specification. D is out by C1. The measurements of S4·S5 support this.
      - The three things counted as remaining gains were not gains in measurement and code paths.
        - **A verdict that does not build the merge:** both verdicts are functions of the same three versions. The exemption query checks the two versions and a version that aligns them (about 1.4 seconds per edit at 1,000 definitions). M is built deterministically by definition operations, and its check is included in the re-proof CPU (about 0.2~0.3 seconds). Where there is no M (write overlap) both verdicts are conflict.
        - **The `expect` commit check (T1-21):** the version to commit is M itself. A mismatch where a read row changed can be settled by decision 4, and the gate reuses that proof (decision 5). The exemption avoids the rejection but does the same job at a higher price.
        - **Edits with large overlapping reads:** even for a contract change with N = 64 callers the query cost more (1.4 vs 0.4 seconds and 3.1 vs 1.3 seconds when the revealed sets are the same; 5.7 vs 0.5 seconds and 11.2 vs 1.3 seconds when they differ per caller). The gate re-encoded the N callers even for pairs judged commuting by the exemption (the caller key of ADR-0043). The exemption saves re-proof only in the verdict and does not save the total work.
      - The refinement verdict tools (`bmb weaken`, `--readers`) stay as they are. Their use of telling the agent "is this contract change a refinement for my proof" (2D-19) is separate from the verdict.
   6. **Impact (information):** the text of decisions 2·3·4 changes. The harness added `decide` of `judge.py` (the recommended verdict: `none` + `resolve`, re-proof with the cache, no exemption queries) and kept the other variants for comparison. The compiler does not change.
   7. **Approvals needed:** together with the acceptance of this ADR (a language specification item, AUTONOMY §4), this fix (decision 3 taken out of the relation, decision 4 as the default verdict) is submitted for approval. "Approval items" below.
   8. **Workarounds:** none.

## Consequences

- **What is added to the language specification:** the definitions of write set, read set, commutation and mergeability (decisions 1·2·4). The refinement exemption does not go in (decision 3). This applies DESIGN §3 principles 1·4·8 and the "contract firewall" of §8 to collaboration. When accepted, it is written as one line next to the contract firewall row of DESIGN §8.1.
- **Measurement (T1-27, section "Re-measurement" of `dev-docs/experiments/commute/results.md`):**
  - The harness `judge.py` implements this relation. The read set is `bmb reads`, and the exemption (previous draft) is `bmb weaken --readers`.
  - On the semantic conflict set (T1-19, 192 pairs, 58 pairs with failed merges) and the 113 BCB pairs, the relation with decisions 1·3 fixed has 0 soundness violations and judged 58/58 failed merges as conflicts. Text merge (`git merge-file`) merged 57 of those 58 pairs cleanly.
  - The draft before the fix had 6 violations (decision 3, what the query read). Exempting by the definition's own `#[uses]` gives 10 violations, and the rule of the first replay using the `id` write set gives 3 violations.
  - False conflicts: on the 73 pairs of 10 separate BCB commits, 9 of 27 conflict pairs (the first replay had 18 of 36 pairs with a weaker ground truth). Text merge has 3 false conflicts. So "fewer false conflicts than text merge" of H-COMMUTE was refuted by the verdict alone. The remaining false conflicts are cases where the other side reads a contract or specification change that is not a refinement, and they were all resolved by the re-proof of decision 4 (T1-29).
  - Exemption queries: at 1,000 definitions, about 2.1 seconds of CPU per `weaken --readers` run, about 13 solver obligations.
- **Measurement (T1-29, "Resolving non-commuting pairs" of `results.md`):** the stage of decision 4 was added to the harness (`resolve` of `judge.py`, compiler `bmb verify --def`).
  - For conflict pairs whose write sets do not overlap, the definition-level merge is checked and only the reading definitions of the conflict elements are proved again.
  - 0 violations: the merges of the resolved pairs were all proved. False conflicts go BCB 9 → 0, semantic conflict set 56 → 0. The 58 pairs with failed merges all stayed conflicts.
  - Cost: the re-proof CPU is 3.3% (about 100 definitions) and 0.8~1.1% (1,000 definitions) of the full proof of the merge of the same pair (no cache).
  - Attaching the stage to the verdict without exemption (recommended B) also gives 0 violations and 0 false conflicts (semantic conflict set 99 → 0, BCB 10 → 0·4 → 0).
- **Measurement (T1-31, "Edits with large overlapping reads" of `results.md`):** a new frozen set (`hot.py`, 24 pairs, 1,000 background definitions): A touches a contract with N ∈ {1, 4, 16, 64} callers, and B touches all of those callers.
  - Both verdicts: 0 violations, 8/8 failed merges left as conflicts, 0 false conflicts after the stage.
  - The 16 pairs whose merge is proved: the exemption (previous draft) judged all 16 pairs commuting without re-proof but with 50.5 seconds of query CPU, and the recommended verdict needs 7.0 seconds of re-proof (339 definitions, 679 obligations). The re-proof is 1.5% of the full cold proof of the merge.
  - The gate of the merge (warm incremental cache) re-encoded 356 definitions regardless of the two verdicts (17.9 seconds). With the re-proof on the same cache the gate's re-encoding drops to 17, and the sum of re-proof and gate is 22.4 seconds (one full check is added, about 0.28 seconds per pair).
- **Premises and remaining decisions:**
  - ADR-0063 (canonical numbers of query symbols): needed so that renames do not change the keys of the read set.
  - T1-28: two structs with the same fields have the same `id`. The write set is the own text written by path (fix of decision 1), so this case is not handled separately. It remains for `expect` (T1-21), which pairs by `id`, and for the sharing in `weaken`.
  - T1-21: the value `expect` compares. Per decision 5 a mismatch of read rows can be settled by decision 4.
- **Refutation and revert:** if even one case appears where the merged result of a pair judged commuting or mergeable is not proved, the tool features that use this relation are turned off until the cause (a missing read set entry, a missing re-proof target) is fixed. Other parts of the language do not rely on this relation.

## Approval items

For the maintainer to approve (AUTONOMY §4: language specification items):

1. Acceptance of this ADR: the write set (decision 1), the read set (decision 2), and commutation and mergeability (decision 4) become language specification, and on acceptance one line is added to DESIGN §8.1.
2. The fix of T1-31: the refinement exemption is not part of the relation (decision 3), and pairs that do not commute are judged by the re-proof of overlapping definitions (decision 4 is the default). If the previous draft (ask the exemption first and re-prove only the remaining overlap) is chosen, decision 3 is reverted to the previous draft.

Done without approval: the harness (`decide` of `judge.py`, `hot.py`), the measurements, the judgement of T1-30·T1-31 in the backlog.
