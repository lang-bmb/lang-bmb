# Cycle 2826: String/Conversion Builtins 문서화

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2825 carry-forward: string/conversion builtins 추가 문서화.

## Scope & Implementation

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**

새 **String Operations** 섹션 추가 (I/O 섹션 바로 아래):
- String concatenation: `+` 연산자 (String + String)
- `int_to_string(n)`: i64 → String
- `str_len(s)`: 길이
- `char_at(s, i)`: 인덱스별 문자
- `s.byte_at(i)`: 바이트 값 (i64)
- `ord(c)` / `chr(n)`: char ↔ i64
- `i64_to_f64(n)` / `f64_to_i64(f)`: 수치 변환
- `read_line()`: stdin에서 줄 읽기

## Verification & Defect Resolution

문서 변경 → cargo test 불필요.

## Reflection

**Scope fit**: 완전히 충족.

**Roadmap impact**: string 변환이 필요한 integration 문제 (숫자→문자열 출력, 문자열 처리)에서 LLM이 올바른 패턴을 사용할 수 있게 됨.

## Carry-Forward

- **Actionable**: 다음 언어 기능 — `while let` 또는 string interpolation 중 결정 필요. 현재 주요 builtins 문서화 완료. Cycle 2827: 구체적인 언어 갭 선택 및 구현 착수.
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: B축 재측정 (API key + 언어 완성 후)
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2827 — hashmap builtins 문서화 (LLM이 자주 사용) 또는 string interpolation 가능성 조사
