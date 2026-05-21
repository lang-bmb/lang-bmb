# Cycle 2828: 문자열 처리 Builtins 구현

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2827 carry-forward: `str_contains`, `str_starts_with`, `str_ends_with`, `str_find`, `str_substr`, `str_trim`, `str_to_int` 구현.

**중요 발견**: 타입 체커(`bmb/src/types/mod.rs`)에도 builtins 등록 필요 — 인터프리터에만 추가하면 "undefined function" 타입 에러 발생.

## Scope & Implementation

**`bmb/src/interp/eval.rs`**
- 7개 새 builtin 함수 등록 (line 271 근처)
- 7개 함수 구현 (tests 섹션 직전):
  - `str_contains(haystack, needle)` → 1/0
  - `str_starts_with(haystack, prefix)` → 1/0
  - `str_ends_with(haystack, suffix)` → 1/0
  - `str_find(haystack, needle)` → byte index or -1
  - `str_substr(s, start, len)` → String
  - `str_trim(s)` → String (whitespace removed)
  - `str_to_int(s)` → i64 (0 on parse failure)

**`bmb/src/types/mod.rs`**
- 7개 함수 타입 시그니처 등록 (line 408 근처)

**`bmb/tests/integration.rs`**
- `test_interp_str_builtins` 테스트 추가 (10개 케이스)

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**
- String Operations 섹션에 새 builtins 추가

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| 초기 테스트 | ❌ FAILED — 타입 체커 미등록 |
| types/mod.rs 등록 후 재빌드 | ✅ |
| `test_interp_str_builtins` | ✅ 1 passed |
| `cargo test --release -p bmb` (전체) | ✅ 2358 passed |

## Reflection

**Scope fit**: 완전히 충족.

**Latent defects**: `str_len`은 Unicode 문자 단위 (O(n)), `str_substr`은 바이트 단위. UTF-8 비-ASCII 문자가 섞인 경우 str_find 인덱스와 str_substr start가 불일치할 수 있음. 현재 BMB 사용 사례(알고리즘 문제)는 ASCII 위주이므로 이슈로 등록할 필요는 없음.

**Philosophy drift**: Rule 6 적용 여부 — 이 builtins는 interpreter runtime library 추가 (compiler 변경 아님). 타입 체커 변경도 기존 Rust 코드에 등록 추가 (새 기능 설계 아님). 허용 범위 판단.

**Roadmap impact**: 문자열 처리 integration 문제 해결 가능. 특히 `str_to_int`는 입력 파싱 시 유용.

## Carry-Forward

- **Actionable**: Stage 1 bootstrap 검증 필요 — `types/mod.rs` 변경이 bootstrap 영향 없음 확인
- **Structural Improvement Proposals**: `str_len`/`str_substr` byte vs char 단위 불일치 → 향후 `str_byte_len` 추가 검토 (P4)
- **Pending Human Decisions**: B축 재측정
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2829 — bootstrap Stage 1 검증 + HANDOFF/ROADMAP 갱신 + 최종 커밋
