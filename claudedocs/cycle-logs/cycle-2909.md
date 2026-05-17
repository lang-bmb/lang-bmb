# Cycle 2909: bmb-compute C 바인딩 scaffold
Date: 2026-05-18

## Re-plan
Cycle 2908 Carry-Forward: bmb-compute C 바인딩. 동일 패턴 적용.

## Scope & Implementation

**생성 파일** (`ecosystem/bmb-compute/bindings/c/`):
- `Makefile`, `example.c`, `test.c`, `README.md`

**주요 발견**:
- `bmb_moving_avg_scaled`는 forward sliding window (out[0]=avg(arr[0..k-1])).
  처음 기댓값 2000(backward trailing window 가정) → 실제 4000. 즉시 수정.
- scaled 반환값 정리: mean×1000, variance×1000000, lerp t∈[0,1000]
- `bmb_median_scaled` requires sorted input.

**테스트 수**: 56 (전 33 함수 커버, RNG는 속성 기반 단언)

## Verification & Defect Resolution

```
./test.exe → 56 passed, 0 failed
./example.exe → All examples passed
```

**수정된 결함**: moving_avg 기댓값 오류 (forward window 미인지) → 즉시 수정.

## Reflection

- **Scope fit**: 33 함수 전체 커버. bmb-algo와 동일 구조.
- **Latent defects**: 없음.
- **Philosophy drift**: 없음.
- **Roadmap impact**: M4 ④ 바인딩 C 항목 (bmb-algo + bmb-compute ✅).
- **Rule 9 검토**: 3개 라이브러리(crypto/text/json) 남음 → 조기 종료 조건 미충족.

## Carry-Forward
- Actionable: bmb-crypto C 바인딩 scaffold (Cycle 2910)
- Structural Improvement Proposals: None
- Pending Human Decisions: B축 재측정, tier3-spawn-overhead
- Roadmap Revisions: M4 ④ C: bmb-algo ✅, bmb-compute ✅
- Next Recommendation: Cycle 2910 — bmb-crypto C 바인딩
