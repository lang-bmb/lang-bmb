# Cycle 2910: bmb-crypto C 바인딩 scaffold
Date: 2026-05-18

## Re-plan
Cycle 2909 Carry-Forward: bmb-crypto C 바인딩.

## Scope & Implementation

**생성 파일** (`ecosystem/bmb-crypto/bindings/c/`):
- `Makefile`, `example.c`, `test.c`, `README.md`

**주요 발견 — arena-free UB (Cycle 2906 동일 문제)**:
- bmb-crypto 14개 함수 모두 `void* → void*` (String 기반).
- `@export` 반환 `void*`는 arena-backed: `bmb_ffi_end()` 때 해제됨.
- `bmb_ffi_free_string(out)` 호출 시 → `STATUS_HEAP_CORRUPTION (0xC0000374)`.
- **수정**: 반환값은 data 읽기 전용, free 금지. 입력(`bmb_ffi_cstr_to_string` 결과)만 free.

**두 번째 발견 — static result_buf 포인터 aliasing**:
- `call1()` 함수가 static buffer 반환 → 여러 포인터가 같은 버퍼 공유 → 비교 시 항상 같음.
- **수정**: 비교 전 `strncpy`로 별도 버퍼에 복사.

**테스트 수**: 23 (전 14 함수 커버; SHA-256/MD5 벡터 포함)

## Verification & Defect Resolution

**수정 전**: `STATUS_HEAP_CORRUPTION` crash (double-free of arena string)
**수정 후**: `./test.exe → 23 passed, 0 failed`

**이 발견은 C 바인딩에만 해당**:
- Cycle 2906에서 Node/C#/Java 수정 시 "arena-free" 규칙 문서화.
- C 바인딩은 기존에 없었으므로 Cycle 2906에서 다루지 않았음.
- README에 CRITICAL 섹션으로 arena 규칙 명시.

## Reflection

- **Scope fit**: 14 함수 전체 커버. SHA-256/MD5 알려진 벡터로 정확성 검증.
- **Latent defects**: 없음 — 23/23 통과.
- **Structural improvement**: arena-free UB 규칙이 bmb-text/bmb-json C 바인딩에도 적용됨 (해당 라이브러리도 String 반환 함수 포함). 사전 인지.
- **Philosophy drift**: 없음.
- **Roadmap impact**: M4 ④ C 바인딩: algo/compute/crypto ✅.

## Carry-Forward
- Actionable: bmb-text C 바인딩 scaffold (Cycle 2911)
- Structural Improvement Proposals: None
- Pending Human Decisions: B축 재측정, tier3-spawn-overhead
- Roadmap Revisions: M4 ④ C: algo/compute/crypto ✅
- Next Recommendation: Cycle 2911 — bmb-text C 바인딩 (arena-free 규칙 선 적용)
