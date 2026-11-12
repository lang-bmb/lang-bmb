You are reviewing the contract of one function in a program written in BMB.
BMB is a Rust dialect in which a function carries a contract (`requires` for
what callers must provide, `ensures` for what the function guarantees;
`spec fn` defines a predicate that contracts use), and the compiler builds a
program only when it has proved that every function meets its own contract.

The program passed that proof: the code does what its contract says. The
proof does not tell whether the contract says what was intended. That is what
you review.

You are given:

- the intent: what the function is meant to do and what its contract must
  state;
- one view of the program. Which kind of view you get is fixed in advance and
  you cannot ask for another. A view is made of one or more of these parts:
  - the code: the whole source file, contracts and bodies;
  - the contract: the signatures, the `requires` and `ensures` clauses and the
    `spec fn` definitions those clauses use. Function bodies are not in it;
  - the output of `bmb examples` for the function under review: a compiler
    tool that reads only the function's `requires` and `ensures` (not its
    body) and asks a solver for concrete values. It prints one JSON object per
    line. A row has a `kind`:
    - `allowed`: the `input` satisfies `requires`, and the contract accepts
      `result` for it (for a `&mut` parameter, `after` is its value on
      return). Two `allowed` rows with the same `input` show two different
      results the contract accepts for that input;
    - `rejected`: the `input` satisfies `requires`, and the contract does not
      accept `result` for it;
    - `excluded`: `requires` does not admit the `input` (no result).
    The tool tries a few inputs, small values first, arrays of at most 4
    elements (`max_len`). For each input it shows an `allowed` row, a second `allowed` row
    when the contract accepts another result for the same input, and a
    `rejected` row. The last line is a summary: the number of rows of each
    kind and `undecided`, the number of questions the solver could not answer
    (`rlimit` is the solver effort spent). A missing row, or `undecided` above
    0, means the tool could not produce that example. It does not mean that no
    such example exists. The tool does not know the intent.

Decide whether the contract is defective:

- `defect`: the function's contract does not guarantee what the intent
  requires.
- `ok`: the function's contract guarantees what the intent requires.

Answer with exactly one JSON object and nothing else:

{"verdict": "defect" | "ok", "reason": "<one line saying why>"}
