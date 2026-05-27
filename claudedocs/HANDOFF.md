# BMB Session Handoff — 2026-05-27 (Cycle 3220)

> **HEAD**: pending commit
> **이번 세션 작업**: Cycle 3220 — **M11-C Phase 2: IPR memset Bug Fix + stack_bytes_new Correctness**
> **M11-C Phase 1 상태**: ✅ **COMPLETE** — `stack_bytes_new` 빌트인 정상 동작
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **M10 상태**: ✅ **COMPLETE** (이전 세션)
> **Stage 2 상태**: ✅ **Fixed Point S2==S3** (Cycle 3220)
> **0-Warning 상태**: ✅ **유지** (lint 0 warnings)
> **Z3 상태**: ✅ **141/141** (이전 Cycle 3219 달성)
> **Golden Tests**: ✅ **2862/2862**

---

## 이번 세션 작업 요약 (Cycle 3220)

### Critical Bug Fix: `ipr_all_calls_readonly` + `stack_bytes_new` 정상화

#### 문제 발견

brainfuck 벤치마크에서 `stack_bytes_new`로 tape 할당 시 "Nested loops" 테스트가 올바른 'X' 대신
'€' 쓰레기 값 출력. 두 가지 연관된 버그 발견:

**Bug 1: `ipr_all_calls_readonly` 과도한 범위화**

```
bootstrap/compiler.bmb:17263
// 기존 (버그):
if find_pattern_at(fn_name, "llvm.", 0) >= 0 { true }

// 수정 (fix):
if find_pattern_at(fn_name, "llvm.", 0) >= 0 {
    find_pattern_at(fn_name, "llvm.memset", 0) < 0
    and find_pattern_at(fn_name, "llvm.memcpy", 0) < 0
    and find_pattern_at(fn_name, "llvm.memmove", 0) < 0
}
```

`@llvm.memset.p0.i64`가 "readonly"로 처리되어 `tape_new()`에 `memory(read)` 어노테이션이 붙음.
그 결과 LLVM이 memset을 "dead store"로 판단, 제거.

**Bug 2: `@inline fn` wrapper의 LLVM inliner lifetime 문제**

`@inline fn tape_new()` 안에서 `stack_bytes_new` 호출 시, LLVM 인라이너가 `ptrtoint` 직후에
`lifetime.end` 삽입 → tape 메모리가 "dead"로 표시됨. `ptrtoint`는 pointer provenance를 끊어서
LLVM이 후속 `inttoptr` 접근과 alloca를 연결하지 못함.

**근본 해결**: `stack_bytes_new`를 직접 `interpret()` 함수 본문에서 호출 (inline wrapper 경유 금지)

#### 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `bootstrap/compiler.bmb` | `ipr_all_calls_readonly`: memset/memcpy/memmove → non-readonly |
| `bootstrap/compiler.bmb` | `ipr_has_store`: memset/memcpy/memmove call 감지 추가 |
| `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main.bmb` | `interpret()` 직접 `stack_bytes_new` 사용 |

### 검증 결과

| 항목 | 결과 |
|------|------|
| Z3 verify | ✅ 141/141 |
| Lint | ✅ 0 warnings |
| Fixed Point | ✅ S2==S3 |
| Golden tests | ✅ 2862/2862 |
| brainfuck 출력 | ✅ 'X' (정상) |
| BMB vs C 성능 | ✅ ~17ms ≈ 17ms (동등) |

---

## 다음 권장 작업 (Cycle 3221+)

### M11-C Phase 2: `[u8; N]` 타입 어노테이션 파서 지원

**목표**: `let x: [u8; N] = stack_bytes_new(N)` 패턴을 타입 시스템에서 지원.

**이전 설계 (Cycle 3219 Carry-Forward)**:
1. `parse_block_let_array_type_aware` — `[u8; N]` 타입 annotation 인식 + N 캡처
2. `lower_stack_array_sb` — `alloca_bytes %_tX, N` MIR 생성
3. codegen — `alloca_bytes` → `alloca [N x i8] + memset`
4. 테스트 + Fixed Point

**하지만**: stack_bytes_new가 이미 올바르게 동작하므로, 타입 확장 없이
`stack_bytes_new(N)` 빌트인 단독으로도 사용 가능. Phase 2는 언어 설계 개선이지 기능 필수가 아님.

### 대안 방향

**M11-A 계속**: 263개 trivial postcondition 중 추가 교체 가능한 후보 탐색.
지금까지 bool충돌/i64산술/semantic_duplication으로 막힌 항목들 재평가.

---

## 주요 알려진 제약

### `stack_bytes_new` 사용 주의사항

```
⚠️ @inline fn wrapper 안에서 stack_bytes_new 사용 금지
   → LLVM 인라이너가 lifetime.end를 ptrtoint 직후 삽입
   → memset이 dead store로 제거됨 → 메모리 미초기화

✅ 올바른 사용: 직접 호출 함수 본문에서 stack_bytes_new(N)
```

### 기존 알려진 제약 (이전 세션에서 이월)

- **semantic_duplication bool 충돌**: `mn_has_memory_op`, `ipr_has_store` 등 bool 함수
  postcondition이 `not it or pos < ir.len()` 공유 → Z3 semantic_duplication 경고
- **inkwell 3 parity gap**: `bmb_exec_with_stdin`, `bmb_file_mtime`, `bmb_str_byte_at`가
  text backend에만 있고 inkwell에 없음 (Rule 7 위반 위험, blocking이 아님)

---

## 기술 현황 스냅샷

| 항목 | 상태 |
|------|------|
| Z3 검증 | ✅ 141/141 (Cycle 3219) |
| Lint warnings | ✅ 0 |
| M11-A trivials | 358 → 263 (-26.5%) |
| Fixed Point | ✅ S2==S3 |
| Golden tests | ✅ 2862/2862 |
| `stack_bytes_new` | ✅ 정상 동작 (Cycle 3220) |
| P-track brainfuck | ✅ BMB ≈ C |
