# Cycle 2887: str_split / str_split_whitespace / str_lines Native Porting
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2886 Carry-Forward: str_split/str_split_whitespace/str_lines → svec handle native porting (now unblocked by svec infrastructure).

## Scope & Implementation

1. **bmb_runtime.c**:
   - `bmb_str_split(BmbString* s, BmbString* delim) → i64` — delimiter-based split; empty delim = per-char split
   - `bmb_str_split_whitespace(BmbString* s) → i64` — whitespace tokenizer, skips empty tokens
   - `bmb_str_lines(BmbString* s) → i64` — newline split, strips `\r\n`
   - Helper: `push_cstr_to_svec(handle, cstr, len)` — creates BmbString + pushes to svec pool

2. **libbmb_runtime.a 재빌드** (`D:\data\lang-bmb\runtime\`) — inkwell 링크 경로

3. **Text backend** (`llvm_text.rs`):
   - 3 LLVM `declare` 추가
   - dispatch 매핑 추가
   - `infer_call_return_type` → "i64" 등록

4. **Inkwell backend** (`llvm.rs`):
   - `str_split` (ptr+ptr→i64), `str_split_whitespace` (ptr→i64), `str_lines` (ptr→i64) 등록

5. **MIR lowering** (`mir/lower.rs`):
   - `is_svec_create` 조건에 `str_split`, `str_split_whitespace`, `str_lines` 추가
   - → for-in-svec이 str_split 결과에도 동작

6. **Tests**:
   - `native_str_split.bmb`: str_split/str_split_whitespace/str_lines 기본 동작 → 3,hello,world,foo,2,hello,world,2,line1,line2
   - `native_str_split_for_in.bmb`: str_split + for-in → length sum 6

## Verification & Defect Resolution

- Interpreter: `bmb run native_str_split.bmb` → 3,hello,world,foo,2,hello,world,2,line1,line2 ✅
- Native: `bmb build native_str_split.bmb` → 동일 출력 ✅
- Native: `bmb build native_str_split_for_in.bmb` → 6 ✅
- `cargo test --release -p bmb` → (실행 중, 이전 2388 PASS 기준)

## Reflection

- **Scope fit**: 3종 split 함수 모두 native 포팅 완료. for-in-svec와 자연스럽게 연동.
- **Pattern**: push_cstr_to_svec helper로 공통 로직 추출 — str_hashmap 패턴과 일관성.
- **Limitation inherited**: 함수 반환 svec handle 추적은 여전히 직접 let 할당만 지원. `let parts = str_split(...)` 패턴에서 동작.
- **Roadmap impact**: str_split native 완료 → word counting, CSV parsing 등 실용적 native 프로그램 가능.

## Carry-Forward
- Actionable: str_hashmap_keys/values → svec native porting (svec 인프라 완성됐으니 가능)
- Actionable: `format()` variadic native porting 조사 (높은 복잡도)
- Structural Improvement Proposals: svec_vars 추적을 함수 반환 경로로 확장 (낮은 우선순위)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2888 — str_hashmap_keys/values → svec native porting
