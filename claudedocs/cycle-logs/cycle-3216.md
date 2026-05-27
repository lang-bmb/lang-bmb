# Cycle 3216: M11-A Phase 5g — get_fn_return_scan + 수익 체감 평가 (1개)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3215 Carry-Forward — M11-A Phase 5g 탐색.
**평가**: 남은 230개 함수 광범위 분석 — 대부분 skip 패턴.

## Scope & Implementation

### 적용 함수 (1개)

| 함수 | pre | 근거 |
|------|-----|------|
| `get_fn_return_scan` | `pre pos >= 0` | 항상 type 문자열 반환: `"i64"`, `"*"+ ...`, known types(`"i32"`, `"f64"`, `"bool"`, `"String"`, `"ptr"`), `"Array<String>"`, `"Array<f64>"` — 모든 분기 non-empty |

### 분석 결과: 230개 분류

| 범주 | 대표 함수 | 수 (추정) |
|------|-----------|---------|
| Accumulator (`acc` 반환) | `collect_lambda_params`, `scan_mir_for_free_vars`, `ipr_collect_*`, `deps_*`, `cov_*` | ~60 |
| Lookup (`""` when not found) | `ctx_find_sig`, `sim_get_calls`, `lint_find_bmb`, `dc_get_calls` | ~40 |
| Pass-through (input 반환) | `replace_free_var`, `resolve_enum_variants_in_ast`, `rename_name_in_ast` | ~30 |
| sb_build (빈 입력 시 `""`) | `fix_typed_ret_placeholders_ir`, `remove_lambda_markers`, `optimize_ir_linear_recurrence` | ~50 |
| 복잡한 조건부 (분석 불가) | `impact_traverse`, `deps_traverse` | ~10 |
| **no-pre skip** | 77개 확인 완료 | 77 |
| **잔여 미분석** | (소수) | ~13 |

### 건너뛴 패턴들 (이번 사이클에서 분석)

| 함수 | 이유 |
|------|------|
| `build_capture_load_mir` | `free_vars == ""` 시 `""` 반환 |
| `scan_mir_for_free_vars` | base: `found` (초기 `""`) |
| `replace_free_var` | `sb_build` — 빈 mir 시 `""` |
| `build_dead_ptrtoint_set` | base: `dead` (초기 `""`) |
| `lookup_contracts_at` | not-found 시 `""` |
| `chain_dfs_calls` | `pos >= calls.len()` 시 `""` |
| `sibl_collect_siblings` | base: `acc` (초기 `""`) |
| `lint_find_bmb` | no .bmb files 시 `""` |
| `impact_visit_callers` | base: `visited` (초기 `""`) |
| `extract_pre_asts`, `extract_post_asts` | TK_EQ, TK_EOF 등 시 `""` |

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

2390 tests passed ✅

### trivials 추적

| 종류 | Cycle 3215 후 | Cycle 3216 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 231 | **230** | **-1** |
| **합계** | **265** | **264** | **-1** |
| String `post it.len() >= 1` | 155 | **156** | +1 |

**누적 진척**: 358 → 264 (-94, 26.3%)

## Reflection

**Scope fit**: 1개 업그레이드. 광범위 분석에서 대부분 skip 패턴.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- M11-A 수익 체감: Phase 5g에서 230개 분석 → 1개 발굴 (0.4% hit rate)
- 남은 230개 중 77개 no-pre skip, 나머지 153개가 accumulator/lookup/pass-through 패턴
- M11-A 추가 진척 위한 새 전략 필요: bool 27개, i64 7개 남은 trivials 도전 고려

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 26.3% 달성. 수익 체감 명확.

## Carry-Forward

- **Actionable**: 전략 평가 — M11-A 지속 vs 언어 갭 작업(M11-C) 전환
  - 남은 230개 중 새 발굴 가능성 매우 낮음 (hit rate 0.4%)
  - bool 27개 `post it or not it` — semantic_duplication 제약 내 추가 분석 가능
  - 언어 갭 작업(stack array / closure capture / generic) 더 높은 임팩트 가능
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→264 (-94, 26.3%). 수익 체감으로 언어 갭 작업 전환 검토
- **Next Recommendation**: M11-A commit 후 언어 갭 작업 전환 OR bool 27개 마지막 분석
