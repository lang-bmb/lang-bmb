# Cycle 2943: ROADMAP/HANDOFF/CLAUDE.md 갱신 + @inline 패턴 문서화
Date: 2026-05-19

## Re-plan
Cycle 2942 carry-forward: ROADMAP P축 현황 갱신 + CLAUDE.md @inline 패턴 문서화.
성능 개선 결과를 영속적으로 기록하고 다음 세션을 위한 상태를 정리.

## Scope & Implementation

### 1. claudedocs/ROADMAP.md 갱신
- 최상단 헤더: Cycles 2939-2942 요약 추가 (언어 갭 + 성능 개선)
- § 5 P축 현황: "real-world 7/7: 6개 BMB faster, 1개 ≤1.06×" 기록
- HEAD: `797d7e3f`

### 2. claudedocs/HANDOFF.md 재작성
- Cycles 2939-2942 세션 작업 요약
- 성능 현황 테이블 (7/7 벤치마크 전체 결과)
- 핵심 변경 사항 문서화
- 다음 사이클 진입점 설정

### 3. CLAUDE.md @inline 패턴 추가
"Benchmark Cycle Guidelines" 섹션에 "### @inline 전략 (Cycles 2941-2942 확립)" 추가:
- LLVM 인라이닝 임계값 초과 함수 탐지 조건
- 적용 사례 (http_parse, brainfuck)
- 컴파일러 결함 vs 사용자 최적화 구분
- BMB가 느린 경우 결정 트리에 @inline 경로 추가

**변경 파일**:
- `claudedocs/ROADMAP.md`
- `claudedocs/HANDOFF.md`
- `CLAUDE.md`

## Verification & Defect Resolution

변경 사항은 문서 갱신만이므로 기능 검증 불필요.
cargo test --release: 2388 PASSED (이전 Cycle에서 확인)

### 결함 없음

## Reflection

### Scope fit
- ✅ ROADMAP 성능 현황 정확하게 반영
- ✅ CLAUDE.md에 재사용 가능한 @inline 패턴 문서화

### 세션 성과 요약 (Cycles 2939-2942)

| 지표 | 이전 | 이후 |
|------|------|------|
| let (a,b) Rust interp | ❌ | ✅ |
| str_byte_at native | ❌ | ✅ |
| println(String) native | ❌ | ✅ |
| csv_parse | 1.204× | 1.057× |
| http_parse | 1.099× | 0.947× BMB faster |
| brainfuck | 1.274× | 0.949× BMB faster |
| 전체 real-world | ~3/7 BMB faster | **6/7 BMB faster** |

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  1. csv_parse 1.057× 추가 최적화: native memset API 추가 → calloc-per-iter 제거
  2. inttoptr UB (P3 flakiness) — Option A codegen 전환 (5-10 cycle scope)
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP P축 현황 갱신 완료
- Next Recommendation: Cycle 2944 — csv_parse 추가 최적화(memset API) 또는 새 언어 갭 탐색
