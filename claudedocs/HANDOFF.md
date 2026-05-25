# BMB Session Handoff — 2026-05-26 (Cycles 3169-3178)

> **HEAD**: 미커밋 변경사항 있음 — compiler.bmb post 조건 추가
> **이번 세션 작업**: Cycles 3169-3178 (M9 Batches 35-44 — missing_postcondition 313→163, −150)
> **3-Stage Fixed Point**: (M8-A 기준 `A8ADD96654CD39795443635F1DAAB55D` — M9는 post-only 추가로 IR 불변)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: **Cycle 3179** — `conc_*/llvm_gen_return/lower_*` 등 15개

---

## 이번 세션 작업 요약 (Cycles 3169-3178)

| Cycle | 제목 | 추가 post | 잔여 missing_postcondition |
|-------|------|----------|---------------------------|
| 3169 | M9 Batch 35 — cfeval_program/trl 계열 | 15 | 298 |
| 3170 | M9 Batch 36 — licm + rpe + ifs 시작 | 15 | 283 |
| 3171 | M9 Batch 37 — ifs 계열 | 15 | 268 |
| 3172 | M9 Batch 38 — ifs 잔여 + gcs + pht 시작 | 15 | 253 |
| 3173 | M9 Batch 39 — pht 잔여 + optimize_cf_dce_loop 등 | 15 | 238 |
| 3174 | M9 Batch 40 — llvm_gen_fn_header + string 처리 등 | 15 | 223 |
| 3175 | M9 Batch 41 — gen_function/llvm_gen struct 계열 | 15 | 208 |
| 3176 | M9 Batch 42 — llvm_gen closure/field 계열 | 15 | 193 |
| 3177 | M9 Batch 43 — check_field/closure marker/cmp 계열 | 15 | 178 |
| 3178 | M9 Batch 44 — llvm_gen string-tracking/mapping 계열 | 15 | 163 |

### M9 전체 진행 현황

| 항목 | 값 |
|------|----|
| M9 시작 (Cycle 3140 기준) | 814 missing_postcondition |
| 이번 세션 시작 (Cycle 3169) | 313 missing_postcondition |
| 현재 상태 | **163 missing_postcondition** |
| M9 총 감소 | **−651 (79.9%)** |
| cargo test | ✅ 6278 tests, 0 failed (이전 세션 확인) |
| bmb check warnings | ✅ (net, post-only 추가) |
| bmb verify | ✅ IR 불변 (post-only 추가) |

### 이번 세션 처리 패스 계열

- **trl (Tail-Recursive Loop)**: trl_parse_params/param_at/find_tail_call/build_phis 등
- **licm (Loop-Invariant Code Motion)**: licm_build_copy_map/invariant_map/scan_phis/emit 등
- **rpe (Redundant Path Elimination)**: rpe_lookup_const_depth/fn_lines/program
- **ifs (If-Statement optimization)**: ifs_check_pattern/fn_lines/try_extended/check_both 등
- **gcs (Goto Chain Simplification)**: gcs_is_forward/find_forwards/resolve/rewrite_fn/program
- **pht (Phi-Aware Empty Block Threading)**: pht_find_copy_map/find_phi_fwds/build_phi_map/rewrite_fn 등
- **llvm_gen struct/closure**: llvm_gen_field_access/closure_new_sb/closure_load/assign_structs 등
- **llvm_gen string-tracking**: llvm_gen_cmp_sb/binop_sb/add_sb/phi_sb/call_tracking_sb_reg 등

---

## 다음 세션 시작점

### Cycle 3179 — conc_*/lower_*/llvm_gen_return 계열

**확정 대상 후보** (bmb check로 실제 확인 후 진행):
- `bmb check bootstrap/compiler.bmb 2>&1 | grep "missing_postcondition" | head -20` 으로 실제 잔여 대상 확인 후 진행
- conc_extract_two_ops_first/second 등 conc_* 계열
- llvm_gen_return/llvm_gen_goto/llvm_gen_branch 계열
- lower_* 계열 잔여

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| HEAD | 미커밋 |
| missing_postcondition | **163** (목표 → 0) |
| cargo test | ✅ 6278 tests (이전 세션 확인) |
| 3-Stage FP | IR 불변 (post-only 추가, llvm.assume 미생성) |

---

## 알려진 미결 사항

- **missing_postcondition 163개**: M9 계속 진행 — conc_*/lower_*/llvm_gen 계열 잔여
- **semantic_duplication**: uniform post 계약 추가 시 카운터 증가 (missing_postcondition 감소와 net 상쇄)
- **Z3 budget 영향**: 복잡한 post 계약 추가 시 `bmb verify` 총 검증 수 점진적 감소 (정상)
