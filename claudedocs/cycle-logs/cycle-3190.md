# Cycle 3190: M10 Phase 2 — 소규모 경고 카테고리 처리 (2121→2064, −57)
Date: 2026-05-26

## Re-plan
Plan valid. M10 Phase 2: unused_return_value 36개 및 기타 소규모 카테고리 처리.
이전 세션(Cycle 3189)에서 unused_binding 781→64 완료, unused_return_value 처리 도중 컨텍스트 초과.

## Scope & Implementation

### 핵심 발견: 프리뤼드 바이트 오프셋
bmb check 경고의 `start` 바이트는 **preprocessed source** (prelude + 파일) 기준.
- 프리뤼드 크기: **2885 바이트**
- raw 파일 위치 = `start - 2885`
- 검증: print_str raw 929894 + 2885 = 932779 (warning start) ✓

### BMB 문법 발견
- `let _ = expr` **미지원** (parser 에러: `Unrecognized token '_'`)
- 올바른 패턴: `let _rN = expr` (명명된 변수, `_` prefix로 unused_binding 억제)

### 처리된 경고 카테고리
| 카테고리 | 이전 | 이후 | 변화 | 방법 |
|---------|------|------|------|------|
| unused_return_value | 36 | 0 | −36 | `let _r{n} = ` 삽입 (prelude offset 보정) |
| redundant_bool_comparison | 6 | 0 | −6 | `X == false` → `not X`, 후 if-else 교환 |
| redundant_if_expression | 8 | 0 | −8 | `if cond { true } else { false }` → `cond` |
| unused_function | 3 | 0 | −3 | 미사용 함수 삭제 (trl_find_block_label, fmt_is_toplevel, build_file) |
| shadow_binding | 1 | 0 | −1 | `t3` → `t3_err` 내부 스코프 리네임 |
| unreachable_code | 1 | 0 | −1 | for 루프 내 `ir = sb_build(sb)` 순서 재배치 |

### 주요 수정 세부사항
- `repl_try_int_first/str_first/fallback`: `negated_if_condition` 경고 회피 위해 if-else 브랜치 교환
- `ipr_inline_pure_pass`: `if new_count == 0 { break } else { () }; ir = sb_build(sb)` → 순서 재배치
- `fmt_is_contract`: if-else 체인 → `or` 표현식으로 단순화

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| bmb check errors | 0 ✅ |
| 총 warnings | 2064 (−57) ✅ |
| negated_if_condition | 16 (변화 없음) ✅ |
| cargo test --release | 6278 passed ✅ |
| Stage 1 bootstrap | ✅ |

## Reflection
- **Scope fit**: 6개 카테고리 완전 제거. −57 warnings 달성.
- **Latent defects**: 없음. redundant_bool_comparison 수정 시 negated_if_condition 신규 발생 → 즉시 교환으로 해결.
- **Philosophy drift**: None — 코드 품질 향상, 의미론적 변경 없음.
- **Roadmap impact**: M10 Phase 2 진행. 남은 대형 타겟: chained_comparison 757, negated_if_condition 16.

## Carry-Forward
- **Actionable**: 
  - `negated_if_condition` 16개: `if not cond { A } else { B }` → `if cond { B } else { A }` 변환
  - `chained_comparison` 757개: `a == b or a == c or ...` → `match` 변환 (자동화 검토)
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: None
- **Roadmap Revisions**: None
- **Next Recommendation**: `negated_if_condition` 16개 처리 (자동화 가능) → 다음 타겟: chained_comparison
