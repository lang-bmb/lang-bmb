# ADR-0043: The caller key does not contain the callee's `#[uses]`

- Status: Accepted (approved 2026-10-25). The name dependence of the caller hash is superseded by ADR-0059 §4 (a nameless caller hash)
- Date: 2026-10-25
- Related: ADR-0019 §2 (contract hash), ADR-0032 (supersedes the callee row of the incremental key), ADR-0042 (BCB), DESIGN.md §8.1, backlog 2D-22

## Context
The incremental key (ADR-0032) contained, for each executable function a definition calls, that function's contract hash (ADR-0019 §2). The contract hash hashes the attributes, signature and contract, and the attributes include `#[uses]`. But `#[uses]` unfolds `spec fn` bodies only in that function's own queries (`verify::uses_of` reads only the attributes of the definition being proved. Unfolding is not transitive, and a caller sees a callee only by its contract, ADR-0017 §4). So when only the callee's `#[uses]` changed, the caller was encoded again even though the caller's query text was unchanged. In the first BCB implementation (2A-14a), 35~43% of the body edits of the N = 100 history changed `#[uses]`. The contract firewall (DESIGN.md §8.1) was wider than necessary.

## Decision
- The callee row of the proof key and of the IR key uses the **caller hash**. The caller hash is the SHA-256 of the qualified normal form without the body and `#[uses]` (the attribute `#[effects]`, the signature, the contract) (`fmt::caller_text`).
- When collecting the `spec fn` declarations a query can read (ADR-0033, `reads` in `incremental.rs`), executable functions are also scanned by their caller text. The `spec fn`s named by a callee's `#[uses]` enter only that function's queries.
- The contract hash (`iface`) itself and `bmb hash` and the map output are not changed. A `#[uses]` change still shows as a surface change in the map.

## Rationale
- The key must be "everything the encoding reads" (ADR-0032). The encoding of a call site reads only the callee's `requires`·`ensures`·signature. It does not read `#[uses]`. This decision fits the key to that fact.
- Audit mode (`incremental::with_audit`) encodes the definitions that hit again and compares the script hash, status and proof list. To `edits_that_shift_spans_or_change_context_keep_outputs_exact` in `tests/incremental.rs` were added an edit that removes an unnecessary entry from a callee's `#[uses]` and an edit that removes a necessary entry so that the function fails. Both have 0 mismatches. The caller of a failed callee assumes only the callee's contract, so it is right that it stays proved.
- Alternative: remove `#[uses]` from the contract hash. The meaning that the map's `iface` represents the surface (ADR-0019) changes, and it spreads to `impact` and the duplicate queries. Changing only the key has a smaller scope. Rejected.

## Consequences
- Measurement (the frozen BCB histories `body-heavy-1k`·`body-only-1k`, 200 commits, the re-encoded set of `build --stats`, excluding commit 0 and cold measurements): the sum of re-encoded definitions is `body-heavy` 458 → 335 (−27%), `body-only` 448 → 306 (−32%), mean per commit 2.29 → 1.68, 2.24 → 1.53. 56 and 62 commits shrank, and at every commit the new set is a subset of the old set. The `build` CPU per commit is almost the same (sum over 200 commits 65 seconds → 65 seconds, 67 seconds → 66 seconds). The breakdown of the warm commit cost was not measured. Re-proving one or two definitions appears small against the fixed cost (check, key computation, code generation, link). Whether the reduced share shows in CPU is looked at with N = 10,000 (2A-14f).
- The B arms of BCB are measured with this compiler. The Rust arms of `body-only-1k` already measured in 2A-14e do not use the BMB compiler, so they are used as is.
- The cache file format is unchanged. The compiler identity (the source hash of `build.rs`) changes, so existing caches are ignored and rebuilt.
