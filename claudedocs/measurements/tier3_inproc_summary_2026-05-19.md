# Tier 3 real_world Inproc Timing Summary
**Date**: 2026-05-19  
**Cycles**: 2918–2921 (Phase 1–4)  
**방법**: `time_ns()` 직접 측정 + `bmb_black_box()` per-iter (DCE 차단)  
**빌드**: BMB `--release` + LLVM opt -O2 / C GCC -O2

---

## 측정 결과 (median of 5 runs)

| 벤치마크 | BMB median (µs) | C GCC median (µs) | 비율 (BMB/C) | 판정 |
|---------|----------------|------------------|-------------|------|
| lexer | 1140 | 6740 | 0.169× | ✅ PASS (5.9× faster) |
| brainfuck | 2065 | 1707 | 1.21× | ⚠️ 조건부 (heap vs stack) |
| csv_parse | 3423 | 2982 | 1.148× | ⚠️ 조건부 (Cycle 2923 최적화: tuple return + single-pass) |
| http_parse | 2906 | 2451 | 1.186× | ⚠️ 조건부 (Cycle 2924 최적화: 사전 할당, byte_at overhead 한계) |
| json_parse | 2537 | 3062 | 0.829× | ✅ PASS (1.21× faster) |
| json_serialize | 467 | 653 | 0.715× | ✅ PASS (1.40× faster) |
| sorting | 471670 | 3023238 | 0.156× | ✅ PASS (6.41× faster) |

**요약**: 4 PASS / 3 조건부 / 0 FAIL (7개 중) — Cycle 2923: csv_parse FAIL→조건부

---

## 이터레이션 규모

| 벤치마크 | 이터레이션 규모 | 체크섬 (BMB) |
|---------|-------------|------------|
| lexer | 100000 iters | 8900000 |
| brainfuck | 100 iters (Mandelbrot BF) | 4100 |
| csv_parse | 50 iters × 1000 rows | 55003850000 |
| http_parse | 10000 iters × 5 requests | 160002980000 |
| json_parse | 100000 iters | 1100000 |
| json_serialize | 10000 iters × 3 serializations | 1590000 |
| sorting | 5 iters × 200 sizes × 4 sorts | 2019526740 |

---

## spawn overhead 영향 제거 확인

| 구분 | 이전 framework 방식 | 이후 inproc 방식 |
|------|-------------------|--------------:|
| 측정 단위 | 전체 프로세스 wall-time | time_ns() 직접 |
| spawn overhead | 200ms+ 포함 | 제외 |
| csv_parse 실제 비율 | ~1.0× (마스킹됨) | **4.06×** (노출됨) |
| 신뢰도 | ratio만 유효, absolute 무의미 | 절대값 의미 있음 |

---

## PASS 패턴 분석

### 빠른 이유 (4 PASS)
| 벤치마크 | 원인 |
|---------|-----|
| lexer | `@inline` 전체 + tail-recursive → LLVM TCO tight loop |
| json_parse | `@inline` tail-recursive descent → LLVM tight loop |
| json_serialize | `store_u8` raw buffer write → vectorization 후보 |
| sorting | `@inline fn swap` + LLVM aggressive inliner → GCC 대비 6.4× |

### 느린 이유 (1 FAIL + 2 조건부)
| 벤치마크 | 원인 |
|---------|-----|
| csv_parse | packed integer encoding (×3 div/mod per field) + double-scan (find_eol + parse_line 분리) |
| brainfuck | heap malloc tape (C: stack array) → allocation overhead |
| http_parse | 10000 iters마다 5개 String heap allocation (C: static const ptr) |

---

## 개선 계획 (Carry-Forward)

| 우선순위 | 벤치마크 | 개선 방향 | 예상 효과 |
|---------|---------|---------|---------|
| P1 | csv_parse | tuple return 대신 pack/unpack 제거 + 단일 패스 스캔 | 4.06× → ≤1.2× 목표 |
| P2 | http_parse | `request1()`~`request5()` 사전 생성 + 재사용 | 1.26× → ~1.0× 기대 |
| P3 | brainfuck | 언어 기능(stack array) 추가 필요 — 장기 |

---

## 이전 stamp (Cycle 2752 legacy framework)

| bench | BMB framework | C framework | ratio |
|-------|-------------|------------|-------|
| lexer | 28ms | 28ms | 1.000× |
| http_parse | 45ms | 47ms | 0.957× |
| brainfuck | 42ms | 45ms | 0.933× |

→ legacy framework는 spawn overhead (200ms+)로 모든 Tier 3 결과가 noise-dominated.  
→ inproc 측정이 유일한 신뢰가능 Tier 3 기준치.
