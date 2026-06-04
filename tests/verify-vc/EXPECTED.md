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

## leaf_accept.bmb — leaf VC-gen (verbatim real predicates + synthetic non-vacuity) (Cycle 3580)

`is_whitespace` .. `is_ident_start` are VERBATIM copies of `bootstrap/compiler.bmb`
leaf/non-recursive-call predicates — their `verified` is a *retroactive* regression
check on real compiler code: the current verdict matches the C3571-documented
verdict (both verified), i.e. no net verdict regression at the endpoints (this is a
verdict comparison, not a source-unchanged or every-point-in-interval claim). The
last three are SYNTHETIC mutations giving leaf non-vacuity (refuted
and unsupported coexist with verified → not rubber-stamping); they guard forward only.

| function             | expected verdict |
|----------------------|------------------|
| is_whitespace        | verified |
| is_digit             | verified |
| is_hex_digit         | verified |
| hex_digit_val        | verified |
| is_alpha             | verified |
| is_alnum_or_underscore | verified |
| is_ident_start       | verified |
| is_digit_bad         | refuted (synthetic: post claims c=58 is a digit → cex c=58) |
| is_hex_digit_badpost | refuted (synthetic: post widens hex bound to 71 → cex c=71) |
| no_pre_caller        | unsupported (synthetic: no caller pre → is_digit's pre undischargeable) |

## rec_accept.bmb — recursive (IH / path-sensitivity / soundness) + trusted-set (Cycle 3580)

All SYNTHETIC (forward-only guards), covering capabilities NOT in cs_accept (which
pins the scanner refute cases). Verdicts are the latest documented state, post
path-sensitivity (C3575): the recursive verified/refuted/abstain mix is the
non-vacuity battery for the IH and interprocedural paths.

| function         | expected verdict | capability |
|------------------|------------------|------------|
| count_down       | verified | basic IH, path-sensitive pre-obligation |
| accum            | verified | two-formal substitution (P2) |
| accum_bad        | refuted  | IH does not rubber-stamp (base case false) |
| rec_sum          | verified | path-sensitive flip (C3575) |
| down_to_zero     | verified | `!=` guard variety |
| bad_base         | refuted  | base case `it>=1` false |
| unsound3         | unsupported_recursion | soundness gate: recursive pre undischargeable |
| leaf_pred        | verified | leaf callee trustworthy |
| trusted_caller   | verified | trusts a leaf-verified callee post |
| untrusted_caller | unsupported | T1 demotes caller of a recursive (non-leaf) callee |

## Why historical per-increment verdicts are not reproducible (finding, Cycle 3580)

The per-increment acceptance probes (p0/o2/p1/t1/p2/ps_accept) lived only in the
gitignored `probe/` dir and evaporated. They cannot be faithfully resurrected with
their *original* verdicts, because **later increments deliberately reclassified
earlier functions** — this is intended evolution, not regression:

- P0 emitted `is_hex_digit -> unsupported` (call boundary); **O2-lite (C3571)
  flipped it to `verified`**.
- P1/P2 turned `unsupported_recursion` into `verified` for recursive scanners;
  **path-sensitivity (C3575) flipped `rec_sum: unsupported_recursion -> verified`**.

So pinning a p0-era verdict on the current compiler would manufacture a false
failure. The honest guard pins the *current* verdict of each capability, cross-checked
against the *latest* documented verdict (not the first). The leaf 7 above are the one
verbatim/retroactive class; everything else is synthetic/forward-only.
