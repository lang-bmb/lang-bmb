# Cycle 3088: Track B 계약 확대 — 문자/줄/ID 언팩 함수군
Date: 2026-05-25

## Re-plan
Cycle 3087 Carry-Forward: Track B 추가 계약 OR M7-4 초기 탐색.
순수 산술 함수 및 스캔 헬퍼 함수 6종에 pre/post 계약 추가.

## Scope & Implementation

### 추가된 계약 (6종)

**`digit_char(d: i64) -> String`** (line 19):
- `pre d >= 0 and d <= 9`
- d는 0-9 범위 digit 인덱스임을 명시

**`count_line_at(src, pos, cur, line) -> i64`** (line 45):
- `pre pos >= 0 and cur >= 0 and line >= 1`
- 줄 번호는 1부터 시작하는 불변식

**`find_line_start(src, pos) -> i64`** (line 50):
- `pre pos >= 0`
- 소스 위치는 비음수

**`find_line_end(src, pos) -> i64`** (line 55):
- `pre pos >= 0`
- 소스 위치는 비음수

**`unpack_temp(packed: i64) -> i64`** (line 3582):
- `pre packed >= 0 / post it >= 0`
- 패킹된 값이 비음수이면 temp_id도 비음수 (나눗셈 결과)

**`unpack_block(packed: i64) -> i64`** (line 3583):
- `pre packed >= 0 / post it >= 0 and it < 1000000`
- 나머지 연산: `packed % 1000000` → [0, 999999] 범위

### 설계 결정
- 재귀 함수(`count_line_at`, `find_line_start`, `find_line_end`): post 없이 pre만 추가
  - 귀납 추론이 필요한 post는 Z3 unknown 유발 가능
  - Pre만으로도 계약 의도(호출 불변식) 문서화 가능
- 비재귀 산술 함수(`unpack_temp`, `unpack_block`): pre + post 추가
  - Z3가 `x / c >= 0` (c > 0, x >= 0), `x % c in [0, c-1]` 직접 증명

## Verification & Defect Resolution

- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅
- `cargo test --release`: **ALL PASS** (3796+47+22+2390) ✅
- 결함 없음

## Reflection

- **Scope fit**: 100%
- **Key finding**: `unpack_block`의 `post it < 1000000`는 나머지 연산의 상한 보장 — Z3 LIA가 직접 증명
- **Philosophy drift**: 없음
- **Roadmap impact**: Track B 계약 누적 확대 (총 +6종)

## Carry-Forward

- **Actionable**: Cycle 3089에서 tok_kind + skip_sp_tab + 추가 scan 함수 계약 적용
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: M7-4 착수 여부 (P3, 자율 결정 가능)
- **Next Recommendation**: Cycle 3089 — tok_kind/skip_sp_tab/include_dirname_scan 계약 + M7-4 초기 구상
