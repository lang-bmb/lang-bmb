# ADR-0160: A status row of `bmb verify` carries escapes and unproved termination

- Status: Accepted (autonomous)
- Date: 2026-11-26
- Related: ADR-0020 §1, §3 (keys added only where they apply; `status` of the map), ADR-0029 §2, §5 (trust levels, the escape keys), ADR-0094 §3 (the mark of unproved termination), ADR-0156 (packages as working material)

## Context

- A package manager that refuses a package with escape hatches, or with termination left unproved, had to run the proof and then `bmb query status`: the status rows of `bmb verify` did not say enough.
- ADR-0029 defined the escape keys (`trust`, `escapes`, `assumed_obligations`, ...) for the L1 rows of the map and the rows of `query status`, and for the status line of `bmb verify` only `"trust"`, when the level is not `proved`; the line of a `proved` definition was to stay as it was. A definition whose proof does not need its `assume` is `proved` (pass A of §2; `bmb lint` reports the `assume` as W1002), so its line was a plain `verified` line.
- ADR-0094 §3 put `"termination":"unproved"` on the L1 and `status` rows of the map; the status line of `bmb verify` had no such key.
- So both were gaps in what the status line is defined to carry, not departures of the code from the records.
- A definition with no obligations is `verified` on its status line and `none` in the map. Both are defined: the status line gives the verdict of the gate (`verified` is "passed the build gate", ADR-0029 §5; chapter 07 of the reference), the map's `status` says whether there was anything to prove (ADR-0020 §3).

## Decision

1. **`"escapes":n`** is on the status line of every definition that has escapes, `n` counted as in the map (its `assume` statements, and one more for `#[external_body]`), whatever its trust level and verdict.
2. **`"termination":"unproved"`** is on the status line of every definition the map marks so (ADR-0094 §3): one with a `loop` without `decreases`, and every definition that calls one, transitively. The map and the status lines take the set from one function over the checker's call graph.
3. Both keys appear only when they apply, after `trust` and before `backstop`. The line of a definition without escapes and with its termination proved is byte-identical to what it was.
4. So one run decides: a program is proved without trusted facts and with termination proved when every status line is `verified` and none has `trust`, `escapes` or `termination`.
5. A definition with no obligations stays `verified` with `obligations` 0 on its status line. The two outputs answer different questions, and the line already carries the count.
6. The `verify` and `write` tools of the MCP server return these lines as they are; their descriptions (`tools/list`) are unchanged. The compact view still folds every `verified` line into the count, as it did with `trust`.

## Rationale

| Option | Verdict |
| --- | --- |
| **Add the two keys where they apply (adopted)** | One run is enough, and the lines of ordinary definitions keep their bytes (the corpora's expected outputs are unchanged) |
| Leave it to `bmb query status` after the proof | Two runs of the whole program, and the second reads proof results that the first had in hand |
| Every key on every line (`"trust":"proved","escapes":0`) | Changes every status line of every program and corpus for no information |
| Make the two `status` values agree (`none` or `verified` in both) | Either changes a defined output that other tools read; the line says `obligations` already |
