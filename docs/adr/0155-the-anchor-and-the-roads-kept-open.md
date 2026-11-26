# ADR-0155: The anchor of the design, and the roads kept open

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0142 (the model of the language), ADR-0143 (the surface as a means), ADR-0145 (the basis of collaboration and deployment), ADR-0040 (outside the guarantee), DESIGN.md §1, §3, §9, ROADMAP.md

## Context

- ADR-0142 says a form is justified by the way its writer works. It does not say what the language is for in the long run, so a decision has nothing to be held against except the present measurement.
- The tools of today (text files, line merges, one linear history, build then run, stop to change) are serialized around the limits of human authors. Some of that serialization is made by tools and can be removed; some is made by meaning (a change of a contract, of the shape of state, an effect that cannot be undone, a decision of intent) and can only be reduced and made explicit.
- Several present choices could close roads the long-run picture needs, without anyone having decided to close them.

## Decision

1. **The anchor.** The design is held against this picture: many agents work on large systems at once; what they change is proved; changes take effect without stopping; several versions live together; a coordinating agent operates the whole. Programming, a language and a runtime are still needed there, and they do not follow the conventions built around human authors.
2. **Four horizons.** Each opens only when the one before it has met its criterion.
   - **H1, one proved program:** an agent writes and changes one program under the proof gate; the compiler builds and proves itself. This is the present roadmap up to layer 9.
   - **H2, change decided by proof:** several agents change one codebase without branches; merges and compatibility are decided by proof; a package's interface is its contracts.
   - **H3, change while running:** a replacement that keeps its contract is applied without a restart; several proved implementations of one contract compete; the shape of state migrates under a proved relation.
   - **H4, systems:** the boundary of the language passes one program: contracts between systems, deployment and configuration as programs, effects that cannot be undone, and authority.
   H2 to H4 are hypotheses. None is measured with a model.
3. **What is built stands on measurement.** The anchor guides decisions; it licenses no machinery. The language fixes relations (commutation, refinement, compatibility) and leaves servers and runtimes until their horizon opens (ADR-0145).
4. **Principle 1 stays.** Code is the only source of truth. One sentence is added: the identity of a contract and the identity of an implementation are kept apart. Making the contract the source of truth is not decided: written specifications reject 67.7% of hidden mutants against a target of 90%.
5. **"Outside the guarantee" is the present boundary.** Failures of configuration and deployment are outside what BMB guarantees today; DESIGN.md says so as the present boundary and not as a permanent one.
6. **Three things are not fixed without a reason.** A new decision that would fix one of them states why:
   - one definition per path, as the only answer of the index from names to definitions;
   - cache keys that read a type as its whole declaration;
   - a list of effects that cannot grow.

## Rationale

- Without an anchor, each decision is judged only by what is measurable now, and what is measurable now is the single task, where an unlearned language is at its weakest.
- Stating the horizons with their order keeps the anchor from pulling work ahead of evidence: the economic premise is not yet shown for one program.
- The three roads of decision 6 are where present structures already lean one way. Each is cheap to keep open and expensive to reopen.
- Alternative: rewrite principle 1 now. Rejected until the strength of specifications is shown.
- Alternative: leave the picture in working notes only. Rejected: a reader of the public design could not tell which restrictions are positions and which are the present state.
