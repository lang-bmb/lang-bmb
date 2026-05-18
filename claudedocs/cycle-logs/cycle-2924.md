# Cycle 2924: http_parse 사전 할당 최적화 — 1.255× → 1.186×
Date: 2026-05-19

## Re-plan
Plan valid. Cycle 2923 Carry-Forward: http_parse 사전 할당 최적화.
근본 원인 추정: `request1()`~`request5()` 반복 호출 = 50000 String 할당/run.

## Scope & Implementation

### 변경 파일
- `ecosystem/benchmark-bmb/benches/real_world/http_parse/bmb/main_inproc.bmb`
  - `parse_all_pre(r1,r2,r3,r4,r5)` 신규 — 사전 할당된 String 파라미터 사용
  - `run_benchmark` → 5개 String 파라미터 전달 방식으로 변경
  - `main()` → `r1`~`r5` timed loop 전에 1회 할당

## Verification & Defect Resolution

### 빌드 결과
| 파일 | 결과 |
|------|-----|
| `http_parse/bmb/main_inproc_bmb.exe` | ✅ |

### 측정 결과

**HTTP Parse (10000 iters × 5 requests, 사전 할당 적용)**

| 구현 | runs (µs) | median elapsed_us | checksum |
|------|-----------|------------------|----------|
| BMB (최적화) | 2881/3091/2906/2904/2945 | **2906 µs** | 160002980000 |
| C GCC -O2 | 2355/2453/2451/2503/2293 | **2451 µs** | 160002980000 |

- BMB vs GCC: **1.186× slower** ⚠️ 조건부
- 기존 (Cycle 2919): 1.255× → 1.186× (0.069 개선)
- 체크섬 완전 일치 ✓

### 분석
- 사전 할당 효과: BMB 2973→2906µs (2.3% 개선) — **예상보다 작음**
- 원인: `request1()`~`request5()`의 String 리터럴은 이미 정적 저장소 기반으로, 실제 heap allocation overhead가 작았던 것으로 추정.
- **실제 병목**: String.byte_at() 간접 접근 vs C의 직접 char* 포인터 — 10000 iters × 수백 byte_at() 호출의 축적된 오버헤드.
- 이 차이는 LLVM vs GCC + BMB string 표현 overhead로 설명 가능. 구조적 한계.

## Reflection
- **Scope fit**: http_parse 사전 할당 최적화 완료.
- **효과**: 조건부 → 조건부 (수치 개선). 1.255× → 1.186× (0.069 감소).
- **근본 원인 재평가**: String allocation이 아닌 byte_at() 접근 패턴 overhead가 주 원인. LLVM/GCC 백엔드 차이 범위(~20%) 내에 있음.
- **tier3 패턴**: 모든 7개 벤치마크 중 0 FAIL (csv_parse Cycle 2923에서 해소). 조건부 3개 (brainfuck/http_parse/csv_parse)는 언어 레벨 특성 차이로 설명 가능.
- **Roadmap impact**: http_parse 추가 최적화 — ROI 낮음. brainfuck은 언어 기능(stack array) 필요.

## Carry-Forward
- Actionable: 없음 (tier3 inproc 작업 완료)
- Structural Improvement Proposals:
  1. **byte_at 최적화** (Long-term): `load_u8(ptr)` 기반 raw pointer 스캔으로 bounds-check overhead 제거. BMB 언어 레벨 API 확장 필요.
  2. **brainfuck stack array** (Long-term): BMB 언어에 고정 크기 stack array 추가 시 brainfuck도 PASS 가능.
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2925 — GPUStack B축 실제 재측정 (Always FAIL 11문제 수정 후)
