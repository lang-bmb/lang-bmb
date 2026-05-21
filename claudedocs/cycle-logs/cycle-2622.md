# Cycle 2622: M4-5 Option::Some(x) 표현식 지원 실현 가능성 조사
Date: 2026-05-10

## Re-plan
Plan valid. Cycle 2621 Carry-Forward: "M4-5 enum 페이로드 표현식 지원 실현 가능성 조사".

## Scope & Implementation

**조사 목표**: `Option::Some(x)` 같은 payload enum 표현식을 bootstrap compiler에서 지원하는 데 필요한 변경 규모 파악.

**현재 enum 표현 분석** (`bootstrap/compiler.bmb`):

1. **`parse_enum_variants_to_registry`** (line ~2909): 변형 이름만 저장, 페이로드 타입 정보 없음.
   ```
   (variant-registry <EnumName> <V1> <V2> ...)
   ```

2. **`resolve_enum_variants_in_ast`** (line ~2956): 모든 enum variant를 정수 ordinal로 변환.
   ```
   (enum_variant <Name> <Variant>) → (int N)
   ```

3. **M4-4 사이드 이펙트**: `Type::Variant(x)` 구문이 이제 `(call <Type_Variant> x)` 로 파싱됨. payload enum constructor를 함수 호출로 처리하는 셈 — 현재는 함수가 없으면 undefined variable.

**필요한 변경 범위**:

| 레이어 | 변경 내용 | 난이도 |
|--------|-----------|--------|
| Enum 레지스트리 | 페이로드 타입 정보 저장 | 중 |
| Enum 표현 | ordinal → tagged union (discriminant + payload ptr) | 높음 |
| 생성자 표현 | `(enum_payload_ctor <Name> <Variant> val_ast)` 노드 추가 | 중 |
| 패턴 매칭 | `match (x) { Some(v) => ... }` payload 추출 로직 | 높음 |
| LLVM codegen | tagged union alloca/store/load/gep | 높음 |
| 기존 enum 호환 | unit enum (no payload) 기존 코드 경로 유지 | 중 |

**결론**: 추정 5-10+ 사이클. 남은 7 사이클(이미 2622로 1 소비 중)에 맞지 않음.

**현실적 대안 평가**:
- M4-5 전체 구현: ❌ 스코프 초과
- Workaround (payload enum을 함수로 구현): ❌ CLAUDE.md Principle 2 위반
- Cycle 2622 = 스코프 분석 + 이슈 업데이트 + 다음 사이클 재방향

**남은 6 사이클 (2623-2628) 방향 결정**:

| 후보 | 예상 cycles | 가치 |
|------|-------------|------|
| M4-6 C# 바인딩 스캐폴드 | 3-5 | M3 생태계 완성 |
| M3-2 벤치마크 하네스 단순화 | 1-2 | 내부 품질 |
| M4-5 payload enum 아키텍처 설계 | 2-3 | 미래 기반 |
| 골든 테스트 커버리지 확대 | 2-3 | 회귀 방지 |
| CLAUDE.md Rule 2 업데이트 | 1 | 개발 마찰 감소 |

→ **선택**: CLAUDE.md Rule 2 업데이트 (Cycle 2623, 1 cycle) + M4-5 아키텍처 설계 문서 (Cycles 2624-2625) + 골든 테스트 커버리지 확대 (Cycles 2626-2628).

## Verification & Defect Resolution
조사 사이클 — 코드 변경 없음, 검증 N/A.

## Reflection

**Scope fit**: M4-5 실현 가능성 분석 완료. "6개월 후 다시 보자"가 아닌 구체적 재방향 결정.

**발견**:
- M4-4 사이드 이펙트: `Type::Variant(x)` 구문이 payload enum constructor와 static method call을 구분 불가. 문서화 필요.
- bootstrap enum 시스템이 "unit enum only" 전제로 설계됨. payload enum은 새 표현 레이어 전체 필요.

**Roadmap impact**: M4-5를 "즉각 구현" 에서 "아키텍처 설계 필요 → 별도 마일스톤"으로 재분류.

## Carry-Forward
- Actionable: Cycle 2623에서 CLAUDE.md Rule 2 업데이트 (let-tuple 지원 + static method call 추가) + M4-4 사이드 이펙트 문서화
- Structural Improvement Proposals: M4-5 payload enum은 M5 마일스톤 첫 번째 항목으로 등록
- Pending Human Decisions: None
- Roadmap Revisions: M4-5 → "M5-1: payload enum 설계" 로 이동, M4-3/M4-4 완료 표시
- Next Recommendation: Cycle 2623 — CLAUDE.md Rule 2 업데이트 + 골든 테스트 커버리지 확대 시작
