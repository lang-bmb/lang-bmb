# Cycle 2972: problem.md 코드 블록 정리 + 패턴 일관성
Date: 2026-05-19

## Re-plan
Cycle 2971 Carry-Forward: 추가 B-axis 개선 또는 언어 갭 발굴.

## Scope & Implementation

### 발견 사항 1: 18개 problem.md의 코드 블록 미닫힘

```
python 분석 결과:
- 18개 파일에서 ` ``` ` 마커 개수가 홀수 (open without close)
- 01/02/04/05/08/14/18/28/30/37/46/47/49/53/60/63/69/86_*
```

**패턴**: BMB Notes 섹션에 ` ``` ` 로 코드 블록을 열었지만 닫지 않음.
**수정**: 각 파일 끝에 ` ``` ` 추가 (append mode)
**검증**: 0개 파일에서 unmatched markers (수정 후)

### 발견 사항 2: 29_bounded_stack — 구 vec_get+vec_pop 패턴

Cycle 2971에서 발견한 `vec_pop` 반환값 수정과 일관성 있게:
```
// 구 (불필요한 get-then-pop):
let val: i64 = vec_get(stk, vec_len(stk) - 1);
vec_pop(stk);
println(val)

// 신 (직접 사용):
let val: i64 = vec_pop(stk);
println(val)
```

**검증**: 12/12 tests PASS (네이티브 빌드)

### 발견 사항 3: 77_state_machine — fn main 래퍼 없음

BMB Notes 코드 예시에 `fn main() -> i64 = { ... };` 래퍼가 없었음.
- 수정: 완전한 함수 형태로 변경
- 코드 블록 태그도 ` ``` ` → ` ```bmb ` 로 개선
- **검증**: 12/12 tests PASS

## Verification & Defect Resolution
- `cargo test --release`: 6260 tests, 0 failed ✅
- 29_bounded_stack 네이티브 빌드 + 12/12 PASS
- 77_state_machine 네이티브 빌드 + 12/12 PASS

## Reflection

- 코드 블록 미닫힘은 AI 모델이 problem.md를 파싱할 때 코드 경계를 잘못 인식하게 만들 수 있음
- `vec_pop` 반환값 수정 (Cycle 2971)과 함께 전체 패턴 일관성 달성
- 29_bounded_stack: AI 모델이 불필요한 2-줄 패턴 대신 간소화된 패턴을 학습

## Carry-Forward
- Actionable: 마지막 사이클 — HANDOFF/ROADMAP 갱신 및 커밋
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: 최종 커밋 + HANDOFF 갱신
