# Cycle 2623: CLAUDE.md Rule 2 업데이트 + 개발 가이드 강화
Date: 2026-05-10

## Re-plan
Plan valid. Cycle 2622 Carry-Forward: "CLAUDE.md Rule 2에 let-tuple + static method call 지원 사실 반영 + M4-4 사이드 이펙트 문서화".

## Scope & Implementation

**변경 파일**: `CLAUDE.md`

**Rule 2 업데이트 내용**:
1. **지원 문법 추가** (Cycle 2620-2621 성과):
   - `let (a, b) = expr` tuple destructuring ✅
   - `Type::method(args)` static method call ✅

2. **미지원 문법 목록 정정**:
   - Tuple destructuring, Static method calls → 지원 목록으로 이동
   - `Option::Some(x)` payload enum → M5-1로 재분류 (단순 미지원이 아닌 아키텍처 이슈)

3. **M4-4 사이드 이펙트 명시**:
   - `Type::Variant(x)` → `Type_Variant(x)` 함수 호출로 처리됨
   - 임시 workaround 방법 + 권장하지 않음 명시

4. **블록 컨텍스트 let 파싱 경로 주의사항** (Cycle 2621 발견):
   - `parse_block_let` 과 `parse_let_expr` 별도 경로 존재
   - 새 let 기능 추가 시 양쪽 수정 필요

## Verification & Defect Resolution
문서 전용 변경 — 코드 변경 없음. CLAUDE.md 내용 검토 완료.

## Reflection

**Scope fit**: Rule 2가 실제 현실을 반영하게 됨. 향후 세션에서 "let-tuple 안된다"는 잘못된 가정 방지.

**발견**: CLAUDE.md의 "미지원 목록"이 오래된 상태로 남아있으면 LLM이 지원 문법을 우회하거나, 이미 구현된 것을 재구현하는 낭비 발생. 사이클마다 Rule 2 상태 동기화 가치 있음.

**Roadmap impact**: 없음. 문서 정합성 개선만.

## Carry-Forward
- Actionable: Cycle 2624에서 골든 테스트 커버리지 확대 시작
- Structural Improvement Proposals: CLAUDE.md Rule 2를 별도 "Language Support Matrix" 파일로 분리 검토 (CLAUDE.md가 커질수록 관리 어려움)
- Pending Human Decisions: None
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2624 — 고차 함수 / 재귀 패턴 골든 테스트 추가
