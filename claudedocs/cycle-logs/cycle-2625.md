# Cycle 2625: M5-1 Payload Enum 아키텍처 설계 문서
Date: 2026-05-10

## Re-plan
Plan valid. Cycle 2624 Carry-Forward: "Cycle 2625 — M5-1 payload enum 아키텍처 설계 문서 (M5 준비)".

## Scope & Implementation

**생성 파일**: `claudedocs/issues/DESIGN-M5-1-payload-enum.md`

**설계 문서 핵심 결정사항**:

1. **레지스트리 포맷 확장**: `"Color:Red,Green,Blue;"` → `"Color:0:Red:unit,1:Green:i64,2:Blue:unit;"`
2. **새 AST 노드**: `(enum_construct <Name> <Variant> payload_ast)` 추가
3. **Tagged Union 표현**: `%EnumValue = type { i64, i64 }` (discriminant + payload)
4. **2단계 해석**: 파서에서 tagged AST 기록 → post-parse에서 레지스트리 참조로 unit/payload 구분
5. **구현 순서**: M5-1a~e, 총 7-12 사이클 예상

**M4-4 사이드 이펙트 명확화**:
- `Type::Variant(x)` 현재 → `(call <Type_Variant> x)` (정적 호출)
- M5-1 완성 후 → 레지스트리 기반으로 `(enum_construct)` vs `(call)` 구분

**결정 필요 사항 문서화**:
- unit enum 하위 호환성 처리 방식
- LLVM 표현: 고정 크기 `{i64, i64}` vs 가변 (boxed ptr)
- `Result<T,E>` 지원 범위

## Verification & Defect Resolution
설계 문서 전용 — 코드 변경 없음.

## Reflection

**Scope fit**: M5-1 구현 준비 완료. 7-12 사이클 예상이 명확해짐 → 다음 마일스톤 첫 작업으로 분류.

**발견**: 
- M4-4 사이드 이펙트(static call override)가 M5-1 설계에서 2단계 해석 필요를 만듦. 파서에서는 `Type::X(args)` 를 중립 노드로 기록하고 semantic phase에서 레지스트리 참조가 깔끔한 설계.
- 현재 bootstrap의 "parse 중 즉시 AST emit" 패턴(single-pass)이 이를 복잡하게 만듦. M5-1은 mini post-processing pass가 필요.

**Roadmap impact**: M5-1 설계 완료 → M5 마일스톤 1번 항목으로 확정.

## Carry-Forward
- Actionable: Cycle 2626에서 추가 골든 테스트 (M4 기능 스트레스 테스트 + enum unit variant 포함)
- Structural Improvement Proposals: DESIGN-M5-1-payload-enum.md를 M5 시작 시 참조하여 결정 필요 사항 해소
- Pending Human Decisions:
  1. unit enum 하위 호환성: 기존 unit enum 표현 변경 여부
  2. LLVM 표현: 고정 크기 vs 가변 (i64 페이로드만 지원 vs any type)
- Roadmap Revisions: ROADMAP에 M5-1 payload enum 설계 완료 표시
- Next Recommendation: Cycle 2626 — enum + let-tuple + static method 통합 스트레스 테스트 + 세션 마무리 준비
