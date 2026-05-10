# Cycle 2633: M5-1 Payload Enum — Core Implementation
Date: 2026-05-10

## Re-plan
Plan valid. Cycle 2632 Carry-Forward: "M5-1 시작 (다음 5 cycles)". 이전 세션에서 M5-1 설계 결정 완료 (`%EnumValue = {i64, i64}`, heap calloc 2-word, unit/payload 통합 표현). Cycle 2633 범위: bootstrap/compiler.bmb에 payload enum 파싱/lowering/codegen 구현 + golden test.

## Scope & Implementation

**M5-1 핵심 구현 — 13개 편집 (bootstrap/compiler.bmb)**:

1. Match arm 파서 — `Type::Variant(v)` payload binding → `(enum_payload_bind ...)` AST 노드
2. `parse_match_arm_body` — `(enum_payload_bind ...)` 패턴 처리
3. `parse_enum_variants_to_registry` — `Variant[Type]` 형식으로 payload 변수 등록
4. `count_variant_index` — `[...]` 제거 후 비교
5. 헬퍼 추가 — `enum_has_payload`, `enum_variant_has_payload`, `variant_has_bracket`
6. `resolve_enum_variants_in_ast` — multi-pass: tag check + payload extract + ev resolution
7. `resolve_ev_sb` — context-aware (match context + standalone 모두 처리)
8. 리졸버 확장 — `resolve_ev_one`, `resolve_ev_match_ctx`, `resolve_tag_checks`, `resolve_payload_extracts`, `resolve_payload_calls_in_ast`
9. `lower_expr_sb` — `enum_val` dispatch 추가 (recursive lowering)
10. 신규 `lower_enum_val_sb` — calloc(2,8) + field-store tag + field-store payload
11. `llvm_gen_field_store` — numeric field name 지원 (`is_digit` 체크)
12. `compile_program` — `resolve_payload_calls_in_ast` pass 추가
13. **[Critical Fix]** `step_expr` — `enum_val` dispatch 추가 (iterative lowering)

**핵심 버그 수정 (편집 13)**: `step_expr`(iterative work-based lowering)에 `enum_val` case 누락. `make_step_leaf(cur_temp)`로 fall-through → temp 진행 0 → `val_id = cur_temp - 1 = -1` → `%_t-1` LLVM IR 생성. `lower_enum_val_sb` 위임으로 수정.

**Golden test 추가**:
- `tests/bootstrap/test_golden_enum_payload.bmb` — `enum Option { None, Some(i64) }` + match
- `tests/bootstrap/golden_tests.txt` 엔트리 2834 추가 (`test_golden_enum_payload.bmb|42`)

**생성 LLVM IR 확인** (None + Some(42)):
```
%_t0_ptr = call ptr @calloc(i64 2, i64 8)   ; None alloc
store i64 0, ptr %_t0_fs0_ptr               ; tag=0
store i64 0, ptr %_t0_fs1_ptr               ; payload=0
%_t6_ptr = call ptr @calloc(i64 2, i64 8)   ; Some(42) alloc
store i64 1, ptr %_t6_fs0_ptr               ; tag=1
store i64 42, ptr %_t6_fs1_ptr              ; payload=42
```
Match arm: tag 비교 후 payload GEP → load → 반환.

## Verification & Defect Resolution

**cargo test --release**: ✅ 23/23 통과

**Stage 1 빌드**: ✅ `bmb-stage1.exe` 정상 생성

**Stage 1 → golden test**: ✅ 출력 `42` 확인

**Stage 2 Fixed Point**: ❌ BLOCKED — arena OOM (`[bmb] FATAL: arena memory limit exceeded (16384 MB / 16384 MB max)`)
- **Pre-existing 확인**: git stash로 M5-1 변경 제거 후 baseline Stage 1 빌드 → 동일 16G OOM
- **M5-1 미기여**: OOM은 Cycle 2237 이후 compiler.bmb 성장에 의한 사전 존재 문제
- Fixed Point 검증 이번 사이클에서 불가

**발견된 결함 (편집 13으로 수정 완료)**:
- `step_expr` 미처리 `enum_val` → `%_t-1` 생성 → Stage 1 빌드 실패
- 두 lowering 시스템(recursive + iterative) 모두 신규 노드 처리 필수 (기존 패턴: struct_init, lambda 동일)

## Reflection

**Scope fit**: M5-1 핵심 목표 달성. `enum Option { None, Some(i64) }` + `match` + payload binding — 기능 작동.

**Latent defects**:
- Stage 2 OOM (pre-existing): compiler.bmb 약 20K LOC → arena 한계 초과. 장기 방치 시 Fixed Point 검증 영구 차단.
- match arm에서 `_` (underscore) wildcard 미지원 — CLAUDE.md에 문서화된 known gap. M5-2 후보.
- Result enum (`Ok(v)`, `Err(e)`) — Option 성공 후 자연스러운 다음 단계. M5-2 후보.

**Structural improvement opportunities**:
- Bootstrap arena OOM: `BMB_ARENA_MAX_SIZE` 환경변수 기본값 32G 이상으로 조정 또는 증분 컴파일 도입 필요. Fixed Point 검증 복원에 필요.
- 두 lowering 시스템(recursive vs iterative) 분기 복잡도: 신규 AST 노드 추가마다 두 곳 모두 수정 필요. 장기적으로 단일화 검토.

**Philosophy drift**: 없음. Heap allocation + calloc은 runtime overhead로 보일 수 있으나, enum 값의 크기가 정적으로 결정되므로 escape analysis 후 stack promotion 가능 (M5+ 최적화).

**Roadmap impact**: M5-1 1차 구현 완료. Fixed Point 차단은 M5 진행을 막지 않음 (Stage 1 + golden test로 기능 검증 가능). Arena OOM은 별도 처리 필요.

## Carry-Forward
- Actionable: 회귀 테스트 — struct/enum golden test 실행 확인 (편집 3, 8, 11 영향 범위)
- Actionable: HANDOFF.md + ROADMAP.md M5-1 상태 반영
- Actionable: PyPI windows-2022 수정 push + 재실행 (Cycle 2632 Carry-Forward)
- Structural Improvement Proposals:
  - **Arena OOM**: compiler.bmb Stage 2 Fixed Point 복원 — 기본 arena 32G 이상 또는 증분 파싱 도입. Rationale: Fixed Point 검증은 bootstrap 정확성의 핵심 게이트.
  - **Lowering 이중화**: recursive + iterative 두 시스템에 동일 노드 추가 필수 규칙 → CLAUDE.md Rule 3 보완 항목으로 추가.
- Pending Human Decisions: 없음
- Roadmap Revisions: M5-1 완료 (Stage 1 + golden test 기준). M5-2 다음: `_` wildcard + Result enum.
- Next Recommendation: Cycle 2634 — 회귀 테스트 확인 + HANDOFF 갱신 + commit + PyPI windows-2022 push
