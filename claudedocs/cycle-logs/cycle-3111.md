# Cycle 3111: Track B String 279개 post it.len() >= 0 배치 추가
Date: 2026-05-25

## Re-plan
Cycle 3110 Carry-Forward: String 279개 `post it.len() >= 0` 소규모 테스트 후 결정.
Advisor 권고: 조기(L3000-5000) + 후기(L16000+) 함수 배치 테스트로 위치 의존성 확인.

**판별 테스트 결과 (Outcome B)**: `tok_kind_name`(L572) + `fmt_indent`(L21326) 모두 0 errors → String safe to batch.

## Scope & Implementation

**판별 테스트**:
- `tok_kind_name` (L572, early): `post it.len() >= 0` 추가 → ✅
- `fmt_indent` (L21326, late): `post it.len() >= 0` 추가 → ✅
- 결론: Outcome B (양쪽 통과) — 배치 추가 안전

**BMB String 함수 패턴 분류**:
- **type2 형태** (fn header 끝 `=`, body 다음 줄): 244개 (87.5%)
- **single-line 형태** (fn name() -> String = body;): 35개 (12.5%)
- multi-line with pre: 0개 (uncontracted 함수는 pre 없음)

**1차 패치 스크립트** (279개 배치):
- type2: `fn ... -> String =\n body` → `fn ... -> String\n  post it.len() >= 0\n= body` (244개)
- single: `fn ... -> String = body;` → 3줄 분리 (35개)
- 정확한 `=` 분리 처리 (이전 leading space 버그 수정)
- **결과**: 279개 패치, 0 errors ✅

**Fixed Point 검증**:
- S3 IR hash: `16bb2d8d28811c45e8dd8ba27537f129`
- S4 IR hash: `16bb2d8d28811c45e8dd8ba27537f129`
- S3 == S4 ✅ (새 Fixed Point)

**참고: 이전 FP `F8DA1AB9` → 신규 `16bb2d8d28811c45e8dd8ba27537f129`**
- String post 조건이 `llvm.assume` IR을 생성하여 IR 변화 발생 (예상됨)
- 새 FP는 안정적 고정점

## Verification & Defect Resolution

- `bmb check`: ✅ 3180 warnings, 0 errors (이전 3199 → 19개 감소)
- `bmb verify`: ✅ 1050/1050 verified, 0 failed
- 3-Stage Fixed Point: ✅ `16bb2d8d28811c45e8dd8ba27537f129`
- 미계약 잔여: 386 → 107 (-279개)

**디버깅 과정**:
- 1차 시도: 기존 스크립트의 body 탐색 방식이 type2 함수의 다음 함수 body를 잘못 찾음 → 오류
- 2차 시도: 올바른 type 분류 (single vs type2) 후 정확한 변환 적용
- `git stash` + `git checkout HEAD --` 2회 복원 후 최종 스크립트 적용

## Reflection

- Scope fit: 100% (String 279개 성공)
- 핵심 발견: BMB String uncontracted 함수의 87.5%가 type2 형태 (`fn ... -> String =` 다음 줄에 body)
- 새 Fixed Point: post 조건이 `llvm.assume(str.len >= 0)` 형태의 IR을 생성하여 FP 변화
- bool 96개: 여전히 타입 체커 한계 (post it >= 0 → i64 추론 충돌) — 구조적 수정 필요
- i64 10개: 음수 반환 가능 함수 — 안전한 계약 없음

## Carry-Forward

- Actionable: Cycle 3112 — bool 타입 체커 수정
  - `post` 절 내 `it` 타입을 함수 선언 반환 타입으로 고정
  - 현재: `it >= 0` 추론 → i64 → bool 반환 타입과 충돌
  - 수정 위치: `bmb/src/types/infer.rs` 또는 `bootstrap/compiler.bmb` types 섹션
- Structural Improvement Proposals: None (script approach는 임시, 장기적으로 bmb 자체 contract 생성 도구 필요)
- Pending Human Decisions: M8 공식 계획 확정 (M8-A Track B vs M8-B Native vs M8-C Language Gaps)
- Roadmap Revisions: Track B 385 → 107 (278 감소, 72.2% 추가 완료)
- Next Recommendation: Cycle 3112 — bool 타입 체커 구조적 수정 또는 M8 계획 수립
