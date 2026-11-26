# ADR-0156: Packages are built as working material before gate E

- Status: Accepted
- Date: 2026-11-26
- Related: ROADMAP.md rules ("The ecosystem comes last"), gate E, ADR-0111, ADR-0154, ADR-0155

## Context

- The rule "the ecosystem comes last" keeps packages and bindings closed until the claims they rest on are shown by measurement.
- Development and measurement keep needing programs to compile, prove and change. So far many were invented for the purpose: generated programs, tasks of one function. Invented material is thrown away, and at the size of one function the comparison of proof with tests did not discriminate in four attempts.
- Programs that will be needed anyway exist: the small libraries every ecosystem has (encodings, checksums, parsers, version comparison), most of them with published test vectors or test suites.

## Decision

1. **New test and measurement material is real packages.** When development or an experiment needs a program, it is built as an official package in the `bmb-packages` repository instead of being invented. Sets that are already frozen stay as they are.
2. **A package's source of judgement is its origin's tests.** A package ported from an existing library carries that library's test vectors or suite as the judge, with the origin's licence and attribution.
3. **A Rust to BMB translator is built along the way.** Its scope grows with the packages that need it. It translates code and leaves contracts to be written; what it cannot translate it names.
4. **The package manager (`gotgan`, the role `cargo` has for Rust) gets only what this needs:** it reads a package's declaration, resolves its dependencies, and runs check, proof, build and tests of a package as one command.
5. **Gate E is unchanged.** Declaring packages usable by others, bindings and a registry service still wait for the measurements gate E names. Until then the repositories say that they are working material.

## Rationale

- Material that is used afterwards is cheaper than material that is discarded, and real libraries have judges nobody on the project wrote.
- The rule was about what is promised to others, not about what is built. Keeping gate E as it is keeps the promise where it was.
- Alternative: keep inventing material until gate E. Rejected for the reasons above.
- Alternative: open the ecosystem now. Rejected: the claims it rests on are not shown.
