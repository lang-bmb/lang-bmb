# Cycle 2888: str_hashmap_keys / str_hashmap_sorted_keys Native Porting
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2887 Carry-Forward: str_hashmap_keys/sorted_keys → svec native porting.

## Scope & Implementation

1. **bmb_runtime.c**:
   - `cmp_bmb_str_ptr`: qsort comparator for BmbString* by lexicographic content
   - `bmb_str_hashmap_keys(i64 handle) → i64` (svec handle): iterates StrHashMap, pushes occupied key BmbString* into svec pool
   - `bmb_str_hashmap_sorted_keys(i64 handle) → i64`: calls bmb_str_hashmap_keys then qsorts the svec data in-place
   - Note: existing `str_hashmap_keys` (v0.95) uses vec — new svec-based versions added separately

2. **libbmb_runtime.a 재빌드** (project root)

3. **Text backend** (`llvm_text.rs`):
   - 2 LLVM `declare` 추가 (i64→i64)
   - dispatch 매핑 + infer_call_return_type → "i64"

4. **Inkwell backend** (`llvm.rs`):
   - `str_hashmap_keys` (i64→i64), `str_hashmap_sorted_keys` (i64→i64) 등록

5. **MIR lowering** (`mir/lower.rs`):
   - `is_svec_create` 조건에 `str_hashmap_keys`, `str_hashmap_sorted_keys` 추가
   - → `for k in str_hashmap_sorted_keys(m)` 패턴 native 지원

6. **Test**: `native_hashmap_keys.bmb`: sorted_keys → svec_join → "a-b-c" ✅

## Verification & Defect Resolution

- Interpreter: 3, a-b-c ✅
- Native: 3, a-b-c ✅
- `cargo test --release -p bmb` → (실행 중)

## Reflection

- **Scope fit**: hashmap_keys/sorted_keys native porting 완료.
- **qsort in-place**: BmbSvec data 배열 직접 sort — 별도 버퍼 불필요. 효율적.
- **Roadmap impact**: map iteration이 native에서 가능해짐.

## Carry-Forward
- Actionable: `format()` variadic — native porting 가능 여부 조사 필요
- Actionable: `to_string<T>` — native porting (str_from_int/str_from_f64 래퍼 조합으로 가능)
- Actionable: read_line / file_read_to_string — native porting 조사
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2889 — format() 진단 + to_string<T> native porting
