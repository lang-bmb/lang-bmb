# Cycle 2890: format() + String Interpolation Native + bmb_reference 정오
Date: 2026-05-15

## Re-plan
Scope adjust. Discovered that format() and string interpolation "Hello {name}" were failing in native — both critical features. Targeted both.

## Scope & Implementation

1. **MIR lowering** (`mir/lower.rs`):
   - `format(template, args...)` 특수 처리 추가 — template이 StringLit인 경우 컴파일타임에 concat 체인으로 변환
   - `lower_format_template_to_pieces()` helper 추가: template을 파싱해 literal 조각과 {N} 플레이스홀더를 분리
   - 각 {N}에 대해 타입 추론 후 `int_to_string`/`bmb_f64_to_string`/`bmb_bool_to_string` 호출 또는 String 통과
   - 조각들을 `string_concat` (= `bmb_string_concat`) 체인으로 연결

2. **Text backend** (`llvm_text.rs`):
   - `"string_concat"` → `"bmb_string_concat"` dispatch 매핑 추가
   - `infer_call_return_type`에 `"string_concat"` → "ptr" 추가

3. **bmb_reference.md 정오** (Cycles 2887-2889 결과 반영):
   - str_split/str_split_whitespace/str_lines → native-supported
   - str_hashmap_keys/sorted_keys → native-supported
   - for-in-svec tracking sources 확장 (str_split, str_hashmap_keys 등)
   - format() / string interpolation → native-supported (이번 cycle)
   - to_string<T> → native-supported for all types

## Verification & Defect Resolution

- Interpreter: `format("Hello {0}, {1}!", "Alice", 30)` → "Hello Alice, 30!" ✅
- Interpreter: `"Hello {name}!"` (string interp) ✅
- Native: `format("Hello {0}, you are {1} years old", "Alice", 30)` → same ✅
- Native: `format("PI is approximately {0}", 3.14159)` → same ✅
- Native: `format("Flag is: {0}", true)` → "Flag is: true" ✅
- Native: `"Hello {name}, number {n}!"` interpolation → "Hello World, number 42!" ✅
- `cargo test --release -p bmb` → (실행 중, 기준 2388 PASS)

**중요**: Cycle 2890 이전에 발견된 P0 — `bmb_f64_to_string` / `bmb_bool_to_string` 미등록 (inkwell) — Cycle 2889에서 수정됨. format() native는 이 수정에도 의존.

## Reflection

- **Scope fit**: format() + string interpolation native가 한 cycle에 완성. 핵심 언어 기능.
- **Design choice**: Compile-time template parsing in MIR lowering. No variadic C needed. Clean.
- **Limitation**: template이 runtime 값일 때(e.g., `let tmpl = get_template(); format(tmpl, ...)`) 미지원 — 이런 패턴은 문서화되지 않은 사용법이며 빠른 MIR 낮춤이 불가능.
- **bmb_reference 정오**: Cycles 2887-2889 결과가 문서에 즉시 반영됨.
- **Roadmap impact**: 이제 native 프로그램에서 전체 string formatting 사용 가능.

## Carry-Forward
- Actionable: `for x in vec` (non-range loops) for 프로그램 실용성 확인 — 이미 native 지원됨 (Cycle 2885)
- Actionable: `read_int()` native 지원 여부 조사 — 이미 native 지원될 것으로 예상
- Structural Improvement Proposals: format() runtime template 지원 — 낮은 우선순위
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2891 — native 지원 현황 전체 감사 + 남은 gaps 식별
