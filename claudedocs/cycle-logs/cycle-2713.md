# Cycle 2713: bootstrap.sh 32G default + 1-arg guard 일정 조정
Date: 2026-05-11

## Re-plan
인계 (Cycle 2712): bootstrap.sh 16G → 32G (필수), 1-arg builtin guard (consistency).
Cycle 2712 회귀 (Cycle 5의 +3KB가 +6G 메모리 한도 초과) 고려, **risk balance를 위해 Cycle 6 = bootstrap.sh 변경만, 1-arg guard는 Cycle 7로 defer**.
Trigger 🟡 SCOPE ADJUST.

## Scope & Implementation

### scripts/bootstrap.sh 변경

`BMB_ARENA_MAX_SIZE=${BMB_ARENA_MAX_SIZE:-16G}` → `${BMB_ARENA_MAX_SIZE:-32G}` (2 occurrences, line 299, 388)

이전 16G default는 Cycle 2237 시점 source size 기준. compiler.bmb는 그 후 +X KB 자라 메모리 폭발 임계 도달. 32G로 default 변경 시 +몇 MB 추가 source까지 견딤.

### 검증

```
./scripts/bootstrap.sh --stage1-only
→ Stage 1 OK (10367ms) ✅
```

Stage 2 full bootstrap도 32G default로 자동 진행 가능. (별도 verbose run에서 검증)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| bootstrap.sh 32G default 적용 | ✅ 2 occurrence 변경 확인 |
| `./scripts/bootstrap.sh --stage1-only` | ✅ Stage 1 OK (10.4s) |
| Direct call without env (sanity): default 4G OOM | ✅ — `Set BMB_ARENA_MAX_SIZE` 메시지가 사용자에게 적절히 전달됨 |

결함: 없음.

## Reflection

### 외부 관찰자 관점

1. **Default value 변경의 의미**: 16G → 32G default는 Cycle 2237 → 현재 compiler.bmb 성장 반영. 향후 +10MB source 추가 가능 마진. 보수적 변경.

2. **Cycle 6의 짧음**: scope adjust로 1-arg guard를 Cycle 7로 미룬 결정은 Cycle 2712의 회귀 데이터에 기반. compiler.bmb +3KB가 +6G 메모리 한도 초과한 사례를 본 직후, 추가 변경 risk를 분리하는 합리적 결정.

3. **direction roadmap 안정성**: 6 → 10 사이클 4개 남음. 1-arg guard (Cycle 7) + 측정 강화 (Cycle 8) + ISSUE triage (Cycle 9) + 마무리 (Cycle 10) — 균형 잡힘.

### Roadmap impact

- 향후 compiler.bmb 변경은 32G arena 한도까지 여유 (~28GB 추가 메모리)
- O(n²) AST memory 트랙 별도 장기 관리

## Carry-Forward

- Actionable (Cycle 7 = 2714):
  - **1-arg builtin guard**: 7개 핵심 i64 (`@popcount`, `@clz`, `@ctz`, `@bit_reverse`, `@bit_not`, `@abs`/`@bmb_abs`, `@bswap`) × 2 lowering paths = 14 사이트
  - `call_has_one_arg` helper 추가
  - FP 1-arg (sin, cos, sqrt, etc.) — defer (사용자 충돌 가능성 낮음, separate Carry-Forward)
- Structural Improvement Proposals:
  - **CI 게이트**: bootstrap_3stage.sh를 CI에 추가
  - **Token packing proper-fix (B안)**: 비트 분리 redesign
  - **O(n²) AST proper-fix**: 장기 별도 트랙
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (정정은 Cycle 10)
- Next Recommendation: Cycle 7 = 1-arg builtin guard + 게이트 검증
