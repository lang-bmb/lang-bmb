# Cycle 3207: M11-A Phase 2 — bool scan function semantic postconditions
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3206 Carry-Forward — M11-A Phase 2, bool trivials 배치 처리.
방향: `pos`-based 스캔 함수들에 `not it or pos < X.len()` / `not it or pos < end` 패턴 적용.

## Scope & Implementation

### 적용 함수 (7개, semantic_duplication 회피 후 net)

**그룹 대표 함수 (중복 없이 적용)**:

| 함수 | new post |
|------|---------|
| `mn_has_memory_op(ir, pos)` | `not it or pos < ir.len()` |
| `ipr_has_store(ir, pos, end)` | `not it or pos < end` |
| `check_fn_in_list(fn_name, list, pos)` | `not it or pos < list.len()` |
| `layer_is_leaf(entries, pos, name)` | `not it or pos < entries.len()` |
| `lint_has_upper(name, pos)` | `not it or pos < name.len()` |
| `check_closure_marker(varname, str_sb, n)` | `not it or n > 0` |
| `check_arg_flag(argc, idx, flag)` | `not it or idx < argc` |

**semantic_duplication 경고로 인해 복원 (9개)**:
- `mn_has_write_op`, `has_user_call_in_body` — same sig/pre as `mn_has_memory_op`
- `ipr_has_mem_op`, `ipr_has_div_rem`, `inline_has_back_edge` — same sig/pre as `ipr_has_store`
- `check_array_fn_in_list`, `check_f64_array_fn_in_list`, `index_has_name_search` — same as `check_fn_in_list`
- `cl_has_name` — same sig/pre as `layer_is_leaf`

**발견**: `semantic_duplication` lint는 sig+pre+post 조합이 동일한 함수 쌍을 경고.
같은 스캔 패턴을 공유하는 함수군에서 대표 1개만 semantic post 가능.

### 의미 정리

- `not it or pos < X.len()`: "true 반환 시 pos가 범위 내였음" — 재귀 스캔 함수의 귀납적 보장
- `not it or pos < end`: "true 반환 시 end 경계 안이었음" — end-bounded 스캔 함수
- `not it or n > 0`: "`check_closure_marker`가 true면 n이 양수였음" — n-step 역방향 스캔
- `not it or idx < argc`: "인수 플래그 발견 시 idx가 argc 내였음"

## Verification & Defect Resolution

### lint

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
```

### Z3 verify

```
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto: post verification failed  (pre-existing)
```

신규 실패 없음 ✅

### cargo test

```
3800 passed; 0 failed ✅
```

### trivials 추적

| 종류 | Cycle 3206 후 | Cycle 3207 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 47 | 40 | -7 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 288 | 288 | 0 |
| **합계** | **342** | **335** | **-7** |

총 누적: 358 → 335 (−23, 6.4%)

## Reflection

**Scope fit**: 7개 semantic 교체 완료. semantic_duplication 제약으로 9개 복원 → 예상보다 적은 수.

**Latent defects**: `semantic_duplication` 경고 자체는 lint 기능이 올바르게 작동하는 것.
같은 스캔 패턴을 공유하는 함수 쌍에서는 "그룹 대표"만 교체 가능 — 이 제약이 명확해짐.

**Structural improvement opportunities**:
- `is_field_f64_by_index_at`, `check_field_f64_at_index`, `dce_var_used_after`, `ube_has_target_at`,
  `cfeval_has_side_effects`, `trl_scan_block_for_return`, `licm_check_args` — 아직 교체 안 됨
- 이들은 각자 다른 sig를 가지므로 semantic_duplication 위험 없이 교체 가능
- `callers_calls_contain`: pre `pos >= 0`, body `if calls == "" { false }` → `not it or calls.len() > 0`
- `variant_has_bracket`: pre `pos >= 0` — 분석 필요

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 계속 진행. 다음 배치에서 남은 bool 22개 + String 288개 처리 필요.

## Carry-Forward

- **Actionable**: M11-A Phase 3 — 나머지 bool 22개 분석 및 적용
  - 명확한 대상: `is_field_f64_by_index_at`, `check_field_f64_at_index`, `dce_var_used_after`,
    `ube_has_target_at`, `cfeval_has_side_effects`, `trl_scan_block_for_return`, `licm_check_args`,
    `callers_calls_contain` (8개)
  - 분석 필요: 나머지 14개 (`variant_has_bracket`, `check_param_name_match`, etc.)
- **Structural Improvement Proposals**:
  - `semantic_duplication` 그룹 내 secondary 함수 처리 전략 확립: 같은 패턴이면 skip이 맞음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: 없음
- **Next Recommendation**: M11-A Phase 3 — remaining bool 22개 중 명확한 8개 먼저 처리
