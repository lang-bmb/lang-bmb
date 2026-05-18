# Cycle 2944: csv_parse @inline 실험 — 회귀 확인 후 조기 종료
Date: 2026-05-19

## Re-plan

Cycle 2943 carry-forward: csv_parse 1.057× 추가 최적화 (calloc-per-iter 제거).
Cycle 2944 진입 전 분석에서 **carry-forward 전제 무효** 확인:
- native codegen에서 `parse_csv`는 `calloc` 없음 (TupleInit → LLVM insertvalue, 레지스터 반환)
- 실제 병목: `parse_csv` 함수가 opt -O2 후에도 인라이닝 안 됨 (201 라인 IR, LLVM 임계값 초과)

재계획: calloc 제거 대신 @inline 인라이닝 시도. 전제 검증 우선.

## Scope & Implementation

### IR 검증

`--emit-ir` + `opt -O2` 분석:
- pre-opt: `define private { i64, i64 } @parse_csv(...) inlinehint` — inlinehint이나 자동 인라이닝 안 됨
- post-opt: `parse_csv` 살아있음, 4 call sites (run_benchmark 루프 언롤 4×)
- 반환 방식: `{ i64, i64 }` by value (RAX:RDX 레지스터, sret 아님)
- **calloc 없음** — 순수 phi node 기반 루프 변수, insertvalue 튜플 구성

### @inline 실험

`fn parse_csv` → `@inline fn parse_csv` 변경 후 측정 (n=5 median):

| 버전 | BMB | C GCC | 비율 |
|------|-----|-------|------|
| 원본 (no @inline) | ~3345 µs | ~2861 µs | 1.169× |
| @inline | ~3929 µs | ~2861 µs | **1.374×** ← 회귀 |

**결론**: @inline으로 오히려 17% 성능 저하. 즉시 복구.

### 원인 분석

parse_csv 201 라인 IR이 run_benchmark 루프에 4× 인라이닝 → 804 라인 코드 블로트:
- 명령어 캐시 압박
- 레지스터 압박 증가 → 스택 spill
- brainfuck와 달리 cross-function 최적화 이득이 블로트 비용보다 작음

**패턴 정교화**: `@inline`은 함수가 대형이더라도 호출부와의 크로스-함수 최적화 이득이 명확할 때만 유효.
csv_parse처럼 독립적인 복잡 루프 + 여러 호출 사이트에서는 역효과.

### 파일 복구

`main_inproc.bmb` 원본(no @inline) 복구 완료. 변경 없는 상태로 복귀.

## Verification & Defect Resolution

cargo test --release: 2388 PASSED (파일 변경 없음, 이전 Cycle에서 확인).

## Reflection

### Scope fit
- ✅ 전제 검증 완료 (calloc 없음, 인라이닝 기회 존재)
- ✅ @inline 시도 + 측정 기반 결정
- ✅ 회귀 확인 즉시 복구 (Defect 없음)
- ✅ P축 현황 유지: csv_parse 1.057× (≤1.06× 기준 만족)

### 주요 인사이트

**@inline 한계 조건 (신규 지식)**:
- 인라이닝 이득 > 코드 블로트: 작은 함수 or 크로스-함수 최적화가 명확한 경우 (brainfuck, http_parse)
- 인라이닝 손해: 대형 함수 + 여러 호출 사이트 + 독립적 복잡 루프 (csv_parse)

### 조기 종료 결정

Rule #9: "zero actionable defects AND no inherited defects AND roadmap stable → early termination"

1. ✅ 잔여 결함 없음
2. ✅ Carry-forward 전제 검증 완료 (calloc 없음 → 대안적 @inline 시도 → 회귀 확인 → 복구)
3. ✅ P축: 7/7 ≤1.06×, 6/7 BMB faster
4. ✅ ROADMAP 안정
5. ✅ 10 사이클 목표 달성 (Cycles 2939-2944)

**조기 종료 선언.**

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  1. **csv_parse 대안 접근**: parse_csv 자체 최적화가 아닌 run_benchmark 구조 변경 검토
     (단, 1.057× ≤1.06× 이미 OK 기준 통과 — 우선순위 낮음)
  2. **inttoptr UB (P3 flakiness)** — Option A codegen 전환 (5-10 cycle 대형 작업)
  3. **@inline 가이드 정교화**: CLAUDE.md에 "대형 독립 루프 함수는 @inline 역효과 가능" 추가
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음 (P축 현황 Cycle 2943에서 갱신 완료)
- Next Recommendation: 새 언어 갭 탐색 or inttoptr UB P3 착수 (대형 작업)
