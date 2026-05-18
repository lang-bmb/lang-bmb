# Cycle 2941: csv_parse break-loop + http_parse @inline — 성능 개선
Date: 2026-05-19

## Re-plan
Cycle 2940 이후 carry-forward 없음. ROADMAP P축 우선순위에 따라 csv_parse(1.204×)와
http_parse(1.099×) 성능 갭 개선 시도.

## Scope & Implementation

### 1. csv_parse: break 기반 quoted loop (v3)

**분석**: `in_quote` 플래그를 사용하는 while 조건(`while p < len and in_quote == 1`)이
loop-carried phi node를 생성 → LLVM이 단순화하기 어려운 패턴.

C 버전은 `break`로 closing quote에서 즉시 탈출 → 더 단순한 제어 흐름.

**변경**: `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main_inproc.bmb`
```bmb
-- before: in_quote 플래그
let mut in_quote = 1;
while p < len and in_quote == 1 { ... in_quote = 0 or 1 ... };

-- after: break 기반
while p < len {
    if qc == 34 {
        if doubled_quote { ... }
        else { p = p + 1; break }  // C와 동일한 구조
    } else { ... }
};
```

### 2. http_parse: @inline on parse_http_flat

**분석**:
- `parse_http_flat`: 355개 assembly instruction (LLVM 인라이닝 임계값 초과)
- `parse_all_flat`: 5회 function call overhead × 10000 iterations = 50000 calls
- C: `parse_request` → `parse_all_requests` 전체 인라이닝 (`static const char*` 배열 + for loop)

**IR 분석**:
```
BMB parse_http_flat alone: 355 instructions
C parse_all_requests total: 297 instructions (5 inlined calls)
```

**변경**: `@inline fn parse_http_flat(...)` 추가
→ LLVM alwaysinline attribute → 5× inlined into parse_all_flat
→ parse_all_flat이 C의 parse_all_requests와 동일한 최적화 기회 확보

## Verification & Defect Resolution

### 성능 결과 (median of 8 runs)

| 벤치마크 | 이전 (BMB) | 이후 (BMB) | C GCC | 이전 비율 | 이후 비율 |
|---------|-----------|-----------|-------|---------|---------|
| csv_parse | ~3408 µs | ~3177 µs | ~3000 µs | 1.204× | **1.059×** |
| http_parse | ~2858 µs | ~2524 µs | ~2617 µs | 1.099× | **0.964×** |

**csv_parse**: 1.204× → **1.059×** (12% 개선)
**http_parse**: 1.099× → **0.964×** ← **BMB가 C보다 빠름** (4% BMB faster)

체크섬 검증: ✅ 두 벤치마크 모두 C와 동일한 checksum 출력

### 결함 없음

## Reflection

### Scope fit
- ✅ csv_parse 갭 크게 축소 (1.204× → 1.059×)
- ✅ http_parse BMB faster than C (1.099× → 0.964×)

### 의의
- **break 기반 루프**: loop-carried phi 제거 → LLVM 최적화 개선 패턴 확립
- **@inline 전략**: 함수가 너무 커서 LLVM 자동 인라이닝 임계값 초과 시 수동 @inline 필요
- **성능 > 1.00x의 근본 원인**: 불필요한 함수 call overhead + 최적화를 막는 루프 패턴

### 잠재적 추가 개선
- csv_parse: i64 vs C int(i32) 산술 — 현재 LLVM이 많은 부분 최적화하나 일부 잔여 가능
- http_parse: 이미 C를 초월. 추가 최적화 불필요.

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  1. **ROADMAP 갱신**: http_parse 0.964× (BMB faster) 기록
  2. **json_parse/json_serialize 갭 확인**: 다음 사이클 후보
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2942 — json_parse / json_serialize 성능 분석 또는 다른 언어 갭
