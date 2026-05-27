# Cycle 3212: M11-A Phase 5c — gen_assumes/hot/stats 체인 (4개)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3211 Carry-Forward — M11-A Phase 5c 분석 계속.
분석: `gen_assumes_for_post_contracts`, `hot_get_detail`, `hot_find_best`, `stats_most_called` 확인 완료 (이전 사이클에서 분석, 이번 사이클에서 실행).

## Scope & Implementation

### 적용 함수 (4개)

| 함수 | 근거 |
|------|------|
| `gen_assumes_for_post_contracts` | `pack_assume_result(raw_counter, fixed)` 항상 반환 → `int_to_string(counter) + ":" + fixed` → 최소 `"0:"` → `>= 1` |
| `stats_most_called` | base case: `best_name + " (" + int_to_string(best_count) + " callers)"` → 최소 `" (0 callers)"` 12자 → `>= 1` |
| `hot_find_best` | base case: `best_name + "\t" + int_to_string(best_score)` → 최소 `"\t0"` 2자 → `>= 1` |
| `hot_get_detail` | base case: `"0\t0"` 3자; match case: `x + "\t" + y` ≥ 1자 → `>= 1` |

### 함수별 분석

**`gen_assumes_for_post_contracts`** (`pre counter >= 0`):
```bmb
= let raw = gen_assumes_for_post_acc(contracts, 0, counter, "");
    let raw_counter = unpack_assume_counter(raw);
    let raw_ir = unpack_assume_ir(raw);
    let fixed = replace_all_str(raw_ir, "%_assume_", "%_post_assume_");
    pack_assume_result(raw_counter, fixed);
```
→ 항상 `pack_assume_result(...)` 반환 → `int_to_string(c) + ":" + fixed` → `>= 1` ✅

**`stats_most_called`** (`pre pos >= 0`):
```
base: best_name + " (" + int_to_string(best_count) + " callers)"
→ 최소 " (0 callers)" (12자)
```
→ 모든 분기 non-empty ✅

**`hot_find_best`** (`pre pos >= 0`):
```
base: best_name + "\t" + int_to_string(best_score)
→ 최소 "\t0" (2자)
```
→ 모든 분기 non-empty ✅

**`hot_get_detail`** (`pre pos >= 0`):
```
base: "0\t0" (3자)
match: callers_get_field(x) + "\t" + callers_get_field(y) ≥ "\t" (1자)
recurse: 재귀 → 항상 base에 도달
```
→ 모든 분기 non-empty ✅

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

2390 tests passed ✅

### trivials 추적

| 종류 | Cycle 3211 후 | Cycle 3212 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 256 | **252** | **-4** |
| **합계** | **290** | **286** | **-4** |
| String `post it.len() >= 1` | 130 | **134** | +4 |

**누적 진척**: 358 → 286 (-72, 20.1%)

## Reflection

**Scope fit**: 4개 String 함수 업그레이드 완료. 다양한 패턴: accumulator-like(gen_assumes), recursive-search-with-base(hot/stats).

**Latent defects**: 없음.

**Structural improvement opportunities**:
- 남은 252개 중 77개 no-pre skip → 175개 with-pre 후보
- `gen_fn_lines_structs`, `gen_function_sb_structs_reuse` 등 codegen chain 추가 탐색 가능
- `llvm_gen_fn_line_structs`, `llvm_gen_line_structs` — `same_mapping_empty()` 경로 때문에 `>= 0` 유지

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 누적 72개 교체. 20% 돌파.

## Carry-Forward

- **Actionable**: M11-A Phase 5d — 남은 252개 추가 탐색
  - with-pre 함수들 중 `llvm_gen_*` 계열 추가 분석
  - `optimize_*` 계열 (MIR 변환): 분기별 확인 필요
  - `gen_fn_lines_structs`, `gen_function_sb_structs_reuse` — codegen 체인 탐색
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→286 (-72, 20.1%)
- **Next Recommendation**: M11-A Phase 5d — 추가 체인/직접분석 탐색 OR 언어 갭 작업 전환 평가
