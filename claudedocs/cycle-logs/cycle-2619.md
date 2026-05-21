# Cycle 2619: 위생 정리 + M4 준비 기반
Date: 2026-05-10

## Re-plan
Plan valid. HANDOFF의 최우선 자율 작업(M3-5 확인 + M4-2 이슈 등록 + 기준선)에 집중.
M3-5(bmb-mcp 미커밋)는 최신 커밋(2c52e043 "불필요 파일 정리 + 서브모듈 업데이트")에서 이미 처리됨 — 확인 완료.

## Scope & Implementation

**M3-5 확인**: `git -C ecosystem/bmb-mcp status` → "nothing to commit, working tree clean" ✅

**cycle-logs 인프라 구축**:
- `claudedocs/cycle-logs/` 디렉토리 생성
- `claudedocs/cycle-logs/ROADMAP.md` 방향성 앵커 작성

**M4-2 언어 갭 이슈 등록**:
- `ISSUE-20260510-let-tuple-destructuring.md` — let-tuple 파서 갭
- `ISSUE-20260510-static-method-call.md` — Type::method() 파서 갭
- `ISSUE-20260510-option-expr-position.md` — Option::Some(x) 표현식 위치 갭

**기준선 확인**:
- `cargo nextest run --release` → 6210/6210 ✅
- `scripts/bootstrap.sh --stage1-only` → Stage 1 OK (10396ms) ✅

**아키텍처 조사 (Cycle 2620 준비)**:
- bootstrap/compiler.bmb (20125줄) 파서 분석
- `::` 처리 위치 특정: parse_call_or_ident 함수 (line ~782)
- 현재: `Name::Variant` → `(enum_variant <Name> <Variant>)`, `(args)` 미처리
- 튜플 표현식은 지원됨 (`lower_tuple_sb`), 필드 접근도 지원 (`lower_field_sb`)
- static method call 데슈가링 전략 확립: `Type::method(args)` → `(call <Type_method> args)`

## Verification & Defect Resolution
테스트/부트스트랩 모두 통과. 새 파일 추가 작업이므로 기능 결함 없음.

## Reflection

**Scope fit**: M3-5 완료 확인, 이슈 3개 등록, 기준선 확인 — 목표 달성.

**아키텍처 관찰**:
- bootstrap compiler의 enum 표현 = 정수 ordinal. 페이로드 변형은 현재 구조에서 지원 불가.
- M4-5 (Option::Some(x))는 enum payload 표현 변경 필요 → 예상보다 큰 스코프. Cycle 2623-2625에서 실현 가능성 재평가.
- M4-4 (static method call)은 순수 파서 변환 → 1-2 cycles에 가능.
- M4-3 (let-tuple)은 튜플 인프라 존재, 숫자 필드명 접근 확인 필요.

**Roadmap impact**: M4-5 예상 소요 상향 조정 필요. M4-4 먼저 → M4-3 → M4-5 재평가 순서 유지.

## Carry-Forward
- Actionable: Cycle 2620에서 M4-4 static method call 파서 구현 시작
- Structural Improvement Proposals: None
- Pending Human Decisions: M3-1 (showcase 선정), M3-3 (npm publish), M3-4 (PyPI publish)
- Roadmap Revisions: cycle-logs/ROADMAP.md에 M4-5 우선순위 조정 반영
- Next Recommendation: M4-4 static method call (1-2 cycles) → M4-3 let-tuple (2 cycles) → M4-5 평가
