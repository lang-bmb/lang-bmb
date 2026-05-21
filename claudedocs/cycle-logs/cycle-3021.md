# Cycle 3021: brainfuck band 255 branchless 최적화
Date: 2026-05-21

## Re-plan
Carry-forward (Cycle 3020): csv_parse/http_parse byte_at 적용 가능성 탐색.
탐색 결과: 양쪽 모두 이미 `str_data + load_u8` 최적 패턴 사용. byte_at 적용 불가.

대안 발견: brainfuck 에서 `% 256` (나머지) → `band 255` (비트 AND) 치환으로 decrement 분기 제거 가능.

## Scope & Implementation

### 탐색

BMB에는 `band` (bitwise AND) 연산자 존재 확인:
- `tests/bench/bench_memory.bmb`: `(seed + i * 31) band 255;`
- `tests/bootstrap/test_golden_binary_gcd.bmb`: `(u bor v) band 1`

### 최적화: band 255 wrapping arithmetic

변경 전:
```bmb
43 => { let v = tape_get(tape, ptr); tape_set(tape, ptr, (v + 1) % 256) },
45 => { let v = tape_get(tape, ptr); tape_set(tape, ptr, if v == 0 { 255 } else { v - 1 }) },
```

변경 후:
```bmb
43 => { let v = tape_get(tape, ptr); tape_set(tape, ptr, (v + 1) band 255) },
45 => { let v = tape_get(tape, ptr); tape_set(tape, ptr, (v - 1) band 255) },
```

**근거**:
- `(v + 1) band 255`: LLVM `and i64 ..., 255` 직접 생성 (나머지 연산 제거)
- `(v - 1) band 255`: 두의 보수에서 0-1=-1, -1 & 0xff = 255 → 분기 없는 wrapping subtract
- `if v == 0 { 255 } else { v - 1 }` 제거 → LLVM select/cmov 분기 제거

**정확성 검증**:
- v=0: (0-1) band 255 = (-1) band 255 = 255 ✓
- v=5: (5-1) band 255 = 4 band 255 = 4 ✓
- v=255: (255-1) band 255 = 254 band 255 = 254 ✓
- v=255 inc: (255+1) band 255 = 256 band 255 = 0 ✓

## Verification & Defect Resolution

- 빌드: `{"type":"build_success"}` ✅
- 체크섬: BMB=0, C=0 (일치) ✅
- 정확성: band 255 wrapping 수동 검증 ✅

### 7-run median 측정 (2026-05-21)

| 측정 | BMB (µs) | C (µs) |
|------|---------|-------|
| 7-run median | **7594** | **8395** |
| **비율** | **0.905×** | — |

**개선 이력**:
| Cycle | 최적화 | 비율 |
|-------|-------|------|
| 3017 (기준) | calloc/free per iter | 1.037× |
| 3018 | memset_fill 단일 alloc | 0.974× |
| 3020 | match dispatch + direct byte_at | 0.958× |
| 3021 | band 255 branchless | **0.905×** |

총 개선: 1.037× → **0.905×** = **-13.2pp** (BMB 9.5% faster than C)

## Reflection

- **Scope fit**: band 255 최적화 완료. 측정으로 0.905× 달성.
- **Latent defects**: 없음.
- **Philosophy fit**: band 255 는 C와 동일한 최적화 (C도 unsigned char wrapping = and 0xff). 언어 수준의 정당한 관용구.
- **Roadmap impact**: brainfuck 0.958× → **0.905×**. P-track 7/7 더욱 여유 있게 PASS.
- **P-track 전체 현황**: brainfuck 이제 가장 많이 BMB가 앞서는 real-world 벤치마크 중 하나.

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  - `band 255` 패턴 문서화 (BMB 이진 wrapping 관용구)
  - 다른 벤치마크에서 `% 2^n` → `band (2^n - 1)` 적용 가능성 탐색 (낮은 우선순위)
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP §5 brainfuck 0.905× 갱신
- Next Recommendation: Cycle 3022 = ROADMAP 갱신 + commit + 다음 최적화 목표 탐색 (또는 Bootstrap 점검)
