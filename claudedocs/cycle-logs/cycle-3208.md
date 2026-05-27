# Cycle 3208: M11-A Phase 3 — bool scan/check semantic postconditions
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3207 Carry-Forward — 나머지 bool 22개 중 명확한 함수 처리.
STEP 0: 분석 대상 26개 중 semantic_duplication 제약 및 "all" 함수 제외 후 6개 적용.

## Scope & Implementation

### 적용 함수 (6개)

| 함수 | 시그니처 | new post |
|------|---------|---------|
| `variant_has_bracket` | `(variants, target, pos)` | `not it or pos < variants.len()` |
| `check_param_name_match` | `(name, params, pos)` | `not it or pos < params.len()` |
| `is_dead_zext_range` | `(ir, start, end, dead)` | `not it or start < end` |
| `is_field_f64_by_index` | `(struct_reg, struct_name, idx)` | `not it or struct_reg.len() > 0` |
| `parse_build_fast` | `(argc)` | `not it or argc > 3` |
| `matches_string_pattern` | `(s, pos)` | `not it or pos + 5 < s.len()` |

### 분석: 건너뛴 함수들

| 함수 | 이유 |
|------|------|
| `mn_has_write_op`, `ipr_has_mem_op`, `ipr_has_div_rem`, `inline_has_back_edge`, `check_array_fn_in_list`, `check_f64_array_fn_in_list`, `has_user_call_in_body`, `index_has_name_search`, `cl_has_name` | semantic_duplication (그룹 대표가 이미 있음) |
| `ipr_all_calls_pure`, `ipr_all_calls_readonly`, `licm_check_args`, `layer_all_callees_leaf`, `match_bytes` | "all" 함수 — empty에서 true 반환, `not it or pos < X` 성립 안 함 |
| `is_dead_ptrtoint_range` | is_dead_zext_range와 동일 sig/pre — semantic_duplication 위험 |
| `check_var_list` | check_fn_in_list와 동일 sig/pre/post(list.len()) — semantic_duplication |
| `call_has_two_args`, `call_has_one_arg` | arg count 분석 — 동일 sig 쌍, 의미있는 post 어려움 |
| `index_is_decl_start` | body: `skip_annotation` 결과 기반 — 복잡한 의존성 |
| `is_string_returning_fn` | 6개 group 함수 disjunction — 긴 enumeration 불필요 |

### 의미 있는 postcondition 유형

- `not it or pos < X.len()`: 스캔 함수 — true면 시작 위치 in-bounds
- `not it or start < end`: 범위 스캔 — true면 유효 범위
- `not it or struct_reg.len() > 0`: 레지스트리 조회 — true면 비어있지 않은 레지스트리
- `not it or argc > 3`: 인수 플래그 — true면 최소 인수 수 보장
- `not it or pos + 5 < s.len()`: 6바이트 패턴 — true면 s가 충분히 길다

## Verification & Defect Resolution

```json
{"type":"lint","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto: post verification failed  (pre-existing)
```

3800 tests passed ✅

### trivials 추적

| 종류 | Cycle 3207 후 | Cycle 3208 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 40 | 27 | -13 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 288 | 288 | 0 |
| **합계** | **335** | **322** | **-13** |

**누적 진척**: 358 → 322 (-36, 10.1%)

## Reflection

**Scope fit**: 6개 적용. 분석 중 많은 함수가 skip인 것이 명확해짐.

**Latent defects**: `is_dead_ptrtoint_range`의 semantic_duplication 위험은 연구 필요.
`index_is_decl_start`는 `skip_annotation`을 호출하는 복잡한 bool 함수 — 아직 분석 미완.

**Structural improvement opportunities**:
- 남은 bool 27개에서 7개는 skip 확정 (no pre), 20개 with pre:
  - 9개 semantic_duplication 그룹 2차 함수 — skip
  - 5개 "all" semantics — skip
  - 3개 복잡/분석 필요 (`is_dead_ptrtoint_range`, `check_var_list`, `index_is_decl_start`)
  - 3개 미분류 (`call_has_two_args`, `call_has_one_arg`, `is_string_returning_fn`)
  → 실질적으로 bool trivials는 거의 소진. 다음 사이클에서 String으로 전환해야.

**Roadmap impact**: Bool trivials 상당수 처리 완료. String 288개가 다음 주요 대상.

## Carry-Forward

- **Actionable**: M11-A Phase 4 — String `post it.len() >= 0` (288개) 중 `pre` 있는 것들 처리
  - `post it.len() >= 1` 가능한 것들: LLVM IR 생성 함수 외 더 있을 수 있음
  - 288개 중 77개 skip 확정 → 211개 검토 대상
- **Structural Improvement Proposals**:
  - `ifs_flex_check_goto`: `pre next_p >= 0` 추가 검토 (pre-existing Z3 failure 해소 가능)
  - bool 남은 20개 중 skip 확정이 아닌 것: `check_var_list`, `index_is_decl_start`, 
    `call_has_two_args`, `call_has_one_arg`, `is_string_returning_fn` — 추가 분석 필요하나 
    semantic 교체 이득 낮음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: 없음
- **Next Recommendation**: M11-A Phase 4 — String trivials 중 `pre` 있는 함수들의
  `post it.len() >= 1` 업그레이드 (14개 llvm_gen_* 이미 완료, 나머지 분석)
