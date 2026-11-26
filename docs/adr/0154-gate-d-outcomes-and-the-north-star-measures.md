# ADR-0154: The outcomes registered for gate D, and the measures that judge the economic premise

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0111 (decision gates, one carry-over), ADR-0102 (the unit of judgement), ADR-0149 (gate B carried over to gate D), ADR-0142 (design cost and the cost of being unlearned), DESIGN.md §2 premise E3, CLAIMS.md

## Context

- Premise E3 (a cost paid once at writing time is repaid over the life of the code) has no confirmed claim behind it: 0 confirmed, 4 refuted. Gate B could not judge it and was carried over to gate D (ADR-0149). ADR-0111 allows one carry-over, so gate D is the last judgement.
- The measurements so far were spent mostly on the token cost of one task, where BMB is not distinguishable from Rust (1.07 to 1.30 times, the interval containing 1). The strongest evidence is elsewhere and deterministic: edits that stay local, merges and compatibility decided by proof.
- Every comparison with Rust so far is the sum of the cost of the design and the cost of being unlearned. DESIGN.md §2 forbids judging the design by a difference that was not separated, and no measurement separates them.
- What happens if gate D again cannot judge is not registered.

## Decision

1. **Three measures judge E3 at gate D.**
   - **Cumulative verification cost:** CPU time over a replayed history of 1,000 commits, against Rust with test selection and caching, at equal or better defect detection.
   - **Life-cycle tokens:** total tokens of writing, later changes and defect repair on the compiler twins.
   - **Concurrent work:** several agents committing to one trunk; integration failures and total tokens against Rust with branches and merges.
   The token cost of one task is reported as a trajectory and judges nothing.
2. **An unlearned control arm.** A small task set is also run in Rust with keywords and standard-library names renamed consistently, so that the cost of being unlearned is measured apart from the cost of the design.
3. **Outcomes, registered before the measurement.**
   - **Holds:** the plan goes on.
   - **Refuted, or cannot be judged:** there is no further carry-over. The public documents say that E3 was not shown, and the claims are narrowed to what was: a higher first-attempt success rate than a verified language, a gate that is not gamed, and changes and merges decided by proof.
4. **Gate E in that case.** The condition for opening the ecosystem is no longer that principles 4 and 6 are shown in general; it is the measurement of the third narrowed claim (changes and merges decided by proof).
5. **Model runs are approved by size and cost before they start.** Extracting tasks and freezing harnesses do not wait for that.

## Rationale

- The measures follow the evidence. Where the language's properties are expected to pay (over many changes, at size, under concurrency) is where they are measured; a single task is the place the premise never claimed.
- Registering the narrowed claims before the measurement keeps the outcome from being argued after it. Each of the three stands without cost parity with Rust.
- Alternative: carry the verdict over again. Rejected by ADR-0111.
- Alternative: keep one task as the judging unit. Rejected: ADR-0102 already moved the judgement to the life cycle, and four attempts at the single-function size did not discriminate.
