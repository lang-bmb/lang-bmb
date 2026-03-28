# Cycle 121: purity_opt 벤치마크 수정 — @pure CSE 활성화

Date: 2026-03-28

## Inherited → Addressed
From cycle 115-120: "MIR optimization pass tests" — deferred (out of scope for performance cycles)

## Scope & Implementation

### 문제 분석
- purity_opt 벤치마크가 `@pure` 어노테이션 없이 `pre`/`post`만 사용
- PureFunctionCSE 패스는 **명시적 @pure 어노테이션 필수** — 계약만으로는 순수성 추론 불가
- 결과: CSE 미동작 → BMB vs C 동등 성능 (0% 이득)

### 근본 원인
1. `@pure` → `is_pure=true` → `is_memory_free=true` → LLVM `memory(none) speculatable`
2. `pre`/`post` → `llvm.assume` (범위 제약) — 순수성과 무관
3. 벤치마크 코드에 `@pure` 누락 → 파이프라인 미동작

### 수정 내역
1. **purity_opt/bmb/main.bmb**: 모든 순수 함수에 `@pure` 어노테이션 추가
2. **purity_opt/c/main.c**: `long` → `int64_t` (Windows 32비트 long 오버플로 수정)
3. 반복 횟수 1,000 → 100,000 (측정 가능한 실행 시간)

### IR 검증
- `@pure` 적용 후 `compute_with_redundancy`에서:
  - 변경 전: `expensive_pure(x)` 3회 호출
  - 변경 후: 1회 호출, 결과 재사용 (`a + a + a`)
- LLVM opt -O2 후: 모든 함수 호출 0개 (완전 인라인 + 최적화)

### 벤치마크 결과

| Runtime | Median (ms) | vs BMB |
|---------|-------------|--------|
| **BMB (@pure)** | **119** | **1.00x** |
| C (Clang -O3) | 343 | 2.88x slower |
| C (GCC -O3) | 586 | 4.93x slower |

**BMB 2.88x FASTER vs Clang** — 계약(@pure → memory(none))이 실제 벤치마크에서 성능 우위 증명.

## Review & Resolution
- 출력값 일치 확인: BMB = GCC = Clang = 50285795812 ✅
- @pure + memory(none) → LLVM CSE/LICM 활성화 확인 (IR 분석) ✅
- C 컴파일러는 same-file 분석으로도 3x 호출 제거 실패 (noinline 없이도)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: @noinline BMB 어노테이션 미지원 — 별도 컴파일 단위 시뮬레이션 불가
- Next Recommendation: 다른 계약 벤치마크(bounds_check, divzero_check)도 동일 패턴으로 강화
