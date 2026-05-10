# Cycle 2651: M5-5 array element of String dispatch — 1차 구현
Date: 2026-05-11

## Re-plan
Cycle 2650 Carry-Forward: M5-5 array element 타입 추적 인프라 설계 시작. 계획 유효.

## Scope & Implementation

**핵심 발견**: 기존 인프라 활용 가능 — `mark_str_ptr` MIR 명령 (R: 마커)이 이미 존재.

**전파 체인** (자동, 수정 불필요):
1. `mark_str_ptr %arr` → arr를 R: 마킹
2. `let arr2 = arr` → assign 시 R: propagation (라인 14461, 14490)
3. `arr[i]` → MIR `gep arr, idx` → llvm_gen_gep_sb에서 base가 R:이면 dest 자동 R: 마킹 (라인 6664)
4. `load_ptr gep_dest` → llvm_gen_load_ptr_sb에서 R:이면 dest를 S: 마킹 (라인 6649)
5. `println(load_dest)` → llvm_try_println_str_dispatch가 S: 인식 → `@println_str` dispatch

**유일한 변경** (`bootstrap/compiler.bmb` 5246 `lower_array_literal_sb`):
```bmb
// M5-5: Mark array of String for dispatch (first element type detection)
let first_elem = if count > 0 { get_child(ast, 0) } else { "" };
let first_type = if first_elem == "" { "" } else { get_node_type(first_elem) };
let w_mark = if first_type == "string" { sb_push_mir(sb, "  mark_str_ptr " + result_tmp) } else { 0 };
```

**참고**: `lower_array_repeat_sb` (5119)에는 이미 동일 패턴 존재 (`["x"; N]`).
                        — `lower_array_literal_sb` (5265)만 빠져 있던 것.

**신규 골든 테스트**: `test_golden_arr_str_println.bmb` (`["hello", "world", "bmb"]` → `println(arr[0])`).

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed (변경 없음)

**신규 골든 테스트**: ✅ "hello\nworld\n", exit=42

**회귀 검증** (5개):
| 테스트 | 결과 |
|--------|------|
| test_golden_struct_str_field | ✅ exit=42 |
| test_golden_println_string | ✅ exit=42 |
| test_golden_let_tuple | ✅ exit=0 |
| test_golden_enum_str_payload | ✅ exit=42 |
| test_golden_struct_str_mut | ✅ exit=42 |

**골든 테스트 카운트**: 2846 → 2847 (test_golden_arr_str_println 추가)

## Reflection

**Scope fit**: M5-5 핵심 (단순 array literal) ✅. 인라인 literal `[s1, s2, ...]` 패턴 완전 동작.

**미해결 잔여 갭**:
- `[String; N]` repeat with non-literal value: `let s = "x"; let arr = [s; 3]` (확장 검증 필요)
- `let arr: [String; N] = func()` 함수 반환 (리턴 타입 추적 필요)
- `for elem in arr` iteration (별도 변환)
- `arr.push(s)` 동적 (Vec 미구현)

**Latent defects**: 없음.

**Philosophy 점검**:
- "Performance > Everything" — 컴파일타임 dispatch (런타임 오버헤드 없음) ✅
- 기존 인프라 재활용 (mark_str_ptr R: 마커) — workaround 아닌 자연스러운 통합 ✅
- 두 lowering 시스템 양쪽 처리 — `step_array_literal`은 `lower_array_literal_sb` 위임이라 단일 수정으로 충분 ✅

**Roadmap impact**:
- M5-5 ⬜ → 부분 완료 (인라인 literal of String ✅)
- 진척도: 일부분이라 "M5-5a" 정도. 잔여 = M5-5b (repeat), M5-5c (return type)

## Carry-Forward
- Actionable: Cycle 2652 — `[String; N]` repeat with non-literal value 검증 + push 인프라 확인
- Structural Improvement Proposals:
  - M5-5b: array repeat `[s; N]` 검증 (lower_array_repeat_sb 인프라 이미 존재 → 검증만)
  - M5-5c: 함수 반환 `[String; N]` 추적 (string_fns 패턴 미러링)
  - M5-4-A: tuple destructuring + String component (별도 lower_tuple_sb 경로)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 첫 단계 완료 (다음 cycle 종합 갱신 시 반영)
- Next Recommendation: Cycle 2652 — `[s; N]` repeat 검증 + 함수 반환 String array dispatch 시도
