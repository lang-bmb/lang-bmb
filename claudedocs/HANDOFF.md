# BMB Session Handoff — 2026-05-19 (Cycles 2958-2962 — B축 100/100 problem.md 완결)

> **HEAD**: `468b16ca` (Cycle 2962 완료, 커밋 완료)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2963

---

## 이번 세션 작업 요약 (Cycles 2958-2962)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2958 | diagnostics 2종 수정 + 첫 배치 | unknown_function(i64_min→min/max) + if_without_else_unit 오해 해소 + 10개 problem.md |
| 2959 | 임계 버그 7개 수정 | `let i = 0` → `let mut i: i64 = 0` (반복 루프 immutable 버그) + 10개 problem.md |
| 2960 | 알고리즘 + 계약 문제 | 03_merge_sort/06_matrix_multiply/33/45/13/17/20 + 계약 문제 21/22/23/26/27 |
| 2961 | 정렬/알고리즘/계약 다수 | 07/09/10/11/12/19/31/32/36/38/40/54 + 15/16/42 + 계약 96/97/98/100 |
| 2962 | 나머지 20개 완결 | 01/02/04/05/08/14/18/28/29/30/37/46/47/49/53/60/63/69/77/87 + 87 헤더 추가 |

### B축 개선 현황

**기준선**: 85.0% (255/300) — Cycle 2914 GPUStack qwen3.6-35b-a3b 측정  
**예상 개선**: 90%+ (재측정 필요) — Cycles 2945-2962 개선 누적  
**달성**: 100/100 problem.md 완전한 BMB 코드 스케치 포함

### 코드 변경 파일

- `bmb/src/diagnostics/patterns.rs` — 2개 패턴 수정 (unknown_function, if_without_else_unit)
- `bmb/tests/diagnostics_test.rs` — 패턴 수정 반영
- `ecosystem/bmb-ai-bench/problems/*/problem.md` — 전체 100개 파일 (이번 세션 ~57개)

### 테스트 결과

```
cargo test --release
  lib.rs:         3778/3778 PASSED
  main.rs:          47/47   PASSED
  diagnostics:      22/22   PASSED
  integration.rs: 2388/2388 PASSED
  총: 6258 tests, 0 failed
```

### 적용된 BMB 패턴 일람 (problem.md 전체 일관 적용)

| 패턴 | 올바른 형식 |
|------|-----------|
| mutable 변수 | `let mut x: i64 = 0; set x = x + 1` |
| vec 핸들 | `let v = vec_new()` (타입 추론, `i64` 명시 불필요) |
| vec_push 반환값 | `vec_push(v, x)` (반환값 무시 가능) |
| 계약 문법 | `fn name(args) -> T pre cond and cond post ret > 0 = body;` |
| 부호 반전 | `0 - x` (unary minus 대신) |
| 공백 구분 출력 | first-flag 패턴 또는 `print_str(" ")` |
| if-without-else | `if cond { side_effect() }` — else 불필요 (Cycle 2822 이후) |

---

## 다음 세션 (Cycle 2963+)

### 권장 우선순위

1. **GPUStack B축 재측정** — Cycles 2945-2962 누적 수정 효과 검증 (API 필요, 수 시간 소요)
   - 목표: 85.0% → 90%+
   - 실행: `python ecosystem/bmb-ai-bench/run_bench.py --model qwen3.6... --runs 3`
2. **잔여 실패 패턴 분석** — 재측정 후 남은 실패 문제 추가 개선
3. **`||`/`&&` 언어 추가** — bool_operators 패턴으로 우회 중이나 근본 해결 필요
4. **inttoptr UB (P3)** — HUMAN 결정 대기 (Option A codegen, 5-10 cycles)

### 잔여 개선 항목

| 항목 | 현재 | 개선 방법 | 비고 |
|------|------|----------|------|
| GPUStack B축 | 85.0% (기준) | 재측정 — 90%+ 예상 | 재측정 대기 |
| `||`/`&&` 지원 | bool_operators 패턴만 | BMB 언어 추가 | 언어 갭 |
| csv_parse | 1.057× | LLVM IR 수준 최적화 | P축 낮은 우선순위 |
| inttoptr UB | P3 flakiness | Option A codegen | HUMAN 결정 필요 |
| claude-sonnet-4-6 재측정 | 98.0% (2026-05-13, stale: 2026-08-13) | `--runs 5` 재측정 | stale 전 수행 권장 |
