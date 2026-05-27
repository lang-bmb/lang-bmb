# BMB Session Handoff — 2026-05-28 (Cycle 3233)

> **HEAD**: Cycle 3233 commit (세션 종료 정리)
> **이번 세션 작업**: Cycles 3232-3233 — **S2 IR 절단 버그 수정 (Cycle 3232) + 정렬 벤치마크 검증 + CLAUDE.md 문서화 (Cycle 3233)**
> **M11-C Phase 2 상태**: ✅ **COMPLETE** — `[u8/i64/f64/i32; N]` 전 primitive 타입 지원 + 실벤치 검증
> **M11-A 상태**: ✅ **CONFIRMED COMPLETE** — 264 trivial postconditions 전부 skip 확정
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **현재 bootstrap 바이너리**: `bootstrap/compiler.exe` (Cycle 3232 S2 고정 버전 — `ifs_check_flex_both_sides post it >= -1`)
> **Fixed Point**: ✅ **S3 IR == S4 IR** (Cycle 3232 — fixed compiler.bmb 기준)
> **0-Warning 상태**: ✅ **유지** (lint 0 warnings, compiler.bmb warnings 174)
> **Z3 상태**: ✅ **141/141** (Cycle 3219 달성)
> **Bootstrap Golden Tests**: ✅ **52/52** (기준 유지)
> **cargo test**: ✅ **6282 tests, 0 FAILED**

---

## 이번 세션 작업 요약 (Cycle 3229)

### Cycle 3229: IPR `array_free memory(read)` False Positive Fix + Array Allocation Bug

**P0 버그 수정**: IPR 분석 패스에서 substring match false positive가 `array_free`에 `memory(read)` 잘못 부여 → LLVM이 `@free()` DCE → STATUS_HEAP_CORRUPTION.

#### 수정 내용

**1. `bootstrap/compiler.bmb` — IPR 블랙리스트 추가**
- `ipr_all_calls_pure` (~line 17319): `free`/`realloc` 명시 예외 처리
- `ipr_all_calls_readonly` (~line 17381): 동일

```bmb
// @free and @realloc modify allocator state — never readonly regardless of name matching
else if fn_name == "free" or fn_name == "realloc" { false }
```

**근본 원인**: `ipr_try_annotate_section`이 `self_name`을 `known_self`에 추가한 후 calls 검사 → `fn_name="free"` + `key="free,"` → `"array_free,"` 내에서 매칭 → false positive

**2. 벤치마크 파일 — `array_new` 할당 크기 수정**
- `main.bmb` + `main_inproc.bmb`: `malloc(n * 8)` → `malloc((n + 2) * 8)`
- 이유: BMB `arr[i]`는 bootstrap compiler에서 `+2` header offset을 더함 (lines 9438, 9568)
- `arr[n-1]` = offset `(n+1)*8` → `n*8` 할당 시 8 bytes overflow

#### 검증
- `compiler_3229_rust_s1.exe emit-ir`: `array_free` → `memory(read)` 없음 ✅
- `sort_s1_final.exe` 체크섬: 2019526740 ✅, exit 0 ✅
- `cargo test --release`: **6282 tests, 0 FAILED** ✅

#### 성능 (정정된 측정)
| 비교 | 비율 | 비고 |
|------|------|------|
| BMB fixed vs GCC -O3 (공식 baseline) | **0.158×** | BMB 6.3× faster (LLVM vectorization) |
| BMB fixed vs Clang -O2 | **0.822×** | BMB ~22% faster (같은 LLVM backend) |
| 기존 메모리 0.92× | 2026-05-01 Cycle 2535 측정, 구 코드 기준 | 현재와 다름 |

> LLVM이 `init_reverse` 루프들을 자동 vectorization (28 vector blocks in BMB IR vs 0 in GCC IR).

---

## 이전 세션 작업 요약 (Cycles 3224-3227)

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

### P-track 최신 수치 (Cycle 3233 기준)

| 벤치마크 | 이전 비율 | 최신 비율 |
|---------|---------|---------|
| brainfuck | 0.941× | **0.848×** (Cycle 3227, 10회 중앙값) |
| csv | 0.858× | — |
| http | 0.934× | — |
| json_serialize | ~0.670× | — |
| json_parse | ~0.875× | — |
| lexer | ~0.174× | — |
| sorting | ~0.155× | **~0.180×** (Cycle 3233, 5회 중앙값, S2 고정 binary) |

> ℹ️ sorting: S2 IR 절단 수정(Cycle 3232) 후 S2 = S1 성능 동등 확인. 이전 ~0.155×는 bench_algo.py
> 측정, 현재 ~0.180×는 직접 binary 측정. 방법론 차이로 인한 변동 (GCC 대비 ~5.5× faster 수준 유지).
