# Cycle 2645: struct String 필드 타입 추적
Date: 2026-05-11

## Re-plan
Cycle 2644의 통합 테스트 중 발견: `println(p.name)` 직접 호출 시 포인터 정수값 출력 (버그).
원인: struct field access가 String 타입을 추적하지 않아 str_sb 마킹 누락.
근본 해결 결정 — 4개 위치 수정.

## Scope & Implementation

**근본 원인**:
- 기존 struct registry: `~d` (f64), `~p-Type` (ptr to struct), 무 suffix (i64). String 미지원.
- `lower_field_sb` → MIR `field-access` → `llvm_gen_field_access` → `is_field_f64`만 체크.
- String 필드 로드 시 어떠한 마커도 push되지 않음 → `println` dispatch 실패.

**4개 변경 (bootstrap/compiler.bmb)**:

1. **`check_field_type` 확장**:
   ```bmb
   if tok_kind(t2) == TK_F64() { 1 }
   else if tok_kind(t2) == TK_STRING_TYPE() { 2 }   // 신규
   else { 0 }
   ```

2. **`parse_struct_fields_to_registry` registry 포맷 확장**:
   ```bmb
   if type_info == 1 { "~d" }
   else if type_info == 2 { "~s" }                  // 신규
   else if ptr_type != "" { "~p-" + ptr_type }
   else { "" }
   ```

3. **`is_field_string` 신규 함수** (`is_field_f64` 패턴 미러링):
   - `is_field_string`, `is_field_string_at`, `check_field_is_string` 3개 함수
   - `~s` suffix 검사

4. **`llvm_gen_field_access`에 string marker push**:
   ```bmb
   let field_is_str = is_field_string(struct_reg, field);
   let _sm = if field_is_str == 1 { push_string_marker(str_sb, dest) } else { 0 };
   ```

**동작 변화**:
```bmb
struct Person { name: String, age: i64 }
let p = Person { name: "Bob", age: 25 };
println(p.name);  // 이전: 포인터 정수값 (예: 140698189959184)
                  // 현재: "Bob"
```

**골든 테스트 1개**:
- `test_golden_struct_str_field.bmb` — `println(p.name)` + `println(p.age)` → "Bob\n25", exit 42

## Verification & Defect Resolution

**Stage 1 재빌드**: ✅

**신규 테스트**: "Bob" + "25" 출력, exit 42 ✅

**회귀 18개 PASS**:
- struct/enum 8개 (enum_match, enum_variant, enum_payload, struct_complex, struct_method, nested_struct, mut_struct, struct_fn) — 각 골든 값 출력
- M5 7개 (enum_wildcard, enum_result, enum_multi_payload, enum_chaining, enum_multi_field, enum_3field, enum_str_payload) — exit 42
- M5-4 println 3개 (string, chain, f64) — exit 42

**cargo test --release**: ✅ 6210 passed

## Reflection

**Scope fit**: struct String 필드 타입 추적 완전 구현. `println(struct.field)` 직접 호출 작동.

**아키텍처 일관성**: 기존 `~d` (f64) / `~p-Type` (struct ptr) 패턴을 그대로 따라 `~s` (String) 추가. 코드 대칭성 유지.

**Latent defects**:
- `nested struct.field.subfield` — 1단 필드 접근만 검증됨, 중첩 미검증 (구조상 작동 가능)
- mut struct field assignment — `set_field`는 별도 경로, 영향 없음 추정

**Philosophy drift**: 없음. 언어 완성도 직접 기여.

**Roadmap impact**: M5 사용성 substantially 향상 — 사용자 코드에서 struct.string_field 자연 사용 가능.

## Carry-Forward
- Actionable: 없음 (실행 가능 항목 없음)
- Structural Improvement Proposals: 중첩 struct String 필드 검증 (test 추가) — 다음 사이클
- Pending Human Decisions: PyPI push (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2646 — 중첩 struct String 검증 or M3-2 벤치마크 분석
