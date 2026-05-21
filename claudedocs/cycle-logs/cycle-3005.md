# Cycle 3005: PyPI 빌드 에러 연속 수정
Date: 2026-05-21

## Re-plan
Plan valid. Cycle 3004 publish=true 트리거 이후 2개 연속 실패 → 에러 수정 모드.

## Scope & Implementation

### 에러 1: `ecosystem/gotgan` workspace member 누락 (run 26210535322)

**원인**: `Cargo.toml` workspace에 `ecosystem/gotgan` 포함 → `submodules: false`로 체크아웃 시 cargo manifest 읽기 실패.

**수정**: checkout 직후 gotgan만 선택적 init:
```yaml
- name: Init gotgan submodule (Cargo workspace member)
  shell: bash
  run: git submodule update --init ecosystem/gotgan
```
커밋: `0341d92c`

### 에러 2: `Constant::FnRef` non-exhaustive patterns (run 26210866275)

**원인**: `llvm.rs` (inkwell 백엔드)가 `Constant::FnRef` variant를 3개 match arm에서 처리하지 않음.
`llvm_text.rs`는 이미 FnRef를 지원함 — 두 백엔드 불일치 (CLAUDE.md Rule 7 위반).

**3개 위치 수정**:
1. MirType 매핑: `Constant::FnRef(_) => MirType::I64`
2. `constant_type()`: `Constant::FnRef(_) => self.context.i64_type().into()`
3. `gen_constant()`: ptrtoint (ptr @fn to i64) 생성

**수정 코드** (`gen_constant` FnRef arm):
```rust
Constant::FnRef(fn_name) => {
    let func = self.module.get_function(fn_name)
        .unwrap_or_else(|| panic!("FnRef: function '{}' not found in module", fn_name));
    let ptr = func.as_global_value().as_pointer_value();
    self.builder
        .build_ptr_to_int(ptr, self.context.i64_type(), "fnref_int")
        .unwrap()
        .into()
}
```

### 에러 3: `bmb_str_char_at` LLVM redefinition (run 26211468381)

**원인**: `ecosystem/bmb-text/src/lib.bmb`가 `@export pub fn bmb_str_char_at(...) -> i64`를 정의.
  - prelude: `declare nonnull ptr @bmb_str_char_at(...)` (반환형 ptr)
  - user code: `define dllexport i64 @bmb_str_char_at(...)` (반환형 i64)
  - 반환형 불일치 → LLVM "invalid redefinition" (Windows text backend에서만 발생; inkwell backend는 충돌 없이 처리)

**수정**: `emit_runtime_declarations`에 `user_fns` 파라미터 추가, `bmb_str_char_at` declare를 user 정의 시 skip.

커밋: `515a3120`  
새 CI run: 26212564823

## Verification & Defect Resolution
- `cargo build --release -p bmb` ✅ (3m46s)
- CI run 26212564823 진행 중

## Reflection
- **Scope fit**: 연속 결함 발견+수정. 총 3개 CI 차단 버그 처리.
- **Latent defects**:
  1. `Constant::FnRef` inkwell 누락 — CLAUDE.md Rule 7 (두 백엔드 동기화) 위반, Cycle 2933 이후 잠재됨
  2. `bmb_str_char_at` 이름 충돌 — text/inkwell 동작 차이로 CI에서만 드러남
- **Philosophy drift**: 없음.

## Carry-Forward
- Actionable: Cycle 3006 — run 26212564823 결과 확인 + publish 완료 + ROADMAP 갱신
- Structural Improvement Proposals: user-defined builtin 이름 충돌 방지를 위한 일반화된 guard 검토
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: 빌드 통과 확인 → PyPI 패키지 검증 → ROADMAP M3-4 ✅
