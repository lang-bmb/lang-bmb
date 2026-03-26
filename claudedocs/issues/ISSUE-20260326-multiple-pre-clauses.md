# Multiple `pre` Clauses Not Supported (Parser Limitation)

**Status: OPEN**
**Priority: LOW**
**Category: Language Design / Parser**

## Summary
BMB only supports a single `pre` clause per function. Multiple `pre` lines cause a parser error. Users must combine with `and`.

## Reproduction
```bmb
// FAILS: parser error
fn safe_get(v: i64, idx: i64, len: i64) -> i64
    pre idx >= 0
    pre idx < len
= vec_get(v, idx);

// WORKS: combined with 'and'
fn safe_get(v: i64, idx: i64, len: i64) -> i64
    pre idx >= 0 and idx < len
= vec_get(v, idx);
```

Error: `Unrecognized token 'pre' found... Expected one of "post", "and", "or", ...`

## Impact
- Low — `and` combinator is a clean workaround
- LLM-generated code sometimes uses multiple `pre` lines (natural pattern)
- 3 AI-Bench contract problems had to be fixed

## Proposed Fix
Either:
1. **Parser**: Allow multiple `pre` clauses, desugar to `and` conjunction
2. **Documentation**: Clearly state single `pre` + `and` pattern in reference

## Acceptance Criteria
- [ ] Either parser support or clear documentation
- [ ] BMB Reference updated with correct contract syntax

## Context
Discovered during AI-Bench problem 96-98 creation (Cycles 2286-2305).
