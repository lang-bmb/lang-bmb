# Cycle 2906: FFI 바인딩 arena-free UB + double-free 전수 수정
Date: 2026-05-17

## Re-plan
HANDOFF Structural Improvement Proposal — str_replace/str_replace_all passthrough 안전성 분석.
이전 세션(컨텍스트 압축 전)에서 Python 바인딩은 SAFE 확인 완료. 이번 사이클에서 Node.js/C#/Java 전수 확인.

## Scope & Implementation

**분석 결과 요약**:

### 발견된 버그 패턴

**근본 원인**: BMB 함수 반환값은 `bmb_alloc`(arena) 할당 — `malloc`이 아님.
- `bmb_ffi_cstr_to_string` → `malloc` (개별 free 필요)
- 모든 BMB string 함수 반환값 → arena (개별 free 금지)

기존 바인딩(Node.js/C#/Java)에서 `BmbStringToCS`/`stringResult`/`_from` 헬퍼가
**반환값에도 `bmb_ffi_free_string` 호출** → arena 메모리에 `free()` = UB.

추가 double-free: str_replace(no-match) / str_trim(empty) 등 passthrough case에서
반환값 == 입력값이므로 입력 free + 반환값 free = double-free.

### 런타임 passthrough 케이스 목록
| 함수 | passthrough 조건 |
|------|-----------------|
| `bmb_string_replace` | no-match 또는 empty old_str |
| `bmb_string_trim` | `len == 0` (empty) |
| `bmb_string_to_upper` | `len == 0` (empty) |
| `bmb_string_to_lower` | `len == 0` (empty) |
| `bmb_string_reverse` | `len == 0` (empty) |

### 바인딩별 상태
| 바인딩 | 상태 | 수정 |
|--------|------|------|
| Python (bmb-text) | ✅ SAFE — 반환값 free 없음 | 불필요 |
| Node.js (bmb-json) | ✅ SAFE — `_from = (ptr) => _str_data(ptr)` 이미 정상 | 불필요 |
| Node.js (bmb-text) | ❌ `_from`에서 `_free_str(ptr)` 호출 | 수정 |
| Node.js (bmb-crypto) | ❌ `_from`에서 `_free_str(ptr)` 호출 | 수정 |
| C# (bmb-text) | ❌ `BmbStringToCS`에서 `bmb_ffi_free_string(p)` | 수정 |
| C# (bmb-crypto) | ❌ `BmbStringToCS`에서 `bmb_ffi_free_string(p)` | 수정 |
| C# (bmb-json) | ❌ `BmbStringToCS`에서 `bmb_ffi_free_string(p)` | 수정 |
| Java (bmb-text) | ❌ `stringResult`에서 `bmb_ffi_free_string(bmbStr)` | 수정 |
| Java (bmb-crypto) | ❌ `stringResult`에서 `bmb_ffi_free_string(bmbStr)` | 수정 |
| Java (bmb-json) | ❌ `stringResult`에서 `bmb_ffi_free_string(bmbStr)` | 수정 |
| C# (bmb-algo) | ✅ SAFE — String 반환 함수 없음 | 불필요 |
| C# (bmb-compute) | ✅ SAFE — String 반환 함수 없음 | 불필요 |
| Java (bmb-algo) | ✅ SAFE — String 반환 함수 없음 | 불필요 |

**수정 내용**: 각 바인딩의 `BmbStringToCS`/`stringResult`/`_from`에서 `bmb_ffi_free_string` 제거.
`_sss`/`_s1str`/`_call1`/`_call2` 헬퍼에서 read-before-free 순서 보장.

### 올바른 FFI 패턴
```
1. bmb_ffi_cstr_to_string(s) → malloc → 개별 free 필요
2. BMB 함수(arena_arg) → arena → free 불필요 (arena bulk-reset)
3. bmb_ffi_string_data(arena_ptr) → 데이터 복사 (managed string)
4. bmb_ffi_free_string(malloc_input) → 입력만 free
```

**변경 파일**:
- `ecosystem/bmb-text/bindings/node/index.js` — `_from` fix + read-before-free
- `ecosystem/bmb-crypto/bindings/node/index.js` — `_from` fix + read-before-free  
- `ecosystem/bmb-text/bindings/csharp/BmbText.cs` — `BmbStringToCS` fix
- `ecosystem/bmb-crypto/bindings/csharp/BmbCrypto.cs` — `BmbStringToCS` fix
- `ecosystem/bmb-json/bindings/csharp/BmbJson.cs` — `BmbStringToCS` fix + docstring
- `ecosystem/bmb-text/bindings/java/io/bmb/text/BmbText.java` — `stringResult` fix
- `ecosystem/bmb-crypto/bindings/java/io/bmb/crypto/BmbCrypto.java` — `stringResult` fix
- `ecosystem/bmb-json/bindings/java/io/bmb/json/BmbJson.java` — `stringResult` fix

## Verification & Defect Resolution
`cargo test --release`: 2388/2388 PASS. 변경 없음 (Rust 코드 미수정).

## Reflection
- **Scope fit**: HANDOFF Structural Improvement 정확 해소.
- **Latent defects**: 없음. Python이 올바른 참조 구현으로 확인됨.
- **Philosophy drift**: 없음. 근본 원인(arena vs malloc 혼용) 정확 수정.
- **Roadmap impact**: Cycle 2907 = libbmb_runtime.a .gitignore로 진행.
- **Python 바인딩이 왜 정확했는가**: ctypes에서 반환값 `out`을 별도 free하지 않고 `bmb_ffi_string_data(out).decode()` 후 입력만 free 패턴을 사용. bmb-json Node.js 작성자도 같은 패턴 적용.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: None
- Pending Human Decisions: B축 재측정, tier3-spawn-overhead Option A/B/C
- Roadmap Revisions: None
- Next Recommendation: Cycle 2907 — libbmb_runtime.a .gitignore 이동
