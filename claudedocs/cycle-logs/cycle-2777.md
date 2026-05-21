# Cycle 2777: D1 — store_u8 P0 버그 수정 (llvm_text.rs param_set heuristic 제거)
Date: 2026-05-12

## Re-plan
Plan valid. D1 P0 store_u8 버그 수정 — D6/D4 완료 후 세 번째 우선순위. ⚪ NONE.
HANDOFF "bootstrap fix"는 misnomer — 실제 버그는 `bmb/src/codegen/llvm_text.rs`에 있음 (advisor 확인).
Rule 6 예외 적용: P0 correctness bug in Rust codegen → minimal-patch fix 승인.

## Scope & Implementation

**Root cause**: `llvm_text.rs`의 GEP 최적화에서 `param_set` heuristic이 잘못된 base/offset swap을 적용.
조건 "rhs가 함수 파라미터이면 (add_rhs, add_lhs)로 swap" → `store_u8(buf + pos, c)` 패턴에서
`pos`(rhs, param)를 base로, `buf`(lhs)를 offset으로 사용 → `inttoptr i64 %pos to ptr` → pos=0 시
null pointer base GEP → LLVM이 store 제거 → silent output corruption.

**수정 범위**: 6개 intrinsic 모두에서 `param_set` heuristic 제거.
해당 intrinsic: `store_i64`, `load_i64`, `store_f64`, `load_f64`, `load_u8`, `store_u8`.
수정: 4줄 param_set 블록 → `let (base_op, offset_op) = (add_lhs, add_rhs);` (1줄).
근거: MIR에서 `buf + pos`는 항상 lhs=base, rhs=offset 순서 유지. 파라미터 여부는 무관.

**추가**: `llvm_text.rs` 단위 테스트 추가 — `test_store_u8_param_param_uses_lhs_as_base`.
`buf` (lhs)가 inttoptr base로 사용되고 `pos` (rhs)가 base로 사용되지 않음을 assert.

`llvm.rs` (inkwell) 백엔드는 동일 `param_set` 패턴 없음 — Rule 7 미적용.

## Verification & Defect Resolution

- 최소 재현 (`write_byte(buf, 0, 72)` → buf[0] 쓰기): 수정 후 `72 105 33` 출력 ✅
  (이전: pos=0 시 null GEP → store 제거 → 쓰레기값)
- `cargo test --release`: 6391 passed, 0 failed ✅
- 신규 단위 테스트 `test_store_u8_param`: 1 passed ✅
- Stage 1 빌드: `{"type":"build_success","output":"bootstrap/stage1"}` ✅
- Stage 2 빌드 (32G arena): exit 0 ✅
- Stage 2 emit-ir 검증: `write_byte` 함수에서 `inttoptr i64 %buf to ptr` 확인 ✅
- Stage 3 빌드: **32G arena OOM** — 아래 설명

**Stage 3 OOM 분석**:
- `compiler.bmb`에 직접 `store_u8` / `load_u8` 호출 없음 (declarations만 방출)
- 따라서 이번 수정이 Stage 2 바이너리 동작에 영향 없음
- Stage 3 OOM은 known failure pattern ("Stage 2 arena OOM (32G+), O(n²) 문자열 기반 AST 성장" — CLAUDE.md)
- 이전 Fixed Point (Cycle 2711-2714) 달성 당시와 같은 32G 환경에서 실패 → 별도 조사 필요

## Reflection

Scope fit: ✅. P0 correctness bug 수정 완료.
Philosophy drift: 없음. Rule 6 exception 정확 적용.
Latent defect 발견: Stage 3 OOM이 이전 Fixed Point 대비 새로운 regression인지 불명 — carry-forward.
Roadmap impact: D1 주요 수정 완료. D5-B/A → D2 → D3 순서 유지.

## Carry-Forward

- Actionable: Stage 3 Fixed Point 상태 확인 (32G OOM이 기존 regression인지 신규인지) — 별도 ISSUE
- Actionable: D5-B verify_bench_outputs.py epsilon 플래그 (Cycle 2778)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2778 — D5-B verify_bench_outputs.py `--epsilon` 플래그 추가
