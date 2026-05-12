# ISSUE-20260413 — SMT/Z3 파이프라인 활성화

**우선순위**: ~~P2~~ → **Deferred** (Cycle 382, 2026-04-13)
**영역**: verify, smt
**상태**: **Deferred** (정식 보류 — 재개 조건 하단 참조)

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-04-13 (deferral 결정) |
| `stale_after` | n/a (deferral 상태 — 재개 조건 충족 시까지 유효) |
| `measurement_source` | `bmb/src/smt/` (2,449 LOC) + `bmb/src/verify/` (1,067 LOC) 인벤토리 |
| `observed_rate` | "93% 달성" (llvm.assume propagation 기준 — 자동 safety check 제거율) |
| `scope` | verify pipeline 단독 (정상 빌드 영향 없음) |
| `env_hash` | n/a (구현 인벤토리 기반, 측정 환경 무관) |

## 판정 요약

- **93%는 이미 달성** (`llvm.assume` + MIR fact propagation + CIR/PIR)
- 남은 7%는 PhD급 형식검증 영역 (quantified formulas, 비선형 산술)
- 현재 투자 대비 효용 낮음 — **재개는 실수요 발생 시**

## 배경

`bmb/src/smt/` (2,449 LOC)와 `bmb/src/verify/` (1,067 LOC)가 구현되어 있으나 **정상 빌드 파이프라인에서 호출되지 않음** (`bmb verify`는 별도 명령). 현재 계약은 `llvm.assume`으로 전파되어 LLVM opt가 93% safety check를 제거.

## Defer 사유

| 축 | 상태 |
|----|------|
| EXISTENTIAL 7/7 | ✅ 충족 |
| Panic 제거 | ✅ 28→2 (93%) |
| 성능 영향 | 현재 경로 이미 최적 — SMT 통합 시 **컴파일 시간 증가**, 런타임 이득 희박 |
| Decision Framework | 언어 스펙(1)/컴파일러 구조(2)/최적화 패스(3)에서 이미 해결. SMT는 4-5 영역에 속함 — 현재 스케줄 적정 |

요약: "SMT 통합"은 자체로 정당한 기능이나, **현 우선순위는 성능 측정 인프라(`@bench`)와 SIMD 1급 타입**에 있음. SMT 재활성화는 이들 완료 후 수요 발생 시점에 재평가.

## 재개 조건 (Resumption Criteria)

아래 중 1개 이상 충족 시 재오픈:

1. **실수요**: 사용자/기여자가 `llvm.assume`으로 증명 불가한 계약 예제를 제시 (ex: `forall x. f(x) > 0`)
2. **성능 기회**: SMT 사전 증명으로 제거 가능한 핫패스 safety check 발견 (IR 분석으로 확인)
3. **안정성 요구**: 스펙에 "증명된 계약만 `@trust` 가능" 정책 도입
4. **에코시스템 요구**: stdlib 구현체가 SMT-검증된 불변식을 요구

## 남은 코드의 처분

- `bmb/src/smt/` (2,449 LOC): **유지** (회귀 방지, 재활성화 자산)
- `bmb/src/verify/` (1,067 LOC): **유지** (`bmb verify` 별도 명령으로 잔존)
- `bmb verify` CLI: **유지** (Z3 존재 시 동작, 미설치 시 skip — 현재 거동 유지)
- 테스트: 기존 스모크 테스트 유지

## 원래 해결 방안 (참고용, 재개 시 이 섹션 복원)

~~1. `bmb verify` 명령 활성화 — CIR → SMT-LIB2 → Z3 호출~~
~~2. 제약: 복잡 quantified formulas, 비선형 연산~~
~~3. 증명 결과를 `.proofcache`에 저장 (ProofDatabase 이미 존재)~~
~~4. 증명된 계약은 PIR을 거쳐 MIR fact로 전파 (이미 동작)~~

## 관련 문서 업데이트

- `docs/ROADMAP.md` — Phase X SMT 섹션에 Deferred 표기
- `CLAUDE.md` — "미구현 핵심" 목록에서 SMT 제거, "Deferred" 섹션으로 이동
- `HANDOFF.md` — §4 P2 순위에서 제거, Deferred 섹션 신설

---

## Close Resolution (Cycle 2755, 2026-05-12)

**Moved to closed/** — 본 ISSUE는 2026-04-13 Cycle 382에서 **Deferred** 결정. 1+ 년 누적 진척 없음. Closed 상태로 이관하여 active backlog 정리.

**재개 조건** (원 ISSUE 본문 참조 — 변경 없음):
- 외부 사용자가 contract 자동 증명 기능 요청
- `@bench` 인프라 + SIMD 1급 타입 완료 후 SMT 재활성화 우선순위 평가
- M5 언어 완성도 트랙 완료 시 (M5-5g 이후) 재검토 가능

**현 상태 (Cycle 2755)**:
- `bmb verify` 명령은 그대로 유지 (regression 위험 없음)
- M5 트랙 진척으로 verify 가능 contract 종류 확장 가능성 — 별도 ISSUE로 재개 시 재등록
