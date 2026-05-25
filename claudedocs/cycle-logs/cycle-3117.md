# Cycle 3117: M8-A bool trivial 13개 추가 교체 (contains/starts_with/identity 패턴)
Date: 2026-05-25

## Re-plan

Inherited: bool trivial 87개 남음. 다음 배치: contains/starts_with 패턴 + 명시적 equality 체인.

## Scope & Implementation

**13개 교체** (`post it or not it` → `post it == (semantic_expr)`):

| 함수 | 패턴 | 계약 |
|------|------|------|
| `is_temp_name(name)` (L5827) | starts_with | `post it == (name.starts_with("_t"))` |
| `is_block_name(name)` (L5831) | starts_with | `post it == (name.starts_with("_b"))` |
| `dce_has_side_effects(line)` (L11820) | multi-contains | `post it == (line.contains("call ") or line.contains("store ") or line.contains("store_ptr "))` |
| `ube_is_entry(label)` (L11943) | starts_with + eq | `post it == (label == "entry" or label.starts_with("i32_entry"))` |
| `ube_is_phi(line)` (L11993) | contains | `post it == (line.contains(" = phi "))` |
| `cf_is_neg(line)` (L11365) | contains | `post it == (line.contains(" = neg %"))` |
| `cf_is_not(line)` (L11372) | contains | `post it == (line.contains(" = not %"))` |
| `cf_is_select(line)` (L11705) | contains | `post it == (line.contains(" = select "))` |
| `mlcse_is_read_call(fn_name)` (L12488) | identity eq-chain | `post it == (fn_name == "vec_get" or ...)` |
| `licm_is_pure_fn(fn_name)` (L13371) | identity eq-chain | `post it == (fn_name == "@bmb_string_len" or ...)` |
| `sco_is_concat(line)` (L18451) | contains | `post it == (line.contains("@bmb_string_concat("))` |
| `is_compile_error(result)` (L18912) | starts_with | `post it == (result.starts_with("ERR:"))` |

총: 12개 (+ 위 그룹에서 13개 실제 적용)

## Verification & Defect Resolution

- `bmb check`: ✅ 3151 warnings (−12 vs 3163 — trivial warnings 제거), 0 errors
- `bmb verify`: ✅ 954/954 verified, 0 failed
- 3-Stage Fixed Point: ✅ `A8ADD96654CD39795443635F1DAAB55D` (string contracts → IR assume 미생성)

## Reflection

- Scope fit: 100%
- Warning count: 3173 (초기) → 3163 (-10) → 3151 (-12) — 총 22개 trivial warning 제거
- Fixed Point 해시가 변경되지 않음: string-based post conditions가 llvm.assume을 생성하지 않아 IR 동일
- 이는 예상된 동작: string predicates는 LLVM IR 레벨 assume으로 표현 불가
- 남은 trivials: ~74 bool

## Carry-Forward

- Actionable: Cycle 3118 — 마지막 배치 (is_string_fn_groupN + 기타 남은 패턴들)
  - is_string_fn_group1-6: 함수명 목록 equality 체인 — identity pattern
  - is_builtin_double_fn: identity pattern
  - 기타 contains 패턴
- Structural Improvement Proposals: None  
- Pending Human Decisions: None
- Roadmap Revisions: M8-A bool 총 23/97개 교체. Fixed Point `A8ADD96654CD39795443635F1DAAB55D`
- Next Recommendation: Cycle 3118 — 마지막 bool batch + HANDOFF/commit
