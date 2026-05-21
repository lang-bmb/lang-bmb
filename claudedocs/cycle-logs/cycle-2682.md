# Cycle 2682: Array<f64> fn return + struct field — M5-5c/d f64 변형
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2681): `Array<f64>` fn return + struct field 4-point fix.
트리거 없음. 본격 구현.

## Scope & Implementation

### Part 1: `Array<f64>` fn return (M5-5c f64 변형)

**5-point fix** (compiler.bmb):

1. **`parse_return_type` (line 2649)** — Array + LT + F64 + GT 시퀀스 → `"Array<f64>"`
2. **`get_fn_return_scan` (line 6537)** — sexp "Array" + "<f64>" 두 토큰 합쳐 인식
3. **`collect_string_fns_acc` (line 13836)** — ret_type=="Array<f64>" → `"F:" + fn_name` prefix (registry 임베드)
4. **`is_dynamic_f64_array_fn` + `check_f64_array_fn_in_list`** (line 13937) — "F:" prefix 매칭 (70=='F', 58==':')
5. **dispatch (line 15363)** — `is_f64_array_fn` → `push_f64_ptr_marker(str_sb, dest)`

### Part 2: `Array<f64>` struct field (M5-5d f64 변형)

**4-point fix**:

1. **`check_field_type` (line 2918)** — Array<f64> 토큰 시퀀스 → 4 반환
2. **`parse_struct_fields_to_registry` (line 2906)** — type_info==4 → `~af` suffix
3. **`is_field_f64_array` + `check_field_is_f64_array`** (신규) — `~af` suffix 매칭
4. **`llvm_gen_field_access` (line 14909)** — `field_is_f64_arr` → push_f64_ptr_marker

### 시나리오 매트릭스 결과

| 시나리오 | Before Cycle 2681 | After Cycle 2681 | After Cycle 2682 |
|---------|------------------|-----------------|------------------|
| `Array<f64>` literal | ❌ raw bits | ✅ "1.500..." | ✅ |
| `Array<f64>` fn return | ❌ raw bits | ❌ raw bits | ✅ "1.500..." |
| `Array<f64>` struct field | 미검증 | 미검증 | ✅ "0.500..." |

### 골든 추가
- `test_golden_arr_f64_fn_return.bmb` → exit 42, 1.5/2.5/3.5 prints
- `test_golden_arr_f64_struct_field.bmb` → exit 42, 0.5/1.25/2.75 prints
- `golden_tests.txt` 2862 → 2864

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (회귀 없음) |
| Stage 1 빌드 | ✅ OK (10.7s) |
| Array<f64> fn return | ✅ exit 42 |
| Array<f64> struct field | ✅ exit 42 |

결함: 없음.

## Reflection

**Scope fit**: M5-5c (fn return) + M5-5d (struct field)의 f64 변형 두 가지를 한 사이클에 완료. 한쪽이 다른 쪽 인프라를 활용하기에 동시 진행이 자연.

**Latent defects**:
- nested `o.inner.fvals[0]` 미검증 (Cycle 2683으로)
- `Array<f64>` alias / for-loop / mut set 미검증 (M5-5 String 변형들과 동형 검증 필요)
- `Array<bool>`, `Array<char>` 등 — 더 거시적 일반화 (낮은 우선순위)

**Structural improvement opportunities**:
- `collect_string_fns_acc` 레지스트리에 "F:" / "A:" / (String) 세 카테고리 → 곧 4-5개로 늘 가능성. 카테고리 표기를 enum-like helper로 추출 검토 (지금은 미경제).
- `~a` / `~af` field suffix — 동일하게 5+ 카테고리 시 helper 추출
- `parse_return_type` + `check_field_type`의 generic type recognition은 거의 동일 로직 — `parse_array_type_args` 공통화 검토 (M6 후보)

**Philosophy drift**: 없음.
- 같은 패턴 두 번 적용 — M5-5의 정확한 미러. workaround 없음.
- 새로운 MIR opcode 추가 없이 기존 `mark_f64_ptr` 재사용 ✅

**Roadmap impact**:
- `Array<X>` 일반화 진척 — String, f64 두 가지 완료. i64는 이미 native.
- 향후 `Array<bool>`, `Array<char>` 도 동일 패턴으로 확장 가능 — 다음 세션 우선순위 낮음.
- Cycle 2683-2684를 nested + 변형 시나리오로 정확화

**User-facing quality**: LLM이 자연스럽게 작성할 `fn() -> Array<f64>` + `struct S { vals: Array<f64> }` 패턴이 직관대로 동작. AI-native 강화.

## Carry-Forward
- Actionable:
  - Cycle 2683: nested Array<f64> 검증 (`o.inner.fvals[0]`)
  - Cycle 2684: Array<f64> alias / for-loop / mut set 검증
- Structural Improvement Proposals:
  - `parse_return_type` / `check_field_type` generic type args 공통화 (M6 후보)
  - `string_fns` 카테고리 prefix 표 (`A:` / `F:` / future `B:` bool 등)
- Pending Human Decisions: 없음
- Roadmap Revisions:
  - cycle-logs/ROADMAP.md: Cycle 2683-2684를 nested + 변형 검증으로 정확화
- Next Recommendation: **Cycle 2683 — nested + variant 시나리오 종합 검증** (구현 없을 가능성 높음, 골든만 추가)
