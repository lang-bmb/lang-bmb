# Cycle 2886: BmbSvec Native Infrastructure + for-in-svec + str_len Fix
Date: 2026-05-15

## Re-plan
Plan valid, inherited scope from Cycle 2885 Carry-Forward: svec native infrastructure.

## Scope & Implementation

1. **BmbSvec C 인프라** (`bmb/runtime/bmb_runtime.c`):
   - `BmbSvec` struct: `{ BmbString** data; int64_t len; int64_t cap }`
   - Global pool 패턴 (handle = pool index)
   - 8종 C 함수: `bmb_svec_new/push/len/get/free/join/index_of/contains`

2. **libbmb_runtime.a 재빌드** (project root — inkwell link path):
   - `D:\data\lang-bmb\runtime\libbmb_runtime.a` 재빌드 필수 (bmb/runtime/*.a 아님)
   - `gcc -O2 -c bmb_runtime.c` → `ar rcs` 로 갱신

3. **Text backend** (`codegen/llvm_text.rs`):
   - 8종 LLVM declare 추가 (nocallback, nounwind 등 속성)
   - dispatch 매핑 + infer_call_return_type 등록

4. **Inkwell backend** (`codegen/llvm.rs`):
   - 8종 함수 등록 (i64→i64, i64+ptr→i64, i64+i64→ptr 시그니처)
   - **bonus fix**: `str_len` alias 추가 — 기존 `"len"` 키만 있었고 `"str_len"` 미등록 버그 수정

5. **MIR lowering** (`mir/lower.rs`):
   - svec_vars HashSet으로 for-in-svec 탐지 (vec_vars 패턴 동일)
   - `for s in svec_var` → index 루프 (`svec_len` + `svec_get`) 생성
   - 루프 변수 elem_mir_ty = `MirType::String`

6. **MIR context** (`mir/mod.rs`):
   - `svec_vars: HashSet<String>` 필드 추가

7. **Tests** (`tests/`):
   - `native_svec.bmb`: svec_new/push/len/contains/index_of/join/free → 3,1,0,1,hello-world-foo
   - `native_for_in_svec.bmb`: for-in-svec + str_len → 총 length 6

## Verification & Defect Resolution

- `bmb build tests/native_svec.bmb` → 3, 1, 0, 1, hello-world-foo ✅
- `bmb build tests/native_for_in_svec.bmb` → 6 ✅
- `cargo test --release -p bmb` → 2388 PASS, 0 FAIL ✅

**발견된 defect**: inkwell backend에서 `str_len` 키 미등록 (기존 `"len"` 만 있었음)
→ 즉시 수정: `self.functions.insert("str_len".to_string(), len_fn)` alias 추가

**발견된 환경 이슈**: inkwell 링크 경로는 `D:\data\lang-bmb\runtime\libbmb_runtime.a` (project root)
— `bmb/runtime/libbmb_runtime.a`가 아님. verbose 빌드로 확인 후 올바른 경로 재빌드.

## Reflection

- **Scope fit**: svec 8종 + for-in-svec + str_len alias fix 모두 완료.
- **Pattern**: BmbSvec global pool (handle-based) — str_hashmap과 동일 패턴으로 일관성 확보.
- **str_len fix**: 잠재 버그로 str_len을 직접 호출하는 native 프로그램이 모두 실패했을 것. 중요 수정.
- **Roadmap impact**: svec ✅ → 이제 str_split (→ svec 반환) native porting 가능해짐.

## Carry-Forward
- Actionable: str_split/str_split_whitespace/str_lines (→ svec) native porting (svec 인프라 완성됐으니 가능)
- Structural Improvement Proposals: str_hashmap_keys/sorted_keys/values도 svec 리턴으로 native 가능 (medium complexity)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2887 — str_split → svec native porting + format() 진단
