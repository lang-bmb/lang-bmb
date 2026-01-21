# Performance Improvement Plan (v0.57+)

**목표:** 모든 벤치마크에서 C 성능 도달 또는 추월 (≤1.10x)

**현재 상태:** 14/48 벤치마크가 C보다 느림 (29% 미달)

---

## Root Cause Analysis

### CRITICAL (>2x slower) - 즉시 해결 필요

| 벤치마크 | 성능 | 근본 원인 | 해결책 |
|----------|------|----------|--------|
| **syscall_overhead** | 2.72x | FFI boundary 비용 | P1: FFI 인라인화, 직접 syscall |
| **http_parse** | 2.29x | 문자열 연결 할당 오버헤드 | P1: 문자열 빌더 패턴, in-place 파싱 |
| **fannkuch** | 2.12x | 재귀 호출 오버헤드 (C는 반복문) | P1: Tail Call Optimization 강화 |

### SEVERE (1.4x-2x slower)

| 벤치마크 | 성능 | 근본 원인 | 해결책 |
|----------|------|----------|--------|
| **simd_sum** | 1.50x | SIMD 자동 벡터화 실패 | P2: LLVM 벡터화 힌트 추가 |
| **json_serialize** | 1.45x | 문자열 연결 오버헤드 | P1: 문자열 빌더 패턴 |
| **fibonacci** | 1.44x | 재귀 호출 오버헤드 | P1: TCO 강화 |

### MODERATE (1.2x-1.4x slower)

| 벤치마크 | 성능 | 근본 원인 | 해결책 |
|----------|------|----------|--------|
| purity_opt | 1.25x | 최적화 미적용 | P2: 순수 함수 인라인 강화 |
| memory_copy | 1.25x | memcpy 대신 루프 | P2: memcpy 인트린직 |
| aliasing_proof | 1.25x | noalias 전파 미흡 | P2: LLVM noalias 강화 |
| parse_bootstrap | 1.25x | 함수 호출 오버헤드 | P2: 인라인 임계치 조정 |
| mandelbrot | 1.20x | FP 연산 최적화 | P3: FMA 인트린직 |
| fasta | 1.20x | 문자열/버퍼 처리 | P2: 버퍼 재사용 |
| pointer_chase | 1.20x | 메모리 접근 패턴 | P3: 캐시 힌트 |
| null_check_proof | 1.20x | 최적화 미적용 | P2: DCE 강화 |

---

## 우선순위별 개선 계획

### P0: Compiler Infrastructure (선행 조건)

| ID | 작업 | 영향 범위 | 예상 효과 |
|----|------|----------|----------|
| P0.1 | **Tail Call Optimization 강화** | fannkuch, fibonacci | 1.5x → 1.0x |
| P0.2 | **함수 인라인 임계치 조정** | 전체 | 5-10% 개선 |
| P0.3 | **LLVM 최적화 패스 순서 조정** | 전체 | 미측정 |

### P1: Critical Performance Fixes

| ID | 작업 | 대상 | 예상 효과 |
|----|------|------|----------|
| P1.1 | **문자열 빌더 구현** | http_parse, json_serialize | 2.3x → 1.1x |
| P1.2 | **FFI 인라인화** | syscall_overhead | 2.7x → 1.2x |
| P1.3 | **TCO 검증 및 수정** | fannkuch, fibonacci | 2.1x → 1.1x |

### P2: Performance Optimizations

| ID | 작업 | 대상 | 예상 효과 |
|----|------|------|----------|
| P2.1 | **SIMD 벡터화 힌트** | simd_sum | 1.5x → 1.0x |
| P2.2 | **memcpy/memset 인트린직** | memory_copy | 1.25x → 1.0x |
| P2.3 | **순수 함수 인라인 강화** | purity_opt, aliasing_proof | 1.25x → 1.0x |
| P2.4 | **DCE 최적화 강화** | null_check_proof | 1.2x → 1.0x |

### P3: Low Priority (목표 달성 후)

| ID | 작업 | 대상 |
|----|------|------|
| P3.1 | FMA 인트린직 | mandelbrot |
| P3.2 | 캐시 힌트 | pointer_chase |
| P3.3 | brainfuck PHI 버그 | brainfuck |

---

## 상세 분석

### 1. fannkuch (2.12x slower)

**C 구현:**
```c
while (1) {
    for (int i = 0; i < n; i++) perm[i] = perm1[i];
    // ... 반복문으로 처리
}
```

**BMB 구현:**
```bmb
fn fannkuch_iter(perm, perm1, count, n, maxFlips, checksum, permCount) =
    // ... 모든 루프가 재귀 함수
    let flips = count_flips(perm, perm1, n);  // 또 재귀
    fannkuch_iter(..., permCount + 1)  // tail recursion
```

**문제:** BMB의 모든 반복이 함수 호출로 변환됨
- 함수 프롤로그/에필로그 오버헤드
- 스택 프레임 생성/해제
- 레지스터 저장/복원

**해결책:**
1. **TCO 검증:** `fannkuch_iter`의 tail call이 실제로 최적화되는지 LLVM IR 확인
2. **강제 인라인:** 작은 헬퍼 함수들(`copy_perm`, `swap_in_array`) 강제 인라인
3. **루프 변환:** 재귀를 반복문으로 변환하는 최적화 패스 추가

### 2. http_parse (2.29x slower)

**문제점:**
```bmb
fn crlf() -> String = chr(13) + chr(10);  // 매번 새 문자열 할당

fn request1() -> String =
    "GET /..." + crlf() +     // 할당 1
    "Host: ..." + crlf() +    // 할당 2
    ...                        // 총 10+ 할당
```

**해결책:**
1. **컴파일 타임 문자열 연결:** 상수 문자열 연결을 컴파일 타임에 처리
2. **문자열 빌더:** `StringBuilder` 타입 도입 (단일 버퍼에 append)
3. **In-place 파싱:** 새 문자열 생성 대신 슬라이스/인덱스 반환

### 3. syscall_overhead (2.72x slower)

**C 구현:**
```c
for (int i = 0; i < 10000; i++) {
    stat(".", &st);  // 직접 syscall
}
```

**BMB 구현:**
```bmb
fn run_syscalls(path, remaining, acc) =
    let result = check_exists(path);  // BMB → runtime.c → stat
    run_syscalls(path, remaining - 1, acc + result)  // 재귀
```

**문제:**
1. FFI boundary: BMB → C runtime → kernel
2. 재귀 호출 오버헤드 (10000회)
3. String 파라미터 변환 오버헤드

**해결책:**
1. **FFI 인라인화:** 단순 syscall wrapper를 LLVM IR로 직접 생성
2. **TCO 적용:** 재귀를 반복으로 변환
3. **문자열 캐싱:** 동일 문자열 반복 사용 시 변환 생략

---

## 검증 기준

### Gate 재정의 (더 엄격하게)

| Gate | 현재 | 목표 |
|------|------|------|
| Gate #3.1 | Compute ≤1.10x (6/10 통과) | **ALL ≤1.10x** |
| Gate #3.5 | ALL benchmarks ≤1.10x | **NEW: 48/48 통과** |

### 완료 기준

- [ ] CRITICAL 3개 모두 ≤1.10x
- [ ] SEVERE 3개 모두 ≤1.10x
- [ ] MODERATE 8개 모두 ≤1.10x
- [ ] 전체 48개 중 SLOW 0개

---

## 일정 (예상)

| Phase | 작업 | 기간 |
|-------|------|------|
| **P0** | TCO, 인라인 인프라 | 1주 |
| **P1** | Critical 3개 수정 | 2주 |
| **P2** | Severe + Moderate 수정 | 2주 |
| **검증** | 전체 벤치마크 재측정 | 1일 |

**총 예상:** 5주

---

*v0.57 완료 조건: 모든 벤치마크 ≤1.10x C*
