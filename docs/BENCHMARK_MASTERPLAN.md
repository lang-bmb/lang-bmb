# BMB Benchmark Master Plan

> **철학**: 인간 편의 희생 → 최고 성능 & 안정성 확보
> **목표**: C/Rust가 포기한 1~20% 최적화 → 이론상 C/Rust 추월

---

## 핵심 원칙

### 1. 성능 최우선 (Performance First)
- 기계어 수준에서 방어 코드 완전 제거
- 런타임 오버헤드 = 0 (모든 검증은 컴파일 타임)
- 최적화 여지가 있으면 반드시 최적화

### 2. 안정성은 언어 복잡도로 달성
- 런타임 체크 대신 타입 시스템/계약으로 증명
- 증명 불가 → 컴파일 에러 (런타임 비용 발생 않음)
- 개발자가 더 많은 정보 제공 → 컴파일러가 더 공격적 최적화

### 3. 증명 가능한 성능 우위
- 모든 주장은 벤치마크로 검증
- C/Rust 대비 정량적 비교 필수
- 재현 가능한 측정 환경

---

## 현재 상태 (2026-01-17)

### 기존 벤치마크 (26개)

| 카테고리 | 개수 | 상태 | 비고 |
|----------|------|------|------|
| Compute | 10 | ✅ | Benchmarks Game 표준 |
| Contract | 6 | ⚠️ | 최적화 효과 미달 |
| Real-World | 7 | ⚠️ | JSON 2.5x 느림 |
| Bootstrap | 3 | ✅ | 자체 컴파일 측정 |

### Gate 상태

| Gate | 기준 | 현재 | 목표 |
|------|------|------|------|
| #3.1 | Clang ≤1.10x | ✅ 1.00-1.08x | 유지 |
| #3.2 | 전체 ≤1.05x | ❌ 미달 | 달성 필요 |
| #3.3 | 3개 C보다 빠름 | ❌ 미달 | 달성 필요 |

---

## 누락 분석 & 강화 계획

### Phase 1: Zero-Overhead 증명 (P0 - 즉시)

**목표**: BMB의 안전성 검증이 성능 비용 0임을 증명

| ID | 벤치마크 | 측정 대상 | 기대 결과 |
|----|----------|-----------|-----------|
| ZO-1 | **bounds_check_proof** | 배열 인덱스 검증 | C unsafe = BMB safe (0% 오버헤드) |
| ZO-2 | **null_check_proof** | Option<T> vs raw pointer | 동일 성능 |
| ZO-3 | **overflow_proof** | 정수 오버플로우 검증 | C unchecked = BMB checked |
| ZO-4 | **aliasing_proof** | 포인터 별칭 최적화 | BMB > C (SIMD 자동화) |
| ZO-5 | **purity_proof** | 순수 함수 최적화 | BMB > C (CSE, 호이스팅) |

**검증 방법**:
```bash
# 어셈블리 비교
bmb build bench.bmb --emit-asm -o bmb.s
clang -O3 bench.c -S -o c.s
diff bmb.s c.s  # 동일해야 함 (또는 BMB가 더 짧음)
```

### Phase 2: 메모리 벤치마크 (P0 - 1주)

**목표**: 시스템 언어 필수 메모리 성능 검증

| ID | 벤치마크 | 측정 대상 | C 대비 목표 |
|----|----------|-----------|-------------|
| MEM-1 | **cache_stride** | 캐시 라인 접근 패턴 | ≤1.00x |
| MEM-2 | **allocation_stress** | malloc/free 사이클 | ≤1.05x |
| MEM-3 | **memory_copy** | memcpy 대체 | ≤1.00x |
| MEM-4 | **stack_allocation** | 스택 변수 접근 | ≤1.00x |
| MEM-5 | **pointer_chase** | 링크드 리스트 순회 | ≤1.00x |
| MEM-6 | **simd_sum** | SIMD 벡터 연산 | ≤1.00x (자동 벡터화) |

### Phase 3: 시스템 콜 벤치마크 (P0 - 1주)

**목표**: OS 인터페이스 성능 검증

| ID | 벤치마크 | 측정 대상 | C 대비 목표 |
|----|----------|-----------|-------------|
| SYS-1 | **syscall_overhead** | 기본 시스템 콜 | ≤1.00x |
| SYS-2 | **file_io_seq** | 순차 파일 읽기/쓰기 | ≤1.02x |
| SYS-3 | **file_io_random** | 랜덤 파일 접근 | ≤1.02x |
| SYS-4 | **process_spawn** | 프로세스 생성 | ≤1.05x |
| SYS-5 | **signal_handling** | 시그널 처리 지연 | ≤1.00x |

### Phase 4: 실제 워크로드 개선 (P1 - 2주)

**문제**: JSON 파싱 2.5x 느림 → 문자열 처리 병목

| ID | 벤치마크 | 현재 | 목표 | 해결책 |
|----|----------|------|------|--------|
| RW-1 | **json_parse** | 2.55x | ≤1.10x | 문자열 인터닝, SSO |
| RW-2 | **json_serialize** | ? | ≤1.10x | StringBuilder 최적화 |
| RW-3 | **regex_match** | N/A | ≤1.20x | 새로 추가 |
| RW-4 | **utf8_validate** | N/A | ≤1.00x | 새로 추가 |
| RW-5 | **compression_lz4** | N/A | ≤1.10x | 새로 추가 |

### Phase 5: 계약 최적화 증명 (P1 - 2주)

**문제**: 계약 최적화가 기대 성능 미달

| ID | 벤치마크 | 기대 | 현재 | 원인 분석 |
|----|----------|------|------|----------|
| CO-1 | **bounds_elim** | 10-30% 빠름 | ~0% | LLVM이 이미 최적화? |
| CO-2 | **null_elim** | 15-25% 빠름 | ~0% | 분기 예측 효율? |
| CO-3 | **branch_elim** | 20-50% 빠름 | ~0% | 데드코드 제거 미작동? |
| CO-4 | **loop_invariant** | 10-20% 빠름 | ~0% | 호이스팅 미작동? |

**디버깅 방법**:
```bash
# LLVM IR 비교
bmb build bench.bmb --emit-llvm -o with_contract.ll
bmb build bench_no_contract.bmb --emit-llvm -o without.ll
diff with_contract.ll without.ll
```

### Phase 6: C/Rust 추월 벤치마크 (P2 - 3주)

**목표**: BMB > C인 케이스 3개 이상 확보 (Gate #3.3)

| ID | 벤치마크 | 추월 전략 |
|----|----------|-----------|
| WIN-1 | **matrix_multiply** | 계약 기반 별칭 분석 → SIMD 최대화 |
| WIN-2 | **sort_presorted** | 사전조건으로 분기 제거 |
| WIN-3 | **tree_balance** | 불변량 기반 재균형 스킵 |
| WIN-4 | **string_search** | 컴파일 타임 패턴 최적화 |
| WIN-5 | **graph_traversal** | 도달성 증명으로 방문 체크 제거 |

---

## 측정 인프라 강화

### 현재 문제점

1. **Median만 측정** → p50/p95/p99 필요
2. **단일 실행 환경** → 재현성 문제
3. **수동 실행** → CI 자동화 필요

### 개선 계획

```yaml
# .github/workflows/benchmark.yml
name: Benchmark CI
on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Daily

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmarks
        run: |
          benchmark-bmb run all -i 10 -w 3 --json > results.json

      - name: Check regression
        run: |
          benchmark-bmb compare results.json baseline.json --threshold 2%

      - name: Gate check
        run: |
          benchmark-bmb gate 3.1 3.2 3.3 --strict
```

### 출력 개선

```json
{
  "benchmark": "fibonacci",
  "language": "bmb",
  "metrics": {
    "p50": 0.016,
    "p95": 0.017,
    "p99": 0.018,
    "min": 0.015,
    "max": 0.019,
    "stddev": 0.001
  },
  "comparison": {
    "vs_c": 1.00,
    "vs_rust": 0.93
  },
  "assembly_size": 1234,
  "llvm_ir_lines": 456
}
```

---

## 우선순위 정리

### 즉시 (이번 주)

| 우선순위 | 태스크 | 이유 |
|----------|--------|------|
| **P0** | Zero-Overhead 증명 (ZO-1~5) | BMB 핵심 가치 증명 |
| **P0** | 계약 최적화 디버깅 (CO-1~4) | 현재 미작동 원인 파악 |
| **P0** | 메모리 벤치마크 추가 (MEM-1~6) | 시스템 언어 필수 |

### 단기 (2주)

| 우선순위 | 태스크 | 이유 |
|----------|--------|------|
| **P1** | JSON 성능 개선 (RW-1) | 실제 워크로드 신뢰성 |
| **P1** | 시스템 콜 벤치마크 (SYS-1~5) | 시스템 언어 검증 |
| **P1** | CI 자동화 | 회귀 방지 |

### 중기 (1개월)

| 우선순위 | 태스크 | 이유 |
|----------|--------|------|
| **P2** | C 추월 케이스 (WIN-1~5) | Gate #3.3 달성 |
| **P2** | 실시간 대시보드 | 커뮤니티 투명성 |
| **P2** | Cross-platform 벤치마크 | 이식성 검증 |

---

## 성공 기준

### Gate #3.2: 전체 벤치마크 C 대비 ≤1.05x

```
[ ] fibonacci      ≤1.05x  (현재: 1.00x ✅)
[ ] mandelbrot     ≤1.05x  (현재: ? )
[ ] spectral_norm  ≤1.05x  (현재: ? )
[ ] binary_trees   ≤1.05x  (현재: 1.39x ❌)
[ ] n_body         ≤1.05x  (현재: ? )
[ ] json_parse     ≤1.05x  (현재: 2.55x ❌)
[ ] ... (20+ more)
```

### Gate #3.3: C보다 빠른 케이스 3개 이상

```
[ ] Case 1: _______ (BMB < C by __%)
[ ] Case 2: _______ (BMB < C by __%)
[ ] Case 3: _______ (BMB < C by __%)
```

### Zero-Overhead 증명

```
[ ] bounds_check: BMB safe == C unsafe (어셈블리 동일)
[ ] null_check: BMB Option == C raw pointer (어셈블리 동일)
[ ] overflow_check: BMB checked == C unchecked (어셈블리 동일)
```

---

## 다음 액션

1. **즉시**: Zero-Overhead 벤치마크 5개 구현 시작
2. **즉시**: 계약 최적화 미작동 원인 LLVM IR 분석
3. **이번 주**: 메모리 벤치마크 카테고리 추가
4. **이번 주**: CI 벤치마크 자동화 설정

---

*Last updated: 2026-01-17*
*Version: v0.50.24*
