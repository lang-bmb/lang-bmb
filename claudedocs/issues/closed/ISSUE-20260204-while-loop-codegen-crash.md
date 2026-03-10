# While Loop Codegen Crash in regex_redux

**Status: RESOLVED (v0.60.262)**
**Severity: Medium**
**Discovered: 2026-02-04**

## Summary

Converting tail-recursive functions to while loops in regex_redux benchmark causes a segmentation fault during LLVM IR optimization.

## Reproduction

Original (works):
```bmb
fn count_pattern1_loop(buf: i64, len: i64, i: i64, count: i64) -> i64 =
    if i >= len { count }
    else {
        let m1 = if match_agggtaaa(buf, i, len) == 1 { 1 } else { 0 };
        let m2 = if match_tttaccct(buf, i, len) == 1 { 1 } else { 0 };
        count_pattern1_loop(buf, len, i + 1, count + m1 + m2)
    };
```

Converted (crashes):
```bmb
fn count_pattern1(buf: i64, len: i64) -> i64 = {
    let mut i: i64 = 0;
    let mut count: i64 = 0;
    while i < len {
        let m1 = match_agggtaaa(buf, i, len);
        let m2 = match_tttaccct(buf, i, len);
        { count = count + m1 + m2; i = i + 1 }
    };
    count
};
```

## Error Message

```
Warning: opt tool failed (exit: Some(1)): opt.exe: regex_bmb.unopt.bc: error: Invalid record (Producer: 'LLVM21.1.8' Reader: 'LLVM 21.1.8')
Segmentation fault
```

## Analysis

The crash occurs during LLVM IR generation or optimization, not during type checking. The specific pattern that triggers the crash seems to be:
1. While loop with multiple let bindings inside
2. Calling `@inline` functions within the loop
3. Multiple mutations in the loop body block

## Workaround

Use tail recursion instead of while loops for this pattern.

## Impact

- Prevents certain loop optimizations
- May affect other benchmarks or user code with similar patterns

## Files

- `ecosystem/benchmark-bmb/benches/compute/regex_redux/bmb/main.bmb`

## Related

- Parameter narrowing optimization may be involved
- LLVM IR generation in `bmb/src/codegen/llvm.rs`
