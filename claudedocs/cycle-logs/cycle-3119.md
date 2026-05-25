# Cycle 3119: M8-A bool trivial 6개 추가 교체 (contains + equality chain 패턴)
Date: 2026-05-25

## Re-plan

Inherited: bool trivial 65개 남음. Carry-Forward: contains 패턴 추가 배치 + is_string_fn_groupN skip.
분석: 65개 중 단순 교체 가능한 후보 6개 확정. 나머지는 복잡한 로직 또는 body 복사.

## Scope & Implementation

**6개 교체** (`post it or not it` → `post it == (semantic_expr)`):

| 함수 | 위치 | 계약 |
|------|------|------|
| `is_fn_node(ast)` | L4129 | `post it == (get_node_type(ast) == "fn" or ... == "fn-trust")` |
| `slf_is_store(line)` | L10502 | `post it == (line.contains("store i64 ") or line.contains("store double "))` |
| `pfcse_is_pure(pure_set, name)` | L12209 | `post it == (pure_set != "" and name != "" and pure_set.contains(name + ";"))` |
| `gcs_label_in_phi(fn_mir, label)` | L14434 | `post it == (fn_mir.contains(", " + label + "]"))` |
| `pht_has_alloca(fn_mir, name)` | L14621 | `post it == (fn_mir.contains("alloca " + name))` |
| `mir_has_self_call(mir, fn_name)` | L15652 | `post it == (mir.contains("call @" + fn_name + "("))` |

**skip된 후보들**:
- `is_string_fn_group1-6`, `is_builtin_double_fn`: body 복사본 계약 — 문서화 가치 낮음
- `enum_has_payload/variant`, `is_float_expr`, `cf_table_has`, `cf_is_int_const`: 복잡한 탐색/재귀 로직
- `is_identity_copy_ir`, `is_aliased_trunc_ir`, `slf_is_bb_boundary`, `ube_is_label`: 복잡한 멀티-조건

## Verification & Defect Resolution

- `bmb check`: ✅ 3135 warnings (−6 vs 3141), 0 errors
- `bmb verify`: ✅ 954/954 verified, 0 failed
- 3-Stage Fixed Point: ✅ `A8ADD96654CD39795443635F1DAAB55D` (해시 불변 — 예상된 동작)

## Reflection

- Scope fit: 100%
- Warning count 누적: 3173 (기준) → 3163 → 3151 → 3141 → 3135 — 총 38개 trivial warning 제거
- `gcs_label_in_phi`, `pht_has_alloca`, `mir_has_self_call`: 계약이 body와 동일한 패턴 — 완전 정확
- `pfcse_is_pure`: 3-조건 and 계약 — body 로직을 정확히 포착
- `is_fn_node`, `slf_is_store`: body의 의미 근사적 표현 — 문서화 가치 있음
- 남은 trivials: ~59개 bool (교체 어려운 복잡한 로직 대다수)

## Carry-Forward

- Actionable: Cycle 3120 — 남은 59개 분류 정리 + HANDOFF/commit 준비
  - 교체 대상 추가 스캔 가능 (L16883-16947 is_X_var_sb 그룹 등)
  - 나머지는 진정한 임의 bool — trivial 유지가 정직한 결정
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-A bool 총 41/97개 교체 완료. Fixed Point `A8ADD96654CD39795443635F1DAAB55D`
- Next Recommendation: Cycle 3120 — is_X_var_sb 패턴 + HANDOFF/commit
