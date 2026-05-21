# Cycle 2684: struct field array mut 파서 갭 발견 → 이슈 등록
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2683): struct field Array<f64> mut set + 안정성 검증.
트리거 없음. 진행.

## Scope & Implementation

### 발견
`set s.values[0] = 99.0` (field-then-index set) 파서 미지원:
```
error[parse]: expected '=', '+=', '-=', '*=', or '/=' in set, got '['
```

`parse_set_field` (line 996)이 `set obj.field = val` 만 처리. 다음 토큰 `[` fall-through.

M5-5d의 String 동형 케이스도 같은 제한 — `set b.tags = [...]` (전체 재할당)만 지원함을 확인.

### 결정
- 본격 작업 (AST 신규 + lowering 추가 + 전수 검색) → 한 사이클 위험
- 이슈 등록 + 다음 세션 task로 이관
- workaround 검증: 전체 재할당 `set s.values = [99.0, ...]` 동작

### 산출물
- `claudedocs/issues/ISSUE-20260511-set-field-index.md` 등록
  - 현상 + 원인 + 4-point fix 예상
  - 다음 세션 자율 작업 (2-3 cycles)
- 골든 회수: 작성한 mut golden 파일은 파싱 실패하므로 제거

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed |
| Stage 1 빌드 | ✅ OK (재빌드 불필요) |
| workaround (전체 재할당) | ✅ 동작 확인 (M5-5d String 케이스에서) |

결함: 1개 발견 + 이슈로 영구 등록. 본 사이클 스코프 외이므로 carry-forward 처리.

## Reflection

**Scope fit**: 안정성 검증 목표는 달성. 무구현 통과 변형 4개 + 파서 갭 1개 발견.

**Latent defects**:
- `set obj.field[idx] = val` — 이슈 등록 (ISSUE-20260511-set-field-index)
- nested set `set o.inner.values[0] = x` — 함께 검토 필요 (이슈 내 기록)

**Structural improvement opportunities**:
- `parse_set_field` 분기 확장 — `[` 또는 `.` 다음 토큰 처리 통합
- AST `set_field_index` 신규 노드 lowering — 이중 lowering 시스템 점검 필수

**Philosophy drift**: 없음.
- 본격 작업이 한 사이클에 무리 → 이슈 등록은 올바른 선택 (Rule 6: 복잡도 회피 ❌ 가 아닌 분할).
- workaround는 "전체 재할당" 으로 명시 (CLAUDE.md Workaround 금지와 충돌 X — 임시 회피책으로 사용자가 명시적 선택).

**Roadmap impact**:
- M5-5는 이미 7/7 완료 선언됨 — 이 갭은 M5-5 외 (set 파서 일반화 문제)
- Cycle 2685-2687은 원래 계획 (측정/도그푸딩) 으로 진행
- 이슈는 다음 세션 우선순위로

**User-facing quality**: LLM이 자연스러운 `set s.values[0] = x` 작성 시 즉시 실패 — UX 개선 필요. Drift C (AI-native 갭) 활성.

## Carry-Forward
- Actionable:
  - Cycle 2685: in-process timing benchmark-bmb 적용 검토
  - Cycle 2686+: 도그푸딩 갭 발견 시 등록
- Structural Improvement Proposals:
  - **다음 세션 우선순위**: `set obj.field[idx] = val` 파서 확장 (ISSUE-20260511)
- Pending Human Decisions: 없음 (이슈는 자율 작업 범위)
- Roadmap Revisions: cycle-logs/ROADMAP.md: Cycle 2685-2687을 측정/도그푸딩 강화로 (원래 계획 유지)
- Next Recommendation: **Cycle 2685 — in-process timing benchmark-bmb 적용 검토**
