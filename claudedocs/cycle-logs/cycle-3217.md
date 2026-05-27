# Cycle 3217: M11-A Phase 5h — bool 업그레이드 (1개) + semantic_duplication 제약 분석
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3216 Carry-Forward — bool 27개 마지막 분석.
**전략**: bool `post it or not it` → `post not it or pos < X.len()` 패턴.

## Scope & Implementation

### 적용 함수 (1개)

| 함수 | pre | 근거 |
|------|-----|------|
| `has_param_ref_in_ir` | 없음 | `pos >= s.len() → false` 기반; `s` 변수명이 post text에서 고유 → 충돌 없음 |

### semantic_duplication 제약 분석

bool 업그레이드의 핵심 제약: `semantic_duplication` lint는 **파라미터 타입 + pre text + post text** 를 비교.
post expression 내 변수명이 같으면 충돌 발생 (파라미터 이름이 달라도).

| 시도 함수 | 충돌 대상 | 이유 |
|----------|----------|------|
| `mn_has_write_op(ir, pos)` | `mn_has_memory_op` | 동일 파라미터 타입 `(String, i64)`, 동일 pre `pos >= 0`, 동일 post text `not it or pos < ir.len()` |
| `check_var_list(name, list, pos)` | `check_fn_in_list` | list 파라미터명 같음, post text `not it or pos < list.len()` 동일 |
| `check_array_fn_in_list(fn_name, list, pos)` | `check_fn_in_list` / `check_var_list` | 동일 |
| `index_has_name_search(list, name, pos)` | `check_fn_in_list` | 동일 |
| `cl_has_name(entries, pos, target)` | `layer_is_leaf` | `entries` 파라미터명 + post text 동일 |
| `ipr_has_mem_op(ir, pos, end)` | `ipr_has_store` | `(String, i64, i64)` 타입 + `end` 변수명 동일 |
| `ipr_has_div_rem(ir, pos, end)` | `ipr_has_store` | 동일 |
| `inline_has_back_edge(ir, pos, end)` | `ipr_has_store` | 동일 |

### 건너뛴 패턴들

| 범주 | 대표 함수 | 이유 |
|------|-----------|------|
| "all" 패턴 | `ipr_all_calls_pure`, `licm_check_args`, `layer_all_callees_leaf` | base case returns `true` → post 불가 |
| 분석 함수 (no pos) | `enum_has_payload`, `is_float_expr`, `is_pure_expr`, `is_var_unused_in_ir` | 자연스러운 semantic post 없음 |
| `index_is_decl_start` | TK_EOF() 시 `true` 반환 | `not it or pos < src.len()` 불성립 |

### 주요 발견: semantic_duplication 규칙

```
충돌 조건: param_types_match AND pre_text == pre_text AND post_text == post_text
충돌 탈출: post expression 내 변수명이 다르면 text 불일치 → 충돌 없음
예시:
  not it or pos < ir.len()  ← mn_has_memory_op  (ir)
  not it or pos < s.len()   ← has_param_ref_in_ir (s) ✅ 충돌 없음
```

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

2390 tests passed ✅

### trivials 추적

| 종류 | Cycle 3216 후 | Cycle 3217 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | **26** | **-1** |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 230 | 230 | 0 |
| **합계** | **264** | **263** | **-1** |
| String `post it.len() >= 1` | 156 | 156 | 0 |

**누적 진척**: 358 → 263 (-95, 26.5%)

## Reflection

**Scope fit**: 1개 업그레이드. semantic_duplication 제약이 예상보다 강함.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- M11-A 수익 체감 심화: bool 26개 중 대부분이 `semantic_duplication`으로 차단
- "all" 패턴 bool 함수들은 구조적으로 `post not it or pos < X.len()` 불가
- i64 7개 trivials 분석 시도 고려 (마지막 남은 영역)

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 26.5% 달성. 수익 체감으로 전략 전환 필요.

## Carry-Forward

- **Actionable**: 전략 평가 — M11-A vs 언어 갭 작업(M11-C) 전환
  - bool 26개: semantic_duplication 장벽, 대부분 충돌
  - i64 7개: `post it == it` → 의미 있는 계약 가능성 탐색
  - 언어 갭 작업이 더 높은 임팩트 가능
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→263 (-95, 26.5%)
- **Next Recommendation**: i64 7개 탐색 OR 언어 갭 작업(M11-C) 전환
