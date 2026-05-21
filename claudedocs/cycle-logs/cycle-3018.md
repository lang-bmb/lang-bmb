# Cycle 3018: memset_fill 빌트인 추가 + brainfuck 재구성
Date: 2026-05-21

## Re-plan
Inherited carry-forward: `memset_fill(ptr, val, count)` 빌트인 추가 → brainfuck 단일 alloc + memset 패턴 재구성.
Plan valid. Proceed.

## Scope & Implementation

### memset_fill 빌트인 추가 (5개 파일)

1. **`bmb/runtime/bmb_runtime.c`**: `bmb_memset` + `memset_fill` alias 추가
   - `memset_fill(int64_t ptr, int64_t val, int64_t count) -> int64_t` (returns 0)

2. **`bmb/src/types/mod.rs`**: 타입 등록 `("memset_fill", [I64, I64, I64] -> I64)`

3. **`bmb/src/codegen/llvm_text.rs`**: IR 선언 `declare i64 @memset_fill(i64, i64, i64) nocallback nounwind`

4. **`bmb/src/mir/mod.rs`**: 반환 타입 등록 `func_return_types["memset_fill"] = MirType::I64`

5. **`bmb/src/codegen/llvm.rs`** (inkwell): 함수 선언 추가 + nounwind attribute

### brainfuck 벤치마크 재구성 (`main_inproc.bmb`)

변경 전: 1000 iterations × (calloc + interpret + free) = 1000 alloc/free pairs
변경 후: calloc 1회 + 1000 iterations × memset_fill + free 1회

핵심 변경:
- `interpret_check_with_tape(prog, tape)`: 외부 tape 수신, `memset_fill(tape, 0, tape_size())` 로 reset
- `run_benchmark(prog, tape, iters, acc, i)`: tape를 매개변수로 전달
- `main()`: 단일 `calloc` + warmup + benchmark + `free`

## Verification & Defect Resolution

### 빌드 결과
- `cargo test --release`: **6260/6260 PASS**, 0 FAILED ✅
- brainfuck BMB 빌드: `{"type":"build_success"}` ✅

### brainfuck 벤치마크 측정 (2026-05-21, 5-run)

| 측정값 | BMB (µs) | C (µs) |
|--------|---------|-------|
| Run 1  | 7803    | 8586  |
| Run 2  | 8082    | 8330  |
| Run 3  | 8112    | 8179  |
| Run 4  | 9143    | 7964  |
| Run 5  | 8279    | 8572  |
| **Median** | **8112** | **8330** |
| **Ratio** | **0.974×** | — |

**판정**: ✅ BMB faster (0.974×). 개선 전 1.037× → 개선 후 0.974×.

## Reflection

- **Scope fit**: memset_fill 빌트인 + brainfuck 재구성 완료. 측정으로 0.974× 달성.
- **Latent defects**: 없음.
- **Philosophy fit**: memset_fill은 성능을 위한 정당한 빌트인 (zero-overhead malloc/free 제거). workaround 아님.
- **Rule 7 준수**: text backend + inkwell backend 양쪽 동시 업데이트 ✅.
- **Roadmap impact**: P-track 7/7 PASS 유지, brainfuck 1.037× → 0.974× 개선.
- **user-facing**: 없음.

## Carry-Forward

- Actionable: 없음 (Cycle 3018 완결)
- Structural Improvement Proposals:
  - memset_fill interpreter 지원 추가 (현재 native-only; brainfuck은 항상 native 빌드이므로 블로킹 아님)
  - bootstrap/compiler.bmb에 memset_fill 선언 추가 (bootstrap self-compile에 사용되면 필요)
- Pending Human Decisions: 없음
- Roadmap Revisions: P-track §5 brainfuck 수치 갱신 필요 (1.037× → 0.974×)
- Next Recommendation: Cycle 3019 = 나머지 P-track 재측정 후 ROADMAP §5 갱신 + ISSUE triage / M4 진척
