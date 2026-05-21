# Cycle 2737: BENCHMARK_REPORT.md stale warning + ROADMAP redirect

Date: 2026-05-11

## Re-plan

인계 Carry-Forward: "Tier all 결과 분석 (Cycle 2729 시작, 진행 중)" — 백그라운드 bench 47분 경과, array_* 진행 중 (50% 추정, 1-2시간 더). Trigger: 🟡 **SCOPE ADJUST** — 의존 작업 회피, 가벼운 doc 작업으로 pivot.

후보 검토:
- SMT-integration formal close: skip (이미 Deferred 명시, visibility 손실 위험)
- multiple-pre-clauses 파서 fix: skip (Rule 6 — compiler.bmb 필요 + bootstrap 헤비, bench 충돌)
- BENCHMARK_REPORT.md stale audit: **선택** (pure doc, bench와 무관, 외부 가시성)

## Scope & Implementation

### Stale 진단

두 파일 모두 3.5개월 stale:

| 파일 | 측정 일자 | BMB 버전 | 현재(2026-05-11) gap |
|------|----------|---------|-----|
| `ecosystem/benchmark-bmb/BENCHMARK_REPORT.md` | 2026-01-25 | v0.51.22 | 3.5개월 + ~47 patch 버전 |
| `ecosystem/benchmark-bmb/results/BENCHMARK_REPORT.md` | 2026-01-21 | v0.50.51 | 3.5개월 + ~48 patch 버전 |

### 갱신 정책

전면 재생성이 아닌 **stale warning + ROADMAP redirect** 패턴 적용:
- ROADMAP § 5 P-track에 이미 v0.98 측정 표 있음 → 단일 진실 원천
- BENCHMARK_REPORT는 historical snapshot으로 보존 (v0.51.22 era 비교 가능)
- 신규 의사결정은 ROADMAP에서 시작하도록 prominent warning + 핵심 변화 요약

### 핵심 변화 강조 (v0.51.22 → v0.98)

| Benchmark | v0.51.22 ratio | v0.98 ratio | 개선 |
|-----------|---------------|------------|------|
| sorting | 1.10x | **0.910x** | **9% BMB FASTER** (19 pp 개선) |
| lexer | 1.09x | 1.000x | parity (9 pp) |
| brainfuck | 1.11x | 1.036x | 7.4 pp |

→ ROADMAP § 5 measurements와 일관성 확인 (Cycle 2725 검증값).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 두 파일 stale 경고 prominent | ✅ |
| ROADMAP redirect 명시 | ✅ |
| 핵심 데이터 차이 강조 | ✅ |
| historical 내용 보존 | ✅ (전체 본문 유지) |
| bench 실행 영향 | ✅ 없음 (doc only) |

결함: 없음.

## Reflection

### 갱신 vs redirect 선택의 정당성

ROADMAP § 5에 이미 v0.98 측정 표 (P 축 Tier 1/3 historic + inproc 4 도메인) 존재. 전면 재생성은:
- 중복 데이터 동기화 부담 (어느 한쪽이 stale 되면 sync 필요)
- 변경 시점마다 두 파일 모두 갱신해야 함 (양식 표준화 정신과 반대)

stale warning + redirect는:
- 단일 진실 원천 (ROADMAP) 강화
- historical context 보존 (v0.51.22 era 비교 가능)
- ISSUE 양식 표준화 원칙 (`measurement_date` + `stale_after`) 적용

### 외부 가시성 leverage

`ecosystem/benchmark-bmb/`는 외부 사용자/contributor가 처음 보는 디렉토리. stale 경고 prominent 표기로 misleading 데이터 방지.

## Carry-Forward

- Actionable: 없음 (Cycle 2737 자체 완결)
- Structural Improvement Proposals: 없음 (redirect 패턴 충분)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (ROADMAP § 5가 이미 v0.98 측정값 보유)
- Next Recommendation: Cycle 2738 — ISSUE backlog deep audit (v0.98 재현 시도 + close 후보 발굴)
