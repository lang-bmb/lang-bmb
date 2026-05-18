# Cycle 2916: Always FAIL 진단 완료 — 나머지 4문제

Date: 2026-05-18

## Re-plan

Carry-Forward: 79_mini_interpreter, 89_topological_sort, 90_nth_prime, 91_ring_buffer 진단 및 수정.

## Scope & Implementation

### 진단 결과 요약

| 문제 | 근본 원인 | 수정 방법 |
|------|----------|----------|
| **79_mini_interpreter** | problem.md 버그: op=5=divide가 아닌 DUP, op=6=pop이 아닌 print-without-pop. + `vec_pop` 반환값 오해 | problem.md 전면 수정 + bmb_reference |
| **89_topological_sort** | stdin 이중 읽기 (edges 두 번), CSR adj_list 구성 오류. 올바른 접근: edge_from/to 배열 한 번 읽기 | problem.md에 알고리즘 힌트 추가 |
| **90_nth_prime** | `if n < 2 { return 0 }` 뒤 `;` 누락 — loop B에서 동일 오류 반복 (10회) | bmb_reference 강화 (return if 패턴) |
| **91_ring_buffer** | problem.md 버그: "discard if full" → 실제 동작은 "overwrite oldest" | problem.md 전면 수정 |

### 수정 내용

**79_mini_interpreter/problem.md**:
- op=5: divide → **DUP** (stack top 복제)
- op=6: "pop and print" → **"print without pop"**
- 예시 추가 (push 5, dup→[5,5], add→[10], print, print → 10,10)

**91_ring_buffer/problem.md**:
- "Full: discard" → **"Full: overwrite oldest (head advances)"**
- 예시 수정: cap=2, push 10,20,30 → dequeue=20 (not 10)

**89_topological_sort/problem.md**:
- Algorithm hint 추가: edge_from/to 병렬 배열로 한 번 읽기, O(n*m) BFS 접근법 설명

**bmb_reference.md**:
- Stack pattern: `vec_pop` CRITICAL 경고 — returns `()`, get before pop 패턴 추가
- Common Pitfalls: `if { return 0 }` 뒤 `;` 필수 예시 추가

### 90_nth_prime 특이사항

문제 자체는 정확. 오류는 순수 BMB 문법 — 모델이 `if n < 2 { return 0 }` 뒤에 `;` 를 붙이지 않음. Loop B로 10회 동일 오류 반복 (fix loop가 작동 안 함). bmb_reference 강화로 다음 측정에서 해소 기대. 근본적으로는 fix loop 메시지에 "eof에서 if 직전에 `;` 추가" 힌트가 명확하지 않아서 발생.

## Verification & Defect Resolution

테스트 변경 없음 (2388 tests, 수정은 problem.md + bmb_reference).

## Reflection

**Always FAIL 11개 진단 완료 요약**:

| 카테고리 | 문제들 | 근본 원인 |
|---------|------|---------|
| Placeholder problem.md | 34,39,41 + (31,32,33,35,36,37,38,40,42,43,44,45) | 내용 없음 |
| problem.md 불일치/버그 | 25,28,71,79,91 | 설명이 solution과 다름 |
| 알고리즘 힌트 부족 | 89 | stdin 2회 읽기 시도, CSR 실패 |
| BMB 문법 오류 (반복) | 90 | if block 뒤 `;` 누락 |
| bmb_reference 부족 | 99 (일부) | vec_pop 반환값 오해 |

**수정 효과 예측** (다음 측정):
- 34, 39, 41 (placeholder): 100% 해소 기대
- 25 (clamp 이름): 높은 확률 해소
- 28 (contract 위치): 높은 확률 해소
- 71 (first/last/count): 100% 해소 기대
- 79 (op5/op6): 높은 확률 해소
- 91 (overwrite): 높은 확률 해소
- 89 (알고리즘 힌트): 중간 확률 해소
- 90 (문법 `;`): 낮은 확률 — fix loop가 여전히 교착 가능
- 99 (vec_pop): 중간 확률 해소

## Carry-Forward

- **Actionable**: tier3-spawn-overhead Phase 1 (ISSUE-20260512) — lexer + brainfuck inproc timing
- **Structural Improvement Proposals**: 재측정 trigger — GPUStack 재측정 (`.env.local` 있으면 즉시 가능)
- **Pending Human Decisions**: GPUStack 재측정 여부 (ANTHROPIC_API_KEY 불필요, 즉시 실행 가능)
- **Roadmap Revisions**: 없음
- **Next Recommendation**: Cycle 2917 — tier3-spawn-overhead Phase 1 (lexer + brainfuck inproc timing 포팅)
