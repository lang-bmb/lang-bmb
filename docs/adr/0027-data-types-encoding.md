# ADR-0027: Proof encoding of data types

- Status: Accepted (approved by the maintainer 2026-10-24). Implementation is at the start of 2B (after 1E-07). The built-in variant notation of the §6 counterexample (`Option::Some(992)`) is superseded by ADR-0026 (bare name `Some(992)`)
- Date: 2026-10-24
- Related: DESIGN.md principles 4·6·9, §8.2 (counterexamples), ADR-0017·0018·0023·0026, backlog 2B-02, experiment `dev-docs/experiments/2B-02/`

## Context
To put the structs and enums of ADR-0026 through the proof gate, the SMT encoding must be decided (ROADMAP 2B criterion 1). There are two candidates.

- **(a) SMT datatypes:** `declare-datatypes`. A struct is one constructor, an enum is one constructor per variant, an array of structs is `(Array Int S)`.
- **(b) Field decomposition:** a struct is a vector of scalar terms, an enum is a tag `Int` plus the fields of all variants, an array of structs is one `(Array Int Int)` per field (SoA).

The current verifier (`compiler/src/verify.rs`) carries values as `Val::S` (a scalar term) and `Val::A` (an element array plus a length), assignment is term substitution, and only merge terms longer than 200 characters are bound to a fresh constant (`merge`, 1B-26). For every obligation it does `(reset)` and sends the base again (ADR-0018).

## Experiment
`python3 dev-docs/experiments/2B-02/run.py` (standard library only, about 7 seconds). A hand-designed generator produces single-obligation queries and runs them on the pinned Z3 4.15.4 with the same options as BMB (`produce-models`, `smt.random_seed`·`sat.random_seed`, `rlimit 5000000`), for seeds 0·1·2 and a repeat of seed 0. The raw data is `results.csv`, the tables are `results.md`. 109 (query, encoding) groups, 436 runs.

| Group | Content | Size |
| --- | --- | --- |
| S1 | Upper bound on every field after n struct field updates | fields k=4·16, n=8·32·128 |
| E1 | `match` on k variants, payload sum per arm | k=2·8·32 |
| E2 | `Option<i64>` chain (checked increment) n times | n=8·32·128 |
| E3 | Rebuild the same value with `match`, then `r == e` | k=2·8·32 |
| E4 | Opaque `spec fn p(E)`: `p(e1) ∧ e1 == e2 ⇒ p(e2)` | k=2·8·32 |
| A1 | Array of structs, loop invariant `∀j<i. a[j].f0 ≤ a[j].f1` preserved | k=2·8·32 |
| A2 | Frame of `a[i].f0 = v` (other elements and other fields unchanged) | k=2·8·32 |
| A3 | Quantified invariant of an `Option` array preserved | — |

Proof variants (unsat expected) and bug variants (sat expected, counterexample) were run together. Encodings: `adt`, `flat` (a canonical form that pins the inactive fields of an enum to 0), `flatj` (E4 only, no canonical form), `hybrid` (S1 only: parameters are datatype constants, the symbolic state is a field vector whose leaves are selectors).

### Results (`rlimit` at seed 0)
| Query | adt | flat | hybrid |
| --- | --- | --- | --- |
| S1 k=4, n=128 proof | 18,666 | 842 | 1,798 |
| S1 k=16, n=128 proof | 49,086 | 1,346 | 1,442 |
| E1 k=32 proof | 46,894 | 80,467 | |
| E2 n=128 proof | 42,894 | 91,361 | |
| E3 k=32 proof | 20,824 | 74,481 | |
| E4 k=32 | 240 | 59,919 (`flatj`: **sat**, 26,503) | |
| A1 k=32 proof | 7,618 | 6,229 | |
| A2 k=32 proof | 5,057 | 1,759 | |
| A3 proof | 2,491 | 2,573 | |

- **Verdict:** every query except `flatj` got the expected verdict, and the verdicts were the same at seeds 0·1·2. The repeat of seed 0 had the same verdict, `rlimit` and model text.
- **Headroom:** the most expensive query is 91,695, which is 1.8% of `rlimit`. At this size neither side is near the limit.
- **Seed variation:** S1 had the same value regardless of seed. In the enum groups `rlimit` differed by seed. The widest spread is 23% for `flat` (E1 k=32 proof, 69,759~85,532) and 16% for `adt` (E1 k=32 bug, 46,095~53,614).
- **E4 `flatj`:** decomposing without a canonical form lets two semantically equal values differ in inactive fields (garbage values), so the congruence of an opaque predicate cannot be used. A false counterexample appears (`e1.V1.0 = -9223372036854775810`, garbage outside the `i64` range). It is not a soundness problem, but completeness is lost. `flat` must assert inactive field = 0 at every entry and construction point, and that cost is most of the `flat` cost of E1·E3·E4.
- **Counterexample readability (raw Z3 `get-value`):** with `adt` one value is one term: `(Some 992)`, `(|E::V1| 0 (- 9223372036854754570))`, `(P 1000 0 712 217)`. `flat` needs k terms for one struct, and a tag plus the field terms of all variants for one enum, filtered by the tag: `(|x.some| true) (|x.v| 992)`. In both cases the compiler can reassemble Rust notation (the encoder knows the field names). For enums, `adt` needs almost no reassembly.

### Interpretation
- The `adt` cost of S1 comes from **building a new datatype value at every update**. `hybrid`, which uses the same datatype sort but carries the symbolic state as a field vector, is almost the same as `flat` (1,442 versus 1,346). So what separates the two for structs is not the sort but the representation of the symbolic state.
- For arrays of structs, field decomposition does not touch the arrays of fields that are not written (A2: `flat` is one `store`, `adt` rebuilds the whole element).
- For enums, field decomposition pays the cost of handling inactive fields (canonical form assertions, per-tag equality). Datatypes handle discrimination, equality and congruence in the solver's theory.

## Decision (proposed)
**Decompose product types, and use datatypes for sum types.**

1. **Struct = field decomposition.** Add `Val::Struct` (a vector of `Val` in field name order). A parameter gets a fresh constant and range facts per field (ADR-0017 §3, asserted at the point of use). A field write changes only that field's term. Spec `==` is the conjunction of field-wise equalities. A struct argument of a `spec fn` is expanded into its leaf fields (congruence holds as is: a struct has no inactive fields).
2. **Array of structs = one array per field.** Each leaf field has the `elems` of a `Val::A`, and the length is shared. The existing array encoding (`store`·`select`, element range facts) is used as is. The length upper bound is computed from the element size (the sum of the field sizes) (ADR-0017 §3).
3. **Enum (including `Option`·`Result`) = monomorphic datatype.** One per instance (`|Option<i64>|`, constructor `|Option<i64>::Some|`). A struct in a payload is expanded into leaf fields, which become constructor fields. If a struct field is an enum, that field is a leaf of datatype sort. An array of enums is `(Array Int E)`.
4. **`match` = testers.** For each arm, push the path condition `((_ is C) e)` and bind the payload with selectors. The obligations of an arm are generated under that path condition. Values are merged with a tester `ite` (the existing `merge` and binding rule).
5. **Range facts:** the integer ranges of a payload are guarded by the tester and asserted at the point of use (`(=> ((_ is C) e) range)`).
6. **Counterexamples:** reassembled into Rust notation. A struct becomes `Point { x: 1, y: 2 }` from its field values, and for an enum the constructor name in the `get-value` result is turned into `Option::Some(992)`.
7. **Trust base:** no new auxiliary fact is added. The datatype axioms are a built-in Z3 theory, and field decomposition is a translation by definition (no new fact that falls under the machine proof rule of ADR-0018 §2). The soundness fuzzer (2B-09) checks combinations of the two encodings against the interpreter.

## Rationale
- **Cost:** in struct update chains decomposition is 6~36 times cheaper (S1 proof, 22~36 times at n=128), and for a field write in an array of structs it is 2.9 times cheaper (A2 k=32). For enums, datatypes are 1.3~3.6 times cheaper (E1·E3 at all sizes, E2 n=32·128. As an exception, at E2 n=8 decomposition was cheaper, 2,455 versus 3,294). Congruence is a constant cost with datatypes (E4: 240 versus 59,919). In A1·A3 the two encodings were within 25% of each other.
- **Completeness:** for a decomposed enum, congruence holds only if the encoder keeps the canonical form invariant at every point (E4 `flatj`). Missing one produces a false counterexample (sound, but it misleads the agent: it harms principle 6 "point out the mistake"). Datatypes do not need this invariant.
- **Determinism and stability:** with both encodings the verdict did not change with the seed, and repeated runs at the same seed had the same verdict, `rlimit` and model. This criterion does not separate the two.
- **Implementation:** struct decomposition reuses the existing `Val::S`·`Val::A` machinery. Datatypes are needed only for enums, so declaration generation is narrow.
- **Ahead:** recursive enums (lists and trees, in 2G together with the heap) can only be expressed as datatypes. If enums are datatypes from the start, the encoding does not change then.

Alternatives considered: (a) All datatypes: struct update and field write cost (S1·A2), and no benefit other than parameter range facts turning into selector terms. (b) All decomposition: the canonical form burden and cost for enums (E1~E4). `hybrid` (structs are also of datatype sort at boundaries, a field vector inside) was similar to decomposition in S1. If `spec fn` arguments growing by the number of fields and enlarging queries becomes a problem, move to this.

## Consequences
- 2B-05 implements this encoding (after the maintainer's approval). `declare-datatypes` enters the `--emit-smt` script, and query text determinism (1B criterion 4) is kept by emitting declarations in path order.
- The ground instance retry (ADR-0023) must also handle predicate applications that contain datatype terms. This is checked in 2B-05.
- Limits of this experiment: the queries are hand-designed encodings, not compiler output. They are small, all below 2% of the limit. Non-linear arithmetic, quantified axioms whose triggers land on selectors, and nested data types were not covered. After 2B-05 it is measured again with the corpus and `stability` (1B-19)·`rlimit_max` (1B-15).
