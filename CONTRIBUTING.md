# Contributing to BMB

BMB is at version 0.x and its design is still being measured. Read this page first: some kinds of change are closed on purpose, and knowing which saves you work.

## What is open and what is not

| Kind of change | State |
| --- | --- |
| A defect: the compiler accepts, rejects, proves or compiles something against [`docs/DESIGN.md`](docs/DESIGN.md) or the [reference](docs/reference/) | Open. An issue with a small program that shows it is the most useful contribution |
| A soundness defect: a wrong program is reported as proved | Open, and urgent. See [`SECURITY.md`](SECURITY.md) |
| Documentation that is wrong or unclear | Open |
| A change to the language surface | Closed. The surface is frozen at `0.1.0` until the compiler builds itself; a change needs a decision record ([`docs/adr/`](docs/adr/)) and the maintainer's approval |
| A new feature in the Rust compiler | Closed except what the self-hosting port needs. The Rust compiler is being replaced by its BMB port in `selfhost/` |
| Packages, bindings, editor support | Closed by design until the claims they rest on are measured ([`docs/ROADMAP.md`](docs/ROADMAP.md), gate E) |

If you are unsure, open an issue before writing code.

## Building and checking

```sh
scripts/setup.sh      # installs the pinned Rust toolchain, Z3 and LLVM (scripts/toolchain.env)
cargo build
scripts/check.sh      # the whole gate; judge it by its exit code
```

`scripts/check.sh` is the one gate: formatting, lints, the tests, the proofs of the standard library and of the port, the differential tests between the Rust compiler and its port, and the fixed point. CI runs the same script. The benchmarks and the self-checks of their harnesses are in the [`bmb-bench`](https://github.com/lang-bmb/bmb-bench) repository, whose own check runs against a checkout of this one. It takes tens of minutes on four cores. Every night CI also runs the fuzzers at length and the port's differential cases too long for the gate (the port verifying, dumping and emitting itself, and the wider input sets: `tests/port/README.md`); `scripts/port_nightly.sh` runs those cases locally (about 3 hours on four cores). While you work, `scripts/fresh.sh` (seconds) tells you which generated file went stale, and `scripts/regen.sh` rewrites them.

A change is ready when the gate passes on it. Do not skip or weaken a check to get there: a check that is wrong is itself a defect to report.

## Rules a change must keep

- **Nothing unproved.** A definition that is not proved does not enter the corpus, the standard library or the port. `assume` and `#[external_body]` are counted and need a stated reason.
- **The design document follows the code.** A change of behavior changes [`docs/DESIGN.md`](docs/DESIGN.md), the reference and [`compiler/README.md`](compiler/README.md) in the same commit. `docs/` always describes the present state.
- **A decision gets a record.** A design decision is one file in [`docs/adr/`](docs/adr/), in the shape its README gives. An accepted record is not edited; a later one supersedes it.
- **Claims come with their measurement.** A statement about what BMB does for agents is written only with a code path or a measurement behind it, and [`docs/CLAIMS.md`](docs/CLAIMS.md) lists it with its state, refutations included.
- **The port follows the Rust compiler.** A component whose port passed the differential test is pinned: a change to its Rust files comes with the same change to the port (`scripts/components.py`).
- **The repository reads on its own.** Comments and documents say the reason in place; they do not point at trackers or documents outside this repository (`scripts/standalone.py`).

## Commits and pull requests

- Write commit messages in English, in the imperative or as a statement of what changed, with the reason in the body when it is not obvious.
- Keep one concern per pull request. Say what changed, why, and how you checked it.
- History on `main` is not rewritten.

## License

By contributing you agree that your contribution is dual licensed under Apache-2.0 and MIT, as the [README](README.md#license) states.
