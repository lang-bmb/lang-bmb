# Cycle 2970: SPECIFICATION.md && bmb_reference.md 단락 평가 문서화
Date: 2026-05-19

## Re-plan
Cycle 2969 Carry-Forward: SPECIFICATION.md에 &&/|| short-circuit semantics 문서화.

## Scope & Implementation

### SPECIFICATION.md (§2.4 Operator Design Rationale)
"Logical operators allow both forms" 섹션에 short-circuit semantics 추가:
```
- Short-circuit semantics: `a && b` / `a and b` evaluates `b` only when `a` is true;
  `a || b` / `a or b` evaluates `b` only when `a` is false.
  This guarantees safe boundary checks like `i < n && vec_get(v, i) > 0`.
```

### bmb_reference.md (Protocol)
Bitwise operators 노트 다음에 short-circuit 노트 추가:
```
- `&&` and `||` (also written `and`/`or`) have **short-circuit semantics** — right side is NOT
  evaluated when left side determines the result. Use `i < n && vec_get(v, i) > 0` safely:
  `vec_get` is skipped when `i >= n`.
```

## Verification & Defect Resolution
`cargo test --release` 실행 중 (이전 사이클 통과, .md 변경만)

## Reflection

- B-axis 모델이 `&&` 를 사용한 경계 체크 패턴을 알게 됨
- 86_heap_sort에서 발생한 버그 패턴(`i < n && vec_get(v, i) > X`)을 명시적으로 문서화
- SPECIFICATION.md와 bmb_reference.md 두 곳에 일관되게 문서화

## Carry-Forward
- Actionable: 커밋 후 추가 언어 개선
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: None
- Next Recommendation: 추가 언어 갭 또는 알고리즘 패턴 추가
