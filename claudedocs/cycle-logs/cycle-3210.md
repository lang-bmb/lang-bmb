# Cycle 3210: M11-A Phase 5a — String always-non-empty (LLVM conc/channel + key entry points)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3209 Carry-Forward — M11-A Phase 5, String trivials 275개 중 198개 비-skip 분석.
Stage 2/Fixed Point: `fixed_point: true` ✅ (이전 Bootstrap 실행 결과 이월).

**STEP 0 판단**: 198개 with-pre 후보 체계적 분석 실행.

## Scope & Implementation

### 분석 방법론

198개 후보를 4가지 패턴으로 분류:
1. **accumulator** (`acc: String` 파라미터, base case에서 `acc` 반환) → CAN return `""` → skip
2. **lookup** (`pos >= X.len()` 시 `""` 반환) → CAN return `""` → skip
3. **pattern-match** (no-match 분기에서 `""` 반환) → CAN return `""` → skip
4. **LLVM codegen entry point** (모든 분기에서 non-empty IR 생성) → `>= 1` 대상

### Phase 5a 적용 함수 (7개)

| 함수 | 근거 |
|------|------|
| `llvm_gen_conc_rhs` | 모든 분기 `"  " + ...` 시작; else는 `same_mapping("  ; unknown-conc: " + line)` — min 17자 |
| `llvm_gen_conc_stmt` | 모든 분기 `conc_gen_call_void_*(...)` → `"  call void @..."` 시작; else는 `"  ; unknown-conc-stmt: "` |
| `llvm_gen_channel_new` | `"  " + sender_dest + "_alloc = alloca i64, align 8" + SEP() + ...` — 항상 non-empty |
| `gen_assumes_for_post_acc` | 항상 `pack_assume_result(counter, acc)` 반환 → `"0:" + acc` min → `>= 1` (확인: Cycle 3209) |
| `lower_function_sb` | `"fn " + name + "(" + params + ") -> " + ret_type + ann_suffix + " {"` → 항상 non-empty |
| `llvm_gen_closure_new_sb` | csb에 최소 5개 line push 후 `same_mapping(sb_build(csb))` → ir 항상 non-empty |
| `compile_program` | error → `ast_raw` (non-empty); empty MIR → `"ERR:lowering produced empty MIR"` (non-empty); 정상 → `pruned_ir + DSEP() + "..."` (non-empty) |

### 건너뛴 패턴들

| 패턴 | 대표 함수 | 이유 |
|------|----------|------|
| Accumulator with `acc` param | `gen_i32_param_sexts`, `format_fn_params`, `format_indirect_call_args`, `build_ir_copy_aliases`, `gcs_find_forwards`, `build_param_sig`, `fmt_lines` | base case에서 `acc` (초기 `""`) 반환 |
| Lookup/search | `lookup_fn_ret_at`, `extract_call_fn_name`, `gcs_is_forward`, `pht_is_phi_fwd`, `skip_annotation`, `extract_contract_text`, `ifs_line_at` | not-found 시 `""` 반환 |
| Pattern-match (conditional gen) | `ifs_check_pattern`, `ifs_check_then_one`, `gcs_resolve` | no-match 분기에서 `""` 반환 |
| `llvm_gen_rhs_with_strings_map_and_fns_reg` | — | `call` 분기 → `llvm_gen_call_with_string_tracking_sb_reg` → `same_mapping(llvm_gen_call_reg(...))` — `llvm_gen_call_reg` CAN return `""` |
| `gen_assumes_for_contracts_acc` | — | base case에서 `acc` (`""` 초기값) 반환 |
| `cont_exit_find_ctrl_vars` | — | `sb_build(sb)` — loop label 없으면 `""` |
| `fmt_rtrim`, `fmt_indent` | — | `end <= 0` 또는 `n <= 0` 시 `""` |

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

3800 tests passed ✅

### trivials 추적

| 종류 | Cycle 3209 후 | Cycle 3210 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 275 | **268** | **-7** |
| **합계** | **309** | **302** | **-7** |
| String `post it.len() >= 1` | 111 | **118** | +7 |

**누적 진척**: 358 → 302 (-56, 15.6%)

## Reflection

**Scope fit**: 7개 String 함수 업그레이드 완료. 198개 후보 중 191개는 `""` 반환 가능 → skip.

**Latent defects**: 없음. `ifs_flex_check_goto` pre-existing.

**Structural improvement opportunities**:
- 198개 with-pre 후보 중 실질적으로 non-empty인 것은 매우 드묾 (7/198 = 3.5%)
- 남은 268개 trivials 중: 77 no-pre skip + 191개 with-pre candidates 중 non-empty 발굴이 어려움
- M11-A 수익 체감 (16→7→6→13→7 per cycle): 다음 사이클에서 전략 재평가 필요

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 계속 가능하지만 수익 체감. 전략 전환 고려 시점.

## Carry-Forward

- **Actionable**: M11-A Phase 5b 또는 전략 전환
  - 남은 after-trivials 268개 → 77 no-pre skip → 191개 with-pre
  - 191개 중 non-empty인 것 발굴이 어려운 상태 (3.5% hit rate)
  - 대안: 언어 갭 작업 (M11-C: stack array / closure capture / generic)
- **Structural Improvement Proposals**: 없음 (이전 제안 유지)
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→302 (-56, 15.6%)
- **Next Recommendation**: M11-A Phase 5b — 남은 191개 후보 중 명확히 비-empty인 그룹 추가 탐색,
  OR 언어 갭 작업으로 전환 (더 높은 임팩트 가능)
