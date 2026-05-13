# Cycle 2808: ROADMAP M4-3/M4-4/M4-5 완료 동기화 + --check-only CI 제안 폐기
Date: 2026-05-13

## Re-plan

Cycle 2807 Carry-Forward 검토:
- Structural Improvement Proposals: "Wire `--check-only` into CI; cross-platform stack check (version stamp)"
- Next Recommendation: Cycle 2808 — Active ISSUE 11 backlog 또는 bootstrap parser iterative (P3)

**`--check-only` CI 제안 실증 무효화 (STEP 0 조사)**:

Advisor 호출 + 실증 조사 결과:
1. `git ls-files bootstrap/compiler.exe` → 빈 출력 — exe가 git에 없음
2. CI fresh checkout → `is_stale()` 에서 `[ ! -f "$BOOTSTRAP_EXE" ]` → 항상 true
3. `--check-only`는 stale 시 exit 1 → CI에서 **항상 실패**
4. `check_stack_mb()`는 PE32+ 헤더 파싱 → Linux ELF에서 0 반환 (ubuntu-latest 운영)
5. `bootstrap.sh`와 `rebuild-bootstrap-exe.sh`는 독립 스크립트 — `bootstrap.sh`는 `rebuild-bootstrap-exe.sh`를 호출하지 않음

**결론**: `--check-only` CI 연동은 exe가 git-tracked 되지 않는 한 구조적으로 불가능. 제안을 **REJECTED**로 명시 폐기.

**Scope pivot**: 실제 문서화 결함 발견 — ROADMAP.md의 M4-3/M4-4/M4-5 항목에 ✅ 누락. CLAUDE.md는 완료(2026-05-10)를 명시하지만 ROADMAP은 "2-3 cycles" 소요 예정 상태로 유지됨. 이번 사이클 범위: ROADMAP 동기화.

STEP 1: Skip — 문서 편집, 패턴 추종 변경, 설계 불필요.

## Scope & Implementation

### 사이클 로그 귀속 검증

M4-4 (`cycle-2620.md`): "M4-4 Static Method Call 구현" — `Type::method(args)` → `Type_method(args)` 망글링, 골든 테스트 통과 ✅
M4-3 (`cycle-2621.md`): "M4-3 Let-Tuple Destructuring 구현" — `let (a, b) = expr` 데슈가링, 두 파싱 경로(표현식/블록) 모두 처리, 골든 테스트 통과 ✅
M4-5 (`cycle-2633.md`): "M5-1 Payload Enum — Core Implementation" — `Option::Some(42)` heap calloc 2-word 표현, match arm payload binding, Stage 1 + 골든 테스트 통과 ✅ (레이블 명칭이 M5-1이지만 CLAUDE.md가 M4-5 기능으로 귀속)

### ROADMAP.md 수정

`claudedocs/ROADMAP.md` M4 준비 태스크 테이블:
- `M4-3`: `2-3 cycles` → `✅ Cycle 2621`
- `M4-4`: `2-3 cycles` → `✅ Cycle 2620`  
- `M4-5`: `1-2 cycles` → `✅ Cycle 2633`

### `--check-only` 제안 처리

Cycle 2806/2807 Structural Improvement Proposals의 "Wire `--check-only` into CI" 항목을 이 로그에서 **REJECTED**로 종결:
- 이유: exe 미 git-tracked, PE32+ Linux 불호환
- 잠재적 대안: exe를 CI artifact로 캐싱하거나 git-tracked 바이너리로 관리 — 별도 ISSUE 등록 없이 제안 수준에서 종결 (P4 미만 우선순위, 현 인프라와 미적합)

## Verification & Defect Resolution

ROADMAP.md 편집은 문서 동기화 — 빌드/테스트 불필요. 귀속 검증은 사이클 로그 3개 실독으로 완료.

결함 없음.

## Reflection

**Scope fit**: 완료. ROADMAP와 CLAUDE.md 간 불일치 해소.

**Latent defects**: 없음.

**Structural improvement opportunities**: M4-5는 사이클 로그에서 "M5-1"로 레이블되지만 ROADMAP/CLAUDE.md에서는 "M4-5"로 참조됨 — 명칭 불일치. 기능 완료 사실은 명확하므로 추가 정정 불필요.

**Philosophy drift**: 없음.

**Roadmap impact**: M4 준비 태스크 테이블이 실제 완료 상태를 반영. 다음 미완료 M4 항목: M4-1 (B축 공식 측정, API key 필요 — HUMAN-blocked), M4-6 (C# 바인딩, multi-cycle), M4-9 (knapsack outlier 분석, 장기). Active ISSUE 11 backlog가 더 실행 가능한 후보.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - `--check-only` CI 연동은 exe를 CI artifact 캐싱 또는 git-tracked 바이너리로 관리하는 인프라 변경 없이는 구현 불가 — 현재 P4 이하로 보류
  - M4-5/M5-1 레이블 명칭 불일치: 기능 완료 확인으로 충분, 별도 정정 불필요
- Pending Human Decisions: None
- Roadmap Revisions: M4-3, M4-4, M4-5 완료 마킹 완료
- Next Recommendation: Cycle 2809 — Active ISSUE 11 backlog에서 선택 (HANDOFF.md 참조)
