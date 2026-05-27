# Cycle 3222: M11-C Phase 1 Cleanup — llvm_intrinsic_is_memory_write helper
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3221 Structural Improvement Proposal — "Consider a general LLVM
intrinsic classification table". ROADMAP also needs updating (ifs_flex_check_goto resolved,
M11-C Phase 1 complete).

**Trigger**: ⚪ NONE — proceed with structural improvement.

## Scope & Implementation

### 1. ROADMAP 업데이트

- `ifs_flex_check_goto Z3 FAIL` → `✅ RESOLVED (Cycle 3219)`
- M11-C Phase 1 완료 현황 테이블 추가 (ipr_all_calls_readonly/pure/ipr_has_store 수정 기록)

### 2. `llvm_intrinsic_is_memory_write` 헬퍼 함수 추출

```bmb
// BEFORE (두 곳에 중복 인라인):
if find_pattern_at(fn_name, "llvm.", 0) >= 0 {
    find_pattern_at(fn_name, "llvm.memset", 0) < 0
    and find_pattern_at(fn_name, "llvm.memcpy", 0) < 0
    and find_pattern_at(fn_name, "llvm.memmove", 0) < 0
}

// AFTER (헬퍼 + 단순화):
fn llvm_intrinsic_is_memory_write(fn_name: String) -> bool =
    find_pattern_at(fn_name, "llvm.memset", 0) >= 0
    or find_pattern_at(fn_name, "llvm.memcpy", 0) >= 0
    or find_pattern_at(fn_name, "llvm.memmove", 0) >= 0;

// 사용처:
if find_pattern_at(fn_name, "llvm.", 0) >= 0 { not llvm_intrinsic_is_memory_write(fn_name) }
```

`ipr_all_calls_pure` (line ~17207)와 `ipr_all_calls_readonly` (line ~17281) 두 곳 모두 헬퍼 사용.

### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `bootstrap/compiler.bmb` | `llvm_intrinsic_is_memory_write` 헬퍼 함수 추가 |
| `bootstrap/compiler.bmb` | `ipr_all_calls_pure` → 헬퍼 사용으로 단순화 |
| `bootstrap/compiler.bmb` | `ipr_all_calls_readonly` → 헬퍼 사용으로 단순화 |
| `claudedocs/ROADMAP.md` | 기술 부채 테이블 업데이트 + M11-C Phase 1 완료 현황 |

Note: `ipr_has_store`는 full IR line에서 `@llvm.memset`을 검색하므로 별도 유지
(fn_name이 아닌 line 검색 — 다른 컨텍스트).

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":141,"failed":0}
```

Fixed Point: **S3 IR == S4 IR ✅** (compiler_3222.exe two runs, diff = 0)

## Reflection

**Scope fit**: Structural improvement proposal from Cycle 3221 fully executed.
No new functionality — pure cleanup that improves maintainability.

**Latent defects**: None found.

**Structural improvement value**:
- Single source of truth for "which LLVM intrinsics write memory"
- Future `llvm.memcpy.inline`, `llvm.memmove.element.unordered.atomic` additions require
  only ONE function update instead of three
- Comment in helper documents the extension pattern

**Philosophy drift**: None — this is internal compiler quality improvement.

**Roadmap impact**: M11-C Phase 1 is fully cleaned up. Ready for Phase 2 decision.

## Carry-Forward

- **Actionable**: Cycle 3223 — Choose direction:
  - Option A: M11-C Phase 2: `[u8; N]` type annotation parser support
  - Option B: Other language gap (closure, generic, improved type inference)
  - Option C: New contract improvements in critical IPR/codegen functions
- **Structural Improvement Proposals**: None new
- **Pending Human Decisions**: None
- **Roadmap Revisions**: ROADMAP § 현재 기술 부채 목록 업데이트 완료
- **Next Recommendation**: Cycle 3223 — Investigate M11-C Phase 2 feasibility
  (parse_block_let_skip_array_type already handles `[T; N]` syntax, but skips N —
  capturing N for auto stack allocation is the remaining gap)
