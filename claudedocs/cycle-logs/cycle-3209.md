# Cycle 3209: M11-A Phase 4 — String always-non-empty functions
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3208 Carry-Forward — String trivials `>= 0` → `>= 1` 업그레이드.
**발견**: String 함수는 bool과 달리 `semantic_duplication` 경고 없음 (Cycle 3206 empirical).
따라서 String 함수는 동일 시그니처라도 독립적으로 교체 가능.

## Scope & Implementation

### 발견: semantic_duplication은 bool만 적용

- bool: 동일 (sig, pre, post) → `semantic_duplication` 경고
- String: 동일 (sig, pre, post) → 경고 없음 (14개 llvm_gen_* 공존 확인)

이로 인해 String 함수 전략 변경: 각 함수를 개별 분석하여 항상 비어있지 않으면 `>= 1` 적용.

### Phase 4a — 7개 (분석 후 적용)

| 함수 | 근거 |
|------|------|
| `llvm_gen_nullable_select` | `"  " + dest + ...` 항상 non-empty |
| `llvm_gen_select` | 두 분기 모두 `"  " + dest + ...` |
| `llvm_gen_closure_load` | `"  " + ptr_tmp + ...` 항상 non-empty |
| `llvm_gen_field_access` | `same_mapping("  " + base + "_p = ..." + ...)` |
| `llvm_gen_neg_sb` | 두 분기 모두 `same_mapping("  " + dest + ...)` |
| `llvm_gen_phi_with_strings_sb` | llvm_gen_phi/phi_typed 위임 (post >= 1) |
| `pack_assume_result` | `int_to_string(counter) + ":" + ir` — counter >= 0 → "0:" min |

### Phase 4b — 6개 (추가 확인 후)

| 함수 | 근거 |
|------|------|
| `llvm_gen_hof_call` | `same_mapping(cast_ir + SEP() + call_ir)` — cast_ir starts "  " |
| `llvm_gen_indirect_call` | `same_mapping(load_ir + SEP() + ... + call_ir)` — load_ir starts "  " |
| `llvm_gen_indirect_call_param` | `same_mapping(env_ir + SEP() + ... + call_ir)` — env_ir starts "  " |
| `llvm_gen_field_store` | `same_mapping(conv + SEP() + gep + SEP() + store)` — conv starts "  " |
| `llvm_gen_binop_sb` | 두 분기 모두 `same_mapping("  " + dest + ...)` |
| `llvm_gen_cmp_sb` | 두 분기 모두 `same_mapping("  " + dest + ... + SEP() + "  " + ...)` |

**건너뛴 함수**:
- `llvm_handle_mark_str_ptr_if` → `same_mapping("")` = "" 반환 (반드시 `>= 0` 유지)
- `llvm_try_println_str_dispatch` → else 분기에서 `""` 반환 가능
- `llvm_gen_copy_struct_aware` → `llvm_gen_rhs_with_strings_map_and_fns_reg` 위임 (불명)
- `llvm_gen_call_with_string_tracking_sb_reg` → dispatch == "" 분기 복잡
- `parse_build_runtime` → `get_arg` 결과 가능성 비어있음

## Verification & Defect Resolution

```json
{"type":"lint","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

3800 tests passed ✅

### trivials 누적 추적

| 종류 | Start (Cycle 3206 전) | Cycle 3209 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 49 | 27 | **-22** |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 302 | 275 | **-27** |
| **합계** | **358** | **309** | **-49 (-13.7%)** |
| String `post it.len() >= 1` | ~84 | 111 | +27 |

## Reflection

**Scope fit**: 13개 String 함수 업그레이드 완료. Phase 4a + 4b 합산.

**Latent defects**:
- `llvm_handle_mark_str_ptr_if`: `same_mapping("")` 항상 반환 → `>= 0` 맞음 (버그 없음)
- 더 많은 String 함수들이 조건부로 `""` 반환 → 더 깊은 분석 없이 업그레이드 불가

**Structural improvement opportunities**:
- 남은 275개 `>= 0` 중 77개는 skip 확정, 198개 잠재 대상
- LLVM codegen 함수 이외에도 다른 패턴 있음: formatting functions, accumulator functions
- `llvm_gen_conc_rhs`, `llvm_gen_conc_stmt`, `llvm_gen_channel_new` 미분석 — 추가 확인 필요

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 1/3 완료 (358 → 309, 13.7%). 남은 302개 분석 필요.

## Carry-Forward

- **Actionable**: M11-A Phase 5 — 남은 String trivials 198개 (non-skip) 추가 분석
  - LLVM conc/channel 함수: `llvm_gen_conc_rhs`, `llvm_gen_conc_stmt`, `llvm_gen_channel_new`
  - Formatting functions: `format_fn_params`, `gen_i32_param_sexts`, `format_indirect_call_args`
  - Accumulator functions: `gen_assumes_for_contracts_acc`, `gen_assumes_for_post_acc`
- **Structural Improvement Proposals**: 없음 (이전 제안 유지)
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 진행 중 (49/358+ 완료)
- **Next Recommendation**: 지금까지 쌓인 변경사항을 commit하고, HANDOFF 업데이트 후
  다음 세션에서 String trivials 추가 분석 또는 별도 테마 작업 선택
