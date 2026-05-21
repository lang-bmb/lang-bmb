# Cycle 2975: format("{} {}") 버그 발견 및 수정
Date: 2026-05-19

## Re-plan
GPUStack 고루프 문제 분석 → `format("{} {}", ...)` 구문이 BMB에서 동작 안 함 발견.

## Scope & Implementation

### 핵심 발견
BMB `format()` 함수는 `{0}`, `{1}` 포지셔널 플레이스홀더만 인식함. `{}` 빈 플레이스홀더는 대체되지 않아 리터럴 `{}` 출력.

**영향 문제들**:
- 17_histogram: avg=3 loops (GPUStack) — 첫 시도에서 `{} {} {}` 출력
- 48_run_length_encode: avg=2 loops (GPUStack) — 첫 시도에서 `{} {}` 출력  
- 56_char_frequency: avg=2 loops (GPUStack) — 첫 시도에서 `{} {}` 출력
- 52_base_convert: 1 loop (모델이 format 미사용으로 우회)

### 수정 사항 (4개 problem.md 파일)

| 파일 | 수정 내용 |
|------|---------|
| 17_histogram | `println_str(format("{} {}", i, c))` → `print(i); print_str(" "); println(c)` + CRITICAL 경고 |
| 48_run_length_encode | IMPORTANT 섹션에 `format("{} {}", ...)` WRONG 주석 추가 |
| 56_char_frequency | IMPORTANT 섹션 + BMB Notes 코드 둘 다 수정 |
| 52_base_convert | `format("{}", d)` → `to_string(d)` |

## Verification & Defect Resolution
- `cargo test --release`: 6260/6260 PASS ✅

## Reflection
- **Root cause**: problem.md 자체가 잘못된 코드 예시를 제공 → 모델이 그것을 따라 실패
- **Impact**: 3개 문제의 첫 루프 실패 패턴이 완전히 사라질 것으로 예상
- **Latent**: `format()` 사용처 전수 검색 완료 — 4개 모두 수정됨

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: BMB 언어에 `format("{}", x)` 지원 추가 검토 (사용성 개선)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 추가 고루프 문제 분석 OR 다른 잠재 패턴 검색
