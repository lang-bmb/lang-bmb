# Cycle 3083: P0 defect — `Expr::It` 타입 체커 수정
Date: 2026-05-25

## Re-plan
Cycle 3082 Carry-Forward: `bmb verify` end-to-end 검증 미완 — advisor 지적으로 발견.
`post it.starts_with("bmb_")` 실행 시 `Type error: unknown method 'starts_with' for i64` 발생.
근본 원인 분석 → `Expr::It` 타입 체커 플레이스홀더 버그 수정.

## Scope & Implementation

### 결함: `Expr::It` → 항상 `Type::I64` 반환

**파일**: `bmb/src/types/mod.rs` (line 3271)

Before:
```rust
Expr::It => {
    // For now, return a placeholder type; actual type comes from context
    Ok(Type::I64)
}
```

After:
```rust
// v0.2: Refinement self-reference — type equals the enclosing function's return type
// (same mechanism as `ret`; fallback to i64 for refined-type constraints outside fns)
Expr::It => Ok(self.current_ret_ty.clone().unwrap_or(Type::I64)),
```

`Expr::Ret`와 동일 패턴: `current_ret_ty`가 설정돼 있으면 해당 타입, 없으면 `i64` fallback.

### 신규 테스트 2개

- `test_tc_post_it_string_method`: `fn get_name() -> String post it.starts_with("bmb_")` → OK ✅
- `test_tc_post_it_i64_comparison`: `fn double(x: i64) -> i64 post it > 0` → OK ✅

### End-to-end 검증

`bmb verify tests/golden/test_it_post_string.bmb --human`:
- `get_bmb_name` (body `"bmb_hello"`, post `it.starts_with("bmb_")`) → **verified** ✅
- `get_other_name` (body `"other_value"`, post `it.starts_with("bmb_")`) → **failed** (counterexample: `__it__ = "other_value"`) ✅

Smoke test 파일 삭제: 회귀는 Rust 유닛 테스트 2개로 커버.

## Verification & Defect Resolution

- `cargo test --release`: **6278 PASS** ✅ (6276 → 6278, +2 신규)
- `bmb verify bootstrap/compiler.bmb`: **1513/1513** ✅ (회귀 없음)

## Reflection

- **Scope fit**: 100% — 단일 버그 수정
- **Philosophy drift**: 없음
- **Roadmap impact**: Cycle 3082의 `post it.method()` 인프라 완성 완결 — 타입 체커 정합성 확보

이 버그가 왜 수십 사이클 동안 미발견이었는가: `Expr::It`는 주로 refinement type `T{it > 0}` 컨텍스트에서 사용돼 왔는데, 그 경우 타입 체커가 `i64` fallback을 반환해도 `it > 0` 같은 비교 연산이 통과했다. `it.starts_with(...)` 같은 String 메서드 호출은 이번에 처음 사용되어 미발견됐다.

## Carry-Forward

- **Actionable**: 없음
- **Structural Improvement Proposals**:
  1. `Expr::It`에 대한 정확한 타입 추론 — 현재는 함수 컨텍스트에서만 작동; refinement type `i64{it > 0}` 형식에서도 베이스 타입 추론이 필요할 수 있음 (현재 타입 체커 플로우 확인 필요)
- **Pending Human Decisions**: M7-3 scope (complex contract 문법 확장: let-in-pre, quantifiers, array contracts) + M8 계획
- **Roadmap Revisions**: 없음
- **Next Recommendation**: M7-3 착수 또는 다음 실용적 개선 탐색 (HUMAN 결정 대기)
