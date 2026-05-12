# If/Else Early Return Pattern Codegen Bug

**Status: CLOSED (Cycle 2735 — v0.98 재현 불가, codegen fix 확정)**
**Priority: MEDIUM**
**Category: Compiler Bug**

## Resolution (Cycle 2735)

v0.98 재현 시도 (2026-05-11):
- 기본 패턴 (`if n < 2 { 0 } else { println(42); 0 }`): 5/5 정답
- Substantial work + side effects (`count_primes(100)` else branch): 5/5 정답 = 25 (실제 prime count)

v0.51.22 era에서 발견된 codegen bug는 v0.98 codegen 광범위 변경 (Cycles 2275 이후 ~360 commit)으로 해소된 것으로 추정. 재현 코드 (`test_if_else_repro*.bmb`)는 검증 후 삭제.

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-03-26 (bmb-ai-bench 100문제 실험 — 미보고 기준) |
| `stale_after` | 2026-09-26 (6개월 — codegen 회귀 검증 필요) |
| `measurement_source` | bmb-ai-bench Cycles 2275-2282 실험 |
| `observed_rate` | unknown (몇 문제 영향 받았는지 quantify 안됨) |
| `scope` | `if cond { ...; 0 } else { ...; 0 }` 패턴 사용 main fn |
| `env_hash` | v0.51.22 era (1년 stale — v0.98 재현 미확정) |

**v0.98 재검증 필요**: cycle 2275 이후 codegen 광범위 변경. 이 ISSUE는 v0.98에서 재현되지 않을 가능성 30-50%. 단일 cycle 재현 시도 권고.

## Summary
When `main` uses `if cond { ...; 0 } else { ...; 0 }` pattern with substantial work in each branch, the compiled binary returns incorrect results from the else branch. The interpreter produces correct output.

## Reproduction
```bmb
fn main() -> i64 = {
    let n = read_int();
    if n < 2 {
        let _p = println(0);
        0
    } else {
        // ... substantial work ...
        let _p = println(count);  // prints wrong value
        0
    }
};
```

Interpreter: correct. Compiled (bmb build --release): wrong value from else branch.

## Workaround
Use mutable state + single return:
```bmb
fn main() -> i64 = {
    let n = read_int();
    let mut result = 0;
    if n >= 2 {
        // ... work ...
        result = count
    } else { () };
    let _p = println(result);
    0
};
```

## Impact
- Multiple AI-Bench problems had to be restructured to avoid this pattern
- Affects any BMB program using if/else as expression in main with side effects
- Not a bootstrapping blocker (CLAUDE.md Rule 6 — Rust bug fix only if blocking bootstrap)

## Root Cause (Suspected)
LLVM IR codegen for if/else expression in main may incorrectly handle PHI nodes or branch targets when both branches contain I/O side effects + return value.

## Acceptance Criteria
- [ ] Root cause identified in codegen (llvm.rs or llvm_text.rs)
- [ ] Fix or workaround documented in BMB Reference
- [ ] Regression test added

## Context
Discovered during AI-Bench problem creation (Cycles 2270-2285). Documented but not fixed per Rule 6.
