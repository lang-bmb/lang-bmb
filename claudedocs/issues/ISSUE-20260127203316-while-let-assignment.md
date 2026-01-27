# While Loop Body: Let Binding + Assignment Limitation

## Summary
Cannot use `let` bindings followed by assignments in while loop body due to BMB's let-body syntax.

## Reproduction

```bmb
// This FAILS
fn test() -> i64 = {
    let mut y = 0;
    while y < 10 { {
        let tmp = foo();
        y = y + tmp;    // ERROR: expected `;` or `}`
        y
    } };
    y
};

// This WORKS
fn test() -> i64 = {
    let mut y = 0;
    while y < 10 { {
        y = y + foo();
        y
    } };
    y
};
```

## Root Cause
In BMB, `let x = v; body` means `let x = v in { body }`. The semicolon after a let binding starts its body expression, not a new statement.

When parsing `let tmp = foo(); y = y + tmp;`:
1. `let tmp = foo()` parses
2. `;` starts the body
3. `y = y + tmp` is expected as an expression, but `=` is unexpected

## Impact
- Makes while loops harder to use with temporary variables
- Forces workarounds like inlining expressions or using tuple returns
- Benchmark code (lexer) cannot easily use while loops

## Proposed Solutions

### Option A: Statement-oriented block syntax (Breaking Change)
Add C-style blocks where statements are truly sequential:
```bmb
while cond {
    let tmp = foo();
    y = y + tmp;
}
```

### Option B: Sequential let syntax
Allow `let!` or similar for statement-style let:
```bmb
while cond { {
    let! tmp = foo();
    y = y + tmp;
    y
} }
```

### Option C: Document the pattern
Recommend using tuple returns or inlining:
```bmb
// Pattern: inline instead of temporary
while cond { {
    y = y + foo();
    y
} }
```

## Priority
Medium - Affects code ergonomics but has workarounds

## Labels
- language-spec
- ergonomics
- while-loop
