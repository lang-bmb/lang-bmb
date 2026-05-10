# Cycle 2639: Dead Code 제거 + M5-4 분석
Date: 2026-05-10

## Re-plan
Cycle 2638 Carry-Forward:
- `resolve_payload_extracts` + `resolve_payload_extracts_sb` 제거 ← 이번 사이클 우선
- M5-4 준비 — `println(String)` 타입 추론 분석

계획 유효. 상속 의무 먼저, M5-4 분석은 후속.

## Scope & Implementation

**Dead code 제거 (bootstrap/compiler.bmb)**:
- `fn resolve_payload_extracts(ast: String) -> String` — 전체 제거
- `fn resolve_payload_extracts_sb(ast: String, pos: i64, sb: i64) -> i64` — 전체 제거
- `resolve_enum_variants_in_ast`에서 `let ast2 = resolve_payload_extracts(ast1);` 호출 제거 → `ast1` 직접 사용

근거: M5-3에서 `build_payload_lets_from_pat`이 `(field ...)` 노드를 직접 생성하므로
`(enum_payload_extract ...)` 노드가 생성되지 않음. 두 함수는 그 노드를 찾아 처리하는
함수였으나 입력이 없으므로 완전한 no-op.

**M5-4 `println(String)` 타입 추론 분석**:
- bootstrap `println` 디스패치 위치: compiler.bmb 약 6696-6760번 줄
- `@println(i64)` vs `@println_str(ptr)` — 구조적 인자 타입으로 선택
- 문제: 사용자 함수 반환값은 call-site에서 타입 추적이 없음 → 항상 i64로 처리
- 예시: `println(greet(m1))` — `greet`가 `String` 반환하지만 bootstrap은 이를 i64로 인식
- 근본 원인: bootstrap 타입 시스템이 함수 반환 타입을 call-site에서 조회하지 않음
- M5-4 구현 범위: 함수 심볼 테이블에서 반환 타입 조회 → 타입별 println 선택

## Verification & Defect Resolution

**Stage 1 재빌드**: ✅
```
./target/release/bmb.exe build bootstrap/compiler.bmb -o target/bootstrap/bmb-stage1 --fast-compile
```

**골든 테스트 8/8 PASS** (회귀 없음):
- enum_match, enum_variant, enum_payload, struct_complex, struct_method,
  nested_struct, mut_struct, struct_fn

**추가 M5 테스트 9/9 PASS** (M5-1~M5-3 전체):
- test_golden_enum_payload(42), test_golden_enum_wildcard(74), test_golden_enum_result(46),
  test_golden_enum_multi_payload(80), test_golden_enum_chaining(137),
  test_golden_enum_multi_field(60), test_golden_enum_3field(36)

**cargo test --release**: ✅ 6210 passed (코드 변경 — 동일 결과)

## Reflection

**Scope fit**: 상속 의무 완전 완료. dead code 2개 함수 + 1 call-site 제거.

**Latent defects**: `println(String)` — call-site 타입 추론 없음. M5-4 작업 범위 명확화됨.

**Structural improvement**: dead code 제거로 코드베이스 정리. 남은 정리 후보 없음.

**Philosophy drift**: 없음.

**Roadmap impact**: M5-4 범위 구체화 — 함수 반환 타입 심볼 테이블 조회 필요.

## Carry-Forward
- Actionable: M5-4 구현 — bootstrap `lower_call_site_println` 수정, 함수 반환 타입 조회
- Structural Improvement Proposals: 없음
- Pending Human Decisions: PyPI push 트리거 (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: 없음 (M5-4는 이미 ROADMAP에 등재)
- Next Recommendation: Cycle 2640 — M5-4 `println(String)` 타입 추론 구현 시작
