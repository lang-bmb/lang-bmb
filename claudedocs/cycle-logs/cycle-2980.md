# Cycle 2980: 고루프 문제 최종 수정 — 3개 파일
Date: 2026-05-19

## Re-plan
Cycle 2979 Carry-Forward: 남은 avg=2 고루프 문제들 (61_mutual_recursion, 42_integer_sqrt, 25_range_clamp).

## Scope & Implementation

### 수정 내용

| 문제 | 실패 원인 | 수정 |
|------|----------|------|
| 42_integer_sqrt | `lo = mid`, `hi = mid - 1` (set 누락) | `set lo = mid`, `set hi = mid - 1` 수정 |
| 61_mutual_recursion | 모델이 t 무시 (`_a = read_int()`) | CRITICAL + fn 정의 포함 완전한 fn main |
| 25_range_clamp | `clamp` 예약어 사용 → linker error | `clamp_val` 이름 사용 CRITICAL + 완전한 구현 |

## Verification & Defect Resolution
- `cargo test --release`: 6260/6260 PASS ✅

## Reflection
- **25_range_clamp**: `clamp`가 BMB 표준 라이브러리 예약어인데 problem.md에 이름 안내가 있었지만 코드 예시가 없어서 모델이 틀린 이름 사용
- **42_integer_sqrt**: 이진탐색 내 `lo = mid` 패턴은 많은 모델이 범하는 오류. `set` 요구를 코드 예시에서 명확히 보여줌

## Carry-Forward
- Actionable: 없음 (모든 주요 고루프 문제 수정 완료)
- Structural Improvement Proposals: `set` 없는 할당에 대한 컴파일러 오류 메시지 개선 검토
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 전체 변경사항 커밋 + HANDOFF/ROADMAP 업데이트 + 메모리 업데이트
