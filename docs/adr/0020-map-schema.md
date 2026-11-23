# ADR-0020: Map JSON schema L0–L2

- Status: Accepted (autonomous, AUTONOMY.md §4)
- Date: 2026-10-24
- Related: DESIGN.md principles 2·7, §7.1, ADR-0008·0016·0019, backlog 1D-04

## Context
The standard navigation maps L0–L2 are fixed in the specification, and later changes are allowed only if backward compatible (ADR-0008). The format is fixed so that third-party tools can depend on it.

## Decision

### 1. Format
- Maps and query outputs are JSONL. One line is one row. Keys are written in the order of the tables below, and rows are sorted by `path` (or `from`, `to`). The same code gives the same bytes.
- The first line of every map output is the header row `{"schema":"bmb.map","version":1}`.
- **Backward compatibility rule:** keys can be added, and new `kind` values can be added. The name, meaning and type of an existing key are not changed, and keys are not removed. A change that breaks this raises `version` and is made with a new ADR.
- A definition is referred to by its full path (`unit::name`). The location is `file` and `line` (from 1), and it changes when the file layout changes. Identity is carried by the path and the hash.

### 2. L0 table of contents
| Key | Value |
| --- | --- |
| `level` | `"L0"` |
| `kind` | `"unit"` or `"def"` |
| `path` | Full path of the unit or definition |
| `summary` | One-line summary of the unit (`unit` only) |
| `units`, `defs` | Lists of the paths of the direct sub-units and definitions (`unit` only) |
| `def_kind` | `"fn"` or `"spec fn"` (`def` only) |
| `hash` | First 12 characters of the body hash (`def` only, ADR-0019) |

### 3. L1 surface
One row per definition.

| Key | Value |
| --- | --- |
| `level`, `path`, `def_kind` | Same as L0 |
| `signature` | The normal form signature, one line |
| `requires`, `ensures` | Lists of the normal form texts of the contract clauses |
| `decreases` | Normal form text or `null` |
| `effects` | `"pure"`·`"io"`, `"spec"` for a `spec fn` |
| `uses` | List of the full paths of the `spec fn`s it unfolds |
| `status` | `"verified"`, `"failed"`, `"none"` (no obligations) |
| `obligations`, `failed` | Number of obligations, number of failures |
| `body`, `iface` | Full hashes (ADR-0019) |
| `file`, `line` | Location of the definition name |

### 4. L2 relations
One row per edge. Every edge connects resolved full paths.

| `kind` | Meaning |
| --- | --- |
| `calls` | A call in executable code (including the builtin `print_i64`) |
| `contract` | A `requires`·`ensures`·`decreases`·invariant·`assert` of `from` uses the predicate `to` |
| `uses` | `from` unfolds `to` with `#[uses]` |
| `spec_calls` | A call in a `spec fn` body |
| `writes` | `from` passes a `&mut` argument to `to` (the argument position in `arg`) |

Keys: `level` (`"L2"`), `from`, `to`, `kind`, and `arg` only for `writes`.

### 5. Queries (the 7 of ADR-0016)
They are run as `bmb query <query> [args] <file>...`, and the results follow the row formats above.

| Query | Result |
| --- | --- |
| `units` | All of L0 |
| `surface DEF` | The L1 row of DEF |
| `edges DEF [out\|in\|both] [depth]` | The L2 edges starting from DEF (default `out`, depth 1). Transitive up to the depth |
| `slice DEF` | `{"path","code","deps":[{"path","text"}],"bytes","approx_tokens"}`. `code` is the normal form of DEF, `deps` is the contract surface of the definitions DEF calls and the definitions of the `spec fn`s it unfolds |
| `find NAME` | The L0 rows of the definitions whose last name segment or full path is NAME |
| `contracts PRED` | `{"pred","required_by","ensured_by","uses_by"}` |
| `status` | A `{"path","status","obligations","failed","effects"}` row per definition and a summary row `{"summary":true,"defs","verified","failed","io_defs"}` |

`approx_tokens` is the approximation `ceil(bytes / 4)`. The exact token unit is fixed in the measurement procedure ADR (1E-01).

## Rationale
- Row-based JSONL is easy to read in part and discard, and it extends just by adding rows.
- The map does not contain code (only `slice` returns the normal form, as an exception) and points by location and hash (DESIGN.md §7.1).

## Consequences
- Filling in the `status` of L1 requires running the proofs. The cost of deriving the map includes the cost of proof. The proof cache (stage 2) reduces it.
