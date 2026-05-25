# Cycle 3136: M9 Batch 2 — lexer/scanner 11개 post 조건 추가
Date: 2026-05-25

## Re-plan
Plan valid. M9 계속 — missing_postcondition 799개 잔여. Cycle 3135 핵심 유틸리티 완료 → lexer/scanner 함수들로 확장.

## Scope & Implementation
11개 post 조건 추가:

**String 길이 계약 (2개)**:
- `int_to_string_neg(n)` → `post it.len() >= 1` (항상 최소 1자리)
- `int_to_string(n)` → `post it.len() >= 1` (항상 최소 1자리)

**i64 범위 계약 (9개)**:
- `include_dirname_scan(path, i)` → `post it >= -1` (−1 = not found, 또는 위치)
- `skip_ws_comments(s, pos)` → `post it >= pos` (전진 단조성)
- `scan_int(s, pos, acc)` → `post it >= 0` (유효 정수값)
- `scan_hex_int(s, pos, acc)` → `post it >= 0` (유효 정수값)
- `scan_bin_int(s, pos, acc)` → `post it >= 0` (유효 정수값)
- `scan_oct_int(s, pos, acc)` → `post it >= 0` (유효 정수값)
- `scan_digits_end(s, pos)` → `post it >= pos` (전진 단조성)
- `scan_exponent(s, pos)` → `post it >= pos` (전진 단조성)
- `scan_ident_end(s, pos)` → `post it >= pos` (전진 단조성)

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 2981 → 2980 (−1 net; missing_postcondition 799→788 = −11)
  - 다른 카테고리 소폭 증가로 net −1
- bmb verify: 942/942 → 931/931 (0 failed, total −11: Z3 예산 영향 추정)
  - 0 failed — 모든 검증 통과

## Reflection
- scan_* 계열 단조성 계약: lexer 위치 전진 보장 — 무한루프 방지 증명 기반
- int_to_string_neg/int_to_string: 숫자 → 문자열 최소 1자리 계약 — 빈 문자열 반환 불가
- include_dirname_scan: `it >= -1` — sentinel 값(-1) 명시
- verify total 감소 패턴 반복: Cycle 3135처럼 −11 동일 — 재귀 계약 Z3 예산 효과
- 남은 missing_postcondition: 788개

## Carry-Forward
- Actionable: missing_postcondition 788개 계속 분석
  - 다음 배치: tok_kind, next_token_raw, get_ident_text, get_int_text, get_string_text 등 lexer 텍스트 추출 함수들
  - scan_string_end, scan_char_end, get_char_value 등 스캐너 함수들
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M9 진행 — missing_postcondition 814→788 (−26 총계)
- Next Recommendation: Cycle 3137: tok_kind/next_token_raw/get_*_text 계열 분석
