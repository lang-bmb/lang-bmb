# verify-vc acceptance probes (tracked, reproducible)

Tracked R2 `verify-vc` acceptance fixtures. Prior cycles' probes lived only in
the gitignored `probe/` dir and evaporated between sessions, leaving
measurement claims with no reproducible artifact. These are tracked so any
future cycle can re-run them and catch a silent verdict regression.

Run:

```bash
./bootstrap/compiler.exe verify-vc tests/verify-vc/<file>.bmb
```

A full automated harness (parse JSON, diff against the table below, fail on
mismatch) is the derive-next; for now this table is the manual oracle.

## ag_accept.bmb — machine-int abstain-guard (Cycle 3576, scope-(b) mul/div/mod)

The guard abstains on overflow-prone `*`/`/`/`%` (SMT models Int as unbounded,
i64 wraps), keeping `+`/`-`/leaf. Establishes that every surviving `verified`
does not rest on overflow-idealized multiplication/division.

| function | expected verdict |
|----------|------------------|
| ag_mul   | unsupported |
| ag_div   | unsupported |
| ag_mod   | unsupported |
| ag_add   | verified (retained-latent: `+` overflow still idealized — scope-(b)) |
| ag_sub   | verified (retained-latent) |
| ag_leaf  | verified |

## cs_accept.bmb — contract-strengthening showcase + counterexample (Cycles 3577 T-CS / 3578 T-CX)

Each pair is a verbatim isomorph of a real compiler.bmb scanner, differing ONLY
in the precondition. `_weak` is refuted (verifier finds the genuine too-weak
contract) and carries a Z3 `counterexample`; adding the indicated `pre`
(`_strong`) verifies. Demonstrates the verifier half of the AI-authored-contract
loop (the contract-authoring step is hand-applied in the probe).

| function              | expected verdict | counterexample pinpoints |
|-----------------------|------------------|--------------------------|
| find_colon_weak       | refuted          | pos=1, len s=0  → missing `pre pos <= s.len()` |
| find_colon_strong     | verified         | — |
| scan_char_end_weak    | refuted          | pos=1, len s=0  → missing `pre pos <= s.len()` |
| scan_char_end_strong  | verified         | — |
| skip_sp_tab_weak      | refuted          | pos=1, limit=0  → missing `pre pos <= limit` |
| skip_sp_tab_strong    | verified         | — |
| count_newlines_weak   | refuted          | count=-1        → missing `pre count >= 0` |
| count_newlines_strong | verified         | — |
| skip_spaces_weak      | refuted          | pos=-1          → missing `pre pos >= 0` |
| skip_spaces_strong    | verified         | — |

### Counterexample genuineness caveat

A `counterexample` is only as real as the refute under the UF abstraction
(uninterpreted byte `b`, `len` constrained only `>= 0`). The witnesses above are
base-case violations (the refuting path never touches `byte_at`), so they are
genuine real-input bugs. In general, a witness could rely on a byte/len
combination unachievable in an actual string — do NOT treat every
`counterexample` as a guaranteed reachable input. (Same false-witness potential
flagged for the deferred sub-term-guard T-SG.)
