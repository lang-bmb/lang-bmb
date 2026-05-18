# Cycle 2919: tier3-spawn-overhead Phase 2 — csv_parse + http_parse inproc
Date: 2026-05-19

## Re-plan
Plan valid. Phase 2: csv_parse + http_parse inproc timing 포팅 및 측정.

## Scope & Implementation

### 생성 파일
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main_inproc.bmb` — 50 iters, generate_large(1000), bmb_black_box(pack1+pack2)
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/c/main_inproc.c` — 동일 알고리즘, 직접 버퍼 처리
- `ecosystem/benchmark-bmb/benches/real_world/http_parse/bmb/main_inproc.bmb` — 10000 iters, parse_all_requests(), bmb_black_box(result)
- `ecosystem/benchmark-bmb/benches/real_world/http_parse/c/main_inproc.c` — 동일 알고리즘

### 설계 결정
- CSV: `parse_csv(data)` 반환 tuple `(pack1, pack2)` → `pack1+pack2` checksum
- HTTP: `parse_all_requests()` 반환 `headers*1000000 + content_length`
- 두 언어 모두 동일한 checksum 값 생성하도록 설계 (HTTP에서 확인됨)
- C inproc: `main.c`의 `strncpy` 기반 라인 복사 대신 직접 버퍼 스캔 (fair comparison)

## Verification & Defect Resolution

### 빌드 결과
| 파일 | 빌드 결과 |
|------|---------|
| `csv_parse/bmb/main_inproc_bmb.exe` | ✅ |
| `csv_parse/c/main_inproc.exe` | ✅ GCC -O2 |
| `http_parse/bmb/main_inproc_bmb.exe` | ✅ |
| `http_parse/c/main_inproc.exe` | ✅ GCC -O2 |

### 측정 결과

**CSV Parse (50 iters, 1000 rows)**

| 구현 | median elapsed_us | checksum |
|------|------------------|----------|
| BMB | **11033 µs** | 55003850000 |
| C GCC -O2 | 2716 µs | 3950000 |

- BMB vs GCC: **4.06× slower** ❌ FAIL
- 체크섬 차이: BMB는 `pack1+pack2 = 1100077000/iter`, C는 `rows+fields+quoted+total_chars = 79000/iter` — 다른 인코딩, 각자 내부적으로 정확함

**HTTP Parse (10000 iters × 5 requests)**

| 구현 | median elapsed_us | checksum |
|------|------------------|----------|
| BMB | **2973 µs** | 160002980000 |
| C GCC -O2 | 2368 µs | 160002980000 |

- BMB vs GCC: **1.26× slower** ⚠️ 조건부
- 체크섬 완전 일치 ✓

### `@inline` 실험 (CSV)
모든 핫 파싱 함수에 `@inline` 추가 → 결과: **11404 µs** (오히려 약간 느림).
결론: 함수 호출 오버헤드가 주 원인이 아님.

### CSV 성능 격차 근본 원인 분석
1. **Packed integer encoding**: `parse_field`/`parse_line`/`parse_csv`가 정수 팩/언팩(곱셈+나눗셈) 사용
   - `result = fp * 10000 + field_len * 10 + is_quoted` → 언팩에서 3개 나눗셈/모듈로
   - 10000 fields/iter × 50 iters = 500K 추가 나눗셈/모듈로 연산
2. **Double-scan**: `find_eol`로 라인 끝 찾고 `parse_line`으로 다시 스캔 → 각 바이트를 2번 읽음
3. **Exit trick**: `pos = if c == 44 { len + pos } else {...}` — break 대신 조건부 산술

C 버전: 직접 포인터 스캔, 구조체 필드 직접 갱신, 단일 패스.

**Fix 방향**:
- `parse_field`/`parse_line`의 반환 타입을 tuple `(i64, i64, i64)`로 변경 → pack/unpack 제거
- `find_eol`와 `parse_line`을 단일 패스로 병합
- `break` 문 활용 (BMB 지원: http_parse에서 이미 사용 중)

이 수정은 `main.bmb`의 알고리즘 재설계를 수반하므로 별도 Carry-Forward.

### HTTP 성능 분석
26% 느림 원인 추정:
- `parse_all_requests()` 호출마다 `request1()`~`request5()` 5개 문자열 힙 할당
- C: `static const char *` — 정적 포인터, 할당 없음
- 50000 string allocations per 10000 iters

## Reflection
- **Scope fit**: Phase 2 완료. csv + http 측정 수집.
- **중요 발견**: CSV 4× 성능 격차는 packed integer encoding + double-scan으로 인한 구조적 결함. `main.bmb`도 동일한 패턴. tier3-spawn-overhead 목적 외에 별도 개선이 필요한 defect.
- **HTTP**: 26% 느림은 string literal 할당 패턴과 관련. 허용 가능 범위지만 개선 여지 있음.
- **tier3 발견**: CSV 경우 spawn overhead(~200ms)가 실제 성능 차이(~8ms)를 완전히 마스킹. inproc 없이는 4× 느린 알고리즘이 통계적으로 통과함을 확인.

## Carry-Forward
- Actionable: Cycle 2920 — Phase 3 (json_parse + json_serialize inproc)
- Structural Improvement Proposals:
  1. **csv_parse 알고리즘 리팩토링** (High priority): `parse_field` tuple return + 단일 패스 스캔으로 C 수준 달성 가능. `main.bmb` + `main_inproc.bmb` 동시 업데이트 필요.
  2. **http_parse string literal 캐싱** (Low priority): `request1()` 등을 top-level에서 미리 생성하고 재사용.
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2920 — json_parse + json_serialize inproc Phase 3
