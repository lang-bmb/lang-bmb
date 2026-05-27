# Cycle 3211: M11-A Phase 5b — String codegen chain (llvm_gen_call_reg >= 1 chain)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3210 Carry-Forward — M11-A Phase 5b 또는 전략 전환.
**발견**: `llvm_gen_call_reg`가 이미 `post it.len() >= 1` 보유 → 전체 RHS codegen 체인이 항상 non-empty.

## Scope & Implementation

### 핵심 발견: llvm_gen_call_reg >= 1 체인

```
llvm_gen_call_reg: post it.len() >= 1  (기존)
    ↓
llvm_gen_call_with_string_tracking_sb_reg: 항상 non-empty (dispatch or same_mapping(call_reg_result))
    ↓
llvm_gen_rhs_with_strings_map_and_fns_reg: 모든 분기 same_mapping("  " + ...) or changed_mapping_empty(...) or 위임
    ↓
llvm_gen_copy_struct_aware, llvm_gen_add_struct_aware, llvm_gen_call_struct_aware, llvm_gen_rhs_structs, llvm_gen_assign_structs
    ↓
llvm_gen_add_with_strings_sb (→ llvm_gen_add_string_concat_sb → "  " + ...) 또는 fadd/add nsw "  " + ...
    ↓
llvm_gen_cmp_with_strings_sb → llvm_gen_string_cmp → "  " + ... 또는 llvm_gen_cmp >= 1
```

### llvm_gen_string_cmp 분석

`llvm_gen_string_cmp_3` → `same_mapping(conv_l + SEP() + ...)` 또는 `same_mapping(conv_l + SEP() + ... + res_ln)`
where `conv_l = "  " + lptr + ...` — 항상 non-empty.

### 적용 함수 (12개)

| 함수 | 근거 |
|------|------|
| `llvm_gen_call_with_string_tracking_sb_reg` | `dispatch != ""` → `same_mapping(dispatch)` non-empty; else `same_mapping(llvm_gen_call_reg(...))` — call_reg `>= 1` |
| `llvm_gen_rhs_with_strings_map_and_fns_reg` | 모든 분기 `same_mapping("  " + ...)`, `changed_mapping_empty(...)`, 또는 `>= 1` 확인 함수에 위임; final else = `same_mapping("  ; unknown: " + line)` |
| `llvm_gen_copy_struct_aware` | `llvm_gen_rhs_with_strings_map_and_fns_reg(...)` 위임 → `>= 1` |
| `llvm_gen_call_struct_aware` | ALLOCA→`indirect_call >= 1`; HOF→`hof_call >= 1`; i64→`indirect_call_param >= 1`; else→`rhs_with_strings >= 1` |
| `llvm_gen_add_struct_aware` | struct path→`same_mapping("  " + ...)` non-empty; regular→`add_with_strings_sb >= 1` |
| `llvm_gen_rhs_structs` | 모든 위임 `>= 1` 확인됨 |
| `llvm_gen_assign_structs` | `llvm_gen_rhs_structs(...)` 위임 → `>= 1` |
| `llvm_gen_add_with_strings_sb` | string→`add_string_concat_sb ("  " + ...)` `>= 1`; float/int→`same_mapping("  " + ...)` `>= 1` |
| `llvm_gen_add_with_strings_sb_2` | `llvm_gen_add_with_strings_sb_3(...)` 위임 → `>= 1` |
| `llvm_gen_cmp_with_strings_sb` | `llvm_gen_cmp_with_strings_sb_2(...)` → `_3` 위임 → `>= 1` |
| `llvm_gen_cmp_with_strings_sb_2` | `llvm_gen_cmp_with_strings_sb_3(...)` 위임 → `>= 1` |
| `llvm_gen_cmp_with_strings_sb_3` | string→`llvm_gen_string_cmp >= 1`; float→`same_mapping("  " + ...)` `>= 1`; int→`same_mapping(llvm_gen_cmp >= 1)` |

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

3800 tests passed ✅

### trivials 추적

| 종류 | Cycle 3210 후 | Cycle 3211 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 268 | **256** | **-12** |
| **합계** | **302** | **290** | **-12** |
| String `post it.len() >= 1` | 118 | **130** | +12 |

**누적 진척**: 358 → 290 (-68, 19.0%)

## Reflection

**Scope fit**: 12개 String 함수 업그레이드 완료. `llvm_gen_call_reg >= 1` 체인 발견이 핵심.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- `llvm_gen_string_cmp`, `llvm_gen_string_cmp_2`, `llvm_gen_string_cmp_3` — no-pre 그룹이지만 확인 완료 (`>= 1` 실질)
- 추가 체인 탐색: `gen_fn_lines_structs`, `gen_function_sb_structs_reuse`, `gen_program_acc_sb_structs_reuse`가 `llvm_gen_line_structs`/`llvm_gen_assign_structs`를 호출하는지 확인 가능

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 368→290 진척. 체인 발견으로 배치 효율성 회복.

## Carry-Forward

- **Actionable**: M11-A Phase 5c — 남은 256개 중 77 no-pre skip → 179개 with-pre 추가 분석
  - `gen_fn_lines_structs`, `gen_function_sb_structs_reuse`, `gen_program_acc_sb_structs_reuse` (lines ~29**): 새 codegen 체인 탐색
  - `llvm_gen_fn_line_structs`, `llvm_gen_line_structs`: 직접 IR 생성 함수들
  - `optimize_*` 계열 함수들: MIR 변환 함수들 분석
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→290 (-68, 19.0%)
- **Next Recommendation**: M11-A Phase 5c — `gen_fn_lines_structs` 등 더 많은 codegen 체인 탐색
