# Cycle 2825: bmb_reference.md — Math Builtins + 오류 수정

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2824 carry-forward: for-loop 현황 정확화 + 다음 언어 갭.

**조사 결과 (STEP 1)**:
- `-x` 단항 부정: ✅ 이미 지원됨 (grammar + interp 모두). `bmb_reference.md` 피타겐 "0-x 사용" 은 **오류**
- `abs(x)`, `sign(x)`, `min(a,b)`, `max(a,b)`, `sqrt`, `floor`, `ceil`, `pow`, `clamp`, `log`, `exp`: ✅ 모두 builtins
- `bmb_reference.md`에 이 builtins가 없어서 LLM이 수동 구현 시도

→ 이번 사이클: 오류 수정 + builtins 문서화 → LLM이 올바른 패턴 사용 가능

## Scope & Implementation

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**

1. **Math Builtins 섹션 신규 추가**: `abs`, `sign`, `min`, `max`, `pow`, `clamp`, `sqrt`, `floor`, `ceil`, `log`, `exp`
2. **Pattern: Absolute value** — `abs(x)` builtin 사용으로 갱신, `-x` 예제 추가
3. **Common Pitfalls** — "Use 0-x instead of -x" 오류 → "-x works" 로 교정

## Verification & Defect Resolution

문서 변경 → cargo test 불필요.

**교차검증**: `bmb/src/interp/eval.rs` 에서 모든 builtins 실제 존재 확인 (grep 검증).

## Reflection

**Scope fit**: 완전히 충족.

**중요 발견**: `bmb_reference.md`에 틀린 정보가 있었음. "0-x 사용" 피타겐은 LLM이 더 장황한 코드를 쓰게 만드는 원인. 이 수정으로 integration 점수 직접 개선 가능.

**Philosophy drift**: 없음. AI-native 언어 → LLM 참조 문서 정확성이 핵심.

**Roadmap impact**: Integration 카테고리 76_multi_function (abs, sign 사용)이 이 수정으로 해결 가능.

## Carry-Forward

- **Actionable**: bmb_reference.md에 string 변환 builtins (`int_to_string`, `str_len` 등) 추가 — LLM이 숫자→문자열 변환 필요 시 자주 사용
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: None
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2826 — string/conversion builtins 추가 + 다음 언어 기능 검토 (while-let vs string interpolation vs neg builtin 추가)
