# Cycle 2861: for-in svec — Value::SvecHandle 도입
Date: 2026-05-15

## Re-plan
Plan valid, inherited scope: Structural Improvement #1 from HANDOFF (for x in svec {}).

## Scope & Implementation
- `bmb/src/interp/value.rs`: `Value::SvecHandle(usize)` 추가 + is_truthy/type_name/Display/PartialEq 업데이트
- `bmb/src/interp/eval.rs`:
  - `svec_idx(v: &Value) -> InterpResult<usize>` helper (SvecHandle + Int 양쪽 수용)
  - `svec_new()` → `Value::SvecHandle` 반환 (기존: `Value::Int`)
  - `str_split()` → `Value::SvecHandle` 반환
  - `str_hashmap_keys()` / `str_hashmap_sorted_keys()` → `Value::SvecHandle` 반환
  - `svec_len/get/free/join/push/sort/contains/remove/clear` 모두 `svec_idx` helper 사용
  - `Expr::For` `Value::SvecHandle(idx)` branch 추가 (SVEC_REGISTRY에서 strings 읽어 Str 바인딩)
- `bmb/tests/integration.rs`: `test_interp_for_in_svec` 5개 assertion 추가

## Verification & Defect Resolution
- `cargo test --release -p bmb`: 2382 → **2383 PASS** ✅
- defect 없음

## Reflection
- Scope fit: ✅ for x in svec {} 완전히 동작
- Type system gap: svec 핸들은 타입 체커에서 `i64`로 추론 → body에서 String 연산 타입 에러. 테스트는 `cnt += 1` 패턴(i64 연산만)으로 우회. 이는 interpreter-only 한계.
- Roadmap: M4 ① 언어 갭 svec iteration 추가 완료.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  1. 타입 체커 `Expr::For`에서 `svec_new()/str_split()` 반환 타입을 추적해 `elem_ty = Type::String` 추론 — 현재 `i64`만 인식
- Pending Human Decisions: 없음
- Roadmap Revisions: M4 ① for-in-svec ✅ 추가
- Next Recommendation: Cycle 2862 — 타입 체커 for-in svec String 추론 OR 새 언어 갭 발굴
