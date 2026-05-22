# Cycle 3056: ISSUE-20260522 GEP Bug 수정 (P1) ✅
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3055): ISSUE-20260522 GEP bug 수정 (P1, native codegen).
계획 유효.

## Scope & Implementation

**Root cause 분석**:
- `let endpoint = getenv("...")` 에서 `getenv` 반환 타입이 `MirType::String` 대신 `MirType::I64`로 인식됨
- `bmb/src/mir/lower.rs:1685` — String 반환 함수 목록에 `"getenv"`, `"exec_with_stdin"` 누락
- `"prefix: " + endpoint` 평가 시: `lhs_ty = MirType::String (ptr)`, `rhs_ty = MirType::I64` → `is_ptr_add = true` 조건 충족
- `MirInst::PtrOffset` 생성 → `getelementptr inbounds i64, ptr "literal", i64 <ptr_as_int>` → 잘못된 IR

**수정** (`bmb/src/mir/lower.rs:1685`):
```rust
// 추가:
| "getenv" | "exec_with_stdin"
```

## Verification & Defect Resolution

1. 재현 테스트:
   ```bmb
   fn main() -> i64 = {
       let endpoint = getenv("TEST_EP");
       let _r = println("prefix: " + endpoint);
       0
   };
   ```
   수정 전: `error: expected value token` (GEP 오류)
   수정 후: `prefix: hello` (정상 출력) ✅

2. `cargo test --release`: 6264+ tests, 0 FAIL ✅

3. ISSUE 종결 기준 2/3 충족:
   - [x] `bmb build` 성공 (Option A)
   - [x] `cargo test --release` 회귀 없음

## Reflection

- Scope fit: 100% (P1 버그 수정 완료)
- 수정 규모: 1줄 (최소 패치 원칙 준수)
- 영향 범위: `getenv`/`exec_with_stdin` 반환값을 직접 변수에 바인딩 후 string concat하는 모든 패턴 수정
- ISSUE-20260522 → closed/ 이동 ✅

## Carry-Forward
- Actionable: M6-P3 gotgan (Rust→BMB) 분석 시작 → Cycle 3057
- Structural Improvement Proposals: lower.rs의 String 반환 함수 목록을 llvm_text.rs와 동기화하는 automated check (P3 제안)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3057 — M6-P3 gotgan 분석
