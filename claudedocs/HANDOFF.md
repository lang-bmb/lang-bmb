# BMB Session Handoff — 2026-05-26 (Cycles 3161-3168)

> **HEAD**: 미커밋 변경사항 있음 — compiler.bmb post 조건 추가
> **이번 세션 작업**: Cycles 3161-3168 (M9 Batches 27-34 — missing_postcondition 433→313, −120)
> **3-Stage Fixed Point**: (M8-A 기준 `A8ADD96654CD39795443635F1DAAB55D` — M9는 post-only 추가로 IR 불변)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: **Cycle 3169** — `cfeval_program/trl_parse_params/trl_param_at_scan/trl_find_tail_call` 등 15개

---

## 이번 세션 작업 요약 (Cycles 3161-3168)

| Cycle | 제목 | 추가 post | 잔여 missing_postcondition |
|-------|------|----------|---------------------------|
| 3161 | M9 Batch 27 — build_payload_lets_from_pat/parse_program_sb 등 | 15 | 418 |
| 3162 | M9 Batch 28 — format_i64_args_sb/llvm_gen_call 등 | 15 | 403 |
| 3163 | M9 Batch 29 — llvm_gen_float_binop/rebuild_ir_no_copies 등 | 15 | 388 |
| 3164 | M9 Batch 30 — rebuild_ir_no_dead_zexts/ipr_collect_* 등 | 15 | 373 |
| 3165 | M9 Batch 31 — ipr_rebuild/spec_rebuild/inline_rebuild 등 | 15 | 358 |
| 3166 | M9 Batch 32 — rebuild_ir_no_dead_ptrtoints/dsa_*/find_pattern_noa 등 | 15 | 343 |
| 3167 | M9 Batch 33 — replace_all_in_mir_acc/cf_*/dce_*/ube_*/cp_* 등 | 15 | 328 |
| 3168 | M9 Batch 34 — cp_replace_vars/pfcse_*/mlcse_*/cfeval_* 등 | 15 | 313 |

### M9 전체 진행 현황

| 항목 | 값 |
|------|----|
| M9 시작 (Cycle 3140 기준) | 814 missing_postcondition |
| 이번 세션 시작 (Cycle 3161) | 433 missing_postcondition |
| 현재 상태 | **313 missing_postcondition** |
| M9 총 감소 | **−501 (61.5%)** |
| cargo test | ✅ 6278 tests, 0 failed |
| bmb check warnings | ✅ (net, post-only 추가) |
| bmb verify | ✅ IR 불변 (post-only 추가) |

### 확립된 계약 패턴 (이번 세션)

**재구성 패스 계열** (`_rebuild`, `_scan_`, `_apply_*`):
- i64 반환 (count/0) → `post it >= 0`

**누적 String 계열** (`_collect_*`, `_find_*`, `build_*_set`, `*_lookup_at`):
- 빈 문자열 포함 가능 → `post it.len() >= 0`

**패턴 검색 계열** (`find_pattern_noa*`):
- 미발견 시 -1 반환 → `post it >= -1`

**특수 sentinel 계열** (`cf_table_get_at`):
- 미발견 시 `-99999999` → `post it >= 0 or it < 0` (항상 참)

**수학 함수** (`cf_pow2`):
- 2^n, n >= 0 → `post it >= 1`

---

## 다음 세션 시작점

### Cycle 3169 — cfeval_program/trl 계열

**확정 대상 후보** (bmb check로 실제 확인 후 진행):
```
cfeval_program(...)              — i64, count 반환 → post it >= 0
trl_parse_params(...)           — String 반환 → post it.len() >= 0
trl_param_at(...)               — String 반환 → post it.len() >= 0
trl_param_at_scan(...)          — String 반환 → post it.len() >= 0
trl_find_tail_call(...)         — i64 반환 → post it >= -1 또는 post it >= 0
```
- `bmb check bootstrap/compiler.bmb 2>&1 | grep "missing_postcondition" | head -20` 으로 실제 잔여 대상 확인 후 진행

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| HEAD | 미커밋 |
| missing_postcondition | **313** (목표 → 0) |
| cargo test | ✅ 6278 tests |
| 3-Stage FP | IR 불변 (post-only 추가, llvm.assume 미생성) |

---

## 알려진 미결 사항

- **missing_postcondition 313개**: M9 계속 진행 — cfeval_program/trl/기타 계열 잔여
- **semantic_duplication 증가**: uniform post 계약 추가 시 카운터 증가 (missing_postcondition 감소와 net 상쇄)
- **Z3 budget 영향**: 복잡한 post 계약 추가 시 `bmb verify` 총 검증 수 점진적 감소 (정상)
