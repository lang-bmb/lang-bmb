# Cycle 2971: vec_pop 문서 버그 수정 + 89_topological_sort BMB Notes 추가
Date: 2026-05-19

## Re-plan
Cycle 2970 Carry-Forward: 추가 언어 개선 기회 탐색. B-axis 나머지 문제 점검.

## Scope & Implementation

### 발견 사항 1: vec_pop 반환 타입 문서 오류 (Critical)

`bmb_reference.md`에서 `vec_pop`이 `()` (unit)을 반환한다고 잘못 문서화되어 있었다.

**실제 타입 시그니처** (`bmb/src/types/mod.rs:560`):
```rust
functions.insert("vec_pop".to_string(), (vec![Type::I64], Type::I64));
```
→ `vec_pop(v)` returns `i64` (제거된 값 반환)

검증:
- 인터프리터 (`eval.rs:7798`): `/// vec_pop(vec: i64) -> i64: Remove and return last element`
- 코드젠 (`llvm_text.rs:6024`): `// vec_pop(vec) -> i64`
- 네이티브 테스트: `let b = vec_pop(stack); let a = vec_pop(stack); println(a+b)` → `7` ✅

잘못된 워크어라운드 패턴 (구):
```
let b = vec_get(stack, vec_len(stack) - 1); let _pb = vec_pop(stack);
```

올바른 패턴 (신):
```
let b = vec_pop(stack);
```

**수정 위치** (`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`):
1. 동적 배열 섹션 (line 145): `let _p = vec_pop(v)` → `let top = vec_pop(v);  // remove and return last element (i64)`
2. Stack 패턴 (lines 323-338): CRITICAL 경고 제거 + 간소화된 `let val = vec_pop(stack)` 패턴
3. DFS 패턴 (line 690): `vec_get + vec_pop` 2-줄 → `let top = vec_pop(stk);` 1-줄
4. Common Pitfalls (line 1005): 반전 — `vec_pop` returns `i64`, unlike `vec_push/vec_set/vec_free`

### 발견 사항 2: 89_topological_sort — BMB 코드 예시 전무

`89_topological_sort/problem.md`에 BMB 코드가 없었음. 알고리즘 힌트만 있었음.

검증된 BMB 구현 추가:
- 병렬 edge 배열 (`edge_from`, `edge_to`) + in-degree 배열
- front-pointer 큐 패턴 (vec_pop 금지 — FIFO)
- O(n*m) BFS 구현
- space-separated 출력

**검증**: 12/12 tests PASS (네이티브 빌드)

## Verification & Defect Resolution
- `cargo test --release`: 6260 tests, 0 failed ✅
- 89_topological_sort 네이티브 빌드 + 12/12 테스트 PASS

## Reflection

- Scope fit: `vec_pop` 문서 오류는 B-axis AI 모델이 불필요한 `vec_get + vec_pop` 패턴을 사용하게 만드는 latent defect
  - 79_mini_interpreter problem.md의 코드는 `let b = vec_pop(stack)` 올바르게 사용했지만 bmb_reference.md가 틀렸음
  - AI 모델이 bmb_reference.md를 참조하면 더 복잡한 패턴을 선택할 수 있음
- 89_topological_sort: 유일하게 BMB 코드가 없던 문제 → 수정 완료
- Philosophy fit: 정확한 문서가 "Performance > Everything" 철학과 정렬 (간소화된 코드 = 더 나은 최적화)

## Carry-Forward
- Actionable: 다음 사이클 — 추가 패턴 또는 문서 개선
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정 (3문제 수정 + vec_pop 수정 + topo_sort 수정)
- Roadmap Revisions: None
- Next Recommendation: 추가 B-axis 개선 또는 언어 갭 발굴
