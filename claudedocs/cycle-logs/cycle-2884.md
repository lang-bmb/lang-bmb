# Cycle 2884: str_hashmap Native Porting
Date: 2026-05-15

## Re-plan
Plan valid, inherited scope: str_hashmap builtin group (8종) → native (llvm_text.rs + llvm.rs + runtime).

## Scope & Implementation

**목표**: `str_hashmap_new/insert/get/contains/len/delete/free/inc` 를 `bmb build` (native) 에서 작동시키기.

**아키텍처**:
- `bmb_runtime.c`: `bmb_str_hashmap_*` wrapper 함수 8종 추가 (BmbString* key arg ABI)
- `codegen/llvm_text.rs` (text backend): declare 8종 + dispatch mapping + infer_call_return_type
- `codegen/llvm.rs` (inkwell backend): 누락된 4종 추가 (str_hashmap_len, contains, delete→remove, inc→bmb_str_hashmap_inc)

**주요 발견**: 기존 inkwell backend에는 `str_hashmap_new/insert/get/free` 만 있었고 `len/contains/delete/inc` 가 누락. text backend는 전체 추가. 두 백엔드 간 handle ABI: inkwell=ptr, text=i64 (둘 다 x86_64에서 동일 레지스터).

**테스트**: `tests/native_str_hashmap.bmb` 신규 작성 (결과=11 검증).

## Verification & Defect Resolution

- `./target/x86_64-pc-windows-gnu/release/bmb build tests/native_str_hashmap.bmb -o /tmp/test_hashmap.exe && /tmp/test_hashmap.exe` → 출력 11 ✅
- `cargo test --release -p bmb` → 0 FAIL ✅
- bmb_reference.md: str_hashmap_* native-supported 마킹 (keys/sorted_keys/values 는 svec 의존으로 interpreter-only 유지)

## Reflection

- **Scope fit**: 8종 핵심 함수 모두 native 포팅 완료. keys/sorted_keys/values는 SvecHandle 반환이므로 svec 포팅 후 처리 가능.
- **Latent defects**: 없음.
- **Philosophy**: inkwell/text 두 백엔드 동기화 — CLAUDE.md Rule 7 준수.
- **Roadmap impact**: for-in-vec native porting (Cycle 2885) 로 이어짐.

## Carry-Forward
- Actionable: for-in-vec native porting (현재 ChannelRecvOpt MIR → index loop 변환 필요)
- Structural Improvement Proposals: inkwell/text 백엔드 handle ABI 불일치 (ptr vs i64) — 기능상 문제 없으나 장기적으로 통일 고려
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2885 — for-in-vec native porting (vec_vars tracking + index loop codegen)
