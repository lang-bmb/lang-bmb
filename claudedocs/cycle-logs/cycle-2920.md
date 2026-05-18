# Cycle 2920: tier3-spawn-overhead Phase 3 — json_parse + json_serialize inproc
Date: 2026-05-19

## Re-plan
Plan valid. Phase 3: json_parse + json_serialize inproc timing 포팅 및 측정.

## Scope & Implementation

### 생성 파일
- `ecosystem/benchmark-bmb/benches/real_world/json_parse/bmb/main_inproc.bmb` — 100000 iters on "[1,...,10]", bmb_black_box(is_valid+count) per iter
- `ecosystem/benchmark-bmb/benches/real_world/json_parse/c/main_inproc.c` — 동일 알고리즘, black_box per iter
- `ecosystem/benchmark-bmb/benches/real_world/json_serialize/bmb/main_inproc.bmb` — 10000 iters, buf/arr 재사용, bmb_black_box(len1+len2+len3) per iter
- `ecosystem/benchmark-bmb/benches/real_world/json_serialize/c/main_inproc.c` — 동일 알고리즘 (manual write_int, heap buf — snprintf 없음)

### 설계 결정
- json_serialize C inproc: 기존 `main.c`의 snprintf 기반 `write_int` 대신 수동 자릿수 기록 함수 사용 (BMB 알고리즘과 동등)
- json_serialize BMB: buf/arr은 timed loop 밖에서 한 번만 할당, 재사용 (main.bmb와 동일 패턴)

## Verification & Defect Resolution

### 빌드 결과
| 파일 | 빌드 결과 |
|------|---------|
| `json_parse/bmb/main_inproc_bmb.exe` | ✅ |
| `json_parse/c/main_inproc.exe` | ✅ GCC -O2 |
| `json_serialize/bmb/main_inproc_bmb.exe` | ✅ |
| `json_serialize/c/main_inproc.exe` | ✅ GCC -O2 |

### 측정 결과

**JSON Parse (100000 iters, "[1,...,10]")**

| 구현 | median elapsed_us | checksum |
|------|------------------|----------|
| BMB (LLVM opt-O2) | **2537 µs** | 1100000 |
| C GCC -O2 | 3062 µs | 1100000 |

- BMB vs GCC: **0.829× (BMB이 1.21× 빠름)** ✅ PASS
- 체크섬 완전 일치 ✓

**JSON Serialize (10000 iters × 3 serializations)**

| 구현 | median elapsed_us | checksum |
|------|------------------|----------|
| BMB (LLVM opt-O2) | **467 µs** | 1590000 |
| C GCC -O2 | 653 µs | 1590000 |

- BMB vs GCC: **0.715× (BMB이 1.40× 빠름)** ✅ PASS
- 체크섬 완전 일치 ✓ (per-iter: 61+22+76=159 chars × 10000 = 1590000)

## Reflection
- **Scope fit**: Phase 3 완료. json_parse + json_serialize 모두 측정.
- **json_parse PASS**: BMB `@inline` tail-recursive descent → LLVM tight loop. C의 while loop 기반과 비교해 BMB가 더 효율적.
- **json_serialize PASS**: BMB의 `store_u8` 기반 raw buffer write vs C의 유사한 접근법. LLVM이 BMB를 더 잘 최적화 (store_u8 체인이 vectorization 후보).
- **Phase 1-3 패턴 정리**:
  - BMB PASS: lexer (5.9×), json_parse (1.21×), json_serialize (1.40×)
  - BMB 조건부: brainfuck (1.21× 느림, heap vs stack), http_parse (1.26× 느림, string allocation)
  - BMB FAIL: csv_parse (4.1× 느림, packed integer encoding + double-scan)

## Carry-Forward
- Actionable: Cycle 2921 — Phase 4 (sorting inproc)
- Structural Improvement Proposals: (Cycle 2919에서 이미 등록)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2921 — sorting inproc Phase 4
