# Cycle 3191: M10 Phase 2 — negated_if_condition 16→0, 기타 경고 제거
Date: 2026-05-26

## Re-plan
Plan valid. Cycle 3190 Carry-Forward: negated_if_condition 16개 처리 (이전 세션에서 15개 완료, 1개 잔여).
이번 Cycle에서 잔여 1개 fix + 이전 세션 누적 경고 제거 검증.

## Scope & Implementation

### 처리 항목 요약 (Cycle 3190 + 3191 통합)

| 경고 종류 | 이전 | 이후 | 변화 |
|-----------|------|------|------|
| unused_return_value | 36 | 0 | −36 |
| redundant_bool_comparison | 6 | 0 | −6 |
| redundant_if_expression | 8 | 0 | −8 |
| unused_function | 3 | 0 | −3 |
| shadow_binding | 1 | 0 | −1 |
| unreachable_code | 1 | 0 | −1 |
| negated_if_condition | 16 | 0 | −16 |
| **총계** | **2,121** | **2,048** | **−73** |

### Cycle 3191 구체 변경 (1개)

- **line 1759 (`parse_bare_assign`)**: `if not is_assign_op(...) { error } else { body }` → `if is_assign_op(...) { body } else { error }`

### Cycle 3190 구체 변경 (72개, 이전 세션)

**unused_return_value (36개)**:
- prelude 2885 byte offset 발견 (경고 start byte = preprocessed 좌표)
- 36개 `print_str`/`print_compile_err` 독립 호출 앞에 `let _rN = ` 삽입

**redundant_bool_comparison (6개)**:
- `inserted == false` → `not inserted`
- `is_compile_error(X) == false` → `not is_compile_error(X)` (5곳, 3개 함수 구조 재배치로 negated_if_condition 방지)

**redundant_if_expression (8개)**:
- 각 `if cond { true } else { false }` → `cond` 직접 표현으로 리팩터

**unused_function (3개)**:
- `trl_find_block_label`, `fmt_is_toplevel`, `build_file` 3개 함수 삭제

**shadow_binding (1개)**:
- line 1671: `let t3 = ` → `let t3_err = `

**unreachable_code (1개)**:
- `ipr_inline_pure_pass`: `ir = sb_build(sb)` 재배치

**negated_if_condition (15/16개, Cycle 3190)**:
- 15개 `if not cond { A } else { B }` → `if cond { B } else { A }` 변환
  (lines: 17969, 13003, 7809, 10332, 10364, 10391, 12867, 13649, 15142, 19295, 19317, 19380, 21548, 1404, 1462)

## Verification & Defect Resolution

- `bmb check` warnings: 2,121 → **2,048** (−73)
  - semantic_duplication: 1,119
  - chained_comparison: 757
  - non_snake_case: 108
  - unused_binding: 64
- `bmb check` errors: **0** ✅
- `cargo test --release`: **6278 passed, 0 failed** ✅
- Stage 1 bootstrap: **✅**

### 발견된 결함 및 수정

1. **`let _ = expr` BMB 미지원**: 42개 삽입 후 파서 오류 → Python 스크립트로 롤백 → `let _rN = expr` 패턴 사용
2. **`== false` → `not` 연쇄**: `negated_if_condition` 5개 신규 생성 → 3개 함수 if-else 분기 교체로 해결
3. **line 7809 `else { 0 }` 누락**: 브랜치 스왑 후 case 누락 → `else { 0 }` 추가
4. **prelude offset**: 경고 start byte는 prelude(2885 bytes) 포함 좌표 → raw position = start − 2885

## Reflection

- **Scope fit**: 7개 경고 분류 모두 제거. negated_if_condition 마지막 1개 포함 완전 제거.
- **Latent defects**: 없음.
- **Philosophy drift**: 없음 — 순수 경고 제거, IR/semantic 변경 없음.
- **Roadmap impact**: M10 Phase 2 핵심 소형 경고군 완전 클리어. 잔여는 대형 자동화 대상(chained_comparison 757, semantic_duplication 1119)과 의도적 이름(non_snake_case 108), lint semantic 이슈(unused_binding 64)만 남음.

## Carry-Forward

- **Actionable**:
  - M10 Phase 3: `chained_comparison` 757개 — `a == b or a == c or ...` → `match` 변환 (자동화 스크립트 필요)
  - `unused_binding` 64개: `sb`/`item`/`cur_exit_label` — BMB lint semantic false positive 여부 추가 분석
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: 
  - `non_snake_case` 108개 (SEP, TK_INT 등 의도적 대문자) — 억제 메커니즘 필요 여부
  - `semantic_duplication` 1,119개 — 장기 목표, postcondition 세분화 필요
- **Roadmap Revisions**: None
- **Next Recommendation**: M10 Phase 3 — `chained_comparison` 757개 자동화 처리. 패턴: `a == b or a == c or a == d` → `match a { b => true, c => true, d => true, _ => false }` 또는 suppress 접근법.
