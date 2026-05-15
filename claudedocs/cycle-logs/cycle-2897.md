# Cycle 2897: M4-6 C# 바인딩 완료 검증 + bmb_json_type FFI 버그 수정
Date: 2026-05-15

## Re-plan
Carry-Forward 없음. ROADMAP ④ 바인딩 항목 점검. C# 바인딩이 ROADMAP에 "3-5 cycles" 예상으로 미완료로 기재되어 있었으나, 조사 결과 5개 라이브러리 모두에 C# P/Invoke 바인딩이 이미 구현되어 있었음. 검증 + 버그 수정 사이클로 전환.

## Scope & Implementation

**발견 사항**: 5개 ecosystem 라이브러리 모두 C# 바인딩 존재
- `bmb-algo/bindings/csharp/BmbAlgo.cs` (276줄) — 33/33 ✅
- `bmb-json/bindings/csharp/BmbJson.cs` (162줄) — 빌드 가능하나 실행 실패
- `bmb-compute/bindings/csharp/BmbCompute.cs` (261줄) — DLL 로드 실패
- `bmb-crypto/bindings/csharp/BmbCrypto.cs` (130줄) — DLL 로드 실패
- `bmb-text/bindings/csharp/BmbText.cs` (188줄) — DLL 로드 실패

### 버그 1: Native DLL 미복사 (bmb-json/compute/crypto/text)
**원인**: 4개 라이브러리 .csproj 파일에 `<Content>` 규칙 누락. 네이티브 DLL이 빌드 출력 디렉토리에 복사되지 않음.
**수정**: 4개 `.csproj` 파일에 `<Content Include="../../bmb_xxx.dll" CopyToOutputDirectory="PreserveNewest"/>` 추가.

### 버그 2: bmb_json_type crash (P0 FFI 정확성 버그)
**원인**: `pub fn bmb_json_type(...) -> String`이 string literal (`"null"`, `"bool"` 등)을 직접 반환. 코드젠이 global BmbString struct 포인터(`@str_null.bmb`)를 ret함. C# `BmbStringToCS` 함수가 이 포인터로 `bmb_ffi_free_string` 호출 → `free(.rodata)` = UB/crash (exit 127).

**근본 원인**: 코드젠의 `v0.51.22` 최적화 ("String constant returns use pre-initialized global BmbString") — global struct를 반환하지만 FFI caller는 heap-allocated string을 기대하고 free() 호출.

**수정**: `bmb_json_src/lib.bmb`의 `bmb_json_type` 함수에서 static literal 대신 `str_repeat(type_name, 1)`로 heap copy 생성 후 반환. `str_repeat`는 `bmb_alloc` (malloc 모드에서 실제 malloc) 사용 → `bmb_ffi_free_string`으로 안전하게 해제 가능.

Rule 6 적용: P0 FFI 정확성 버그 — `pub fn -> String`이 global pointer를 반환하여 caller가 free() 시 crash. 최소 패치 (lib.bmb 1함수, 8줄 수정).

**Files changed**:
- `ecosystem/bmb-json/bindings/csharp/BmbJson.csproj` — DLL content 추가
- `ecosystem/bmb-compute/bindings/csharp/BmbCompute.csproj` — DLL content 추가
- `ecosystem/bmb-crypto/bindings/csharp/BmbCrypto.csproj` — DLL content 추가
- `ecosystem/bmb-text/bindings/csharp/BmbText.csproj` — DLL content 추가
- `ecosystem/bmb-json/src/lib.bmb` — `bmb_json_type` heap-allocate fix
- `ecosystem/bmb-json/bmb_json.dll` — rebuild
- `claudedocs/ROADMAP.md` — M4-6 ✅ 완료 마킹

## Verification & Defect Resolution
재빌드 후 전체 테스트:
- bmb-algo: **33/33 ✅**
- bmb-json: **14/14 ✅** (Type object/array/number 포함)
- bmb-compute: **17/17 ✅**
- bmb-crypto: **10/10 ✅**
- bmb-text: **19/19 ✅**
- **총계: 93/93 ✅**

cargo test --release: 코드 변경 없음 (Rust 소스 미수정), 불필요.

## Reflection
- **Scope fit**: M4-6 "C# 바인딩 scaffold" 실제로는 이미 구현됨. 이번 사이클은 검증 + 버그 수정.
- **Latent defects**: bmb_json_type FFI crash는 모든 "static literal -> String" 패턴에 잠재. 다른 라이브러리에 같은 패턴이 있으면 동일 crash 발생. bmb-text/compute/crypto는 동적 문자열만 반환하므로 이 버그 없음.
- **Structural improvement**: 코드젠이 `@export pub fn -> String` 반환 시 global pointer를 직접 반환하는 최적화(`v0.51.22`)는 FFI safety 위반. 장기적으로 export 함수의 String 반환은 반드시 malloc-based copy를 생성해야 함. 이는 코드젠 레벨 수정 (bootstrap → Rule 6 제약).
- **Roadmap impact**: M4-6 ✅ 완료. ROADMAP 갱신.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - 코드젠: `@export pub fn -> String`이 static literal return 시 `bmb_ffi_cstr_to_string` equivalent를 경유해 항상 malloc-based copy 반환하도록 수정 (bootstrap 레벨, Rule 6)
  - 다른 lib.bmb 파일들에도 static literal return이 있는지 점검 (`bmb_json_get`, `bmb_json_array_get` 등 검토 필요 — 현재 동적 구성이므로 안전)
- Pending Human Decisions: None
- Roadmap Revisions: M4-6 ✅ 완료 마킹
- Next Recommendation: Cycle 2898 — HANDOFF 갱신 후 세션 마무리 또는 추가 작업 점검
