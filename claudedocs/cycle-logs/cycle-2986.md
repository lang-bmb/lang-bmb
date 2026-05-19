# Cycle 2986: Multi-shot 문제 원인 분석 + fn main 래퍼 수정
Date: 2026-05-20

## Re-plan
Plan valid. Carry-Forward: GPUStack 2차 측정 준비 + multi-shot 문제 개선.
GPUStack API 키 미설정으로 2차 측정 불가 → multi-shot 문제 분석으로 전환.

## Scope & Implementation

### GPUStack 2차 측정 불가
GPUSTACK_API_KEY 환경변수 미설정. 서버(http://172.30.1.53:8080) 인증 필요.
다음 세션에서 사용자가 API 키 설정 후 직접 실행 필요.

### Multi-shot 분석 (Cycle 2984 측정 기준)

| 문제 | 루프수 | 타입 | 근본 원인 | 수정 |
|------|--------|------|-----------|------|
| 04_fibonacci | 2 | Type D | fn main 래퍼 없음 | ✅ 추가 |
| 29_bounded_stack | 2 | Type C | vec_push i64 vs println_str unit 타입 불일치 | ✅ `let _p = vec_push` |
| 36_array_rotation | 2 | Type D | fn main 래퍼 없음 + set 불일치 | ✅ 추가 |
| 69_overflow_detect | 3 | Type B | fn main 끝 `}` → `};` 누락 | ✅ 수정 |
| 75_longest_plateau | 2 | Type B | fn main 끝 `}` → `};` 누락 | ✅ 수정 |

추가 발견:
- 72_alternating: fn main `}` 누락 → `};` 수정

### 수정 내용

**04_fibonacci**: BMB Notes에 `fn main() -> i64 = { ... };` 래퍼 추가

**36_array_rotation**: BMB Notes에 fn main 래퍼 추가 + `set first = 0` 일관성

**69_overflow_detect**: 마지막 `}` → `};` 수정

**75_longest_plateau**: 마지막 `}` → `};` 수정

**72_alternating**: 마지막 `}` → `};` 수정

**29_bounded_stack**: `vec_push(stk, x)` → `let _p = vec_push(stk, x)` (타입 일관성)

### fn main 래퍼 현황
- fn main 있는 파일: 62개
- fn main 없는 파일: 38개 (GPUStack에서 대부분 통과함 — AI 자체 처리)
- 추가 작업 불필요 (선별적 수정이 더 효과적)

## Verification & Defect Resolution
수정 파일 6개 확인. 별도 cargo test 실행 없음 (problem.md 전용 텍스트 수정).

## Reflection

- **Scope fit**: Multi-shot 5개 문제 근본 원인 파악 + 예방적 수정 완료
- **Key finding**: fn main 래퍼 누락과 `};` 미종결이 Type B syntax 실패의 주요 원인
- **Type checking insight**: `vec_push` 반환값 (i64) vs `println_str` 반환값 (()) — if-else 분기 타입 불일치
- **Impact**: 다음 GPUStack 측정에서 5개 문제 1-shot 예상

## Carry-Forward

- Actionable:
  1. 사용자: GPUSTACK_API_KEY 설정 후 3-run 측정 실행 (100/100 예상)
  2. GPUStack 2차 측정 결과 분석
- Structural Improvement Proposals: 38개 fn main 없는 파일들 — 필요 시 추가 가능
- Pending Human Decisions: GPUSTACK_API_KEY 설정
- Roadmap Revisions: None
- Next Recommendation:
  1. 나머지 active ISSUE들 재검토 (clang-knapsack, golden-flakiness)
  2. 다른 문제 카테고리 전반적 quality 검사
  3. HANDOFF 갱신
