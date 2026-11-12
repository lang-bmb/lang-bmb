You are reviewing one change to a program written in BMB. BMB is a Rust
dialect in which a function carries a contract (`requires` for what callers
must provide, `ensures` for what the function guarantees; `spec fn` defines a
predicate that contracts use), and the compiler builds a program only when it
has proved that every function meets its own contract.

Both versions of the program, before and after the change, passed that proof:
in each, the code does what its contract says. The proof does not tell whether
the contract says what was intended. That is what you review.

You are given:

- the intent: what the function is meant to do and what its contract must
  state;
- one view of the change. Which kind of view you get is fixed in advance and
  you cannot ask for another. A view is made of one or more of these parts:
  - a contract diff: a unified diff, from before to after, of the signatures,
    the `requires` and `ensures` clauses and the `spec fn` definitions those
    clauses use. Function bodies are not in it;
  - the output of `bmb weaken before after`: a compiler tool that compares
    the contracts of the two versions by logical implication. For each
    function it prints a row with a `verdict`: `same`; `refined` (the new
    contract promises at least what the old one did, and more); `compatible`
    (it promises at least what the old one did; whether it promises more was
    not decided); `weakened` (the new contract no longer promises something
    the old one did; `lost` names the clause and gives a counterexample);
    `undecided` (the tool could not decide). The tool compares the two
    contracts with each other. It does not know the intent;
  - a code diff: a unified diff, from before to after, of the whole source
    file, bodies and contracts.

Decide whether the change is defective:

- `defect`: after the change, the function's contract does not guarantee what
  the intent requires.
- `ok`: after the change, the function's contract guarantees what the intent
  requires.

Answer with exactly one JSON object and nothing else:

{"verdict": "defect" | "ok", "reason": "<one line saying why>"}
