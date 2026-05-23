# Cycle 3077: M7-1 Track B — Z3 String 조건 테스트 + 주석 불변식 문서화
Date: 2026-05-23

## Re-plan
Cycle 3076 Carry-Forward: Track B Z3 String 조건 테스트 + method_to_runtime_fn/get_call_return_type/is_string_returning_fn 처리 결정.
이번 사이클 범위: Z3 테스트 → 결과에 따른 Track B 처리.

## Scope & Implementation

### Z3 String 조건 테스트

```
fn str_classify(s: String) -> i64
  pre s.len() > 0
= ...

bmb verify → {"type":"verify_result","total":0,"verified":0,"failed":0}  (exit code 1)
```

**결과**: `total:0` — Z3 백엔드가 String pre-condition 함수를 완전히 무시함.
Z3는 현재 정수/불리언 조건만 지원. String length 조건은 처리 불가.

### Track B 결정: 주석 불변식 문서화

Z3 미지원으로 인해 계약 추가는 무의미 (total:0 = verifiable 0). 
대신 `// invariant:` 주석으로 불변식 명시:

1. `method_to_runtime_fn`: `// invariant: method != "" → ret starts with "bmb_" or known special; unknown → "__unknown_method_"`
2. `get_call_return_type`: `// invariant: ret ∈ {"void", "ptr", "i64", "double"}`
3. `is_string_returning_fn`: `// invariant: returns true iff name is a known stdlib fn that returns heap-allocated String (ptr). Must stay in sync with get_call_return_type "ptr" branch.`

### Track B 결론 (M7-2 연결)

String 조건 계약은 M7-2 범위 — Z3 IPC에서 SMT String theory 지원 추가 필요.
현재 Track B 불변식은 주석으로 보존, M7-2에서 실제 계약으로 승격 예정.

## Verification & Defect Resolution

- Stage 1 빌드: `build_success` ✅
- `cargo test --release`: 6264 PASS ✅
- Z3 String 조건: total:0 → 명확한 미지원 확인

## Reflection
- **Scope fit**: 100% — Track B 결정 완료
- **핵심 발견**: Z3 백엔드는 String length 조건을 silent skip. 계약이 아닌 주석으로 처리
- **M7-2 임플리케이션**: Z3 SMT String theory 지원이 M7-2의 핵심 구현 과제
- **M7-1 완료**: Track A 17종 contracts (25 llvm.assume) + Track B 3종 주석 불변식

## Carry-Forward
- **Actionable**: ROADMAP M7-1 ✅ COMPLETE 마킹 + HANDOFF 갱신
- **Structural Improvement Proposals**:
  - M7-2 요구사항 명확화: Z3 IPC pipeline에 String length → bitvector 인코딩 추가
  - Track A 추가 후보: `find_pattern_at`, `find_pattern_at_slow` (pos >= 0) — 낮은 우선순위
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: ROADMAP M7-1 ✅ COMPLETE 마킹
- **Next Recommendation**: Cycle 3078 — ROADMAP M7-1 완료 선언 + M7-2 범위 확정
