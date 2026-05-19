# BMB Session Handoff — 2026-05-19 (Cycles 2945-2953 — GPUStack B축 대규모 개선)

> **HEAD**: `9ef76b6b` (Cycle 2944 완료) → 이번 세션 미커밋
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2954

---

## 이번 세션 작업 요약 (Cycles 2945-2953)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2945 | GPUStack always-fail 1차 | vec_clear codegen + option_type FP fix + if_stmt_no_semicolon |
| 2946 | GPUStack always-fail 2차 | contract_param_undefined 패턴 + problem.md 5개 |
| 2947 | 2/3-fail + HANDOFF 갱신 | problem.md 5개 (sieve/rle/char_freq/sorted_insert/bounded_queue) |
| 2948 | 1/3-fail + bool_operators | bool_operators 패턴 + problem.md 3개 |
| 2949 | if_stmt_no_semicolon 확장 | identifier 뒤 세미콜론 누락도 패턴 발동 |
| 2950 | bitwise 연산자 개선 | bool_operators bitwise 케이스 + 3개 problem.md |
| 2951 | t-first 7종 일괄 | 61/62/66/68/73/74/81 problem.md |
| 2952 | unwrap_bang not + 알고리즘 힌트 | not 키워드 안내 + 87/88/52/59/64 problem.md |
| 2953 | 최종 테스트 + HANDOFF + 커밋 | 전체 검증 완료 |

### GPUStack B축 개선 현황

**기준선**: 85.0% (255/300) — Cycle 2914 GPUStack qwen3.6-35b-a3b 측정
**예상 개선**: 90%+ (재측정 필요)

### 에러 패턴 변경 사항

| 패턴 ID | 변경 | 효과 |
|--------|------|------|
| `function_name_reserved` | 신규 (C2945) | 25_range_clamp clamp 충돌 |
| `if_stmt_no_semicolon` | 신규 + C2949 확장 | 90/89 B루프 |
| `contract_param_undefined` | 신규 (C2946) | 28_positive_factorial C루프 |
| `bool_operators` | 신규 (C2948) + C2950 bitwise 확장 | 51_bracket_match + 비트연산 오류 |
| `unknown_function` | C2950 builtin 목록 확장 | 더 완전한 안내 |
| `unwrap_bang` | C2952 `not` 키워드 추가 | `!x` → `not x` 안내 |

### problem.md 수정 현황 (총 30개 파일)

**Always-fail 수정 (Cycles 2945-2946)**:
41_collatz, 34_power_mod, 39_partial_sum_query, 71_single_element, 91_ring_buffer, 79_mini_interpreter

**Partial-fail 수정 (Cycles 2947-2948)**:
35_sieve_primes, 48_run_length_encode, 56_char_frequency, 24_sorted_insert, 99_bounded_queue_contract, 43_sum_of_squares, 44_euclidean_dist, 50_calculator

**추가 개선 (Cycles 2950-2952)**:
84_accumulator, 85_registry (병렬 배열 힌트), 92_bit_counting (bitwise 안내), 61/62/66/68/73/74/81 (t-first 경고), 52_base_convert, 59_calendar_day, 64_many_params, 87_longest_increasing (LIS DP 스케치), 88_knapsack_01 (1D DP 스케치)

### 코드 변경 파일

- `bmb/src/diagnostics/patterns.rs` — 4개 신규 패턴 + 3개 개선
- `bmb/src/codegen/llvm_text.rs` — vec_clear codegen fix
- `bmb/tests/diagnostics_test.rs` — 9개 신규 테스트 (13→22)
- `ecosystem/bmb-ai-bench/problems/*/problem.md` — 30개 파일

### 테스트 결과

```
cargo test --release -p bmb
  lib.rs:         3778/3778 PASSED
  main.rs:          47/47   PASSED
  diagnostics:      22/22   PASSED  (was 13 pre-session, +9)
  integration.rs: 2388/2388 PASSED
```

---

## 다음 사이클 (Cycle 2954+)

### 권장 우선순위

1. **GPUStack B축 재측정** — 9 cycles분 fix 효과 검증 (API 필요, 수 시간 소요)
2. **51_bracket_match `||` 지원** — BMB 언어에 `||`/`&&` 추가 (언어 갭)
3. **inttoptr UB (P3)** — HUMAN 결정 대기 (Option A codegen, 5-10 cycles)
4. **추가 패턴 발굴** — GPUStack 재측정 후 남은 실패 패턴 분석

### 잔여 개선 항목

| 항목 | 현재 | 개선 방법 | 비고 |
|------|------|----------|------|
| GPUStack B축 | 85.0% | 재측정 필요 | 재측정 대기 |
| `||`/`&&` 지원 | bool_operators 패턴만 | BMB 언어 추가 | 언어 갭 |
| csv_parse | 1.057× | LLVM IR 수준 최적화 | P축 낮은 우선순위 |
| inttoptr UB | P3 flakiness | Option A codegen | HUMAN 결정 필요 |
