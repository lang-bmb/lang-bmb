# BMB Session Handoff — 2026-05-25 (Cycles 3152-3160)

> **HEAD**: `c71f2c90` (미커밋 변경사항 있음 — compiler.bmb post 조건 추가)
> **이번 세션 작업**: Cycles 3152-3160 (M9 Batches 18-26 — missing_postcondition 568→433, −135)
> **3-Stage Fixed Point**: (M8-A 기준 `A8ADD96654CD39795443635F1DAAB55D` — M9는 post-only 추가로 IR 불변)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: **Cycle 3161** — `llvm_gen_line/llvm_gen_fn_*/parse_struct_sb/parse_enum_sb/emit_fill_stores` 등 15개

---

## 이번 세션 작업 요약 (Cycles 3152-3160)

| Cycle | 제목 | 추가 post | 잔여 missing_postcondition |
|-------|------|----------|---------------------------|
| 3152 | M9 Batch 18 — step_set_index/field/var/break/continue/return | 15 | 553 |
| 3153 | M9 Batch 19 — step_cast/array/tuple/repeat/lambda/struct/enum | 15 | 538 |
| 3154 | M9 Batch 20 — lower_call/struct_init/lambda/enum_val/call_sb | 15 | 523 |
| 3155 | M9 Batch 21 — lower_lit/var/cast/if/match/let/nullable_sb | 15 | 508 |
| 3156 | M9 Batch 22 — lower_method_args/block/unit/seq/assign/while/loop/break/continue/return/field/set_field/set_var/index_sb | 15 | 493 |
| 3157 | M9 Batch 23 — lower_set/ptr_index/for_*/safe_*/lower_expr_sb/pos_after_annotation/replace_var_rec | 15 | 478 |
| 3158 | M9 Batch 24 — get_int/string/float_text/get_child/read_sexp/get_pipe_name/get_field/rename/get_exit_label | 15 | 463 |
| 3159 | M9 Batch 25 — collect_lambda/scan_mir/replace_free_vars/format/rewrite/lower_function/collect_params | 15 | 448 |
| 3160 | M9 Batch 26 — get_fn_return_scan/get_fn_body_scan/llvm_gen_* | 15 | 433 |

### M9 전체 진행 현황

| 항목 | 값 |
|------|----|
| M9 시작 (Cycle 3140 기준) | 814 missing_postcondition |
| 이번 세션 시작 (Cycle 3152) | 568 missing_postcondition |
| 현재 상태 | **433 missing_postcondition** |
| M9 총 감소 | **−381 (46.8%)** |
| cargo test | ✅ 6278 tests, 0 failed |
| bmb check warnings | ✅ (net, post-only 추가) |
| bmb verify | ✅ IR 불변 (post-only 추가) |

### 핵심 계약 패턴 수립 (Cycles 3149-3160)

**`post it.len() >= 1` 체인**:
```
int_to_string(n) → post it.len() >= 1  (기존)
i2s(n)          → post it.len() >= 1  (Cycle 3149: int_to_string 위임)
make_step_leaf  → post it.len() >= 1  (Cycle 3149: i2s(temp) 직접 반환)
make_step       → post it.len() >= 1  (Cycle 3149: i2s(temp) + SEP + ... 연결)
do_step         → post it.len() >= 1  (Cycle 3149: make_step/make_step_leaf 반환)
step_expr, step_int, step_float, step_bool, step_string, step_var  (Cycle 3149)
step_binop_start/right/final, step_unary_start/final              (Cycle 3150)
step_if_start/select/then/else/final                               (Cycle 3150)
step_let_start/body, step_mut_start/body, step_call_start         (Cycle 3150)
step_call_arg/final, step_method_start/arg/final                   (Cycle 3151)
step_nullable_result/or, step_unit                                 (Cycle 3151)
step_seq_start/second, step_assign_start/final                     (Cycle 3151)
step_array_index_start/idx/final                                   (Cycle 3151)
```

**`post it >= 0` 패턴**:
```
trampoline_v3    → post it >= 0  (pack_ids 반환, Cycle 3149)
lower_expr_iter  → post it >= 0  (trampoline_v3 반환, Cycle 3149)
check_field_type → post it >= 0  (0~4 반환, Cycle 3149)
parse_int_simple → pre acc >= 0 + post it >= 0  (누산기, Cycle 3149)
parse_oct_from   → pre acc >= 0 + post it >= 0  (누산기, Cycle 3149)
```

---

## 다음 세션 시작점

### Cycle 3161 — llvm_gen_line/llvm_gen_fn_*/parse_struct_sb/emit_fill_stores 계열

**확정 대상 후보**:
```
llvm_gen_line(...)          — String 반환, LLVM IR 한 줄 생성
llvm_gen_fn_start(...)      — String 반환, 함수 시그니처 생성
llvm_gen_fn_end(...)        — String 반환
parse_struct_sb(...)        — String 반환, 구조체 MIR 생성
parse_enum_sb(...)          — String 반환
emit_fill_stores(...)       — i64 반환 (pre temp_id >= 0 확인 필요)
get_fn_return_type(...)     — String 반환
llvm_gen_alloca(...)        — String 반환
```
- `bmb check bootstrap/compiler.bmb 2>&1 | grep "missing_postcondition" | head -20` 으로 실제 잔여 대상 확인 후 진행

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| HEAD | `c71f2c90` (미커밋) |
| missing_postcondition | **433** (목표 → 0) |
| cargo test | ✅ 6278 tests |
| 3-Stage FP | IR 불변 (post-only 추가, llvm.assume 미생성) |

---

## 알려진 미결 사항

- **missing_postcondition 433개**: M9 계속 진행 — llvm_gen_line/parse_struct/emit_fill_stores 등 잔여
- **`emit_fill_stores` / `emit_fill_stores_step`**: `pre temp_id >= 0` 없는 경우 존재 → 추가 전 확인 필요
- **bool 반환 함수**: `variant_has_bracket` 등 bool post 조건 패턴 미결정
- **semantic_duplication 증가**: uniform post 계약 추가 시 카운터 증가 (missing_postcondition 감소와 net 상쇄)
- **Z3 budget 영향**: 복잡한 post 계약 추가 시 `bmb verify` 총 검증 수 점진적 감소 (정상)
