# ADR-0037: Mandatory self-hosting bootstrap and language maturity phases

- Status: Accepted (the maintainer's instruction 2026-10-25)
- Date: 2026-10-25
- Related: ADR-0012 (own compiler; supersedes "self-hosting is decided after the semantics are stable"), ADR-0034 (mission goals), genesis §3 (v0: the cost of parity between two compilers)

## Context
ADR-0012 deferred self-hosting to "decided after the semantics are stable". The maintainer's instruction (2026-10-25): BMB **must pass every stage a language has to go through, including the bootstrap (first and second stage).** v0 kept a Rust compiler and a self-hosting compiler together and paid a large cost matching parity on every feature commit (genesis §3.5). A procedure is needed that makes it mandatory without repeating that failure.

## Decision
1. **Self-hosting is mandatory.** The goal of phase 4 is not "decide" but "achieve".
2. **Bootstrap definition (fixed point):**
   - stage0: the current Rust compiler (`compiler/`).
   - stage1: the compiler written in BMB, built with stage0.
   - stage2: the same source built with stage1.
   - stage3: the same source built with stage2.
   - **Pass:** the stage2 and stage3 binaries (or IR) are byte-identical, and stage2 gives the same output as stage0 on the whole corpus, benchmarks and fuzzers (differential test). CI checks on every commit.
3. **The two compilers are not grown together (the lesson of v0):**
   - During the porting period no language feature is added to stage0. The feature freeze is declared at the start of porting, and after that stage0 changes are only bug fixes and the minimal changes the bootstrap needs.
   - Porting goes per component (lexer → parser → checker → prover → code generation → map and tools), and each component moves on only after a differential test against stage0 confirms identical output.
   - After the fixed point is reached, stage0 is frozen and preserved as the "bootstrap seed" (tag and binary), and development is on the single BMB compiler.
4. **Self-proof:** the compiler written in BMB must pass its own proof gate. Parts that use escape hatches (ADR-0029 proposal) are counted and reported as unproved area, and an upper bound on that area is put in the phase pass criteria.
5. **Language maturity phases:** besides self-hosting, the stages a language must have (specification and conformance tests, standard library, system interface, stability and version policy, reproducible build and distribution, multiple targets, debug information, documentation) are sub-phases of ROADMAP phases 4·5.
6. **Relation to phase 2:** the features needed to write the compiler in BMB (strings and bytes, heap collections, data types, error handling, files and processes) become scope and priority input for the phase 2 language sub-phases (gap analysis 4A-01). The 2R syntax decisions are finished before porting.

## Rationale
- Compiling and proving itself is the strongest demonstration of the mission "AI can handle large code in proved form". A compiler is a real program of tens of thousands of lines (a real test of the phase 3 scale hypothesis).
- The fixed point and the differential test are measurable pass criteria (genesis §3.5: frozen and done only when the criteria pass).
- The feature freeze and per-component porting directly prevent v0's cost of dual maintenance.

## Consequences
- Phase 4 depends on the phase 2 language sub-phases (2B·2C·2G etc.). The order is decided by 1E-07 and the 4A-01 gap analysis together.
- From the stage0 feature freeze on, language extension happens only on the BMB compiler side.
- Reverting needs an ADR that supersedes this one (the maintainer's approval).
