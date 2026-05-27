# Cycle 3215: M11-A Phase 5f — index_* parse 체인 (7개)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3214 Carry-Forward — M11-A Phase 5f index_* parse 체인 탐색.
**핵심 발견**: `index_parse_param_text`, `index_read_type_text`, `index_collect_params`, `index_read_ret_type`, `index_scan_body`, `index_struct_field`, `index_struct_fields` — 모두 `pack_result(...)` 반환.

## Scope & Implementation

### 적용 함수 (7개)

| 함수 | 근거 |
|------|------|
| `index_parse_param_text` | 모든 분기 `pack_result(...)` → `>= 2` |
| `index_read_type_text` | 모든 분기 `pack_result(...)` → `>= 2` |
| `index_collect_params` | 모든 분기 `pack_result(...)` → `>= 2` |
| `index_read_ret_type` | `TK_ARROW` → `index_read_type_text >= 1`; else → `pack_result(pos, "?") >= 2` |
| `index_scan_body` | 모든 분기 `pack_result(...)` → `>= 2` |
| `index_struct_field` | 모든 분기 `pack_result(...)` → `>= 2` |
| `index_struct_fields` | 모든 분기 `pack_result(...)` → `>= 2` |

### pack_result 체인 패턴 완성

이번 사이클로 BMB 인덱서/파서의 전체 `pack_result` 체인이 `>= 1`로 승격:
```
pack_result (>= 2)
  ← index_parse_param_text, index_read_type_text, index_collect_params (>= 1)
  ← index_read_ret_type (>= 1)
  ← index_scan_body (>= 1)
  ← index_struct_field, index_struct_fields (>= 1)
  ← query_one_fn, query_one_struct, callers_collect_fn (>= 1, Cycle 3214)
```

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

2390 tests passed ✅

### trivials 추적

| 종류 | Cycle 3214 후 | Cycle 3215 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 238 | **231** | **-7** |
| **합계** | **272** | **265** | **-7** |
| String `post it.len() >= 1` | 148 | **155** | +7 |

**누적 진척**: 358 → 265 (-93, 26.0%)

## Reflection

**Scope fit**: 7개 업그레이드. `index_*` 파서 체인 전체 완성.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- 남은 231개 중 추가 pack_result 체인 함수 탐색 가능
- `extract_pre_asts`, `extract_post_asts` 등 — 조건부 empty 반환 가능성 높음

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 26.0% 달성.

## Carry-Forward

- **Actionable**: M11-A Phase 5g — 남은 231개 추가 탐색
  - parse chain 함수들 중 `pack_result` 또는 `same_mapping("  " + ...)` 패턴
  - `build_contracts_map`, `build_post_contracts_map` — 빈 결과 가능성 확인 필요
  - `deps_find_calls`, `sim_get_calls` 등 search 함수들
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→265 (-93, 26.0%)
- **Next Recommendation**: M11-A Phase 5g 계속 OR 언어 갭 작업 평가 (수익체감 점검)
