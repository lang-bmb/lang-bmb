# Cycle 3092: Track B 계약 확대 — 파서/토크나이저 함수군
Date: 2026-05-25

## Re-plan
Cycle 3091 Carry-Forward: Track B 추가 계약 + M7-4 초기 구현 착수 여부.
파서·토크나이저 핵심 함수에 pre pos >= 0 계약 추가.

## Scope & Implementation

### Track B 계약 추가 (5종)

**`starts_with(s, pat, pos)`** (line 79):
- `pre pos >= 0`
- 패턴 매칭 시작 위치 비음수

**`has_pattern(s, pat, pos)`** (line 82):
- `pre pos >= 0`
- 패턴 탐색 시작 위치 비음수

**`next_token_raw(s, pos)`** (line 646):
- `pre pos >= 0`
- 토크나이저 진입점 — 위치 비음수 필수

**`escape_parens_sb(s, pos, sb)`** (line 692):
- `pre pos >= 0`
- 괄호 이스케이프 처리 시작 위치 비음수

**`unescape_parens_sb(s, pos, sb)`** (line 722):
- `pre pos >= 0`
- 괄호 언이스케이프 처리 시작 위치 비음수

### Track B 누적 요약 (M7-3 이후 추가분)

| 사이클 | 함수 | 계약 |
|--------|------|------|
| 3085 | pack_int_tok | pre acc/pos >= 0, post >= 0 |
| 3086 | hex_digit_val | pre 범위, post [0,15] |
| 3086 | tok_val/tok_end/make_tok/pack_ids | pre/post 범위 |
| 3088 | digit_char, count_line_at, find_line_start/end, unpack_temp/block | pre 범위, post 산술 |
| 3089 | tok_kind, skip_sp_tab | pre 범위 |
| 3090 | make_caret_line, get_char_value, unpack_pos_acc, find_colon | pre 범위 |
| 3091 | find_separator, low_skip_ws, low_find_ident_end | pre 범위 |
| 3092 | starts_with, has_pattern, next_token_raw, escape/unescape_parens_sb | pre 범위 |

## Verification & Defect Resolution

- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅
- `cargo test --release`: 이전 사이클과 동일 ALL PASS (코드 변경 없음, 계약만 추가)

## Reflection

- **Scope fit**: 100%
- **Philosophy drift**: 없음
- **Roadmap impact**: Track B pre-condition 커버리지 대폭 확대. next_token_raw 같은 핵심 토크나이저 진입점도 이제 계약 보유.

## Carry-Forward

- **Actionable**: Cycle 3093 — 세션 종료 정리 + 단일 커밋
- **Structural Improvement Proposals**: None  
- **Pending Human Decisions**: None
- **Next Recommendation**: Cycle 3093 — HANDOFF.md 갱신 + 단일 커밋 (10사이클 완결)
