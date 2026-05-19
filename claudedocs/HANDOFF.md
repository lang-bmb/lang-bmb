# BMB Session Handoff — 2026-05-19 (Cycles 2964-2973 — B-axis 개선 완료)

> **HEAD**: `c1bf68de` (Cycle 2973, 세션 종료)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2974

---

## 이번 세션 작업 요약 (Cycles 2964-2973)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2964 | B-axis 3문제 수정 | 01_binary_search/30_contract_chain/86_heap_sort problem.md 근본 수정 |
| 2965 | &&/\|\| MIR short-circuit | `bmb/src/mir/lower.rs`: BinOp::And/Or phi 노드 기반 단락 평가 |
| 2966 | &&/\|\| 문서화 | 86_heap_sort CRITICAL 경고 제거, LANGUAGE_REFERENCE.md 업데이트 |
| 2967 | short-circuit 테스트 | 인터프리터 OOB 보호 테스트 2개 추가 |
| 2968 | csv_parse 벤치마크 | short-circuit 변경 후 C 파리티 확인 (~1.0×, 회귀 없음) |
| 2969 | ROADMAP/HANDOFF 갱신 | 세션 요약 + GPUStack 재측정 권장 |
| 2970 | SPECIFICATION.md 문서화 | short-circuit semantics §2.4 + bmb_reference.md 노트 추가 |
| 2971 | vec_pop 버그 수정 | bmb_reference.md CRITICAL 오류 정정 + 89_topological_sort BMB Notes |
| 2972 | 코드 블록 정리 | 18개 problem.md 미닫힌 코드 블록 수정 + 패턴 일관성 |
| 2973 | HANDOFF/ROADMAP 갱신 | 세션 마무리 |

### B-axis 3문제 수정 내용

| 문제 | 실패 원인 | 수정 내용 |
|------|----------|----------|
| 01_binary_search | 모델이 leftmost search 생성 | "first mid-comparison" 패턴 CRITICAL 경고 + 완전한 코드 예시 |
| 30_contract_chain | Z3 counterexample `limit=0` | bound() pre 조건에 `x >= 0` 추가 |
| 86_heap_sort | `&&` OOB 메모리 접근 | bubble sort 예시 + 잘못된 "&&미지원" 경고 제거 |

### &&/|| Short-Circuit 구현

**파일**: `bmb/src/mir/lower.rs`

**변경**: `Expr::Binary { BinOp::And/Or }` 매치 암에 short-circuit lowering 추가:
- `a && b` → `if a { b } else { false }` (phi 노드)
- `a || b` → `if a { true } else { b }` (phi 노드)

**검증**:
- Bootstrap 컴파일러: 이미 short-circuit 구현 완료 (lines 5679-5681)
- Interpreter: 이미 short-circuit 구현 완료 (lines 688-703)
- `&&`/`||`은 파서에서 v0.32부터 지원 (grammar.lalrpop)
- `test_ir_boolean_logic`: phi i1 패턴 검증으로 업데이트
- `test_short_circuit_and/or_prevents_oob`: 새 테스트 추가

### vec_pop 문서 버그 수정 (Cycle 2971)

**발견**: bmb_reference.md에 `vec_pop`이 `()` 반환한다는 CRITICAL 오류 존재
**실제**: `vec_pop(v) -> i64` (제거된 요소 반환)
**수정**:
- Stack 패턴: `let b = vec_pop(stack)` 직접 사용으로 간소화
- DFS 패턴: `let top = vec_pop(stk)` 1줄
- Common Pitfalls: 올바른 설명으로 교체
- 29_bounded_stack, 77_state_machine, 89_topological_sort 코드 예시 개선

### 89_topological_sort BMB Notes 추가 (Cycle 2971)

유일하게 BMB 코드가 없던 문제. Kahn's BFS 완전한 구현 추가.
- 12/12 테스트 통과 (네이티브)

### 코드 블록 일관성 수정 (Cycle 2972)

18개 problem.md 파일에서 BMB Notes 섹션의 코드 블록 미닫힘 수정.
- 영향: AI 모델의 마크다운 파싱 개선

### 테스트 결과

```
cargo test --release
  lib.rs:         3778/3778 PASSED
  main.rs:          47/47   PASSED
  diagnostics:      22/22   PASSED
  integration.rs: 2390/2390 PASSED
  총: 6260 tests, 0 failed
```

### P축 상태 (csv_parse)

| 벤치마크 | 이전 | 현재 |
|----------|------|------|
| csv_parse | 1.057× (C 대비) | ~0.991× (C 파리티) |

---

## 다음 세션 (Cycle 2974+)

### 권장 우선순위

1. **GPUStack 재측정** — B-axis 3문제 수정 + vec_pop 수정 + short-circuit 구현 반영
   - 예상: 97.0% → ~99-100% (01/30/86 모두 수정됨)
2. **추가 언어 개선** — 언어 갭 식별 후 구현
3. **inttoptr UB (P3)** — HUMAN 결정 대기 (Option A codegen, 5-10 cycles)
4. **claude-sonnet-4-6 재측정** — 98.0% (2026-05-13, stale: 2026-08-13)

### 잔여 개선 항목

| 항목 | 현재 | 비고 |
|------|------|------|
| GPUStack B축 | 97.0% (2026-05-19) | 3문제 수정 후 재측정 필요 |
| csv_parse P축 | ~1.0× | C 파리티 달성 |
| inttoptr UB | P3 flakiness | HUMAN 결정 필요 |
| claude-sonnet-4-6 | 98.0% (stale: 2026-08-13) | 재측정 권장 |

### 알려진 언어 갭 (현재 없음)
- `&&`/`||` short-circuit: ✅ 완전 지원
- `break`/`continue`/`return`: ✅ 지원
- vec/str/svec/hashmap builtins: ✅ 완전 native 지원
- string interpolation/format: ✅ 지원
- `vec_pop`: ✅ `i64` 반환 (제거된 요소) — 직접 사용 가능
