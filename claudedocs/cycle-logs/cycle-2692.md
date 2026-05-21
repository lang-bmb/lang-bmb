# Cycle 2692: Nested field path 일반화 (set field chain)
Date: 2026-05-11

## Re-plan
Carry-Forward (Cycle 2691): nested field path 갭. Trigger 없음.

## Scope & Implementation

### `parse_set_field` 재귀화

`parse_set_field`를 `parse_set_field_chain` helper 호출로 단순화. chain helper는 한 번에 한 field를 읽고 다음 토큰으로 분기:
- `.` → chain 계속 (재귀, prev_base 누적)
- `[` → `parse_set_field_chain_index` (terminal)
- assign op → set_field terminal

`prev_base`는 `"(var <name>)"` → `"(field <prev> f1)"` → ... 누적.

### AST 생성 예시

| 입력 | 생성 AST |
|------|----------|
| `set o.f = v` | `(set_field (var <o>) f v)` (기존과 동일) |
| `set o.f[i] = v` | `(set_index (field (var <o>) f) i v)` (Cycle 2690) |
| `set o.a.b = v` | `(set_field (field (var <o>) a) b v)` (**신규**) |
| `set o.a.b[i] = v` | `(set_index (field (field (var <o>) a) b) i v)` (**신규**) |
| `set o.a.b.c = v` | `(set_field (field (field (var <o>) a) b) c v)` (**신규**) |

## Verification & Defect Resolution

| 케이스 | 결과 |
|--------|------|
| basic i64 (회귀) | ✅ exit 42 |
| Array<f64> (회귀) | ✅ exit 42 |
| Array<String> (회귀) | ✅ exit 42 |
| compound `+=`/`-=`/`*=` (회귀) | ✅ exit 42 |
| **nested set_index** `set o.inner.tags[0] = ...` | ✅ exit 42 ("BLUE green") |
| **nested set_field simple** `set o.inner.name = ...` | ✅ exit 42 ("bob") |
| **3-level chain** `set o.mid.inner.val = ...` | ✅ exit 42 ("done") |

결함: 없음.

## Reflection

**구조적 평가**:
- AST 차원 desugar 패턴이 nested chain까지 자연 확장 — step_set_field가 base를 EX로 평가하므로 nested `(field ...)` 표현도 정상
- 신규 AST/MIR 노드 0개 — Rule 5 (이중 lowering) 자동 만족
- LLM 자연 패턴 1개 추가 해소 (Drift C)

**도그푸딩 영향**:
- compiler.bmb 자체에서 nested struct field set이 가능 → 향후 AST 변환 코드 단순화 가능 (사용처 발굴 필요)

**Roadmap impact**:
- Phase 1 단축: Cycle 2693에서 골든 등록 + 회귀 → Phase 2 (Tier 1 inproc) 조기 진입

## Carry-Forward
- Actionable: 7개 신규 골든 등록 + 회귀 검증 (cargo test 6210 유지 확인)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음 (Phase 1 마무리 정상 진행 중)
- Next Recommendation: ISSUE-20260511 closed 이동 + Phase 2 (Tier 1 bench inproc) 진입
