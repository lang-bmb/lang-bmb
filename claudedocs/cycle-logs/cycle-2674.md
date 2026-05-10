# Cycle 2674-2675: M5-5d 구현 — struct field `Array<String>` dispatch ✅
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2673): M5-5d 진단. ⚪ Plan 유효. M5-5c와 동일한 패턴이 struct field에 적용 가능한지 검증 → 적용 가능.

## Scope & Implementation

### 1. 진단 (Cycle 2674)

기존 struct field type registry는 이미 카테고리화:
| Suffix | 의미 |
|--------|------|
| `~d` | f64 |
| `~s` | String |
| `~p-Foo` | pointer to struct Foo |
| (없음) | default i64 |

`Array<String>` 카테고리 누락 — `check_field_type`이 인식 못함 → default i64 → field_is_str_arr 없이 그냥 i64 load → 포인터 정수 출력.

### 2. 구현 (Cycle 2675) — `~a` 카테고리 추가

**(a) `check_field_type` — 인식 추가**:
```bmb
else if tok_kind(t2) == TK_IDENT() {
    let name = get_ident_text(src, tok_end(t1), t2);
    if name == "Array" and tok_kind(t3) == TK_LT() {
        if tok_kind(t4) == TK_STRING_TYPE() and tok_kind(t5) == TK_GT() { 3 }
        ...
```

**(b) `parse_struct_fields_to_registry` — `~a` suffix 발행**:
```bmb
let type_suffix = if type_info == 1 { "~d" } 
    else if type_info == 2 { "~s" } 
    else if type_info == 3 { "~a" }    // NEW
    else if ptr_type != "" { "~p-" + ptr_type } else { "" };
```

**(c) `is_field_str_array` + `check_field_is_str_array` 신규** — `is_field_string` 패턴 미러.

**(d) `llvm_gen_field_access` — push_str_ptr_marker 발행**:
```bmb
let field_is_str_arr = is_field_str_array(struct_reg, field);
let _am = if field_is_str_arr == 1 { push_str_ptr_marker(str_sb, dest) } else { 0 };
```

### 3. 골든 (Cycle 2675)

| 골든 | 시나리오 | 결과 |
|------|---------|------|
| `test_golden_arr_str_struct_field.bmb` | `p.tags[0]` direct access | exit 42, "red\ngreen\nblue\n" ✅ |
| `test_golden_arr_str_struct_field_loop.bmb` | `for i in 0..N { p.tags[i] }` | exit 42, "alpha\nbeta\ngamma\n" ✅ |

Golden count: 2854 → **2856**.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (회귀 없음) |
| Stage 1 빌드 | ✅ OK (10.5s) |
| M5-5 골든 (5+3+2) | ✅ 10/10 모두 PASS |
| M5-5d 변형 (for-loop) | ✅ PASS |

## Reflection

**Scope fit**: 의도 = M5-5d. 두 사이클 내 구현 + 변형 골든 추가까지 완료. M5-5c의 `A:` prefix 패턴과 동형 (struct field에서는 `~a` suffix). 

**Latent defects**:
- (제로) 회귀 통과
- `let mut p; set p.tags = ...` 같은 mut 패턴은 별도 검증 필요 (set_field 경로) — 다음 세션 또는 필요 시
- nested struct (`p.inner.tags[0]`)는 미검증 — 일반화 인프라(get_field_ptr 패턴) 사용해야 할 수 있음

**Structural improvement opportunities**:
- struct field type registry가 4종 (i64/d/s/a/p) — 추가 종류 (`Array<i64>` `~ai`, `Result<T,E>` 등) 일관적 확장 가능
- `check_field_type` 패턴이 M5-5c `parse_return_type` 인식 로직과 중복 — 공통 `parse_type_signature` 헬퍼 후보 (M6)
- `field_is_X / push_X_marker / load_type / load_line` 4중복 — single dispatch 매핑으로 단순화 가능 (현재는 명료성 ↑)

**Philosophy drift 점검**:
- "Workaround 없는 근본 해결" — struct field type registry 정확한 layer에 `~a` 추가, 기존 카테고리화 패턴 일관 ✅
- "AI-native 언어 확장" — LLM이 가장 자주 작성할 패턴 `struct { tags: Array<String> }` 자동 dispatch ✅
- "복잡도는 기피 사유 아니다" — 4-point fix (parse + suffix + lookup + codegen) 정확한 위치 처리 ✅

**Roadmap impact**:
- **M5-5 매트릭스 6/7 → 7/7 ✅ 완료**
- 더 이상 M5-5 잔여 없음 — M5 전체 작업이 M5-1~M5-5 모두 완료
- 새 작업: M5-6 (다른 generic types), M6 (type-checker 분리), 또는 M3 publish (HUMAN)

**User-facing quality**:
- LLM 자연 패턴 (struct 정의 + field 접근 + indexing + println) 작동
- 명시적 type annotation 일관 (BMB explicit 철학)

## Carry-Forward
- Actionable:
  - Cycle 2676: ROADMAP.md / HANDOFF.md 갱신 (M5-5 7/7 완료 반영)
  - Cycle 2677: 추가 edge case 검증 (mut struct field, nested struct field)
  - Cycle 2678: 종합 commit
  - Cycle 2679: 세션 마무리
- Structural Improvement Proposals:
  - 공통 type signature parsing helper (parse_return_type + check_field_type 통합) — M6 후보
  - `Array<X>` 일반화 (i64, f64, struct ptr 등) — 현재는 String만 지원
  - struct field 카테고리화 single-dispatch (현재는 각각 4중복 코드)
- Pending Human Decisions: 변경 없음 (npm/PyPI publish 등 기존 잔여)
- Roadmap Revisions: M5-5 7/7 완료 — Cycle 2676에서 ROADMAP/HANDOFF 갱신
- Next Recommendation: Cycle 2676 — ROADMAP/HANDOFF 갱신 + 추가 edge case 검증
