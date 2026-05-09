# BMB AI Output Schema (v1)

> **Audience**: AI agents (LLMs, code assistants) and tooling that consume BMB compiler output.
> **Stability**: v1 — additive changes only (new fields, new types). Breaking changes require v2 namespace.
> **Default mode**: machine/JSON. `--human` flag switches to colored ariadne output.
> **Anchor**: CLAUDE.md Rule 8, `docs/superpowers/specs/2026-05-01-vision-v1.0-realignment.md` § 4.2 Track M.

---

## 1. Format conventions

- **One event per line** (JSONL — no pretty printing in default machine mode).
- All events have a `type` discriminator at the top level: `{"type": "<event_kind>", ...}`.
- Numeric fields are integers unless explicitly float.
- Path fields use forward slashes (`/`), even on Windows.
- `_schema` (optional, future): `"bmb.v1"` — reserved for explicit version tagging.

### Event flow per command

| Command | Typical event sequence |
|---------|----------------------|
| `bmb check <file>` | 0..N `error`/`warning` → `success` (if no errors) |
| `bmb build <file>` | 0..N `error`/`warning` → `build_success` (terminal) |
| `bmb run <file>` | program stdout (program-defined) → 0..1 `error` |
| `bmb test <file>` | 0..N `test_fail` → 1 `test_result` |
| `bmb bench <file>` | 0..N `bench`/`bench_fail` → 1 `bench_result` |
| `bmb bench --compare a b` | 0..N `compare` → 1 `compare_result` |
| `bmb fmt <file>` | 0..N `fmt_needed`/`fmt_formatted` → 1 `fmt_result` |
| `bmb lint <file>` | 0..N `warning` per file + 1 `lint`/file → 1 `lint_summary` |
| `bmb verify <file>` | 0..1 `verify_skip` → 1 `verify_result` |

---

## 2. Event types

### 2.1 Diagnostic events

#### `error`
Compilation error. Terminates the affected command path. Multiple errors may be reported per check.

**Fields**:
- `type` (string, const `"error"`)
- `message` (string) — human-readable description
- `kind` (string, optional) — error category, e.g. `"parse"`, `"type_check"`, `"resolve"`, `"verify"`
- `file` (string, optional) — source file path (forward slashes)
- `start`, `end` (integer, optional) — byte offsets in source
- `line`, `col` (integer, optional) — 1-indexed
- `suggestion` (string, optional) — fix hint

**Example**:
```json
{"type":"error","kind":"type_check","file":"src/foo.bmb","start":120,"end":130,"line":5,"col":10,"message":"Expected i64, found String","suggestion":"Add explicit conversion: x.parse()"}
```

#### `warning`
Non-blocking diagnostic. Always carries position info.

**Fields**: `type`, `kind`, `file`, `start`, `end`, `line`, `col`, `message`.

**Example**:
```json
{"type":"warning","kind":"unused_var","file":"src/foo.bmb","start":40,"end":42,"line":2,"col":8,"message":"Unused variable: x"}
```

#### `hint`
Informational nudge for the user/agent. Often paired with errors.

**Fields**: `type`, `message`.

**Example**:
```json
{"type":"hint","message":"Fix this error and re-run check — more errors may be found after parsing succeeds."}
```

---

### 2.2 Build events

#### `build_success`
Successful native/object/IR build. Terminal event of `bmb build`.

**Fields** (variants depending on output type):
- `type` (const `"build_success"`)
- `output` (string) — output file path
- `size` (integer, optional) — output size in bytes
- `target` (string, optional) — target triple, e.g. `"x86_64-unknown-linux-gnu"`
- `functions`, `structs`, `extern_fns` (integer, optional) — symbol counts (emit-ir mode)

**Example**:
```json
{"type":"build_success","output":"target/foo.exe","target":"x86_64-pc-windows-gnu","size":1234567}
```

---

### 2.3 Check / Lint events

#### `success`
Terminal event of `bmb check`. Reports total warning count.

**Fields**: `type`, `file`, `warnings` (integer).

#### `lint`
Per-file lint result during multi-file lint.

**Fields**: `type`, `file`, `warnings`.

#### `lint_summary`
Terminal event of `bmb lint`.

**Fields**: `type`, `files`, `warnings`, `errors` (all integer).

---

### 2.4 Verify events (Z3 contract verification)

#### `verify_skip`
Z3 not available. Verification bypassed.

**Fields**: `type`, `reason` (e.g. `"z3_not_found"`), `hint`.

#### `verify_result`
Terminal event of `bmb verify`.

**Fields**: `type`, `total`, `verified`, `failed` (all integer counts).

---

### 2.5 Test events

#### `test_fail`
Per-test failure during `bmb test`.

**Fields**: `type`, `name`, `file`, `reason`, `ms` (integer, optional — execution time).

#### `test_result`
Terminal event of `bmb test`.

**Fields**: `type`, `tests`, `passed`, `failed`, `ms` (integer, optional).

---

### 2.6 Bench events

#### `bench`
Per-benchmark result.

**Fields**:
- `type` (const `"bench"`)
- `name` (string) — bench function name
- `file` (string)
- `samples`, `warmup` (integer)
- `min_ns`, `median_ns`, `p99_ns`, `stddev_ns` (integer, nanoseconds)
- `mean_ns` (float, nanoseconds)
- `mode` (string, optional) — `"interp"` (default) or `"native"`

**Example**:
```json
{"type":"bench","name":"sort","file":"benches/sort.bmb","samples":1000,"warmup":100,"min_ns":12345,"median_ns":12500,"p99_ns":13000,"mean_ns":12567.4,"stddev_ns":50}
```

#### `bench_fail`
Bench couldn't execute or no samples.

**Fields**: `type`, `name`, `reason`.

#### `bench_native_build_fail`, `bench_native_exec_fail`
Native bench harness build/exec failure.

**Fields**: `type`, `file`, `stdout`/`stderr`.

#### `bench_result`
Terminal event of `bmb bench`.

**Fields**: `type`, `benches` (integer count), `mode` (optional).

---

### 2.7 Compare events (regression detection)

#### `compare`
Per-bench comparison result.

**Fields**:
- `type` (const `"compare"`)
- `name` (string)
- `baseline_median_ns`, `current_median_ns` (integer)
- `delta_pct` (float) — percent change ((current - baseline) / baseline × 100)
- `status` (string) — `"OK"`, `"REGRESSION"`, `"IMPROVEMENT"`, `"MISSING"`, `"NEW"`

#### `compare_result`
Terminal event of `bmb bench --compare`.

**Fields**: `type`, `ok`, `regressions`, `improvements`, `missing`, `new` (integer counts), `threshold_pct` (float).

---

### 2.8 Format events

#### `fmt_needed`
File requires reformat (in `--check` mode).

**Fields**: `type`, `file`, `first_diff` (string, optional — line number or excerpt).

#### `fmt_formatted`
File was reformatted (in default mode).

**Fields**: `type`, `file`.

#### `fmt_result`
Terminal event of `bmb fmt`.

**Fields**: `type`, `files` (integer).

---

## 3. AST/IR dump events

### `bmb parse <file>` — ✅ implemented (Cycle 2558)

Outputs the full AST as JSON. Unlike other commands, this is **not** wrapped in a `{"type":...}` envelope — the AST object itself is the output.

**Format selection** (`--format` / `-f`):

| Value | Output |
|-------|--------|
| `compact` (default) | Single-line JSON (Rule 8: machine-friendly) |
| `json` | Alias for `compact` (backward compat) |
| `pretty` | Multi-line indented JSON |
| `sexpr` | S-expression tree |
| *(with `--human`)* | Overrides to pretty regardless of --format |

**Note**: The `type` discriminator wrapper (`{"type":"ast_dump","ast":...}`) is deferred — direct AST JSON is more token-efficient for downstream tooling.

---

## 4. Stability guarantees

### 4.1 v1 stable types

Listed in §§ 2.1–2.8. AI agents may rely on:
- `type` discriminator presence
- Field names of documented fields
- Field types (string vs integer vs float)
- Mandatory fields not disappearing

### 4.2 v1 additive changes (allowed without version bump)

- New optional fields on existing types
- New `type` values (agents must handle unknown types gracefully)
- New `kind` values within `error`/`warning` (e.g. `"new_kind"`)

### 4.3 v2 breaking changes (require namespace bump)

- Renaming or removing existing mandatory fields
- Changing field types
- Changing event flow ordering for a command

---

## 5. Recommended AI agent integration

```python
# Example: parse BMB JSONL output
import json
import subprocess

result = subprocess.run(
    ["bmb", "check", "src/main.bmb"],
    capture_output=True, text=True
)

events = []
for line in result.stdout.splitlines():
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        event = json.loads(line)
        events.append(event)
    except json.JSONDecodeError:
        pass

errors = [e for e in events if e["type"] == "error"]
warnings = [e for e in events if e["type"] == "warning"]
```

### Best practices

- **Tolerate unknown types**: filter by `event["type"] in known_types`, log unknowns.
- **Treat positions as advisory**: not all errors carry `start`/`end`/`line`/`col`.
- **Don't parse stderr**: `Warning: Z3 solver not available` etc. are stderr-only chatter, not events.
- **Stream incrementally**: each line is independently valid JSON; no need to wait for end.

---

## 6. Current implementation references

| Event type | Source location |
|-----------|----------------|
| `error` | `bmb/src/error/mod.rs:942`, `bmb/src/main.rs:539,1039,1057` |
| `warning` | `bmb/src/error/mod.rs:966` |
| `build_success` | `bmb/src/main.rs:747,830,890,947` |
| `hint` | `bmb/src/main.rs:1114` |
| `success`, `lint`, `lint_summary` | `bmb/src/main.rs:1217,1336,1450` |
| `verify_skip`, `verify_result` | `bmb/src/main.rs:1494,1569` |
| `test_*` | `bmb/src/main.rs:1644,1712,1722,1747` |
| `bench`, `bench_*` | `bmb/src/main.rs:1775,1848,1879,1894,1928,2015,2032,2061,2097,2118` |
| `compare`, `compare_result` | `bmb/src/main.rs:2194,2208` |
| `fmt_*` | `bmb/src/main.rs:2459,2507,2518` |

---

## 7. Migration path (v0.x → v1 stable)

This document is the v1 baseline. Prior to v1 freeze, additions or refinements may occur without notice. Once v1 freezes (current expectation: M2 complete), this schema becomes a stability guarantee.

**Track M completion criteria** (M2 게이트):
- [x] AI_OUTPUT_SCHEMA.md — ✅ this document (Cycle 2516, Phase 1)
- [x] dump-ast `--format compact|pretty` — ✅ Cycle 2558 (`bmb parse --format compact|pretty|sexpr|json`)
- [x] 회귀 테스트: `bmb parse` default → compact JSON (single-line, valid JSON) — ✅ Cycle 2559
- [ ] CI gate (선택) — deferred (low priority)

---

**작성**: 2026-05-01 (Cycle 2516)
**갱신**: 2026-05-09 (Cycles 2558-2559 — dump-ast format + Track M closure)
**Anchor**: CLAUDE.md Rule 8, ROADMAP § "Vision v1.0 Framework" Track M
