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

## ag_accept.bmb — machine-int, now sound under bitvectors (T-BV, Cycle 3582)

**Verdicts superseded by T-BV.** The C3576 abstain-guard emitted `unsupported`
for `*`/`/`/`%` because SMT modelled Int as unbounded while i64 wraps. T-BV models
i64 as signed `(_ BitVec 64)`, so wrap is sound and the multiplicative ops are
re-enabled with genuine verdicts — and the previously-idealized `+`/`-` are now
sound too. This fixture now pins the *sound* behaviour: every `verified` holds
under i64 wrap, and genuine overflow is `refuted` with the exact witness.
(The fixture's in-file header comment still narrates the old C3576 expectation —
this table is the authoritative oracle.)

| function | expected verdict | why (under i64 wrap) |
|----------|------------------|----------------------|
| ag_mul   | refuted  | `x*2` overflows; witness x=2^62 (#x4000000000000000) → it=i64::MIN |
| ag_div   | verified | x≥0 ⇒ x/2 ≥ 0, no overflow (bvsdiv) |
| ag_mod   | verified | x≥0 ⇒ x%3 ∈ [0,2], no overflow (bvsrem) |
| ag_add   | refuted  | **the headline soundness fix**: x=i64::MAX ⇒ x+1 wraps to MIN<0 |
| ag_sub   | verified | x≥5 ⇒ x−5 ≥ 0 (now *sound*, no longer idealized) |
| ag_leaf  | verified | no arithmetic |

## cs_accept.bmb — contract-strengthening showcase + counterexample (Cycles 3577 T-CS / 3578 T-CX)

Each pair is a verbatim isomorph of a real compiler.bmb scanner, differing ONLY
in the precondition. `_weak` is refuted (verifier finds the genuine too-weak
contract) and carries a Z3 `counterexample`; adding the indicated `pre`
(`_strong`) verifies. Demonstrates the verifier half of the AI-authored-contract
loop (the contract-authoring step is hand-applied in the probe).

**T-BV exception (Cycle 3582):** under sound i64 wrap the bounds `pre` alone no
longer suffices for two pairs — `scan_char_end_strong` is `refuted` (`pos+2`
overflows at extreme index) and `count_newlines_strong` is
`unsupported_recursion` (`count+1` recursive actual not dischargeable). Those
contracts need a further strengthening (no extreme index / `count ≤ limit`). The
other three pairs (all `pos+1`) still verify. See the per-row notes and the T-BV
section below.

| function              | expected verdict | counterexample pinpoints |
|-----------------------|------------------|--------------------------|
| find_colon_weak       | refuted          | pos=1, len s=0  → missing `pre pos <= s.len()` |
| find_colon_strong     | verified         | `pos+1` (k=1) cannot overflow when pos≤len≤i64::MAX |
| scan_char_end_weak    | refuted          | pos=1, len s=0  → missing `pre pos <= s.len()` |
| scan_char_end_strong  | refuted (T-BV)   | `pos+2` overflows at pos≈2^63; adding `pre pos<=s.len()` is NOT enough under sound BV (len is only `len≥0`-bounded) — see T-BV note |
| skip_sp_tab_weak      | refuted          | pos=1, limit=0  → missing `pre pos <= limit` |
| skip_sp_tab_strong    | verified         | — |
| count_newlines_weak   | refuted          | count=-1        → missing `pre count >= 0` |
| count_newlines_strong | unsupported_recursion (T-BV) | `count+1` recursive actual can overflow ⇒ pre-obligation `count+1≥0` not dischargeable; contract is overflow-incomplete (needs e.g. `count ≤ limit`) |
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

**T-BV broadens this caveat.** Under sound i64 wrap, a recursive result is only
known to satisfy its own (often weak) post — e.g. `it ≥ 0`. The IH can therefore
take the value i64::MAX, so a body like `delta + IH` overflows and the function
is `refuted` with a witness where the recursive result is MAX (unreachable for a
real, short string). Such refutes are *sound* (the contract as-written genuinely
cannot prove overflow-freedom) but the witness is practically unreachable — the
fix is a stronger post (`it ≤ s.len()`), i.e. the AI-authored-contract loop.

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
| count_down       | verified | basic IH, path-sensitive pre-obligation (`pos-1`, no overflow) |
| accum            | unsupported_recursion (T-BV) | `acc+1` recursive actual can overflow ⇒ pre-obligation `acc+1≥0` not dischargeable. accum IS genuinely overflow-buggy (accum(1,MAX) wraps) → sound abstain. Two-formal substitution still runs (then discharge fails). |
| accum_bad        | unsupported_recursion (T-BV) | same `acc+1` discharge failure masks the base-case refute (coarse, like C3576 parse_len_acc). `bad_base` below still pins "IH no rubber-stamp" via a genuine base-case refute. |
| rec_sum          | refuted (T-BV) | `n + rec_sum(n-1)` overflows for large n (witness n=i64::MAX) — genuine reachable overflow, like cf_pow2. |
| down_to_zero     | verified | `!=` guard variety (`n-1`, no overflow) |
| bad_base         | refuted  | base case `it>=1` false (witness n=0) |
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

## T-BV: signed-bitvector migration (Cycle 3582) — sound i64 wrap

i64 is now modelled as signed `(_ BitVec 64)` instead of unbounded `Int`, so
wrap is faithful. This is the *proper* fix that subsumes the C3576 machine-int
abstain-guard (mul/div/mod) and resolves the previously-documented `+`/`-`
overflow latent. The byte/`len` axiom is grounded per String param (decidable
QF_UFBV) rather than quantified (the `forall` over BV hung Z3); a `-t:10000`
soft Z3 timeout is the backstop.

**Soundness invariant (the point):** after T-BV no `verified` rests on
overflow-idealized arithmetic. Verified count moving *down* is not a regression
when the composition is sound.

**div/mod boundary (checked):** `bvsdiv`/`bvsrem` are *total* (div-by-zero and
`INT_MIN÷-1` return defined values, not traps). The only verified functions that
contain div/mod are the three `unsupported→verified` cases below; all three
divide by a callee that pins a nonzero constant (`tok_pack_mul()==5000000`,
`block_id_mul()==1000000`), so the divisor is provably positive — no div-by-zero,
no `INT_MIN÷-1`. (For their `it≥0`/`it<1000000` posts a zero divisor would in any
case *refute* — `bvsdiv(r,0)=-1`, `bvsrem(p,0)=p` violate the post — not
spuriously verify; the safe direction.) Variable divisors not provably-nonzero
would need a guard before claiming verified; none exist in the corpus today.

Real-corpus delta (baseline `372b7c83` exe+source vs T-BV, `verify-vc
bootstrap/compiler.bmb`), 17 functions changed:

- **verified 154 → 150**, **refuted 17 → 26**, **unsupported_recursion 8 → 10**,
  **unknown 0 → 1** (rest `unsupported`).
- **8 machine-int abstain → genuine** (C3576's `unsupported` resolved): cf_pow2 /
  pack_int_tok / make_tok / pack_ids → refuted (genuine overflow); tok_val /
  unpack_temp / unpack_block → **verified** (BV *proves* the bit-ops
  overflow-safe — precision recovery the guard could not give); tok_end →
  unknown — its body `r-(r/M)*M` has a `bvmul` that QF_UFBV can't decide in
  10s, so the `-t:10000` backstop *fires* (measured 10.1s). This `unknown` is
  therefore timeout-driven and machine-dependent: the lone `unknown 0→1` in the
  histogram is not perfectly reproducible on slower hardware (it would still be
  `unknown`, just via the same timeout). No `unknown` is pinned in any fixture,
  so the T-REG harness stays deterministic regardless.
- **2 lost-refute recovered**: cf_log2, **parse_len_acc** (unsup_rec → refuted).
  parse_len_acc's genuine refute was *masked* by C3576's coarse mul-abstain;
  T-BV restores it (the documented "genuine refute LOST" is back).
- **3 verified → refuted** (sound, contract too weak to prove overflow-freedom):
  count_commas / count_top_commas (unbounded IH ⇒ result=MAX ⇒ `delta+IH`
  overflows; needs `post it ≤ s.len()`), sim_find_start_rev (pos=i64::MAX
  false-witness; pre lacks `pos ≤ len`).
  **[Resolved live in Cycle 3583 — T-CS-live]** count_commas / count_top_commas
  received `pre pos≤s.len()` + `post it ≤ s.len()-pos` in `compiler.bmb`; both
  flipped back to `verified` (corpus verified 150→152, refuted 26→24), all
  callers discharge `pos≤s.len()` (every external caller passes `pos=0`), S3==S4
  FIXED_POINT_OK. sim_find_start_rev (extreme-index false-witness) remains refuted
  — its fix is option-B (bounded-length axiom), deferred. See cslive_accept.bmb.
- **4 verified → unsupported_recursion** (recursive accumulator, sound abstain):
  count_line_at / find_separator / trl_count_chars / count_string_bytes_acc —
  `count+1` recursive actual not dischargeable; contract overflow-incomplete.
  **[2 resolved live in Cycle 3584 — T-CS-live-acc]** trl_count_chars /
  count_string_bytes_acc received `pre pos≤s.len() and count≤pos` in
  `compiler.bmb`; both flipped `unsupported_recursion → verified` (corpus verified
  152→154, unsup_rec 10→8), all callers pass `(_,0,0)`, S3==S4 FIXED_POINT_OK. The
  other two stay abstained as the **option-B** boundary: count_line_at's `line+1`
  is 1-indexed (`line≤cur+1` ⇒ `line+1≤src.len()+1` overflows at MAX) and
  find_separator's guard computes `pos+2` (overflows at extreme index, logic-bound
  not contract-bound). Both need the bounded-length axiom. See accum_accept.bmb.

**Design choice — signed BV only, NO bounded-length axiom (option A).** A
`forall x. len(x) < 2^62` axiom would recover the lone extreme-index false-witness
(scan_char_end-class), but the corpus shows that class is ~1 function, and the
axiom adds a soundness caveat (strings < 4 EB). Not worth it; A is soundness-pure.

The takeaway is thesis-consistent: most non-machine-int losses are the verifier
correctly reporting that a bounds-contract is too weak to guarantee
overflow-freedom (`it ≤ len`, `pos ≤ len`, `count ≤ limit`) — exactly the
strengthening the AI-authored-contract loop is meant to supply.

## cslive_accept.bmb — unbounded-IH overflow, strengthen-and-verify (Cycle 3583, T-CS-live)

Pins the exact class T-BV exposed and Cycle 3583 resolved *live* in
`compiler.bmb`: a recursive counter `1 + f(pos+1)` whose only contract is
`post it >= 0`. Under signed-BV the self-call's IH gives only `it >= 0` (may be
MAX), so `1 + IH` overflows → the post is `refuted` with a MAX false-witness.
The function is correct (return ≤ scanned positions); the contract is too weak.
Adding `pre pos ≤ s.len()` + `post it ≤ s.len()-pos` bounds the IH so
`1 + IH ≤ s.len()-pos ≤ s.len() ≤ 2^63-1` (no wrap) → verifies.

`cc_*` is a verbatim isomorph of live `count_commas`, `ctc_*` of live
`count_top_commas` (the two functions that received this strengthening in
Cycle 3583). Synthetic/forward-only fidelity (the pre/post differ from the live
functions only in name); the live verdict flip is recorded in the T-BV section
above. This demonstrates the AI-authored-contract loop end-to-end: verifier
refutes (overflow gap) → bounded post supplied → same function verifies.

| function   | expected verdict | why |
|------------|------------------|-----|
| cc_weak    | refuted   | `1+cc_weak(pos+1)` overflows; IH=MAX false-witness (only `post it≥0`) |
| cc_strong  | verified  | `post it ≤ s.len()-pos` bounds the IH ⇒ `1+IH ≤ s.len()-pos`, no wrap |
| ctc_weak   | refuted   | same overflow gap with a 3-param depth scanner |
| ctc_strong | verified  | bounded post; depth `+/-` irrelevant to the return-value bound |

## accum_accept.bmb — recursive accumulator overflow, strengthen-and-verify (Cycle 3584, T-CS-live-acc)

The `count+1`/`line+1` accumulator class T-BV exposed as `4 verified →
unsupported_recursion`. Unlike cslive's `1 + f(...)` (RETURN position, bounded by a
post), these accumulate in ARGUMENT position; they are `unsupported_recursion` (not
refuted) because the IH's call-site pre-obligation `count+1 ≥ 0` can't discharge
under signed-BV without an upper bound on the accumulator.

**CLEAN** (tcc / csba): the accumulator counts a subset of scanned positions, so
`count ≤ pos` is inductive; with `pre pos ≤ s.len()` + path-sensitivity
(`pos < s.len()` on the recursive branch), `count ≤ pos < s.len() ⇒ count+1` no
overflow. tcc/csba are verbatim isomorphs of the live functions strengthened in
Cycle 3584 (trl_count_chars / count_string_bytes_acc). **OPTION-B boundary** (cla /
fsep): kept abstained — count_line_at's `line+1` is 1-indexed (the `+1` eats the
headroom `count≤pos` has) and find_separator's guard computes `pos+2` (overflows by
construction); both need the bounded-length axiom (a future cycle).

| function    | expected verdict | why |
|-------------|------------------|-----|
| tcc_weak    | unsupported_recursion | `count+1≥0` pre-obligation undischargeable (count unbounded) |
| tcc_strong  | verified  | `count ≤ pos < s.len()` ⇒ `count+1` no overflow |
| csba_weak   | unsupported_recursion | same, with pos+2/pos+1 branches |
| csba_strong | verified  | `count ≤ pos` inductive across both advance steps |
| cla_weak    | unsupported_recursion | `line+1` accumulator unbounded |
| cla_strong  | unsupported_recursion | even `line ≤ cur+1` ⇒ `line+1 ≤ src.len()+1` overflows at MAX (option-B) |
| fsep_weak   | unsupported_recursion | guard `pos+2` overflows; recursive `pos+1` undischargeable |
| fsep_strong | unsupported_recursion | `pre pos≤s.len()` insufficient — `pos+2` overflows by construction (option-B) |
