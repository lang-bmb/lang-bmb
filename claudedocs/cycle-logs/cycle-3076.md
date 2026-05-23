# Cycle 3076: M7-1 Track A — 정수 파라미터 함수 17종 Contract 부착
Date: 2026-05-23

## Re-plan
Cycle 3075 Carry-Forward: `find_char` 스모크 테스트 완료 (Track A 1/17). 이번 사이클 범위: Track A 나머지 16개 추가 + Z3 + Fixed Point 검증.

## Scope & Implementation

### Track A 완료 — 정수 파라미터 함수 17종 (25 llvm.assume 주입)

| 함수 | 계약 | assume 수 |
|------|------|---------|
| `find_char` | `pre pos >= 0 and ch >= 0` | 2 |
| `skip_ws` | `pre pos >= 0` | 1 |
| `skip_ws_comments` | `pre pos >= 0` | 1 |
| `scan_int` | `pre pos >= 0 and acc >= 0` | 2 |
| `scan_hex_int` | `pre pos >= 0 and acc >= 0` | 2 |
| `scan_bin_int` | `pre pos >= 0 and acc >= 0` | 2 |
| `scan_oct_int` | `pre pos >= 0 and acc >= 0` | 2 |
| `scan_digits_end` | `pre pos >= 0` | 1 |
| `scan_exponent` | `pre pos >= 0` | 1 |
| `scan_ident_end` | `pre pos >= 0` | 1 |
| `scan_string_end` | `pre pos >= 0` | 1 |
| `scan_char_end` | `pre pos >= 0` | 1 |
| `find_comma` | `pre pos >= 0` | 1 |
| `find_comma_or_end` | `pre pos >= 0` | 1 |
| `find_pattern_noa` | `pre pos >= 0` | 1 |
| `match_bytes` | `pre s_pos >= 0 and p_pos >= 0` | 2 |
| `find_pattern_noa_range` | `pre pos >= 0 and end >= 0` | 2 |

**합계**: 17개 함수, 25개 llvm.assume

### 발견 — 다중 pre 절 문법

`pre A\n  pre B` 형식은 파싱 에러. `pre A and B` 단일 절로 결합 필수.
(Cycle 3075 스모크 테스트에서 발견)

### Track B 결정 (HANDOFF 타겟)

`method_to_runtime_fn`, `get_call_return_type`, `is_string_returning_fn` — String 반환 함수.
- llvm.assume 혜택: 없음 (assume은 정수 비교만 지원)
- Z3 pre-condition: `pre fn_name.len() > 0` 등 String 조건 → Z3 지원 여부 불명

**결정**: Track B는 Cycle 3077에서 String pre-condition Z3 테스트 후 결정.

## Verification & Defect Resolution

- Stage 1 빌드: `build_success` ✅
- Fixed Point: S2 IR == S3 IR, 해시 `dc57beff` ✅ (이전: `745082F5`)
- Z3 검증: `bmb verify bootstrap/compiler.bmb` → 1513/1513 ✅
- `cargo test --release`: 6264 PASS ✅

## Reflection
- **Scope fit**: 100% — Track A 17종 완료
- **효과**: 파서 핵심 경로 (스캐너, 패턴 매처) 모두 pos >= 0 불변식 선언 완료
- **LLVM 혜택**: 25개 assume → LLVM이 음수 포지션 케이스 제거 가능, bounds check 최적화 활성화 기대
- **Fixed Point 변경**: `745082F5` → `dc57beff` (계약 주입으로 IR 변경, 결정론적)

## Carry-Forward
- **Actionable**: Cycle 3077 — Track B 결정
  1. `pre fn_name.len() > 0` 등 String 조건 Z3 테스트 (pass/fail/timeout 확인)
  2. Pass → Track B 계약 작성 (method_to_runtime_fn, get_call_return_type, is_string_returning_fn)
  3. Fail → Track B는 주석으로 불변식 문서화, M7-2 Z3 string 지원 추가 필요 기록
- **Structural Improvement Proposals**:
  - Track A 추가 후보: `find_pattern_at`, `find_pattern_at_slow`, `scan_number` (pos >= 0)
  - assume 범위 확장: String.len() 기반 조건 (`pos < s.len()`) — `ptrtoint + icmp` 형태로 확장 가능
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: 없음
- **Next Recommendation**: Cycle 3077 — Track B Z3 String 조건 테스트 + 추가 Track A 함수
