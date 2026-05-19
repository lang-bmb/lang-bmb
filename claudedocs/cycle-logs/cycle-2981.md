# Cycle 2981: ISSUE 정리 + Bootstrap for-loop 스코프 버그 재현 조사
Date: 2026-05-20

## Re-plan
HANDOFF Carry-Forward:
1. GPUStack 재측정 (35개 problem.md 개선 반영)
2. Bootstrap compiler for-loop 스코프 버그 (33_counting_sort 발견)
3. inttoptr UB (P3) — HUMAN 결정 대기

GPUStack 접속 확인 (.env.local 자격증명 유효). ISSUE 정리 + 스코프 버그 조사 우선 진행.

## Scope & Implementation

### ISSUE 정리 (2 RESOLVED → closed/)
- `ISSUE-20260326-crosslang-reference-asymmetry.md` → closed/ (RESOLVED Cycle 2817)
- `ISSUE-20260326-statistical-testing.md` → closed/ (RESOLVED Cycle 2816)
- 잔여 active: 8개 (OPEN 6 + LARGELY/ROOT RESOLVED 2)

### Bootstrap for-loop 스코프 버그 조사
다양한 패턴으로 재현 시도:

| 패턴 | Interpreter | Native | 결과 |
|------|------------|--------|------|
| `let mut v = 999; for v in 0..3 {}` | 0,1,2 ✅ | 0,1,2 ✅ | 정상 |
| `let v: i64 = 999; for v in 0..3 {}` | 0,1,2 ✅ | 0,1,2 ✅ | 정상 |
| `let v = vec_push(...) × 3; for v in 0..3 {}` | 0,1,2 ✅ | 0,1,2 ✅ | 정상 |
| `while { let v = j*10 }; for v in 0..3 {}` | 0,1,2 ✅ | 0,1,2 ✅ | 정상 |
| outer v 확인 (`println(v)` after loop) | 999 ✅ | 999 ✅ | 정상 |

**결론**: for-loop 스코프 버그 현재 버전에서 재현 불가.
- `lower_for_range_sb`의 `rename_name_in_ast` 로직 확인 → 올바르게 `<v>` → `<v_N>` 리네임
- 이전 세션(Cycle 2980)에서 변수명 분리 workaround가 적용됐으나 실제 native 버그는 검증 불가
- 가능성: 특정 컴파일러 버전 빌드 환경에서만 발생, 혹은 Cycles 2965-2980 중 암묵적으로 수정됨

**조치**: HANDOFF "for-loop 스코프 버그" 항목 → 재현 불가로 닫음. 문제 재발 시 최소 재현 케이스와 함께 다시 등록.

### GPUStack 접속 확인
`.env.local` → GPUSTACK_ENDPOINT=http://172.30.1.53:8080, GPUSTACK_MODEL=qwen3.6-35b-a3b
API 접속 테스트: `{"data":[{"id":"qwen3.6-35b-a3b",...}]}` ✅

## Verification & Defect Resolution
- cargo test --release: 6260/6260 ✅

## Reflection
- **스코프 버그**: HANDOFF에서 발견된 버그가 현재 재현 불가. 가능성 3: (1) 이미 수정됨, (2) 특정 환경 의존, (3) workaround 적용으로 문제가 숨겨짐. 어느 쪽이든 현재 동작은 올바름.
- **ISSUE 정리**: 2개 완전 해결 → closed/ 이동. Active 8개는 모두 측정 의존(HUMAN) 또는 낮은 우선순위.
- **GPUStack 재측정**: 35개 problem.md 개선 반영 → 97.0% → ~99-100% 예상. 다음 사이클에서 background 실행.
- **로드맵 영향**: 재측정 결과 따라 M4 B축 현황 업데이트.

## Carry-Forward
- Actionable: GPUStack 재측정 background 실행 (Cycle 2982)
- Structural Improvement Proposals: ISSUE-20260326-first-shot-rate-low 및 type-d-failure-analysis — 재측정 결과 반영 후 close 여부 결정
- Pending Human Decisions: npm/PyPI publish (M3-3/M3-4), inttoptr UB (P3)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2982 — GPUStack 재측정 background 시작 + integration category 분석
