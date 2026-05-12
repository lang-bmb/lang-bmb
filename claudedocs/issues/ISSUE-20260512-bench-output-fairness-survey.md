# ISSUE-20260512 — Bench output fairness survey (Cycle 2769 verify tool)

## 핵심 메타

**우선순위**: P2 (measurement fairness 영향, 다수 bench)
**영역**: ecosystem/benchmark-bmb / bench correctness
**상태**: Open — multi-bench scope, sub-ISSUE로 분리 진행

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2769) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | `python scripts/verify_bench_outputs.py --tier all --rebuild --verbose` |
| `observed_rate` | **11/17 PASS (65%)** — 4 unfairness + 2 build/run fail |
| `scope` | Tier 1 (10 benches) + Tier 3 (7 benches) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / gcc MinGW |
| `estimated_cycles` | 5-10 cycles total (per sub-ISSUE) — hypothesis |

## 문제

Cycle 2769 신규 작성 `scripts/verify_bench_outputs.py` 로 BMB ↔ C bench 출력 정합 검사. 4개 bench가 unfair comparison (BMB와 C가 다른 작업 수행 가능성), 2개 build/run 실패.

## 측정 결과

### Tier 1 (compute/) — 10 benches

| bench | 결과 | 진단 |
|-------|------|------|
| binary_trees | ✅ PASS | |
| fannkuch | ✅ PASS | |
| fasta | ✅ PASS | |
| **fibonacci** | ❌ FAIL (C run) | C 빌드 정상이나 실행 시 returncode≠0 — runtime check |
| hash_table | ✅ PASS | (cycle 2767 검증) |
| knapsack | ✅ PASS | |
| mandelbrot | ✅ PASS | |
| **n_body** | ⚠️ MISMATCH | FP precision diff (~7th decimal) — normal? |
| nqueen | ✅ PASS | |
| spectral_norm | ✅ PASS | |

### Tier 3 (real_world/) — 7 benches

| bench | 결과 | 진단 |
|-------|------|------|
| brainfuck | ✅ PASS | |
| **csv_parse** | ❌ MISMATCH | 구조적 diff — Strings 라인 누락, fields/quoted/chars 다름 |
| http_parse | ✅ PASS | |
| json_parse | ✅ PASS | |
| **json_serialize** | ❌ MISMATCH | 단일 char bug — `Array: {1,2,3,4,5]` (BMB) vs `Array: [1,2,3,4,5]` (C). `[` 대신 `{` 출력 |
| **lexer** | ❌ MISMATCH | pre-existing 0-token bug (별도 ISSUE-20260512-bmb-lexer-bench-zero-tokens.md) |
| **sorting** | ❌ FAIL (BMB run) | **재빌드 binary hangs** (main_verify.exe 무한 wait, main.exe 정상) — 잠재 컴파일러 회귀 |

## 영향 평가

| 영역 | 영향 |
|------|------|
| **P-track 측정 fairness** | 🚨 **4/17 = 24% benches unfair comparison** — ratio 측정 의미 부족 |
| **M1 가설 (≤1.05x 16/16)** | ⚠️ 8/9 Tier 1 fair (n_body FP 정상 가정 시 PASS). Tier 3는 3/7만 fair |
| **CI gates** | ❌ 미존재 — 회귀 시 발견 못함 |
| **개발 마찰** | ⚠️ 측정 변화 시 (e.g., cycle 2750 1.040x) 원인 추적 ambiguity (compiler vs correctness) |

## 분리 ISSUE (sub)

| sub-ISSUE | bench | 우선순위 | scope |
|-----------|-------|---------|-------|
| ISSUE-20260512-bmb-lexer-bench-zero-tokens.md | lexer | P2 | 별도 등록 (Cycle 2765) |
| 신규: `bmb-sorting-rebuild-hang` | sorting | **P1** | 컴파일러 회귀 가능성 |
| 신규: `bmb-json-serialize-bracket-bug` | json_serialize | P2 | 단일 char bug |
| 신규: `bmb-csv-parse-output-divergence` | csv_parse | P2 | 구조적 diff |
| 신규: `bmb-fibonacci-c-runtime-fail` | fibonacci | P3 | C exe runtime check |
| 신규: `bmb-n-body-fp-precision-fairness` | n_body | P3 | FP 정밀도 정상 vs algorithm diff 판별 |

## 해결 방안

### 단기 (1-3 cycles)

1. **CI 통합**: `scripts/verify_bench_outputs.py` 를 `scripts/quick-check.sh` / CI workflow에 추가. mismatch / fail 시 alert.
2. **각 sub-ISSUE 진단**: BMB 측 출력이 실제 동등 작업인지 검증 (sorting hang root cause + json_serialize char bug 등)

### 장기 (multi-cycle)

3. **FP fairness gate**: n_body / mandelbrot 같은 FP-heavy bench에 absolute / relative tolerance 적용 (epsilon)
4. **golden test 통합**: bench output을 골든 테스트 케이스로 사용 → 회귀 즉시 검출

## HUMAN 결정 필요

- 단기 sub-ISSUE 우선순위 (sorting hang P1 우선 권고)
- CI workflow 추가 시점 (Cycle 2769 추가 또는 다음 세션)
- FP tolerance 정책 결정 (절대값 vs 상대값)

## 종결 기준

- [ ] 17 Tier 1+3 benches 모두 PASS 또는 정당화된 MISMATCH (FP precision 등 명시)
- [ ] CI workflow에 `verify_bench_outputs.py` 통합
- [ ] 회귀 시 alert + cycle 진입

## 메타

- 관련 ISSUE:
  - `ISSUE-20260512-bmb-lexer-bench-zero-tokens.md` (sub)
- 인용 cycle: cycle-2769.md (도구 작성 + 1차 측정)
- 외부 참조: `scripts/verify_bench_outputs.py`
