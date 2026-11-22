# ADR-0059: Hash references and names as metadata

- Status: Accepted (approved by the maintainer 2026-10-28 "ADR-0059 to 0062 accepted"). The query symbol row of §4 is superseded (ADR-0063, 2026-11-01). Supersedes the definition hash of ADR-0019 §2 and the parts of the key rows of ADR-0032·0043 that contain names
- Date: 2026-10-28
- Related: ADR-0019 §2 (definition hash), ADR-0032·0043 (incremental keys), ADR-0011 (storage is text), ADR-0009 (transactions), ADR-0006 (rationale on the node hash), ADR-0020 (map schema), DESIGN.md §3 principles 1·7·8, `dev-docs/reviews/direction-2026-10-28.md` §5·§6 (H-ID, H-PAR)

## Confirmed at acceptance (2026-10-28)
- Implementation phase: ROADMAP M2-c (identity and collaboration), backlog T1-10. T1-ID-01~05 of the Consequences table are its subtasks.
- The refutation conditions (Consequences section) are checked during implementation. If even one holds, this ADR is rejected and reverted (no data migration).
- Exposing the `rename` and `expect` of §5 over MCP follows the rule that preserves measurement conditions (ADR-0021, 2D). The tool list of a frozen task set is not changed, and exposure comes with a new edition.

## Confirmed at implementation (2026-10-28, backlog T1-10)
- **§1 Nominal types and traits.** Hashing structure only gives two types with the same structure (`struct Sq { side: i64 }`, `struct Big { n: i64 }`) the same `id`. In Rust they are different types and a trait call dispatches on which one it is. So `fn labels(a: Sq, b: Big)` and the definition with the parameter types swapped mean different things but get the same `id` (confirmed in `tests/run/trait_defaults.bmb`). The same holds for two traits (same methods, same `id`). The implementation puts the trait `impl`s of a type into the text of that type, and every `impl` of a trait into the text of that trait (`impl #trait for <type> { #trait#k = #fn, .. }`). Then the trait, the `impl` functions and the implementing types become one recursive component, and inside the component each member has a different position. Cost: editing the body of one `impl` function changes the `id` of that component and of every definition that uses it (as content addressing defines). Unison solves the same problem with the GUID of `unique type`. BMB stores text (§3), so there is no place to put a GUID.
- **§1 Component order.** With only "sort by the erased hash, ties by path", ties are common in the component above (`impl` functions whose erased text is the same, like `<Sq as Shape>::area` and `<Big as Shape>::area`). Then renaming a type changes the order and the `id`s ripple. So colour refinement was added: re-hash with the text that writes the other members by their previous hash, until the partition splits no further, and only the remaining ties are decided by path (the remaining ties are truly symmetric members, so the order does not change the text). The order of trait methods is decided by the method text and the hashes of the `impl` functions that define that method, and the remaining ties by name. Only two methods with the same signature, contract and all implementations are in this case, and renaming them can change the `#trait#k` of calls (the meaning is the same).
- **§1 Remaining details.** A default body is part of the trait text, and a default body definition is `default #trait#k` plus the function. Type arguments are written by segment position counted from the end of the path (`f::<A>` and `u::f::<A>` are the same). `_` (the wildcard) is not a name. An inherent `impl` function with no generics has no head, like the qualified text of ADR-0019. The rename invariant is checked by the fuzzer: in generated programs and the corpus, renaming every occurrence of one name (definition, unit, type, field, variant, parameter, local, label) leaves every `id`, `body` and `iface` the same, and a literal edit changes the `id` of that definition (`renames_keep_identities_and_edits_change_them` in `tests/fmt_fuzz.rs`; run once with every name of the corpus, passed).
- **§4 `body`·`iface`.** Changed to identity hashes. Consequence: when the `id` of a referenced definition changes, the `id` and `body` of the referrer change too. When the body of a `spec fn` called by a contract changes, `iface` changes too. Other members of the same component are written `#self.k`, so editing their bodies does not change `body` or `iface`.
- **§4 `changed` of a commit.** Compared by identity, one body edit puts every transitive referrer in `changed`. That is the question `impact` answers, so `changed` stays as "the definition's own text (the qualified hash that writes other definitions by path) changed". The MCP commit response is unchanged too.
- **§4 Incremental keys and query symbols (T1-ID-02).** A nameless proof key is right only when the query text has no names. Queries now contain names: definition symbols `|u::f|`, local constants `|x@0|`, callee results `|u::g.result@0|`, field selectors, datatype constructors. And there are places where the encoder walks locals in name order (the merge of `Enc::branch` walks `vars` in name order, declaring constants and adding assertions). If names are taken out of the key while they remain in the query, a renamed definition reuses a result (script, `rlimit`) made with different symbols, and the `rlimit` of the status line differs depending on whether the cache was hit. So T1-ID-02 was split in two. This time (T1-10d): the stored obligation positions changed from byte offsets to syntax node positions (`incremental::node_spans`), the source text row was removed from the proof key (layout and comment edits inside a definition do not re-encode), and the file format went to v2. The rows that contain names (path, qualified normal form, callee surface, `spec fn` declarations, type rows) remain as the legend of the names the query contains. Remaining work (T1-10e): build query symbols without names and remove the legend rows. This needs changes across symbol naming in `verify.rs` and the name-order walks, so it was postponed to avoid overlapping with the concurrent work implementing ADR-0062.
- **§4 Problem with the symbol naming rule (decision needed).** "Name definition symbols with an `id` prefix" ties the symbol to the body of the definition (the `id` contains the body). Then when the body of a `spec fn` that is not unfolded changes, the text of every query that uses its name changes and misses the proof cache. Now the symbol is the name, so it does not miss (contract firewall, ADR-0032). Two different `spec fn`s with the same signature have to be told apart, so the surface hash cannot be used for naming either. For the same reason, putting the callee's `id` in the incremental key also breaks the firewall. The canonical way is canonical names assigned in order of first appearance per query (alpha normalization) plus a legend, with the key in the same shape (a reference is a position number in the key, and per number what the query reads: for an executable callee the nameless surface, for a `spec fn` the declaration or body). This changes the symbol rule of §4, so the maintainer's decision is needed before starting T1-10e.
- **`expect` and H-PAR.** `id` is transitive, so a body edit of a callee also changes the `id` of the caller. So `expect` (path → `id`) sees body edits of different definitions (callee and caller) as a conflict. It does not conflict with renames (`expect_rejects_a_changed_definition_only` in `tests/workspace.rs`). The value `expect` compares in H-PAR (transitive `id`, own text) is decided by measurement (T1-21).
- **Scope of `rename`.** It accepts functions, `spec fn`s, `impl` functions, types and traits. It commits only if the multiset of all `id`s is the same after the change, and rejects if the new name captures another reference (shadowing the built-in `print_i64` with the same signature). Trait methods and moves between units are not done yet (T1-23). Not exposed over MCP (T1-20).
- **Measurement (T1-ID-05, `dev-docs/experiments/rename-ripple/results.md`).** Over the same 424 renames, cases where the `id`, body or contract hash of another definition changed: previous compiler 367 (87%, bodies 626, contracts 384) → 0. Definitions re-encoded 848 → 847, proof cache misses 3,560 → 3,560 (unchanged until T1-10e). Readability: the maps of the 85 corpus files are byte-identical to before once `id` is removed and hash values are masked. V7 is unchanged because query text did not change (0.0, baseline 0.0, `scripts/health.py`).
- **BNB.** The bench BNB checks that the duplicate groups of the BMB map and of the Rust twin analyzer are the same. Grouping by identity groups more definitions that differ only in unit and call the same predicate (at N=100, 5 groups → 10 groups), so the analyzer was also changed to write calls by the identity of the target (`bench/bnb/analyze.py`). The answers of the frozen tasks `n26` and `n27` (duplicate questions) did not change at N=100, 1,000 and 10,000.
- **Verdict on the refutation conditions.** (1) Does a rename change the `id` of another definition: no (measurement above, fuzzer). The incremental key in the T1-10d state changes on rename. This is because of the legend rows and can be judged only when T1-10e is done. (2) Audit mismatches: 0 (including layout and rename edits). Whether the nameless key contains all inputs is looked at again in T1-10e. (3) V7: hash symbols are not used yet, so not applicable. So far no refutation condition has held.

## Context

Names are now part of the **identity** of a definition. Names enter in four places.

| Place | Code path | How names enter |
| --- | --- | --- |
| Definition hash | `fmt::qualified_text`(`compiler/src/fmt.rs:799`), `fmt::def_hash`(`fmt.rs:936`) | References to other definitions are written by full path (`Printer::path`, `fmt.rs:92`). Only the definition's own name is replaced by `_`. Self-recursive calls, parameters, locals and field names remain as they are |
| Incremental key | `incremental.rs:505~620` | The source text piece of the definition (because obligation positions are byte offsets), the qualified normal form, the `caller_text` hash of callees (ADR-0032·0043) |
| Proof cache key | `cache.rs:30`(`ProofCache::key` = solver version + query text) | The definition symbols in the query are path names (`verify::sym`, `verify.rs:1879`: `\|unit::f\|`) |
| Map, workspace | `map.rs:824·909`(L1 `body`·`iface`, `duplicates`), `workspace.rs:157·308`(`changed` of a commit) | Use the definition hash above as is |

Measurement (`dev-docs/experiments/rename-ripple/`):

- In the run programs and `lib/bytes.bmb`, 412 definitions were renamed one at a time.
- In 356 cases (86%) the hash of another definition changed.
- The referrers whose hash changed are 611 bodies and 377 contracts. When a contract hash changes, the incremental keys of that referrer's callers change too.
- Parameter names are in the contract hash, so renaming a parameter ripples the same way (not measured).
- `duplicates` (2D-06) groups by body hash. So it does not find alpha-equivalent definitions that differ only in local names.

This state blocks two things.

- **Preserving refactoring (3C).** Renames and moves break unrelated proof caches.
- **Concurrent editing (H-PAR).** Changing one name makes every definition that uses that name a conflict candidate.

## Decision

1. **Identity hash.** The identity (`id`) of a definition is the SHA-256 of its nameless normal form.
   - A reference to another definition (function, `spec fn`, type, trait) is written as the target's `id` (`#<hex>`).
   - A reference inside the same recursive component is written `#self.k`. The component is the Tarjan SCC used now (`check.rs:7541`, `Checker::recursive`), and type cycles are the same.
     - The hash of a component is the hash of the member texts joined in a fixed order.
     - The `id` of a member is `#<component hash>.k`.
     - The order is decided by sorting on the hash with references inside the component erased (the same rule as Unison).
     - Members with the same hash are alpha-equivalent to each other. Only in this case is the order decided by path, and that fact is written in the documentation.
   - Bound names are replaced by positional names (`_0`, `_1`, ..) in binding order. This covers parameters, result names, locals, labels, type parameters and `forall` variables (alpha normalization).
   - Fields and variants are written by their declaration order position inside the type. Declaration order fixes layout and discriminant values, so it is part of the meaning.
   - Attributes, effects, contracts, `decreases` and triggers go in as they are. The entries of `#[uses]` are also written as `id`s.
   - Not included: the `#[summary]` of a unit, rationale prose. Both are metadata.
   - Language built-ins (`Vec`, `Option`, `std::mem::replace`, integer types) are not definitions, so they stay as names.
2. **Names are an index.** The workspace derives the index `path → id` from the text.
   - One definition per name (DESIGN §4.1 "names", principle 8) stays. Two definitions at the same path are rejected as now (E0302).
   - Unison keeps two definitions for one name after a merge, as hash-qualified names. BMB does not follow this, for zero ambiguity.
3. **Storage stays text (ADR-0011).** The file keeps the normal form written with names, and `id` is computed deterministically at load. There is no separate codebase store.
4. **Keys are built without names.**
   - **`bmb hash` and map L1:** `body` and `iface` are redefined as identity hashes (`iface` = the identity without the body). So they are invariant under renames. An `id` field is added.
   - **Incremental keys (ADR-0032·0043):**
     - The callee rows use the nameless caller hash.
     - The "source text" row is replaced by the nameless normal form.
     - The stored obligation positions are AST node paths instead of byte offsets. They are mapped to positions in the current source only when a diagnostic is issued.
   - **Proof queries:**
     - Definition symbols are named with an `id` prefix (`|#5568cad7c114|`). Local symbols are named with positional names.
     - `--emit-smt` writes the name legend to a separate file outside the query text.
     - Then the proof cache key (the query text) is invariant under names. Renaming symbols did not change verdicts (1B-40: for each of 718 obligations, over 16 variants including 4 symbol-renaming variants, verdicts changed in 0%).
5. **Writing is by name.**
   - The agent writes by name as now (Rust-shaped code in `put`, a path in `remove`). The tool resolves to `id` at commit (direction §4 "structural address, familiar content").
   - Two operations are added to transactions.
     - `rename` (path → new name): the tool fixes the index and the text of referrers. It changes the `id` of no definition.
     - `expect` (path → `id`) precondition: if the current `id` differs, the commit is rejected. This is optimistic concurrency (H-PAR).
   - So a rename and a body edit of different definitions do not conflict.
6. **The display side is names.**
   - The map, diagnostics, counterexamples and `impact` print names as now and append the `id`. The agent never has to read a hash.
   - `duplicates` groups by identity hash. Then it finds alpha-equivalent definitions, and the merging of isomorphic functions in DESIGN §8.1 gets an exact key.
   - Rationale (ADR-0006, 2E) is tied to the `id`.

## Rationale

1. **Problem.** 86% of 412 renames changed the hash of another definition. Identity and name are mixed in one value, so an edit that changes only a name costs the same as an edit that changes meaning.
2. **Cause and layer.** The compiler and tool layer (the definition of the hash, the composition of the keys). The cause is that ADR-0019 §2 fixed the "qualified normal form" as the hash input. The surface grammar does not change, so it is not the language layer. ADR-0019 solved the same problem only for the definition's own name ("taking its own name out of the body hash lets the same definition be found after a move or rename").
3. **Canonical solution and sources.**
   - Content-addressed code: Unison identifies a definition by the hash of its syntax tree, keeps names as metadata attached to the hash, and references dependencies by hash. A cycle of mutually recursive definitions gets one hash, and the members are `#x.n` (cycle hash and position). The order is sorted by the individual hashes with the cycle erased ([hashes](https://www.unison-lang.org/docs/language-reference/hashes), [tour](https://unison-lang.org/docs/tour), [unisonweb/unison](https://github.com/unisonweb/unison). The sites did not open in this environment, so this was confirmed from search summaries; the original text was not checked).
   - Deciding alpha equivalence of bound names: positional names (de Bruijn indices) are a standard PL technique (S1 (d)).
   - Building build keys from the hash of the input content rather than from names is the standard of content-addressed builds (S1 (c)).
4. **Comparison of candidates.**

   | Candidate | C1 Soundness | C2 Determinism | C4 Definition | S1 Canonical | S2 Root cause | S3 Consistency | S4 Goal |
   | --- | --- | --- | --- | --- | --- | --- | --- |
   | (a) Current: path hash | Pass | Pass | Pass | Partial | 86% of renames ripple | Only the own name is an exception | Bad for refactoring and concurrent editing |
   | **(b) Text storage + hash references at load, names are an index** | Pass (`with_audit` checks that the key contains everything the encoding reads) | Pass (component order rule, ties by path) | Pass (storage is text, principle 1) | Pass (Unison, alpha normalization) | Removes the whole class of names entering identity | One `id` ties map, keys, rationale and duplicates | 0 invalidation on rename and move, definition-level concurrency |
   | (c) Unison-style codebase store (no text files) | Pass | Pass | Violation: ADR-0011 and R1 (storage is text, layout is the tool's) would have to change | Pass | Same as (b) | Same as (b) | Same as (b), and moves away from git and text tools |
   | (d) Current + only a `rename` tool | Pass | Pass | Pass | — | The text of referrers changes, so hashes ripple as before | — | Only writing gets easier |

5. **Recommendation and decisive criterion.** (b) is recommended.
   - (a) and (d) fail on S2. The cause (names in identity) remains.
   - (c) fails on C4. It needs a revision of the definition and gains nothing over (b). The only cost of keeping text is computing hashes at load.
6. **Impact (information).**
   - `fmt.rs`: add the identity printer. The qualified printer stays for display.
   - `check.rs`: reuse the SCC.
   - `incremental.rs`: replace key rows, obligation positions as node paths.
   - `verify.rs`: symbol naming.
   - `cache.rs`: format version bump. Existing caches become invalid once.
   - `map.rs`: add the `id` field, change the meaning of `body` and `iface` (revision of ADR-0020).
   - `workspace.rs`·`mcp.rs`: `rename`·`expect`.
   - Tests: `tests/incremental.rs` (audit), `fmt_fuzz` (invariant added), `mcp_fuzz`.
   - `--emit-smt` becomes harder for a person to read because the symbols are hashes. The legend file covers this.
   - Solver stability (V7) has to be measured again.
7. **Approvals needed.**
   - Superseding the accepted ADR-0019 §2 and the key rows of ADR-0032·0043 (AUTONOMY §4).
   - Changing the meaning of the map's `body` and `iface` (ADR-0020). It is 0.x, so there is no backward compatibility obligation (JUDGMENT §6).
8. **Workarounds.** None.

## Consequences

**Proposed tasks** (IDs are assigned in the backlog)

| Task | Content | Done check |
| --- | --- | --- |
| T1-ID-01 Identity hash | Nameless printer (references are `id`, components are `#self.k`, bound names are positions, fields and variants are positions). `id` in `bmb hash` | Invariant in `fmt_fuzz`: randomly renaming definitions, parameters, locals and fields leaves every `id` the same, and an edit that changes meaning changes the `id` of that definition |
| T1-ID-02 Keys and proof cache | Incremental key rows without names, obligation positions as node paths, query symbols as `id`, `--emit-smt` legend, cache format version | 0 `with_audit` mismatches (`tests/incremental.rs`), V7 re-measured |
| T1-ID-03 Map and queries | L1 `id`, `duplicates` by identity, rationale binding (together with 2E) | Determinism and layout invariance tests (`tests/map.rs`), alpha-equivalent duplicate corpus |
| T1-ID-04 Transactions | `rename` and `expect` operations. MCP exposure follows the rule that preserves measurement conditions (2D) | `tests/workspace.rs`: every `id` unchanged after `rename`, an `expect` mismatch is rejected |
| T1-ID-05 Measurement | Re-run rename-ripple, replay rename and move edits in the BCB history, input for the H-PAR experiment | Verdict below |

**Measurements that close H-ID**

- **Cost side:** the number of referrers in rename-ripple (bodies, contracts) must become 0. On rename edits the set of definitions the incremental build re-encodes (`build --stats`) and the proof cache misses must also be 0. The renamed definition itself keeps its `id` even though its text changes, so it is not re-encoded.
- **Benefit side:**
  - On an edition of the BCB history with rename and move edits mixed in, compare re-encodings and CPU per commit before and after this ADR.
  - In the H-PAR replay, compare the definition-level conflict rate the same way.
- **Readability side:** map and diagnostic output must be byte-identical to before once the `id` field is removed (names are shown as before).

**Refutation and revert**

- If even one of the refutation conditions holds, this ADR is rejected.
  - After T1-ID-02 a rename still changes the `id` or key of another definition.
  - The nameless key fails to contain all encoding inputs so the audit reports mismatches, and those mismatches cannot be fixed without names.
  - V7 gets worse than the baseline because of hash symbols.
- Reverting is easy. `id` is a value derived from the text and the stored files do not change, so there is no data migration. Revert the printer to qualified paths and bump the cache format version.
