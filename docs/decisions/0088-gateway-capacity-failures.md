# ADR-0088: A gateway capacity failure is not a session result

- Status: Accepted (autonomous, AUTONOMY.md §4: handling of the measurement infrastructure within the scope of ADR-0066·ADR-0021). Supplements the "errors that a retry does not fix" rule of ADR-0066
- Date: 2026-11-04
- Related: ADR-0021 §1·§3, ADR-0066 (gateway characteristics), ADR-0070, backlog INF-23·T0-33·T1-41

## Context

On 2026-11-04 from 06:50 to 07:07 three measurements were run at the same time (qwen `bvb-write` repeats at parallelism 4, qwen `brb-write` repeats at parallelism 1, sonnet T1-41 at parallelism 4). Two kinds of failure occurred.

- **sonnet spend cap:** the gateway returned HTTP 402 `cap_reached` ("spend cap on all its wallets", daily, reset at 2026-11-05 00:00 UTC). It is a 4xx, so the adapter did not retry, and as in ADR-0066 the session ended with `stop: "api_error"` and was judged in the state it was in. Later sessions were recorded as failures after 0 seconds without ever reaching the model. That was `bvb-ast` 44/45, `brb-diff` 22/24, `brb-ast` 24/24, `m1b-ast` 3/25.
- **qwen concurrent request limit:** HTTP 429 "The wallet has 8 requests in flight" and 502 outlasted the retries (about 2 minutes), and one session in repeat 1 of `bvb-write` ended with `api_error`. The cause is that the total of 9 concurrent sessions exceeded the wallet's concurrent request limit of 8 (a measurement operation mistake).

The rule of ADR-0066 assumed a cause on the model side (context overflow, HTTP 400). A capacity failure has nothing to do with the model, the language or the task.

- **The spend cap applies to qwen too (confirmed after 07:09).** The same 402 `cap_reached` came for `openai:qwen3.8-27b` requests ("spend cap on all its wallets", with `required` equal to the prompt size on each request). The harness of this measurement checkout does not have fix 1, so the 20 runs of the qwen repeats (ADR-0086) ended within minutes, mostly as 0-second sessions. "qwen has no repeat cost" in ADR-0066 §2 and the rationale of ADR-0086 is wrong. qwen also uses a share of the gateway's daily cap, so the measurement volume is bound by the daily cap.

## Decision

1. **The spend cap stops the run.** HTTP 402 `cap_reached` is raised as `model_api.CapReached` and no agent catches it. The row of that session is not written and the run ends. When resumed with the same command after the cap is lifted, that session runs once from the start (INF-16: a session cut off before its row was written has no result, so it is not a retry).
2. **The total of concurrent sessions is at most 8.** The sum of `--jobs` of all runs going at the same time on one wallet (gateway credential) is kept at 8 or less (the BRB arm is 1). 429·5xx are retried as transport failures, as now.
3. **Handling of runs before this fix.** A set run with even one session that ended on a capacity failure (402 `cap_reached`, a 429 that outlasted the retries) is voided as a whole and run again at the same commit and under the same condition. Voiding is decided only by a rule fixed before looking at the result (the kind of error message), regardless of success or failure. Voided runs are not kept in the results directory, and the measurement report records the set, the number of sessions and the kind of error. This covers: the five runs above, sonnet `m1b-write` of T0-13 (around 04:00, one exploration task got a 429), and all 20 runs of the qwen repeats from 07:09 to 07:33 (context). Runs of the same day without a capacity-failure session (the rest of T0-13·T0-30 sonnet, T0-14 qwen, T1-41 sonnet `m1b-diff`·`bvb-diff`) are valid.
4. **Plan within the daily cap.** The measurement plan of a day is split by set so that it fits within the cap (reset 00:00 UTC), and when running at a commit without this fix, after each set finishes the kind of error of the `api_error` rows is looked at, and if there is even one 402 the rest of that day is stopped (the run script checks).

## Rationale

- Counting capacity-failure sessions as failures makes the measurement measure the gateway's operational state. Running the same set again at the same commit is not the "task retry" of ADR-0021 §3. It does not select after seeing the model's answer; it discards by rule a run in which the model could not answer.
- **Rejected:** (가) running only the voided sessions again and swapping in the rows. That selects sessions within one run to run again, which does not fit the run-level rule (ADR-0077 §1). (나) retrying 402 too. The cap is daily, so a retry would wait for hours, which is the same as the run standing still in the meantime.

## Consequences

- Code: `CapReached` and `post_json` in `bench/m1/model_api.py`. Test: `test_the_spend_cap_stops_the_run` in `bench/m1/test_model_api.py`.
- Operation: the harness of the measurement checkout (`e5247e8d`) does not have this fix. When running sonnet at that commit, the runs are split by set so as not to exceed the cap, and after they finish the kind of error of the `api_error` rows is checked.
- Reverting: delete the `CapReached` branch and it is ADR-0066 as before.
