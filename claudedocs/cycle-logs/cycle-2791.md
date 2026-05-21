# Cycle 2791: fibonacci fair fix → 17/17 PASS fair comparison
Date: 2026-05-13

## Re-plan

⚪ NONE. Carry-Forward에서 명시적으로 인계: fibonacci @pure constant-fold fairness 문제.
Cycle 2790에서 verify timeout을 해결(6B→100M)했지만 BMB는 여전히 LLVM constant-fold(~17ms),
C는 실제 100M 루프 실행(~1-3s) → P-track 비율 무의미. ROADMAP "신뢰도 ✅" claim이 false였음.

## Scope & Implementation

### fibonacci 공정 비교 수정

**근본 원인**: `@pure fibonacci_iter` + 상수 인수 50 → LLVM이 컴파일타임에 fold.
BMB: ~17ms (constant-fold). C: ~1-3s (실제 100M 루프). 출력은 일치하지만 측정 비율 신뢰 불가.

**수정 전략** (main_inproc.bmb의 기존 패턴 적용):
- **BMB**: `fibonacci_iter(50)` → `fibonacci_iter(bmb_black_box(50))` — 인수를 불투명하게 만들어 LLVM이 @pure fn을 fold 못하게 강제
- **C**: `fibonacci_iter` 에 `__attribute__((noinline))` 추가 — GCC가 루프 밖으로 호출을 hoisting하거나 constant-propagate하는 것 방지

**변경 파일**:
- `ecosystem/benchmark-bmb/benches/compute/fibonacci/bmb/main.bmb`
  - `fibonacci_iter(50)` → `fibonacci_iter(bmb_black_box(50))`
  - 주석: "bmb_black_box(50) makes the argument opaque to LLVM — prevents constant-fold of @pure fn"
- `ecosystem/benchmark-bmb/benches/compute/fibonacci/c/main.c`
  - `int64_t fibonacci_iter(int64_t n)` → `__attribute__((noinline)) int64_t fibonacci_iter(int64_t n)`
  - 주석: "noinline prevents GCC from constant-propagating fibonacci_iter(50) across iterations"

**패턴 출처**: `benches/compute/fibonacci/bmb/main_inproc.bmb` (Cycle 2686)
`bmb_black_box(n_seed)` + `bmb_black_box(fib)` 패턴이 이미 in-process 벤치에서 사용됨.

### ROADMAP 수정

false claim 수정:
- "측정 신뢰도 회복: 17/17 benches 모두 fair comparison. P-track ratio 신뢰도 ✅"
  → "측정 신뢰도 완전 회복 (Cycle 2791): fibonacci BMB: bmb_black_box(50), fibonacci C: noinline. P-track ratio 신뢰도 ✅"
- bench verify 최신 섹션: 16/17 → 17/17 fair comparison 갱신
- 섹션 헤더: Cycle 2788-2790 → Cycle 2788-2791

## Verification & Defect Resolution

```
python scripts/verify_bench_outputs.py --tier all --epsilon 1e-6 --rebuild
→ 17/17 PASS, 0 mismatch, 0 failed, Time: 43.6s
```

| 항목 | 결과 |
|------|------|
| fibonacci | ✅ PASS (C: ~2s noinline, BMB: ~2s black_box, output 일치) |
| 전체 --tier all | ✅ **17/17 PASS** |
| 총 소요 시간 | 43.6s (↑ Cycle 2790의 12.7s — fibonacci C/BMB 이제 실제 실행) |

verify 시간이 12.7s → 43.6s로 증가한 것은 정상. fibonacci가 이제 real workload (~2s×2).

## Reflection

**Scope fit**: ✅ 1-cycle 작업으로 fair fix 완전 해결.
**Correctness**: bmb_black_box(50)는 LLVM가 50이라는 값을 알 수 없게 만들어 @pure fn의 constant-fold를 방지. C noinline은 GCC가 루프 내 순수 함수 호출을 hoisting하는 것을 방지. 두 방어선 모두 필요.
**Philosophy alignment**: P-track 신뢰도가 이제 실질적 의미를 가짐. "Performance > Everything"은 측정이 공정해야 의미 있음.
**ROADMAP false claim 해소**: Cycle 2790에서 "신뢰도 ✅"를 기록했지만 당시에는 fibonacci fair fix가 완료되지 않았음. Cycle 2791에서 사실이 됨.
**Roadmap impact**: bench verify 인프라 100% 완성 + 공정성 보장. P-track 측정 신뢰도 ✅ (이제 진짜).
**Philosophy drift**: 없음.

## Carry-Forward

- Actionable: None
- Structural: None
- Pending Human Decisions:
  - D5-A (GitHub Actions verify workflow) — HUMAN approval
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline)
- Roadmap Revisions: ROADMAP.md fibonacci fair fix 반영, bench verify 최종 상태 17/17 fair ✅
- Next Recommendation: Cycle 2792 — or-chain-lowering P2 (단락 평가 의미론 위반 수정) — bootstrap/compiler.bmb lowering 변경
