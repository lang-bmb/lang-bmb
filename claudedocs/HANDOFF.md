# BMB Session Handoff — 2026-05-19 (Cycles 2976-2980 — B축 problem.md 대규모 개선)

> **HEAD**: `8c131005` (세션 종료 정리 커밋)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2981

---

## 이번 세션 작업 요약 (Cycles 2976-2980)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2976 | palindrome_check + memory_pool | avg=3 고루프 2문제 완전한 fn main 래퍼 추가 |
| 2977 | avg=2 고루프 7문제 수정 | `set` 누락, `break`/`loop` 미지원, `for v in` 변수 충돌 |
| 2978 | Claude 고루프 8문제 수정 | `set` 누락, t 무시, bool→i64 패턴 |
| 2979 | Claude 고루프 9문제 수정 | `set` 누락, bound contract, mini_interpreter 완전 구현 |
| 2980 | 최종 3문제 수정 | integer_sqrt binary search, mutual_recursion, clamp_val 예약어 |

### problem.md 개선 요약 (35개 파일)

#### 주요 패턴별 수정

| 패턴 | 영향 문제 수 | 설명 |
|------|------------|------|
| `set` 누락 | ~25개 | `i = i + 1` → `set i = i + 1` 등 |
| `break`/`loop`/`return` 미지원 | 5개 | `while` 패턴으로 교체 |
| 완전한 fn main 래퍼 없음 | 15개 | 완전한 구현 예시 추가 |
| 다중 쿼리 t 무시 | 8개 | CRITICAL: "loop t times" 경고 |
| format("{} {}") 버그 | 4개 | print/print_str/println 패턴 |
| 논리 오류 | 5개 | 62_deep_nesting (-1 not n), 99_bounded_queue (empty dequeue) 등 |
| 변수 스코프 충돌 | 1개 | 33_counting_sort: v → val/vi 분리 |

#### 수정된 파일 목록
12_queue_simulation, 17_histogram, 25_range_clamp, 30_contract_chain, 33_counting_sort,
34_power_mod, 35_sieve_primes, 37_binary_exp, 41_collatz_length, 42_integer_sqrt,
43_sum_of_squares, 48_run_length_encode, 49_roman_to_int, 51_bracket_match,
52_base_convert, 55_token_count, 56_char_frequency, 57_zigzag_print, 61_mutual_recursion,
62_deep_nesting, 66_acc_recursion, 69_overflow_detect, 70_empty_input, 71_single_element,
72_alternating, 73_palindrome_check, 74_majority_element, 75_longest_plateau,
76_multi_function, 79_mini_interpreter, 83_pipeline, 85_registry_pattern, 90_nth_prime,
95_memory_pool, 99_bounded_queue_contract

### 테스트 결과

```
cargo test --release
  lib.rs:         3778/3778 PASSED
  main.rs:          47/47   PASSED
  diagnostics:      22/22   PASSED
  integration.rs: 2390/2390 PASSED
  총: 6260 tests, 0 failed
```

### 잠재 버그 발견 (Bootstrap 컴파일러)

**33_counting_sort**에서 발견: BMB 네이티브 컴파일러에서 `for v in 0..N`이 외부 스코프에
이미 선언된 `v` 변수를 덮어쓰지 않는 스코프 버그.
- 인터프리터: 정상 (변수 스코프 올바름)
- 네이티브: 이전 `let v = ...`의 마지막 값 유지
- 임시 해결: 변수명 분리 (v → val/vi)
- 실제 fix: bootstrap compiler `for` 루프 변수 스코프 수정 필요

---

## 다음 세션 (Cycle 2981+)

### 권장 우선순위

1. **GPUStack 재측정** — 35개 problem.md 개선 반영. 예상: 97.0% → ~99-100%
   - 실행: `cd ecosystem/bmb-ai-bench && python3 -m bmb_ai_bench.run_cmd --model <gpustack-model> --out results/2026-05-19-v2`
2. **Bootstrap 컴파일러 for-loop 스코프 버그** — 변수 섀도잉 미작동 (33_counting_sort 발견)
3. **inttoptr UB (P3)** — HUMAN 결정 대기

> **Claude 재측정 비활성화** (2026-05-20): ANTHROPIC_API_KEY 환경변수 제거됨. Claude B-axis 98.0% (2026-05-13) 을 고정 베이스라인으로 유지. 비용 절감 목적.

### 알려진 언어 갭
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `break`/`continue`/`return`: ✅ 지원
- vec/str/svec/hashmap builtins: ✅ 완전 native 지원
- format: ✅ `{0}`, `{1}` 포지셔널 플레이스홀더 지원 (`{}` 빈 플레이스홀더 미지원)
- `vec_pop`: ✅ `i64` 반환 (제거된 요소)

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (재측정 없음) |
| GPUStack | 97.0% (2026-05-19) | 재측정 필요 (~99-100% 예상) |
