# Phase Plan: v0.46 → v0.47

> **작성일**: 2026-01-14 | **업데이트**: 2026-01-14 | **범위**: v0.46 완료 → v0.47 성능 검증

---

## 현재 상태 분석

### v0.46 (Independence) - 95% 완료

```
v0.46 Progress
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[██████████████████████████████████████████████████░░] 95%

✅ 완료:
  - LLVM 백엔드 검증 (WSL에서 네이티브 컴파일 성공)
  - Golden Binary 생성 (첫 번째 네이티브 컴파일러)
  - CLI 인자 전달 (bmb run file.bmb arg1 arg2)
  - v0.32 문법 지원 (// 주석, braced if-else)
  - String 반환 타입 수정 (395개 테스트 통과)
  - 3-Stage 스크립트 준비 (scripts/bootstrap_3stage.sh)
  - 빌드 문서화 (docs/BUILD_FROM_SOURCE.md)
  - 런타임 확장 (33개 함수)

⏳ 남은 작업:
  - 3-Stage 자체 컴파일 검증 (WSL에서 실행)
  - Cargo.toml 제거 (BMB-only 빌드 체인)
```

### 프로젝트 철학 정렬

**BMB Core Principles** (README.md에서 발췌):

| Priority | Principle | 검증 상태 |
|----------|-----------|----------|
| **P0** | **Performance** — No syntax that constrains optimization. Target: exceed C/Rust. | ✅ 0.89x-0.99x vs C |
| **P0** | **Correctness** — If it can be verified at compile time, it must be. | ✅ 계약 시스템 완료 |

**핵심 철학**: "Hard to write. Hard to get wrong. And that's what AI prefers."

---

## Phase v0.46.3: 3-Stage Bootstrap 검증 (즉시 실행)

### 목표
BMB 컴파일러가 자신을 컴파일하고, 그 결과물이 동일한 바이너리를 생성하는지 검증

### 태스크

| ID | 태스크 | 설명 | 예상 시간 |
|----|--------|------|----------|
| 46.3.1 | WSL 환경 준비 | LLVM 21, clang, 환경변수 설정 | 5분 |
| 46.3.2 | Stage 1 빌드 | Rust BMB → bmb-stage1 | 2분 |
| 46.3.3 | Stage 2 빌드 | bmb-stage1 → bmb-stage2 | 2분 |
| 46.3.4 | Stage 3 빌드 | bmb-stage2 → bmb-stage3 | 2분 |
| 46.3.5 | 동일성 검증 | diff bmb-stage2 bmb-stage3 | 1분 |
| 46.3.6 | 테스트 실행 | Stage 3로 부트스트랩 테스트 | 5분 |

### 검증 스크립트

```bash
#!/bin/bash
# scripts/bootstrap_3stage.sh

set -e

echo "=== BMB 3-Stage Bootstrap Verification ==="

# Stage 1: Rust BMB로 Bootstrap 컴파일
echo "[Stage 1] Building with Rust BMB..."
cargo run --release --features llvm -- build bootstrap/compiler.bmb -o bmb-stage1

# Stage 2: Stage 1으로 재컴파일
echo "[Stage 2] Building with bmb-stage1..."
./bmb-stage1 build bootstrap/compiler.bmb -o bmb-stage2

# Stage 3: Stage 2로 재컴파일
echo "[Stage 3] Building with bmb-stage2..."
./bmb-stage2 build bootstrap/compiler.bmb -o bmb-stage3

# 동일성 검증
echo "[Verify] Comparing Stage 2 and Stage 3..."
if diff bmb-stage2 bmb-stage3; then
    echo "✅ SUCCESS: Stage 2 == Stage 3"
else
    echo "❌ FAILURE: Stage 2 != Stage 3"
    exit 1
fi

# 테스트 실행
echo "[Test] Running bootstrap tests with Stage 3..."
./bmb-stage3 run bootstrap/lexer.bmb
./bmb-stage3 run bootstrap/types.bmb

echo "=== 3-Stage Bootstrap Verification COMPLETE ==="
```

### 완료 기준

- [ ] Stage 2 바이너리 생성 성공
- [ ] Stage 3 바이너리 생성 성공
- [ ] `diff bmb-stage2 bmb-stage3` 결과 동일
- [ ] Stage 3로 부트스트랩 테스트 통과

---

## Phase v0.47: Performance (성능 검증)

### 목표
모든 벤치마크 Gate 통과, 성능 회귀 방지 체계 구축

### 태스크 상세

#### 47.1: Gate #3.1 완전 검증

| ID | 태스크 | 설명 | 우선순위 |
|----|--------|------|----------|
| 47.1.1 | fibonacci 검증 | fib(35), fib(40), fib(45) 모두 ≤1.10x C | P0 |
| 47.1.2 | mandelbrot 검증 | 1000x1000 ≤1.10x C | P0 |
| 47.1.3 | spectral_norm 검증 | n=8000 ≤1.10x C | P0 |
| 47.1.4 | n_body 구현 | f64 + sqrt intrinsic 필요 | P0 |
| 47.1.5 | binary_trees 구현 | 동적 메모리 할당 필요 | P1 |

**n_body 구현을 위한 선행 작업:**
```bmb
// stdlib/math/f64.bmb 필요
fn sqrt(x: f64) -> f64 = @intrinsic("sqrt");
fn pow(x: f64, y: f64) -> f64 = @intrinsic("pow");
fn sin(x: f64) -> f64 = @intrinsic("sin");
fn cos(x: f64) -> f64 = @intrinsic("cos");
```

#### 47.2: Gate #3.2 - Benchmarks Game 완전 통과

| ID | 벤치마크 | 상태 | 필요 작업 |
|----|----------|------|----------|
| 47.2.1 | fasta | ⏳ | 완료 확인 |
| 47.2.2 | k-nucleotide | ⏳ | HashMap 의존 |
| 47.2.3 | reverse-complement | ⏳ | 문자열 연산 |
| 47.2.4 | fannkuch | ⏳ | 배열 순열 |

#### 47.3: Gate #3.3 - C 초과 성능 달성

목표: 3개 이상의 벤치마크에서 C보다 빠른 성능

| 벤치마크 | 현재 | 목표 | 전략 |
|----------|------|------|------|
| fibonacci | 0.99x | <1.00x | ✅ 이미 달성 |
| mandelbrot | 0.93x | <1.00x | ✅ 이미 달성 |
| spectral_norm | 0.89x | <1.00x | ✅ 이미 달성 |
| bounds_check | TBD | <0.90x | 계약 기반 최적화 |
| aliasing | TBD | <0.70x | SIMD 벡터화 |

#### 47.4: 성능 회귀 방지

```yaml
# .github/workflows/benchmark.yml 업데이트
name: Benchmark Regression

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Benchmarks
        run: |
          cd ecosystem/benchmark-bmb
          ./runner/target/release/benchmark-bmb run all -i 5

      - name: Check Regression
        run: |
          # 2% 임계값 초과시 실패
          ./runner/target/release/benchmark-bmb gate 3.1 --threshold 0.02
```

#### 47.5: 계약 기반 최적화 구현

| 최적화 | 메커니즘 | 예상 성능 향상 |
|--------|---------|--------------|
| **Bounds Check Elimination** | `pre i >= 0 && i < len` | 10-30% |
| **Null Check Elimination** | `pre opt != None` | 15-25% |
| **Pure Function Detection** | 자동 CSE, 메모이제이션 | 20-50% |
| **SIMD Vectorization** | `post no_alias(a, b)` | 30-50% |
| **Dead Branch Elimination** | `post ret > 0` | 10-30% |

---

## 비판적 검토 및 개선 사항

### 현재 아키텍처 문제점

| 문제 | 영향 | 개선안 |
|------|------|--------|
| **String 오버헤드** | json_parse 2.5x 느림 | String View/Slice 도입 |
| ~~**sqrt intrinsic 부재**~~ | ~~n_body 미완료~~ | ✅ 이미 구현됨 |
| **힙 할당 제한** | binary_trees 미완료 | 동적 컬렉션 완성 |
| **Windows LLVM 부재** | 크로스 플랫폼 제한 | WSL 의존 문서화 |

### 철학적 일관성 검토

**질문**: BMB가 "AI-native"를 주장하면서 왜 성능이 중요한가?

**답변**:
1. AI가 생성한 코드도 실행되어야 함 - 성능은 실용성의 핵심
2. 계약 기반 검증으로 AI 생성 코드의 정확성 보장
3. 명시적 사양은 AI가 올바른 코드를 생성하도록 유도
4. 컴파일 타임 검증으로 런타임 오류 방지

**일관성 확인**: ✅ P0 (Performance + Correctness) 유지

### 리스크 분석

| 리스크 | 가능성 | 영향 | 완화 전략 |
|--------|--------|------|----------|
| 3-Stage 검증 실패 | 낮음 | 높음 | 개별 단계 디버깅 |
| n_body 성능 미달 | 중간 | 중간 | SIMD 최적화 |
| 회귀 테스트 누락 | 중간 | 높음 | CI 자동화 |

---

## 실행 계획

### 즉시 실행 (v0.46.3)

```bash
# 1. WSL 진입
wsl

# 2. 환경 설정
export LLVM_SYS_210_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# 3. BMB 빌드
cd /mnt/d/data/lang-bmb
cargo build --release --features llvm

# 4. 3-Stage 검증
./scripts/bootstrap_3stage.sh

# 5. 결과 기록
```

### 단기 계획 (v0.47)

| 태스크 | 산출물 | 상태 |
|--------|--------|------|
| sqrt intrinsic | stdlib/math/f64.bmb | ✅ 이미 구현됨 |
| 벤치마크 구현 | 26개 BMB 벤치마크 | ✅ 모두 완료 |
| CI 벤치마크 자동화 | .github/workflows/benchmark.yml | ✅ 완료 |
| 3-Stage 스크립트 개선 | scripts/bootstrap_3stage.sh | ✅ 완료 |
| 계약 최적화 검증 | Gate #3.3 진행 | ⏳ WSL 필요 |

---

## 완료 기준

### v0.46 완료

- [ ] 3-Stage Bootstrap 검증 통과 (WSL에서 실행 필요)
- [ ] Stage 2 == Stage 3 동일성 확인
- [ ] 부트스트랩 테스트 Stage 3로 통과

### v0.47 완료

- [x] Gate #3.1: 모든 compute 벤치마크 ≤1.10x C (✅ 0.89x-0.99x 달성)
- [ ] Gate #3.2: Benchmarks Game 11개 ≤1.05x C
- [ ] Gate #3.3: 3+ 벤치마크 < C (fibonacci, mandelbrot, spectral_norm 이미 달성)
- [x] CI 벤치마크 회귀 테스트 구축 (.github/workflows/benchmark.yml)
- [x] 성능 문서 최신화 (docs/BENCHMARK_COMPARISON.md)
- [x] 26개 BMB 벤치마크 구현 완료

---

## 2026-01-14 세션 완료 작업

### 생성된 파일

| 파일 | 설명 |
|------|------|
| `.github/workflows/benchmark.yml` | 벤치마크 CI 자동화 워크플로 |
| `docs/BENCHMARK_COMPARISON.md` | C/Rust/BMB 성능 비교 리포트 |
| `docs/PHASE_PLAN_v0.46-v0.47.md` | 페이즈 계획 문서 |

### 개선된 파일

| 파일 | 변경 내용 |
|------|----------|
| `scripts/bootstrap_3stage.sh` | 진정한 3-Stage 검증 (Stage 2 바이너리 → Stage 3) |
| `README.md` | v0.46 상태, 성능 결과 추가 |
| `docs/ROADMAP.md` | Gate #3.1 완료 상태, 벤치마크 결과 추가 |

### 발견 사항

1. **sqrt intrinsic 이미 구현됨** - n_body 벤치마크 실행 가능
2. **26개 BMB 벤치마크 모두 완료** - 미완료 구현 없음
3. **3-Stage 스크립트 버그 수정** - Stage 3가 인터프리터 대신 Stage 2 바이너리 사용
