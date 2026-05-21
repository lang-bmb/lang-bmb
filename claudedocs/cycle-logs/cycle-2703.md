# Cycle 2703: Track Q lint - builtin_name_collision 규칙 추가
Date: 2026-05-11

## Re-plan
인계받은: builtin/hardcoded 이름 충돌 감지 lint 규칙. Trigger ⚪ NONE.

## Scope & Implementation

### 신규 규칙
`bootstrap/lint/lint.bmb`:
- `is_reserved_builtin_name(name)`: 25개 reserved 이름 (bit_or 가족 8 + str fn 가족 17)
- `check_builtin_collision(...)`: pub fn / fn 정의에서 이름 추출 후 reserved 매칭

### Reserved set
- **Builtin intrinsic** (compiler.bmb:7142 분기): `bit_or`, `bit_and`, `bit_xor`, `popcount`, `clz`, `ctz`, `bit_reverse`, `bit_not`
- **Hardcoded String fn** (compiler.bmb:is_string_fn_group*): `chr`, `slice`, `concat`, `concat3/5/7`, `read_file`, `make_error`, `sb_build`, `parse_source`, `compile_program`, `gen_function`, `gen_program`, `get_field`, `trim_end`, `i2s`, `int_to_string`, `digit_char`

### `tokenize` 처리
Cycle 2702에서 hardcoded 리스트에서 제거됨 → 더 이상 reserved 아님 → lint도 경고 안 함 (정합성).

### 헤더 갱신
`bootstrap/lint/lint.bmb` 파일 상단 docstring: 9 → 11 checks (10 double_negation + 11 builtin_name_collision).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 신규 테스트 `test_lint_builtin_collision.bmb` | ✅ 2 warnings (bit_or + read_file 정확) |
| 기존 lint target 회귀 | ✅ 3 warnings (non_snake_case 2개 보존) |
| `tokenize`는 더 이상 경고 안 함 | ✅ Cycle 2702 결정과 정합 |

## Reflection

**핵심 통찰**:
- Lint은 컴파일러 fix와 다른 시점에 작동 (정적 검사) — Cycle 2702 변경에 따라 reserved 리스트도 업데이트해야 정합성 유지
- 25개 reserved 이름 리스트는 `is_string_fn_group*` 5개 그룹의 부분집합 (LLVM helper, BMB-prefix 등은 사용자 충돌 위험 낮음)
- "사용자 친화 prefix 권장" 메시지 (`user_xxx`, `my_xxx`)는 회피 가이드 명확

**도그푸딩 가치**:
- 컴파일러 결함이 lint 규칙으로 환원됨 (사용자 사전 경고 → silent IR corruption 방지)
- BMB 1차 사용자 (LLM)의 코드 작성 패턴에서 일반 명사 사용 빈도 높음 — lint 가치 큼

**Roadmap impact**:
- Track Q lint check count 10→11
- Cycle 2702의 Option C (dynamic 우선화) 구현 시 reserved 리스트도 함께 정리 (양방향 커플링)

## Carry-Forward
- Actionable:
  - Cycle 2704: M4-9 clang knapsack outlier IR diff
- Structural Improvement Proposals:
  - **Lint**: reserved 리스트 자동 동기화 — compiler.bmb의 `is_string_fn_group*` 변경 시 lint도 업데이트해야 하는 ergonomic 부담. Build script로 자동 추출 검토 (장기)
  - **Lint**: `compile_function`, `lower_function_sb` 등 group2 항목도 reserved set 후보 (현재 미포함, 사용 빈도 낮음으로 판단)
- Pending Human Decisions: 없음
- Roadmap Revisions: Track Q lint count 10→11
- Next Recommendation: Cycle 2704 M4-9 IR 분석
