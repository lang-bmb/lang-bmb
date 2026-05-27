# Cycle 3214: M11-A Phase 5e — pack_result/parse_source/lambda 체인 (6개)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3213 Carry-Forward — M11-A Phase 5e 추가 탐색.
**전략**: pack_result 체인, parse_program_sb 위임, lambda 분석.

## Scope & Implementation

### 적용 함수 (6개)

| 함수 | pre | 근거 |
|------|-----|------|
| `llvm_gen_string_ref` | 없음 | `"  " + dest + " = ptrtoint ptr @str_bmb_" + idx_str + " to i64"` → 항상 non-empty |
| `parse_source` | 없음 | `parse_program_sb(src, 0, sb)` 위임 → `parse_program_sb`가 `post it.len() >= 1` |
| `query_one_fn` | `pre pos >= 0` | 모든 exit path: `pack_result(...)` → `pack_result`는 `post it.len() >= 2` |
| `query_one_struct` | `pre pos >= 0` | 모든 exit path: `pack_result(...)` → `>= 2` → `>= 1` |
| `callers_collect_fn` | `pre pos >= 0` | 모든 exit path: `pack_result(...)` → `>= 2` → `>= 1` |
| `get_lambda_body` | `pre idx >= 0` | base: `"(int 0)"` (7자); else: `child` (child != "" 확인됨); recurse: always non-empty |

### pack_result 체인 발견

`pack_result` 이미 `post it.len() >= 2`. 이를 항상 반환하는 함수들:
- `query_one_fn`: 모든 분기 `pack_result(body_end, "1"/"0")` or `pack_result(tok_end(t1), "0"/"0")`
- `query_one_struct`: 모든 분기 `pack_result(..., "0"/"1")`
- `callers_collect_fn`: 모든 분기 `pack_result(..., name+"\t"+calls+"\t"+sig)` or `pack_result(..., "")`

### `get_lambda_body` 분석

```bmb
= let child = get_child(ast, idx);
    if child == "" { "(int 0)" }
    else if get_node_type(child) != "param" { child }  // child != "" 보장
    else { get_lambda_body(ast, idx + 1) }  // 재귀 → 항상 base에 도달
```
- `child == ""` → `"(int 0)"` (7자) ✅
- `child != ""` and not param → `child` (non-empty) ✅
- param → recurse → eventually "" or non-param child

### 건너뛴 패턴들

| 함수 | 이유 |
|------|------|
| `ctx_find_fn` | TK_EOF 시 `""` 반환 |
| `ctx_find_sig` | not-found 시 `""` 반환 |
| `callers_collect_source` | base: `acc` (초기 `""`) |
| `impact_traverse` | visited 변수 흐름 복잡 |
| `clust_prefix` | `name` or slice — 빈 문자열 가능 |
| `resolve_tag_checks` | `sb_build` — 빈 입력 시 `""` |

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

2390 tests passed ✅

### trivials 추적

| 종류 | Cycle 3213 후 | Cycle 3214 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 244 | **238** | **-6** |
| **합계** | **278** | **272** | **-6** |
| String `post it.len() >= 1` | 142 | **148** | +6 |

**누적 진척**: 358 → 272 (-86, 24.0%)

## Reflection

**Scope fit**: 6개 업그레이드. `pack_result` 체인 패턴 확립.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- 남은 238개 분석 계속. 많은 함수가 특정 탐색 결과("" 반환) 패턴이라 skip 확률 높음
- `index_scan_body`, `index_struct_field/fields` 등 체인 탐색 필요

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 24.0% 달성.

## Carry-Forward

- **Actionable**: M11-A Phase 5f — 남은 238개 추가 탐색
  - `index_scan_body` 등 parse-chain 함수들 (항상 `pack_result` 반환 가능성)
  - 더 많은 codegen chain 탐색
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→272 (-86, 24.0%)
- **Next Recommendation**: M11-A Phase 5f — index_* parse 체인 탐색 OR 언어 갭 작업 평가
