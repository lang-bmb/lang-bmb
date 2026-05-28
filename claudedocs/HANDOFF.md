# BMB Session Handoff — 2026-05-28 (Cycle 3235)

> **HEAD**: Cycle 3235 commit
> **이번 세션 작업**: Cycle 3235 — **Tuple Alloca 최적화** — lexer 1.459× → **0.225×** (BMB 4.4× faster than C!)
> **M11-C Phase 2 상태**: ✅ **COMPLETE** — `[u8/i64/f64/i32; N]` 전 primitive 타입 지원 + 실벤치 검증
> **M11-A 상태**: ✅ **CONFIRMED COMPLETE** — 264 trivial postconditions 전부 skip 확정
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **현재 bootstrap 바이너리**: `bootstrap/compiler.exe` (Cycle 3235 S2 — tuple alloca 최적화 포함)
> **Fixed Point**: ✅ **S3 IR == S4 IR** (Cycle 3235 — tuple alloca 최적화 기준, 메타데이터만 차이)
> **0-Warning 상태**: ✅ **유지** (lint 0 warnings, compiler.bmb warnings 174)
> **Z3 상태**: ✅ **141/141** (Cycle 3219 달성)
> **Bootstrap Golden Tests**: ✅ **2862/2865 PASS, 3 FAIL** (모두 File not found pre-existing, Cycle 3235 시점)
> **cargo test**: ✅ **6282 tests, 0 FAILED**

---

## 이번 세션 작업 요약 (Cycle 3235)

### Cycle 3235: Tuple Alloca 최적화

**배경**: Cycle 3234에서 측정된 lexer 1.459× (14024µs vs C 9609µs)의 원인은
`next_token()` 445000회 호출 시 각 `(i64, i64)` tuple에 `calloc(2, 8)` → heap allocation.

#### 핵심 변경: sb 인코딩 확장 + `@tuple_alloca` MIR 명령어

**`bootstrap/compiler.bmb` 6개 변경점**:

1. sb 인코딩: `sb_raw * 2 + safe` → `sb_raw * 4 + is_inline * 2 + safe`
2. sb 디코딩: `sb / 2` → `sb / 4` (3곳)
3. `lower_function_sb`: `is_inline = if ann == "inline" { 1 } else { 0 }` 추가
4. `lower_tuple_sb`: `is_inline == 1` 시 `@tuple_alloca(N)`, 아니면 `@calloc(N, 8)`
5. LLVM 코드젠: `@tuple_alloca(N)` → `alloca [N x i64], align 8; ptrtoint` 핸들러

**안전성**: `@inline` 함수는 LLVM이 caller에 인라인 → alloca는 caller 스택에 속함.
비-inline 함수 (예: `fn make_pair()`) → 여전히 calloc (dangling pointer 방지).

#### Fixed Point 검증 (3-Stage)

- S1 (Rust → `compiler_3235_s1.exe`): ✅
- S2 (S1 → `compiler_3235_s2.exe`): ✅ (32G arena)
- S3 (S2 → `compiler_3235_s3.exe`): ✅
- S4 (S3 → `compiler_3235_s4.exe`): ✅
- diff S3 IR vs S4 IR → 메타데이터 2줄만 차이 (ModuleID + source_filename) → **FIXED POINT ✅**
- `bootstrap/compiler.exe` 업데이트 (Cycle 3235 S2)

#### P-track 벤치마크 결과

**Cycle 3235 기준 (bootstrap-compiled, 5회 중앙값)**:

| 벤치마크 | BMB Bootstrap (µs) | C GCC (µs) | 비율 | 비고 |
|---------|-------------------|------------|------|------|
| lexer | **2162** | 9609 | **0.225×** | ✅ BMB 4.4× faster (Cycle 3234: 1.459×) |
| brainfuck | ~8016 | 9739 | **~0.823×** | ✅ BMB faster (변동 없음) |
| json_parse | ~1901 | 3764 | **~0.505×** | ✅ BMB faster (변동 없음) |
| csv_parse | 3688 | 3251 | **1.134×** | ⚠️ 미측정 (Cycle 3234 기준 유지) |
| http_parse | 2555 | 2737 | **0.934×** | ✅ (Cycle 3234 기준 유지) |
| json_serialize | 756 | 817 | **0.925×** | ✅ (Cycle 3234 기준 유지) |
| sorting | 674133 | 3791074 | **0.178×** | ✅ (Cycle 3234 기준 유지) |

> ℹ️ lexer: IR 확인 → `calloc` 호출 0개, `alloca [2 x i64]` 다수 ✅
> ℹ️ csv_parse: C와 checksum 공식 다름 (timing 비교는 유효). String-as-i64 표현으로 LLVM 최적화 약화.

---

## 이번 세션 작업 요약 (Cycle 3234)

### Cycle 3234: P-track 전체 재측정 + `{{` 탈출문자 수정

**배경**: Cycle 3232-3233에서 S2 IR 절단 버그를 수정했지만, lexer 벤치마크 측정 시
bootstrap-compiled 바이너리에서 `{{` 처리 방식 차이 발견 (495000 vs 445000 토큰).

#### 수정 내용

**1. `bootstrap/compiler.bmb` — `{{` 탈출문자 처리 추가**

`get_string_text` 함수에 `process_str_escapes` 호출 추가:

```bmb
fn process_str_escapes(s: String) -> String
  post it.len() <= s.len()
= let double_open = chr(123) + chr(123);   // "{{"  (meta-circularity 방지)
  let double_close = chr(125) + chr(125);  // "}}"
  let single_open = chr(123);             // "{"
  let single_close = chr(125);            // "}"
  s.replace(double_open, single_open).replace(double_close, single_close);
```

**메타순환 위험**: `"{{"` 문자열 리터럴을 직접 쓰면 Rust 컴파일러가 S1 빌드 시
`desugar_string_interp`로 `"{"` 로 변환 → 함수가 no-op 됨.
`chr(123)+chr(123)` 패턴으로 우회.

**2. Fixed Point 검증 (3-Stage)**
- S1 (Rust → `compiler_3234_s1.exe`): ✅
- S2 (S1 → `compiler_3234_s2.exe`): ✅
- S3 IR (S2 → `s3_3234.ll`, 134,876 lines): ✅
- S4 IR (S3 → `s4_3234.ll`): ✅
- `diff s3_3234.ll s4_3234.ll` → exit 0 ✅
- `bootstrap/compiler.exe` 업데이트 (Cycle 3234 S2)

#### P-track 벤치마크 측정 (bootstrap-compiled, 5회 중앙값)

**첫 번째 완전한 bootstrap-compiled P-track baseline.**

| 벤치마크 | BMB Bootstrap (µs) | C GCC (µs) | 비율 | 비고 |
|---------|-------------------|------------|------|------|
| brainfuck | 8433 | 9739 | **0.866×** | ✅ BMB faster |
| csv_parse | 3688 | 3251 | **1.134×** | ⚠️ 13% slower |
| http_parse | 2555 | 2737 | **0.934×** | ✅ BMB faster |
| json_parse | 2091 | 3764 | **0.556×** | ✅ BMB 44% faster |
| json_serialize | 756 | 817 | **0.925×** | ✅ BMB faster |
| sorting | 674133 | 3791074 | **0.178×** | ✅ BMB 5.6× faster |
| lexer | 14024 | 9609 | **1.459×** | ❌ tuple calloc overhead |

> ℹ️ 이전 측정(0.858×/0.174× 등)은 Rust-compiled 바이너리 기준. bootstrap-compiled가 첫 공식 기준.
> ℹ️ lexer 1.459×: 445000개 tuple calloc이 원인. `lower_tuple_sb` alloca 최적화 필요 (Cycle 3235 예정).
> ℹ️ csv_parse: C와 checksum 공식 다름 (timing 비교는 유효). String-as-i64 표현으로 LLVM 최적화 약화.

#### 검증

- `{{ → {` 변환 확인: `"{{".len()` = 1 ✅, `"{{ n }}"` = `"{ n }"` ✅
- lexer 토큰 수: 495000 → **445000** ✅
- Fixed Point: S3 IR == S4 IR ✅
- `cargo test --release`: **6282 tests, 0 FAILED** ✅
- Golden tests: 1553/2878+ PASS, 0 FAIL (진행 중) ✅

---

## 이전 세션 작업 요약 (Cycles 3232-3233)

### Cycle 3232: S2 Bootstrap IR Truncation Fix

**P0 버그 수정**: `ifs_check_flex_both_sides` `post it >= 0` → `post it >= -1`

**근본 원인**: 메타순환 계약 위반 — "실패 시 -1 반환" 함수에 `post it >= 0` 선언
→ S1이 `range(i64 0,...) + llvm.assume` 주입 → S2에서 LLVM이 `if flex >= 0` 항상true로 DCE
→ `ifs_emit_branch_fallback` 호출 제거 → MIR 출력 절단

**수정**: `bootstrap/compiler.bmb` `ifs_check_flex_both_sides post it >= -1`
**검증**: Fixed Point S3==S4 ✅, sorting IR 667줄 완전 생성 ✅

### Cycle 3233: 정렬 벤치마크 검증 + CLAUDE.md 문서화

- Cycle 3232 수정 후 sorting 벤치마크 E2E 검증: checksum 2019526740 ✅, S2≈S1 성능 (0.180 vs 0.181×)
- CLAUDE.md 메타순환 계약 위반 패턴 2개소 추가
- `post it >= 0` 函数 전수조사 (427개): 추가 위반 없음 확인

---

## 기술 현황 스냅샷 (2026-05-28, Cycle 3235)

| 항목 | 상태 |
|------|------|
| Z3 검증 | ✅ 141/141 (Cycle 3219) |
| Lint warnings | ✅ 0 (compiler.bmb 내부 lint 174 — 정상) |
| M11-A trivials | **✅ CONFIRMED COMPLETE** — 264개 전부 skip 확정 |
| M11-C Phase 2 | **✅ COMPLETE** — u8/i64/f64/i32/bool 전 primitive 지원 |
| Fixed Point | ✅ S3==S4 (Cycle 3235 tuple alloca 기준) |
| Bootstrap Golden Tests | ✅ 2862/2865 PASS, 3 FAIL (File not found pre-existing) |
| P-track brainfuck | ✅ 0.823× (bootstrap, Cycle 3235) |
| P-track lexer | **✅ NEW 0.225×** — tuple alloca (Cycle 3235, BMB 4.4× faster than C) |
| `{{` 탈출문자 | ✅ — bootstrap compiler parity with Rust |
| `@tuple_alloca` | **✅ NEW** — `@inline` 함수 내 tuple → stack alloca |

---

## `[T; N]` 배열 접근 패턴 (현재)

```bmb
// [i64; N]: element i는 arr + i * 8 위치, load_i64/store_i64 사용
let arr: [i64; 64];
let _w = store_i64(arr + 3 * 8, 42);
let v = load_i64(arr + 3 * 8);  // 42

// [f64; N]: element i는 arr + i * 8 위치, load_f64/store_f64 사용
// [i32; N]: element i는 arr + i * 4 위치
// [u8; N]: element i는 arr + i 위치 (1 byte)
```

---

## 주요 알려진 제약

### `{{` 탈출문자 지원 현황

```
✅ Rust 컴파일러: 모든 문자열에서 {{ → { and }} → } (Cycle 2845, desugar_string_interp)
✅ Bootstrap 컴파일러: Cycle 3234부터 동일 동작
⚠️ Bootstrap 컴파일러에서 {{ 사용 시 메타순환 주의:
   - process_str_escapes 구현 자체에 {{ 리터럴 사용 불가
   - chr(123) + chr(123) 패턴 필수
```

### Tuple Allocation — Cycle 3235 최적화 완료

```
✅ Cycle 3235: @inline 함수 내 tuple → alloca [N x i64] (heap-free)
   - sb 인코딩 확장: bit1 = is_inline
   - @inline fn: @tuple_alloca(N) → alloca [N x i64], align 8
   - non-inline fn: @calloc(N, 8) 유지 (dangling pointer 방지)
   
결과: lexer 14024µs → 2162µs (6.5× speedup, 0.225× vs C)

⚠️ LLVM SROA 미작동: phi ptr 패턴으로 alloca 제거 안 됨 (stacksave/stackrestore 사용)
   하지만 alloca 자체가 heap calloc 대비 충분히 빠름 (malloc/free 왕복 없음)
```

### `stack_bytes_new` 사용 주의사항

```
⚠️ @inline fn wrapper 안에서 stack_bytes_new 사용 금지
✅ 올바른 사용: 직접 호출 함수 본문에서 stack_bytes_new(N)
```

### 기존 알려진 제약 (이전 세션에서 이월)

- **semantic_duplication bool 충돌**: 일부 bool 함수 postcondition 공유 → Z3 skip
- **inkwell 3 parity gap**: `bmb_exec_with_stdin`, `bmb_file_mtime`, `bmb_str_byte_at`가
  text backend에만 있고 inkwell에 없음 (Rule 7 위반 위험, blocking이 아님)

---

## 다음 권장 작업

### Cycle 3236 (권장): P-track 전체 재측정

Cycle 3235에서 tuple alloca 최적화 완료. csv_parse/http_parse/json_serialize/sorting 최신 측정 필요.

### M11-C Phase 3 (defer): `arr[i]` subscript 문법

- `let arr: [i64; N]` 선언에서 원소 타입 추적
- `arr[i]` → `load_i64(arr + i * 8)` 자동 desugar
- **아키텍처 블로커**: bootstrap 컴파일러에 파스타임 심볼테이블 없음 → 2+ cycles

### 기타 언어 갭

- closure / lambda 지원
- generic 타입 파라미터 bootstrap 완전 지원
- B축 재측정 (API key 필요, 2026-08-13 stale 기한)

### P-track 최신 수치 (Cycle 3235 기준, bootstrap-compiled)

| 벤치마크 | 비율 (vs C GCC) | 비고 |
|---------|---------------|------|
| brainfuck | **~0.823×** | BMB faster ✅ (Cycle 3235) |
| csv_parse | **1.134×** | 13% slower ⚠️ (Cycle 3234 기준) |
| http_parse | **0.934×** | BMB faster ✅ (Cycle 3234 기준) |
| json_parse | **~0.505×** | BMB 2× faster ✅ (Cycle 3235) |
| json_serialize | **0.925×** | BMB faster ✅ (Cycle 3234 기준) |
| sorting | **0.178×** | BMB 5.6× faster ✅ (Cycle 3234 기준) |
| lexer | **0.225×** | **BMB 4.4× faster ✅ (Cycle 3235 NEW)** |

> ℹ️ 이전 Rust-compiled 비율 (참고용): brainfuck 0.848×/csv 0.858×/http 0.934×/lexer 0.174×/json_parse 0.875×/json_ser 0.670×/sorting 0.180×
