# ADR-0158: The standard library is in the normal form

- Status: Accepted
- Date: 2026-11-26
- Related: DESIGN.md §3 principle 8, ADR-0019 (normal form and definition hash), ADR-0092 (standard library core), ADR-0117 (definition summaries)

## Context

- The units of the standard library, `lib/std/*.bmb` (11 files, 4,262 lines), were written by hand: each file had 15 to 67 `//` comment lines (402 in all: section headers and explanations), items in reading order and compact one-line bodies. None passed `bmb fmt --check`. The normal form drops comments and puts the items of a unit in name order (ADR-0019).
- Nothing said that the library was outside the normal form, and nothing checked it. The gate ran `bmb fmt --check` only on `selfhost/*.bmb`, the compiler's port (about 60,000 lines), which lives in the normal form and keeps every explanation in `#[summary("..")]`. The generated copy of the library the port embeds, `selfhost/std_units.bmb` (`scripts/std_units.py`), already went through `bmb fmt`.
- The compiler embeds `lib/std/NAME.bmb` (`include_str!` in `compiler/build.rs`) and adds the units a program names (ADR-0092). The embedded text is part of the compiler's identity digest.

## Decision

1. Every file of `lib/std/` is the normal form of the unit it declares.
2. What a comment said that the code and its contracts do not (why a contract has its shape, what a group of definitions is for, algorithm and proof notes, references to RFC 4648, RFC 8259 and FIPS 180-4) is in the `#[summary]` of the definition it explains. A section header became part of the summaries of the definitions of its section, worded so that each still reads on its own in name order; a note inside a body became part of its function's summary; the note before an `impl` block, which takes no attributes, became part of the summaries of the block's functions. A section header that only named the topic of the definitions under it (`search`, `encode`) was dropped.
3. The gate checks it: the job that runs `bmb fmt --check selfhost/*.bmb` also runs `bmb fmt --check lib/std/*.bmb` (`scripts/check.sh`).

## Rationale

- One normal form for all BMB code (principle 8). A reader, a tool and an agent see the library as they see any other program: `bmb fmt` of a program that holds a library file no longer rewrites it, and map L1 carries the explanations of library definitions, which a comment never reached.
- The port already lives this way, and packages are written in the normal form from their start. The library is code a program pulls in by path, and the maps show it with the program
- Meaning is unchanged, measured on `lib/std/*.bmb` as one program before and after: `bmb verify` gives the same rows, status, obligations, `rlimit` and `rlimit_max` for each of its 218 definitions; `bmb dump smt-queries` gives the same 436 rows in the same order, whose scripts differ only in the source offsets of their `;` comment lines; `bmb hash` is the same (the summary is not part of the identity, ADR-0117). The compiler's identity digest changes, as with any edit of the embedded text.
- Alternative: declare the library outside the normal form. Rejected: a second form of BMB code, with no check to keep either one.
- Alternative: make the normal form keep comments. Rejected: that changes the frozen surface and ADR-0019, and comments would enter the text the tools compare without entering the maps; a summary already carries what a definition is for.
- Cost: the normal form writes a long expression or a `match` arm with a block body on one line, and a 64-entry constant table as one array literal, so some lines of the library are long (the longest 774 characters).
