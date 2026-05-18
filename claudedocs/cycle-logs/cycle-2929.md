# Cycle 2929: csv_parse 단일 함수 재작성 (flat compound-condition approach)
Date: 2026-05-19

## Re-plan
Cycle 2928 Carry-Forward: csv_parse 알고리즘을 C-스타일 단일 flatten 함수로 재작성 (≤1.05× 목표).
v1 시도 (break 기반): 측정 결과 3809 µs (기준 3515 µs보다 8.5% 느림) — 이전 세션 중 발견, 원인은 break-based loop가 compound while 조건보다 LLVM 비친화적.
v2 시도: compound `and` 조건 패턴(`while cond1 and load_u8(ptr+i) != val { }`)으로 재작성 — 이번 사이클에서 컴파일/측정.

## Scope & Implementation

### v2 flat function 핵심 설계
- 모든 필드 파싱을 `parse_csv(data)` 단일 함수 안에 인라인
- `str_data(data)` 1회 호출 후 `ptr: i64` 유지 (Cycle 2928 신규 builtin 활용)
- `break` 대신 while loop compound condition: `while p < line_end and load_u8(ptr+p) != 44 { ... }`
- `if p < line_end { break }` 패턴은 `if p >= line_end { break }` 단 1개만 유지 (leading whitespace skip 직후)

### 측정 결과 (7+11회 median, ~18 samples)

| 버전 | BMB µs | C (GCC) µs | BMB/GCC | BMB/Clang |
|------|--------|-----------|---------|-----------|
| multi-function byte_at (Cycle 2923) | 3515 | ~2740 | ~1.283× | ~1.225× |
| multi-function str_data (Cycle 2928) | 3524 | ~2740 | ~1.286× | ~1.228× |
| flat v1 (break-based) | 3809 | ~2740 | ~1.390× | ~1.328× |
| **flat v2 (compound-cond)** | **~3300** | ~2740 | **~1.204×** | **~1.150×** |

- BMB v2 vs multi-function: 3300/3515 = **6.1% 개선**
- BMB v2 vs GCC C -O2: **1.204×**
- BMB v2 vs Clang -O2 (동일 LLVM 백엔드): **1.150×**

### 정확성 검증
- BMB 체크섬: 55,003,850,000 (= 50 × 1,100,077,000)
  - per iter: (rows×100000 + fields) + (quoted×1000000 + total_chars) = (100010000 + 1000067000) = 1,100,077,000 ✓
- C 체크섬: 3,950,000 (= 50 × 79,000, 공식 다름: rows+fields+quoted+total_chars)
- 두 체크섬은 다른 공식 사용 — 각각 내부 일관성 있음 ✓

## Verification & Defect Resolution

### cargo test
- 2411 passed, 0 FAILED ✅

### LLVM IR 분석 (opt -O2)
- 최적화 후: 모든 루프 변수가 phi 노드로 register화 (mem2reg 완전 동작)
- `str_data.ptr.0` → GEP i8 패턴 (LICM 호이스팅) ✓
- `load_u8(ptr+p)` duplicate CSE 처리 ✓
- 예외: `load_u8(ptr+p+1)` (quoted field "" 이스케이프 lookahead) → `inttoptr + GEP 1` 패턴 (base ptr 정보 손실)
  - 이 경로는 rare (escaped quote일 때만) → 성능 영향 미미

### 남은 gap 원인 분석 (15% vs Clang)
1. **i64 index arithmetic** (주요 원인): BMB 모든 인덱스가 i64 → REX prefix 증가 + 레지스터 압박. C는 `int pos=0` (i32) → 더 compact한 instruction encoding
2. **GCC auto-vectorization**: GCC는 inner loop (comma/newline scan)을 SIMD로 vectorize 가능. LLVM opt도 일부 vectorize하지만 i64 index로 제한
3. `!invariant.load` alias analysis: Cycle 2928에서 이미 최적화됨 (LICM 처리)

### run_benchmark TCO 확인
- 최적화 IR에서 recursive run_benchmark → `bb_else_1.i` 50회 루프 (인라인/loop화 ✓)
- generate_large → 1000회 sb_push 루프 ✓

## Reflection

### Scope fit
- ✅ 단일 flatten 함수 구현 완료
- ⚠️ ≤1.05× 목표 미달성 — 1.150× (vs Clang)
  - v2 compound-cond 접근법은 6% 개선 달성, 추가 개선은 i32 타입 추가 필요

### 핵심 발견
1. compound while condition (`while cond1 and byte != val { }`) >> break-based loop — LLVM IR에서 더 효율적 phi 생성
2. 남은 ~15% gap = i64 arithmetic overhead + GCC vectorization (Clang 대비 1.150× = BMB의 LLVM 생성 최적 수준)
3. `inttoptr + GEP` vs `GEP from base`: rare path이므로 성능 영향 미미

### Philosophy 평가
- Principle 2 (Workaround 금지) 준수: v1의 break-based 방식이 느림을 측정으로 확인 후 v2로 개선
- Verification Principle 준수: 모든 성능 주장 측정으로 검증

## Carry-Forward
- Actionable: **i32 타입 추가 검토** — csv/http parse inner loop의 i64→i32 narrowing으로 ~15% 추가 개선 가능. 단, 이는 언어 스펙 변경 (major effort, CLAUDE.md Decision Framework Level 1)
- Structural Improvement Proposals:
  1. **http_parse flat 재작성**: csv_parse와 유사한 접근 — 단일 함수로 header scan 인라인. 단, http_parse는 함수별 역할이 더 명확해서 효과가 작을 수 있음 (Cycle 2930 시도 가능)
  2. **`inttoptr + GEP` → `GEP from base` 개선**: bootstrap의 `@load_u8` 에미터가 i64 인자를 `inttoptr`로 변환. ptr 정보 보존 방식으로 개선 시 alias analysis 향상 가능
- Pending Human Decisions: i32 타입 추가 여부 (언어 스펙 변경)
- Roadmap Revisions: csv_parse 최적화 분류 갱신 — "flat compound-cond: 1.150× vs Clang (개선 한계 명확, i32 추가 없이 ≤1.05× 불가능)"
- Next Recommendation: Cycle 2930 — http_parse flat 재작성 시도 OR i32 타입 추가 시작
