# ISSUE-YYYYMMDD — {간결 제목}

> **이 파일은 신규 ISSUE 작성 template이다.** 복사 후 파일명을 `ISSUE-{측정일}-{slug}.md` 형식으로 변경하라.
> 양식 변경 이력:
> - 2026-05-11 (Cycle 2730): measurement_date / stale_after / measurement_source / observed_rate / scope / env_hash 필수 필드 추가
> - 2026-05-12 (Cycle 2768): **estimated_cycles + hypothesis_until_verified** 필수 필드 추가 — cycle 2765/2766/2767 연속 estimate 갭 패턴 (3 cycle 연속 ISSUE 본문 추정과 실측 차이 ≥1.5x) 회귀 방지

## 핵심 메타

**우선순위**: P0 / P1 / P2 / P3 (P0=blocking M-축, P1=cycle 내 처리 가능, P2=multi-cycle, P3=long-term)
**영역**: codegen / runtime / bootstrap / stdlib / parser / typecheck / ci / docs / ecosystem
**상태**: Open / In Progress / Blocked / Closed (closed 시 `closed/` 디렉토리로 이동)

## 측정 stamp (필수)

> **2026-05-11 Cycle 2730 표준화**: 모든 측정 기반 ISSUE는 아래 필드 필수. 측정 기반 아닌 ISSUE (e.g., 언어 갭 spec)는 `n/a` 명시.

| 필드 | 값 | 비고 |
|------|-----|------|
| `measurement_date` | YYYY-MM-DD | 측정한 날짜 (commit date 아님) |
| `stale_after` | YYYY-MM-DD | 이 측정값이 stale로 간주되는 시점 (기본 +3개월) |
| `measurement_source` | 파일 경로 / 명령 | e.g., `target/benchmarks/v098-historic.json`, `scripts/benchmark.sh --tier 3 --runs 10` |
| `observed_rate` | % / ratio / count | e.g., `1.085x BMB slower`, `20% failure rate`, `41 inttoptr count` |
| `scope` | 단일 / 도메인 / 전역 | 영향 범위 — single test / single benchmark / N tests / all |
| `env_hash` | OS / LLVM / GCC 버전 | e.g., `win32 / LLVM 21.1.8 / MSYS2 UCRT64 / gcc MinGW` |

**측정 추이** (재측정 시마다 append, 가장 최근이 위):

| date | source | observed | 변화 |
|------|--------|----------|------|
| YYYY-MM-DD | path/cmd | X.XXX | (+/-N pp) |

## 문제

{1-3 문단으로 증상 기술. 재현 절차 포함. 최소 명령어 / 입력 / 기대값 / 실제값.}

## 핵심 증거

{데이터 + 인용. 가설 기각/확정 모두. 측정 기반이면 raw numbers, 코드 기반이면 grep 결과 / 인용.}

## 추정 root cause

{가설 — "확정 아님" 명시. 가능하면 1-3 옵션 비교.}

## 영향 평가

| 영역 | 영향 |
|------|------|
| CI | ... |
| 부트스트랩 | ... |
| 개발 마찰 | ... |
| M축 (B/P/A/D/C) | ... |

## 해결 방안 (옵션 비교)

> **Cycle 2768 강화**: 모든 `scope: N cycles` 추정은 **검증 전까지 가설**. 1 cycle 진단 cycle을 먼저 두고 추정 적합성 점검 권고. cycle 2765/2766/2767 패턴에서 ISSUE 본문 1-2 cycle 추정이 실측 3-7 cycle로 확장된 사례 누적.

### Option A: {proper fix}
- `estimated_cycles`: N **(hypothesis — verify via 진단 cycle)**
- 절차: ...
- 리스크: ...
- 검증 절차: 첫 cycle은 measurement / IR diagnosis only. 가설 적합성 확인 후 진행.

### Option B: {임시/우회}
- `estimated_cycles`: 1-2 **(hypothesis)**
- 절차: ...
- 트레이드오프: ...

## HUMAN 결정 필요

- 옵션 선택
- 우선순위 vs 다른 진행 중 작업
- 외부 도구/라이브러리 의존 여부

## 종결 기준 (close criteria)

- [ ] {측정 가능한 기준 1}
- [ ] {측정 가능한 기준 2}
- [ ] {재발 방지 — golden test / lint rule / CI gate 추가}

## 메타

- 관련 ISSUE: ...
- 인용 cycle: cycle-NNNN.md
- 외부 참조: docs URL, RFC 등

---

## 양식 보존 가이드 (보존 — 삭제 금지)

1. 측정값 변경 시: 측정 추이 표에 row append, `measurement_date`/`source` 갱신
2. `stale_after` 도달 시: `Blocked — stale data` 상태 전환 → 재측정 후 close 또는 갱신
3. `observed_rate` 변화 5pp 이상 시: 우선순위 재검토
4. close 시: `closed/` 디렉토리 이동 + 종결 cycle 명시
5. 측정 기반 아닌 ISSUE (e.g., 언어 갭 spec, 문서 누락): 측정 stamp 필드는 `n/a` 명시하되 다른 메타데이터(영역/상태/추정 소요)는 채울 것
6. **Cycle 2768 강화 — `estimated_cycles` 검증**: 모든 옵션의 cycle 추정은 가설. 1 cycle 진단 cycle (측정/IR 분석)을 먼저 두고 가설 적합성 확인 후 implementation 진행. 추정과 실측 차이 ≥1.5x 시 ISSUE 본문 갱신 + 우선순위 재평가.
