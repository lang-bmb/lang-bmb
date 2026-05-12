# Recursive Function with Mutable Parameters Codegen Issue

**Status: CLOSED (Cycle 2735 — v0.98 재현 불가)**
**Priority: MEDIUM**
**Category: Compiler Bug**

## Resolution (Cycle 2735)

heapify 재현 코드 (`test_heap_recurse.bmb`)로 v0.98 재시도:
- 입력 `[5,3,8,1,9,2,7,4,6,0]`에 `heapify(v, 10, 0)` 호출
- 예상 출력: v[0]=8, v[2]=7 (root swap with v[2]=8, 그 후 v[2]가 v[6]=7과 swap)
- **5/5 deterministic**: 정확히 `8\n7\n` 출력

v0.51.22 era에서 발견된 "garbage values" 버그는 v0.98 codegen 변경 (Cycles 2286-2305 이후 ~440 commit)으로 해소된 것으로 추정. Cycle 2735 추가 검증 없이 close.

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-03-26 (bmb-ai-bench 실험) |
| `stale_after` | 2026-09-26 (6개월) |
| `measurement_source` | bmb-ai-bench Cycle 2275-2282 |
| `observed_rate` | unknown (단일 문제 발견) |
| `scope` | `vec_set` + 재귀 호출 패턴 |
| `env_hash` | v0.51.22 era (1년 stale — v0.98 재현 미확정) |

**v0.98 재검증 필요**: `vec_set` 관련 codegen 변경 가능성. 단일 cycle 재현 권고.

## Summary
Recursive functions that modify parameters via vec_set and then recurse produce garbage output in compiled binary. The interpreter is correct.

## Reproduction
```bmb
fn heapify(v: i64, n: i64, root: i64) -> i64 = {
    let mut largest = root;
    // ... find largest child ...
    if largest != root {
        // swap
        let _r = heapify(v, n, largest);  // recursive call
        ()
    } else { () };
    0
};
```

Compiled output: garbage values (e.g., `-8070449075963733112`).

## Workaround
Convert recursive functions to iterative using while loops:
```bmb
fn sift_down(v: i64, n: i64, start: i64) -> i64 = {
    let mut root = start;
    let mut done = 0;
    while done == 0 {
        // ... find largest, swap, update root ...
    };
    0
};
```

## Impact
- Heap sort had to be rewritten as iterative
- Any recursive function with vec mutations may be affected
- Not blocking bootstrapping (bootstrap compiler uses different patterns)

## Root Cause (Suspected)
Stack frame management for recursive calls may not preserve vec handle correctly, or LLVM optimization of recursive + side-effect patterns causes issues.

## Acceptance Criteria
- [ ] Minimal reproduction case
- [ ] Root cause identified
- [ ] Fix or documentation in BMB Reference

## Context
Discovered during AI-Bench problem 86_heap_sort creation (Cycle 2286-2305).
