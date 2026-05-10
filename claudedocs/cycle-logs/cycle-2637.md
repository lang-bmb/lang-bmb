# Cycle 2637: M5-3 다중 필드 enum 구현
Date: 2026-05-10

## Re-plan
Plan valid. DESIGN-M5-3 문서 기반으로 4개 변경 구현.

## Scope & Implementation

**4개 편집 (bootstrap/compiler.bmb)**:

1. **`lower_enum_val_sb` 재작성**: n-field 루프 지원
   - 기존: `calloc(2, 8)` + 1-payload 하드코딩
   - 신규: `count_children(ast)` → `calloc(1+N, 8)`, `lower_enum_payload_fields_sb` 재귀 루프
   - 새 헬퍼 `lower_enum_payload_fields_sb(ast, field_idx, n_children, ...)` 추가

2. **`parse_match_arms` 수정**: `Type::Variant(a, b, ...)` 다중 바인딩
   - 새 헬퍼 `parse_payload_bind_list` 추가 (콤마 구분 bind name 수집)
   - pat_expr 형식: `(enum_payload_bind <Type> <Variant> <a> <b> ...)`

3. **`build_payload_lets_from_pat` 신규**: 다중 let 체인 생성
   - 형식: `(let <a> (field M 1) (let <b> (field M 2) body))`
   - pat_expr에서 `<name>` 토큰 순서대로 처리

4. **`parse_match_arm_body` 수정**: 다중 바인딩 처리
   - `enum_payload_extract` 대신 `build_payload_lets_from_pat` 사용
   - `vn_start = v_end + 2` (기존 `v_end + 3`에서 변경 — 새 포맷에 맞게)

**LLVM 표현** (`Node::Branch(20, 30)`):
```llvm
%ptr = call ptr @calloc(i64 3, i64 8)  ; 3-word: tag + 2 fields
store i64 1, ptr %tag_ptr             ; tag=1 (Branch)
store i64 20, ptr %field1_ptr         ; field 1
store i64 30, ptr %field2_ptr         ; field 2
```

**골든 테스트 2개**:
- `test_golden_enum_multi_field.bmb` — `Node::Branch(20,30)` → 60
- `test_golden_enum_3field.bmb` — `Triple::Three(1,2,3)` → 36

## Verification & Defect Resolution

**cargo test --release**: ✅ 6210 passed

**Stage 1 golden tests**: ✅ 9/9 PASS
- 기존 7개 enum golden tests 모두 통과 (회귀 없음)
- test_golden_enum_multi_field.bmb (=60) ✅ (신규)
- test_golden_enum_3field.bmb (=36) ✅ (신규, 3-field variant)

**발견된 결함**: 없음

## Reflection

**Scope fit**: DESIGN-M5-3 기준 전체 달성. 단일 → 무제한 필드로 일반화.

**Latent defects**:
- `parse_payload_bind_list`가 `_` wildcard를 `<_>` 이름으로 처리 → `(let <_> ...)` 생성. `_`라는 이름의 변수가 할당되지만 사용 안 됨. 현재 동작 확인됨.
- `build_payload_lets_from_pat`의 `vn_start = v_end + 2` 계산: `v_end` 이후 공백이 없으면 `v_end + 1`이 `<`. 공백이 있으면 `v_end + 2`가 `<`. 현재 형식은 항상 공백 있음 (`" <name>"`). 안전.

**Structural improvement opportunities**:
- `enum_payload_extract` + `resolve_payload_extracts` 패스가 이제 사용되지 않음 (신규 코드는 `(field ...)` 직접 생성). 정리 후보.

**Philosophy drift**: 없음. 다중 필드는 언어 완성도에 직접 기여.

**Roadmap impact**: M5-3 완료. golden_tests.txt: 2840개.

## Carry-Forward
- Actionable: `enum_payload_extract` + `resolve_payload_extracts` 사용 여부 확인 — 미사용이면 제거 가능
- Actionable: HANDOFF + ROADMAP M5-3 완료 반영
- Actionable: CLAUDE.md Rule 2 업데이트 — M5-3 다중 필드 지원 명시
- Structural Improvement Proposals:
  - 미사용 `enum_payload_extract` 패턴 제거: `resolve_payload_extracts` 함수가 dead code가 됨
- Pending Human Decisions: 없음
- Roadmap Revisions: M5-3 완료. M5-4 후보: `enum_payload_extract` 정리 + `println(String)` 타입 추론
- Next Recommendation: Cycle 2638 — HANDOFF 갱신 + M5 완료 커밋 + 다음 목표 설정
