# BMB Session Handoff — 2026-05-27 (Cycles 3212-3218)

> **HEAD**: `606c4ebc`
> **이번 세션 작업**: Cycles 3212-3218 — **M11-A Phase 5c-5j: trivial postconditions 27개 교체**
> **M11-A 상태**: 🔵 수익 체감 확인 (358 → 263 trivials, -26.5%)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **M10 상태**: ✅ **COMPLETE** (이전 세션)
> **Stage 2 상태**: ✅ **RECOVERED** (Cycle 3202)
> **0-Warning 상태**: ✅ **유지** (lint 0 warnings)
> **M11-B 상태**: ✅ **COMPLETE** (Cycle 3205)

---

## 이번 세션 작업 요약 (Cycles 3212-3218)

### M11-A Phase 5c-5j: 추가 trivial postcondition 교체

7개 사이클에 걸쳐 27개 trivial postcondition을 의미있는 계약으로 교체.

#### 변경 요약

| 사이클 | 내용 | 변경 수 |
|--------|------|---------|
| 3212 | gen_assumes/hot/stats 체인 (String 4개) | 4 |
| 3213 | contract/lr2l/cx_most/repl 체인 (String 8개) | 8 |
| 3214 | pack_result/parse_source/lambda 체인 (String 6개) | 6 |
| 3215 | index_* parse 체인 전체 (String 7개) | 7 |
| 3216 | get_fn_return_scan (String 1개) + 광범위 분석 | 1 |
| 3217 | has_param_ref_in_ir (bool 1개) + semantic_duplication 분석 | 1 |
| 3218 | i64 7개 탐색 → 전부 skip (산술/추출 함수) | 0 |
| **합계** | | **27** |

#### 발견된 패턴

1. **pack_result 체인**: `pack_result(pos, ast)` → `post it.len() >= 2` 이므로 이를 반환하는 모든 함수가 `>= 1`. `index_*`, `query_*`, `callers_collect_fn` 등 7개 일괄 업그레이드.

2. **repl compile 체인**: `compile_program` → `post it.len() >= 1`. `repl_try_*` 전체 체인 자동 업그레이드.

3. **semantic_duplication bool 제약 심화**: post expression 내 파라미터 변수명이 같으면 함수명이 달라도 충돌. 대부분 bool 함수가 `not it or pos < X.len()` 같은 패턴을 공유해 차단됨.

4. **i64 trivials 전부 skip**: 산술(+,-,*,shift,bitwise) 및 파싱 함수 — 전체 i64 범위 반환.

#### M11-A 최종 상태

| 종류 | 세션 전 | 현재 |
|------|---------|------|
| bool `post it or not it` | 27 | **26** |
| i64 `post it == it` | 7 | 7 |
| String `post it.len() >= 0` | 231 | **230** |
| **합계** | **265** | **263** |
| String `post it.len() >= 1` | 155 | **156** |

**누적 진척**: 358 → 263 (-95, **26.5%**)

#### skip 확정 목록 (변경 금지)

| 종류 | 수 | 이유 |
|------|---|------|
| bool 충돌군 | ~20 | semantic_duplication (mn_has_memory_op/ipr_has_store/check_fn_in_list/layer_is_leaf) |
| bool "all" 패턴 | ~5 | base returns true → post `not it or pos < X.len()` 불성립 |
| bool 분석 함수 | ~4 | 자연스러운 semantic post 없음 |
| i64 all 7개 | 7 | 산술/추출, 음수 포함 — skip 확정 |
| String accumulator | ~60 | base: `acc` (초기 "") |
| String lookup | ~40 | not-found 시 "" |
| String pass-through | ~30 | 입력 반환 |
| String sb_build | ~50 | 빈 입력 시 "" |
| String no-pre 77개 | 77 | 확인 완료 skip |

---

## 다음 세션 시작점

### 추천 작업 순서 (우선순위)

| 순위 | 작업 | 예상 임팩트 | 비고 |
|------|------|------------|------|
| **1** | **언어 갭 작업(M11-C)** | 높음 | stack array / closure capture / generic 미지원 기능 |
| 2 | ifs_flex_check_goto 수정 | 낮음 | `pre next_p >= 0` 추가 → Z3 141/141 달성 |
| 3 | M11-A 잔여 탐색 | 매우 낮음 | hit rate < 0.4%, 수익 체감 확인됨 |

**M11-A 수익 체감 명확**: 이번 세션 7개 사이클 → 27개 발굴. **다음 세션은 M11-C 전환 강력 추천**.

### M11-C 언어 갭 작업 후보

ROADMAP § M11-C 또는 아래 갭 중 사용자 결정:

| 갭 | 현황 | 접근 |
|----|------|------|
| Stack array `[T; N]` | 미지원 | 파서 + MIR + codegen + bootstrap |
| Closure capture (upvalue) | 부분 지원 | MIR capture analysis |
| Generic fn bootstrap | 부분 지원 | type param 처리 강화 |
| `ifs_flex_check_goto` Z3 | pre-existing 1 FAIL | `pre next_p >= 0` 단순 추가 |

### 기술 상태 스냅샷

| 항목 | 값 |
|------|----|
| chained_comparison | **0** ✅ |
| missing_postcondition | **0** ✅ |
| 총 warnings | **0** ✅ |
| Stage 1 bootstrap | ✅ |
| Stage 2 bootstrap | ✅ |
| bootstrap.sh fixed_point | ✅ **true** |
| BMB-internal Fixed Point | ✅ |
| 테스트 | 2390 passed ✅ |
| M11-A trivials | 358 → 263 (-95, 26.5%) 🔵 |

---

## 알려진 미결 사항

- **trivial postconditions**: 263개 잔여 (원래 358개 대비 -95). M11-A 수익 체감 확인.
- **ifs_flex_check_goto**: `post it >= 0` Z3 실패 (pre-existing) — `pre next_p >= 0` 누락.
- **BMB IR → opt 최적화 불가**: printf 기반 IR 방출 코드가 opt O1+ 적용 시 절단됨.
- **lint.exe 재빌드 불가**: `bootstrap/lint/lint.exe`의 PHI node IR 오류.
- **Unix 링크 스택 미설정**: `bootstrap.sh` Unix 브랜치 carry-forward.

---

## 핵심 기술 메모

### semantic_duplication 제약 (bool) — 상세

bool 함수의 semantic_duplication 충돌 규칙:
- **타입 비교**: 파라미터 TYPES (이름 아님)로 그룹화
- **pre text 비교**: 리터럴 텍스트 일치
- **post text 비교**: 리터럴 텍스트 일치 (변수명 포함!)

충돌 탈출: post expression의 변수명이 다르면 text 불일치 → 충돌 없음
예: `not it or pos < ir.len()` ≠ `not it or pos < s.len()` (ir vs s)

### `is_int_literal` 범위 주의 (핵심!)

```bmb
fn is_int_literal(kind: i64) -> bool =
  kind < 2000000000 + 100 or kind >= 2000000000 + 1000;
```

**2000000100–2000000999 범위는 제외!** (토큰 상수 범위)

### bootstrap.sh opt 제거 (핵심!)

Stage 2 바이너리 컴파일: `opt` 사용 금지, `llc -filetype=obj -O2`만 사용.
