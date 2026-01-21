# BMB v0.51 Benchmark Comprehensive Report

**Generated:** 2026-01-21
**Compiler Version:** v0.51
**Test Configuration:** 5 iterations, 2 warmup runs
**Platform:** Windows x86_64

---

## Executive Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Benchmarks** | 48 | 100% |
| **FAST (BMB < C)** | 26 | 54% |
| **OK (≤1.10x C)** | 11 | 23% |
| **SLOW (>1.10x C)** | 11 | 23% |
| **Target (≤1.10x C)** | 37 | 77% |

**결론**: 37/48 (77%) 벤치마크가 목표(C 대비 ≤1.10x) 달성

---

## Detailed Results by Category

### FAST - BMB가 C보다 빠름 (26개)

| Benchmark | BMB (ms) | C (ms) | Ratio | Category |
|-----------|----------|--------|-------|----------|
| n_body | 4.66 | 23.80 | **0.20x** | Compute |
| typecheck_bootstrap | 4.62 | 20.31 | **0.23x** | Bootstrap |
| sorting | 5.03 | 18.55 | **0.27x** | Real World |
| hash_table | 7.84 | 14.78 | **0.53x** | Compute |
| lex_bootstrap | 5.04 | 7.84 | **0.64x** | Bootstrap |
| csv_parse | 6.14 | 8.29 | **0.74x** | Real World |
| simd_sum | 4.55 | 6.04 | **0.75x** | Memory |
| bounds_check_proof | 5.03 | 6.60 | **0.76x** | Zero Overhead |
| invariant_hoist | 7.05 | 8.68 | **0.81x** | Contract |
| mandelbrot | 4.57 | 5.61 | **0.81x** | Compute |
| loop_invariant | 4.04 | 4.93 | **0.82x** | Contract Opt |
| tree_balance | 4.78 | 5.81 | **0.82x** | Surpass |
| bounds_elim | 5.03 | 6.09 | **0.83x** | Contract Opt |
| sort_presorted | 4.91 | 5.57 | **0.88x** | Surpass |
| k-nucleotide | 8.27 | 9.26 | **0.89x** | Compute |
| parse_bootstrap | 4.44 | 4.92 | **0.90x** | Bootstrap |
| pointer_chase | 5.68 | 6.28 | **0.90x** | Memory |
| overflow_proof | 5.38 | 5.79 | **0.93x** | Zero Overhead |
| bounds_check | 5.30 | 5.72 | **0.93x** | Contract |
| string_search | 5.28 | 5.60 | **0.94x** | Surpass |
| file_io_seq | 794.70 | 834.18 | **0.95x** | Syscall |
| graph_traversal | 7.99 | 8.21 | **0.97x** | Surpass |
| cache_stride | 4.62 | 4.75 | **0.97x** | Memory |
| spectral_norm | 4.59 | 4.70 | **0.98x** | Compute |
| matrix_multiply | 5.13 | 5.20 | **0.99x** | Surpass |

### OK - 목표 달성 (≤1.10x) (11개)

| Benchmark | BMB (ms) | C (ms) | Ratio | Category |
|-----------|----------|--------|-------|----------|
| fasta | 5.22 | 5.12 | 1.02x | Compute |
| lexer | 4.63 | 4.51 | 1.03x | Real World |
| aliasing_proof | 4.92 | 4.78 | 1.03x | Zero Overhead |
| binary_trees | 105.53 | 102.54 | 1.03x | Compute |
| process_spawn | 673.28 | 656.21 | 1.03x | Syscall |
| aliasing | 5.31 | 5.04 | 1.05x | Contract |
| purity_opt | 5.32 | 5.05 | 1.05x | Contract |
| purity_proof | 5.33 | 5.06 | 1.05x | Zero Overhead |
| branch_elim (opt) | 5.31 | 4.90 | 1.08x | Contract Opt |
| memory_copy | 4.89 | 4.48 | 1.09x | Memory |
| null_elim | 5.09 | 4.68 | 1.09x | Contract Opt |

### SLOW - 목표 미달 (>1.10x) (11개)

| Benchmark | BMB (ms) | C (ms) | Ratio | Category | Root Cause |
|-----------|----------|--------|-------|----------|------------|
| null_check | 5.47 | 4.97 | **1.10x** | Contract | 경계선 |
| null_check_proof | 4.77 | 4.30 | **1.11x** | Zero Overhead | Null 검사 |
| reverse-complement | 4.98 | 4.40 | **1.13x** | Compute | 문자열 처리 |
| stack_allocation | 5.63 | 4.88 | **1.15x** | Memory | 스택 프레임 |
| branch_elim | 5.02 | 4.32 | **1.16x** | Contract | 분기 예측 |
| brainfuck | 5.84 | 4.71 | **1.24x** | Real World | 인터프리터 |
| json_serialize | 27.35 | 20.01 | **1.37x** | Real World | 문자열 O(n²) |
| fibonacci | 24.40 | 16.91 | **1.44x** | Compute | Non-tail 재귀 |
| http_parse | 24.38 | 14.57 | **1.67x** | Real World | 문자열 연결 |
| fannkuch | 169.15 | 79.26 | **2.13x** | Compute | 재귀 오버헤드 |
| syscall_overhead | 635.93 | 172.72 | **3.68x** | Syscall | BmbString 래퍼 |

---

## Critical Issues Analysis

### 1. syscall_overhead (3.68x) - CRITICAL

**현상**: FFI 호출당 오버헤드가 매우 큼
**측정값**: BMB 635.93ms vs C 172.72ms (10,000회 file_exists 호출)

**Root Cause**:
- BmbString 구조체 래퍼 사용
- 함수 호출 체인 (file_exists → bmb_file_exists → stat)
- 포인터 역참조 + Null 검사 오버헤드

**개선 방안**: 문자열 리터럴 FFI 최적화 (직접 char* 전달)

### 2. fannkuch (2.13x) - HIGH

**현상**: 깊은 재귀 호출 오버헤드
**Root Cause**: 재귀 함수 호출 스택 설정 비용

**개선 방안**: while 루프로 재작성 (v0.51 문법 지원)

### 3. http_parse / json_serialize (1.67x / 1.37x)

**현상**: 문자열 연결 성능 저하
**Root Cause**: + 연산자가 매번 새 문자열 할당

**개선 방안**: StringBuilder 사용 권장, 함수 인라인 패스

---

## Category Performance Summary

| Category | Total | FAST | OK | SLOW | Pass Rate |
|----------|-------|------|-----|------|-----------|
| Compute | 10 | 6 | 2 | 2 | 80% |
| Contract | 6 | 2 | 2 | 2 | 67% |
| Contract Opt | 4 | 2 | 2 | 0 | **100%** |
| Memory | 5 | 3 | 1 | 1 | 80% |
| Real World | 7 | 2 | 1 | 4 | **43%** |
| Syscall | 3 | 1 | 1 | 1 | 67% |
| Zero Overhead | 5 | 2 | 2 | 1 | 80% |
| Surpass | 5 | 5 | 0 | 0 | **100%** |
| Bootstrap | 3 | 3 | 0 | 0 | **100%** |

**문제 영역**: Real World (43%) - 문자열 처리 중심

---

## Improvement Priority

### P0 - Critical
1. **syscall_overhead**: 문자열 리터럴 FFI 최적화
2. **fannkuch**: 벤치마크 루프 재작성

### P1 - High  
3. **http_parse/json_serialize**: 문자열 최적화

### P2 - Medium
4. **branch_elim/stack_allocation**: 경계 케이스

---

## Conclusion

- **77% (37/48)** 벤치마크가 목표 달성
- **54% (26/48)** 벤치마크에서 C 추월
- **100%** Bootstrap/Surpass/ContractOpt 카테고리 달성
- **주요 이슈**: syscall_overhead (3.68x), fannkuch (2.13x)
