# Cycle 2923: csv_parse 알고리즘 최적화 — 4.06× FAIL → 1.148× 조건부
Date: 2026-05-19

## Re-plan
Plan valid. Cycle 2921/2922 Carry-Forward: csv_parse 알고리즘 재설계.
근본 원인: packed integer encoding + double-scan (find_eol + parse_line).

## Scope & Implementation

### 변경 파일
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main.bmb` — 알고리즘 전면 재설계
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main_inproc.bmb` — 동일 최적화 적용

### 핵심 변경 사항

**변경 전 (Cycle 2919 — FAIL)**:
- `parse_quoted_field` → `i64` (packed: `pos * 1000 + field_len`)
- `parse_unquoted_field` → `i64` (packed: `pos * 1000 + field_len`)
- `parse_field` → `i64` (packed: `fp * 10000 + field_len * 10 + is_quoted`)
- `parse_line` → `i64` (packed: `fields * 1000000 + quoted * 10000 + total_chars`)
- `parse_csv` → `find_eol` → `parse_line` (이중 스캔)
- 언팩: 각 필드당 ×7 div/mod 연산 (10000, 10, 10, 1000000, 10000, ×2)

**변경 후 (Cycle 2923)**:
- `parse_quoted_field` → `(i64, i64)` (new_pos, field_len)
- `parse_unquoted_field` → `(i64, i64)` (new_pos, field_len)
- `parse_field` → `(i64, i64, i64)` (new_pos, field_len, is_quoted)
- `find_eol` + `parse_line` → **완전 제거**
- `parse_csv` → 단일 패스 nested-while (각 바이트를 한 번만 스캔)

**div/mod 제거**: 10000 fields × 7 ops → 0 ops (tuple 접근은 레지스터 직접 읽기)

## Verification & Defect Resolution

### 빌드 결과
| 파일 | 결과 |
|------|-----|
| `main.bmb` interpreter run | ✅ 동일 출력 (11 rows/44 fields/4 quoted/274 chars; 1000 rows/10000 fields/1000 quoted/67000 chars) |
| `main_inproc_bmb.exe` | ✅ build_success |

### 측정 결과

**csv_parse 최적화 비교**

| 버전 | median elapsed_us | BMB/C 비율 |
|------|------------------|-----------|
| 기존 (packed int + double-scan) | 11033 µs | 4.06× FAIL |
| 최적화 v1 (tuple + parse_row_loop 재귀) | 3481 µs | 1.17× |
| 최적화 v2 (tuple + nested-while 반복) | **3423 µs** | **1.148×** |
| C GCC -O2 | 2982 µs | 1.00× |

**체크섬**: BMB 55003850000 ✅ (변경 전후 동일)
**판정**: FAIL (4.06×) → **조건부 OK (1.148×)** — GCC 대비 14.8% 느림

### 개선 비율
- 절대 성능: 11033 µs → 3423 µs = **3.22× 향상**
- 상대 성능: 4.06× → 1.148× = **3.54× 격차 개선**

## Reflection
- **Scope fit**: csv_parse 근본 원인(packed integer encoding + double-scan) 해소.
- **결과**: 4.06× FAIL → 1.148× 조건부. PASS 기준(≤1.0×)에는 아직 미달이나 GCC 대비 차이는 LLVM vs GCC 백엔드 차이 + `byte_at` 간접 접근 vs 직접 포인터 차이로 설명 가능.
- **3-tuple return**: BMB `(i64, i64, i64)` 반환 및 destructuring 정상 지원 확인 (Cycle 2621 추가, 골든 테스트 확인).
- **반복 vs 재귀**: nested-while이 parse_row_loop 재귀 대비 약 1.7% 빠름 (3481→3423µs). 두 버전 모두 정확.
- **Roadmap impact**: csv_parse FAIL → 조건부로 상향. tier3_inproc_summary 갱신 필요.
- **남은 격차(14.8%)**: LLVM + 재귀 해소 후에도 차이 존재. `byte_at` 간접 접근이 C의 직접 포인터 대비 상수 오버헤드. 언어 레벨 fix 필요 시 raw pointer 기반 스캔 가능 (별도 검토).

## Carry-Forward
- Actionable: Cycle 2924 — http_parse 사전 할당 최적화 (1.26× → ~1.0× 목표)
- Structural Improvement Proposals:
  1. **csv_parse 추가 최적화** (Low): `byte_at` → raw pointer 스캔으로 전환. BMB `load_u8(ptr)` intrinsic + string `.as_ptr()` 사용 가능 여부 확인 필요. 목표 ≤1.05×.
  2. **tier3_inproc_summary 갱신**: csv_parse 결과 4.06× → 1.148× 업데이트.
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2924 — http_parse 최적화
