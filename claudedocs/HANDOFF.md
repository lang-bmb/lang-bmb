# BMB Session Handoff — 2026-05-25 (Cycles 3148-3151)

> **HEAD**: `c71f2c90`
> **이번 세션 작업**: Cycles 3148-3151 (M9 Batches 14-17 — missing_postcondition 628→568, −60)
> **3-Stage Fixed Point**: (M8-A 기준 `A8ADD96654CD39795443635F1DAAB55D` — M9는 post-only 추가로 IR 불변)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: **Cycle 3152** — `step_set_index/field/var/break/continue/return/cast` 계열 15개

---

## 이번 세션 작업 요약 (Cycles 3148-3151)

| Cycle | 제목 | 추가 post | 잔여 missing_postcondition |
|-------|------|----------|---------------------------|
| 3148 | M9 Batch 14 — make_step_leaf/step_literal 계열 | 15 | 613 |
| 3149 | M9 Batch 15 — i2s/trampoline/make_step/step_basic | 15 | 598 |
| 3150 | M9 Batch 16 — step_binop/unary/if/let/mut/call | 15 | 583 |
| 3151 | M9 Batch 17 — step_call/method/nullable/seq/assign/array_index | 15 | 568 |

### M9 전체 진행 현황

| 항목 | 값 |
|------|----|
| M9 시작 (Cycle 3140 기준) | 814 missing_postcondition |
| 이번 세션 시작 (Cycle 3148) | 628 missing_postcondition |
| 현재 상태 | **568 missing_postcondition** |
| M9 총 감소 | **−246 (30.2%)** |
| cargo test | ✅ 6278 tests, 0 failed |
| bmb check warnings | ✅ 2949 (net) |
| bmb verify | ✅ 715/715, 0 failed |

### 핵심 계약 패턴 수립 (Cycles 3149-3151)

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

### Cycle 3152 — step_set_index/field/var/break/continue/return/cast 계열

**확정 대상 (모두 `post it.len() >= 1` — make_step/make_step_leaf 반환)**:

#### Group 1: step_set_index (4개)
```
step_set_index_start  — make_step(cur_temp, ...) 단일 경로
step_set_index_idx    — make_step(cur_temp, ...) 단일 경로
step_set_index_val    — make_step(cur_temp, ...) 단일 경로
step_set_index_final  — make_step_leaf(cur_temp + N) 또는 make_step(...)
```

#### Group 2: step_field_access (2개)
```
step_field_access_start  — make_step(cur_temp, ...) 단일 경로
step_field_access_final  — make_step_leaf(cur_temp + 1) 단일 경로
```

#### Group 3: step_set_field (3개)
```
step_set_field_start  — make_step(cur_temp, ...) 단일 경로
step_set_field_val    — make_step(cur_temp, ...) 단일 경로
step_set_field_final  — make_step_leaf(cur_temp + N) 단일 경로
```

#### Group 4: step_set_var (2개)
```
step_set_var_start  — make_step(cur_temp, ...) 단일 경로
step_set_var_final  — make_step_leaf(cur_temp + 1) 단일 경로
```

#### Group 5: 제어 흐름 (4개)
```
step_break         — make_step_leaf(cur_temp) (exit label 설정)
step_continue      — make_step_leaf(cur_temp) (loop-back jump)
step_return        — make_step_leaf(cur_temp) (void return)
step_return_value  — make_step(cur_temp, ...) (value return, 평가 필요)
```

**합계: 15개** — Cycle 3152 배치 목표

#### 다음 배치 후보 (Cycle 3153+)
```
step_cast_to_i64/f64/i32              (3개)
step_cast_ptr_f64_start/finish        (2개)
step_array_literal                    (1개)
step_tuple                            (1개)
step_array_repeat, step_array_repeat_lit
step_array_repeat_lit_with_val, step_array_repeat_expr
```

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| HEAD | `c71f2c90` |
| missing_postcondition | **568** (목표 → 0) |
| bmb check warnings | 2949 |
| bmb verify | 715/715, 0 failed |
| cargo test | ✅ 6278 tests |
| 3-Stage FP | IR 불변 (post-only 추가, llvm.assume 미생성) |

---

## 알려진 미결 사항

- **missing_postcondition 568개**: M9 계속 — step_* 계열 75% 이상 처리 완료 추정, 잔여 40여 개 + 기타 함수군
- **`get_child` / `get_child_at` / `read_sexp_at`**: 빈 문자열 `""` 반환 경로 존재 → `post it.len() >= 1` 불가, 별도 분석 필요
- **semantic_duplication 증가**: uniform post 계약 추가 시 카운터 증가 (missing_postcondition 감소와 net 상쇄)
- **Z3 budget 영향**: 복잡한 post 계약 추가 시 `bmb verify` 총 검증 수 점진적 감소 (정상)
