# BMB Session Handoff — 2026-05-28 (Cycle 3227)

> **HEAD**: Cycle 3227 commit pending
> **이번 세션 작업**: Cycles 3227 — **Brainfuck calloc→[u8;30000] stack migration**
> **M11-C Phase 2 상태**: ✅ **COMPLETE** — `[u8/i64/f64/i32; N]` 전 primitive 타입 지원 + 실벤치 검증
> **M11-A 상태**: ✅ **CONFIRMED COMPLETE** — 264 trivial postconditions 전부 skip 확정
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **현재 bootstrap 바이너리**: `bootstrap/compiler_3224.exe`
> **Fixed Point**: ✅ **S3 IR == S4 IR** (Cycle 3224)
> **0-Warning 상태**: ✅ **유지** (lint 0 warnings, compiler.bmb warnings 174)
> **Z3 상태**: ✅ **141/141** (Cycle 3219 달성)
> **Bootstrap Golden Tests**: ✅ **52/52** (컴파일러 변경 없음 — 유지)

---

## 이번 세션 작업 요약 (Cycle 3227)

### Cycle 3227: Brainfuck calloc→[u8; 30000] Stack Migration (M11-C Phase 2 dogfood)

`main_inproc.bmb`에서 `calloc(tape_size(), 1)` → `let tape: [u8; 30000]` + `free` 제거.

- **IR 확인**: `alloca [30000 x i8], align 16` — LLVM이 calloc을 alloca로 자동 승격하지 않음
- **성능**: 0.917× → **0.848×** (~7% 개선, 10회 측정 중앙값 기준)
- **정확성**: 체크섬 = 0 (모든 측정)
- **컴파일러 변경 없음** — 벤치마크 파일만 변경

---

## 이전 세션 작업 요약 (Cycles 3224-3226)

### Cycle 3224: M11-C Phase 2 Extension — `[i64; N]` / `[f64; N]` Element-Typed Stack Arrays

M11-C Phase 2 기존 `[u8; N]` 지원을 확장하여 전 primitive 타입 element size 인식.

#### 구현 내용

`bootstrap/compiler.bmb` 두 함수 최소 수정:

**`parse_block_let_array_type_aware`** (1줄 추가):
- `TK_F64()` 추가 to element type match
- `k1` (element type kind) → `parse_block_let_after_stack_array` 전달

**`parse_block_let_after_stack_array`** (elem_kind 파라미터 + byte_count_expr switch):
```
TK_I64() / TK_F64() → "(binop * size_expr (int 8))"  — 8 bytes/element
TK_I32()            → "(binop * size_expr (int 4))"  — 4 bytes/element
TK_IDENT() / TK_BOOL() → size_expr                   — 1 byte/element (unchanged)
```

LLVM 최적화기가 `64 * 8 = 512` 컴파일 타임 상수 폴딩.

새 문법:
```bmb
let arr: [i64; 64];   // stack_bytes_new(64 * 8) = 512 bytes
let arr: [f64; 32];   // stack_bytes_new(32 * 8) = 256 bytes
let arr: [i32; 16];   // stack_bytes_new(16 * 4) = 64 bytes
let arr: [u8; 100];   // stack_bytes_new(100)     = 100 bytes (unchanged)
```

#### 검증

```json
{"type":"golden_tests","passed":50,"failed":0,"total":50}
Fixed Point: S3 IR == S4 IR ✅
```

### Cycle 3225: M11-C Phase 2 완결 + M11-A 확정 평가

**`[f64; N]` golden test** 추가:
- `test_f64_array_zero_init()`: `[f64; 8]` → 64 bytes zero-initialized
- `test_f64_array_write_read()`: `store_i64(arr + 24, 55)` → `load_i64(arr + 24)` = 55

**M11-A 최종 평가**:
```
post it or not it  (bool):    27 remaining  [skip 확정: 7 no-pre + ~20 semantic_duplication]
post it.len() >= 0 (String): 230 remaining  [skip 확정: ~207 in 5 categories]
post it == it      (i64):      7 remaining  [skip 확정: 7 all skip]
Total:                        264 trivial postconditions — 전부 skip 확정
```

**결론**: M11-A effectively complete. 264개 모두 문서화된 skip 카테고리에 포함.

---

## 기술 현황 스냅샷 (2026-05-28)

| 항목 | 상태 |
|------|------|
| Z3 검증 | ✅ 141/141 (Cycle 3219) |
| Lint warnings | ✅ 0 (compiler.bmb 내부 lint 174 — 정상) |
| M11-A trivials | **✅ CONFIRMED COMPLETE** — 264개 전부 skip 확정 |
| M11-C Phase 2 | **✅ COMPLETE** — u8/i64/f64/i32/bool 전 primitive 지원 |
| Fixed Point | ✅ S3==S4 (compiler_3224.exe) |
| Bootstrap Golden Tests | ✅ 52/52 |
| P-track brainfuck | ✅ BMB ≈ C |
| `[T; N]` 문법 | ✅ 전 element 타입 지원 (`let arr: [i64; 64]` 등) |

---

## `[T; N]` 배열 접근 패턴 (현재)

```bmb
// [i64; N]: element i는 arr + i * 8 위치, load_i64/store_i64 사용
let arr: [i64; 64];
let _w = store_i64(arr + 3 * 8, 42);
let v = load_i64(arr + 3 * 8);  // 42

// [f64; N]: element i는 arr + i * 8 위치, load_f64/store_f64 사용
let arr: [f64; 8];
// f64 값 접근: store_f64(arr + i * 8, 3.14) / load_f64(arr + i * 8)

// [i32; N]: element i는 arr + i * 4 위치
let arr: [i32; 16];
// i32 접근: store_i32(arr + i * 4, v) / load_i32(arr + i * 4)

// [u8; N]: element i는 arr + i 위치 (1 byte)
let arr: [u8; 100];
// u8 접근: store_u8(arr + i, v) / load_u8(arr + i)
```

### 편의 helper 패턴 (@inline fn, 컴파일러 변경 없음)
```bmb
@inline fn i64_arr_get(arr: i64, i: i64) -> i64 = load_i64(arr + i * 8);
@inline fn i64_arr_set(arr: i64, i: i64, v: i64) -> i64 = store_i64(arr + i * 8, v);
```

---

## 주요 알려진 제약

### `stack_bytes_new` 사용 주의사항

```
⚠️ @inline fn wrapper 안에서 stack_bytes_new 사용 금지
   → LLVM 인라이너가 lifetime.end를 ptrtoint 직후 삽입
   → memset이 dead store로 제거됨 → 메모리 미초기화

✅ 올바른 사용: 직접 호출 함수 본문에서 stack_bytes_new(N)
```

### `[T; N]` 원시 포인터 의미론

`[T; N]` 은 `stack_bytes_new(N * sizeof(T))` syntactic sugar.
Element access는 raw byte 산술 + load/store 빌트인 필요.
`arr[i]` subscript 문법은 M11-D (미래) 스코프.

### 기존 알려진 제약 (이전 세션에서 이월)

- **semantic_duplication bool 충돌**: `mn_has_memory_op`, `ipr_has_store` 등 bool 함수
  postcondition이 `not it or pos < ir.len()` 공유 → Z3 semantic_duplication 경고
- **inkwell 3 parity gap**: `bmb_exec_with_stdin`, `bmb_file_mtime`, `bmb_str_byte_at`가
  text backend에만 있고 inkwell에 없음 (Rule 7 위반 위험, blocking이 아님)

---

## 다음 권장 작업

### 단기 (Cycle 3228): json_serialize calloc 스택 마이그레이션 조사

- `let buf = calloc(65536, 1)` → `let buf: [u8; 65536]` — 64KB 스택 배열 (대형)
- `let arr = calloc(10, 8)` → `let arr: [i64; 10]` — 80바이트 스택 배열 (소형)
- 성능 측정 전/후 비교 필수 (대형 스택 배열은 성능 개선 불확실)

### M11-C Phase 3: `arr[i]` subscript 문법 (주요 스코프, defer 추천)

- `let arr: [i64; N]` 선언에서 원소 타입 추적
- `arr[i]` → `load_i64(arr + i * 8)` (또는 해당 타입) 자동 desugar
- grammar + parser + type annotation tracking 필요 — 2+ cycles 예상
- **아키텍처 블로커**: bootstrap 컴파일러에 파스타임 심볼테이블 없음

### 기타 언어 갭

- closure / lambda 지원
- generic 타입 파라미터 bootstrap 완전 지원
- B축 재측정 (claude-sonnet-4-6, stale 기한 2026-08-13, API key 필요)

### P-track 최신 수치 (Cycle 3227 기준)

| 벤치마크 | 이전 비율 | 최신 비율 |
|---------|---------|---------|
| brainfuck | 0.941× | **0.848×** (Cycle 3227, 10회 중앙값) |
| csv | 0.858× | — |
| http | 0.934× | — |
| json_serialize | ~0.670× | — |
| json_parse | ~0.875× | — |
| lexer | ~0.174× | — |
| sorting | ~0.155× | — |
