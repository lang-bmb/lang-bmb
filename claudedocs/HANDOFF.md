# BMB Session Handoff — 2026-05-17 (Cycles 2901-2905 — P0 수정 + CI 스크립트 + Java 바인딩 배치)

> **HEAD**: `0c4e0a11` (이번 세션 완료)
> **이전 HEAD**: `c0070797` (Cycles 2877-2900)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2822, 120790 lines) — 이번 세션 bootstrap 변경 없음
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2906

---

## 이번 세션 작업 요약 (Cycles 2901-2905)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2901 | @export → String P0 FFI 수정 전파 | bmb-text 3곳 + bmb-crypto 6곳 `""` → `str_repeat("", 1)` |
| 2902 | inkwell/text 백엔드 정합성 검사 스크립트 | `scripts/check_backend_parity.py` 신규 (146 shared, 0 unexpected mismatches) |
| 2903 | bmb_runtime.c CI 자동 재빌드 스크립트 | `scripts/rebuild-runtime.sh` 신규; libbmb_runtime.a 갱신 (540→1695 symbols) |
| 2904 | Java bindings batch | bmb-json/compute/crypto/text 4개 JNA scaffold; bmb-json P0 7곳 추가 수정 |
| 2905 | @export String safety 자동 스캔 | `scripts/check_export_string_safety.py` 신규; quick-check.sh/full-cycle.sh Step 0a/0b/0c 통합 |

### 테스트 변화
2388 tests (변화 없음). Java scaffold는 native .so/.dll 로드 필요 — `mvn test`는 native 빌드 후 실행.

---

## CI 스크립트 현황

### 신규 추가 (이번 세션)

| 스크립트 | 목적 | --ci |
|----------|------|------|
| `scripts/rebuild-runtime.sh` | libbmb_runtime.a staleness 감지·재빌드 | ✅ exit 1 on stale |
| `scripts/check_backend_parity.py` | inkwell/text 백엔드 bmb_* 함수 일치 검사 | ✅ exit 1 on mismatch |
| `scripts/check_export_string_safety.py` | @export→String P0 static literal 탐지 | ✅ exit 1 on P0 |

### quick-check.sh + full-cycle.sh Step 구성 (업데이트)
```
Step 0a: rebuild-runtime.sh --ci (staleness)
Step 0b: check_backend_parity.py --ci (Rule 7)
Step 0c: check_export_string_safety.py --ci (P0 FFI)
Step 1:  cargo test --release
Step 2:  bootstrap (Stage 1 / Full 3-stage)
Step 3+: benchmarks
```

---

## @export → String P0 패치 현황 (이번 세션 기준 전체)

| 파일 | 패치 Cycle | 패치 수 |
|------|-----------|---------|
| `ecosystem/bmb-json/src/lib.bmb` — `bmb_json_type` | 2897 | 1 |
| `ecosystem/bmb-text/src/lib.bmb` — str_reverse/str_trim | 2901 | 3 |
| `ecosystem/bmb-crypto/src/lib.bmb` — b64/b32 encode/decode | 2901 | 6 |
| `ecosystem/bmb-json/src/lib.bmb` — json_get/get_string/array_get | 2904 | 7 |

**현재 상태**: `check_export_string_safety.py --ci` → 5/5 OK

---

## Java 바인딩 현황

| 라이브러리 | scaffold | tests |
|-----------|---------|-------|
| bmb-algo   | ✅ Cycle 2899 | 24 |
| bmb-json   | ✅ Cycle 2904 | 25 |
| bmb-compute | ✅ Cycle 2904 | 27 |
| bmb-crypto | ✅ Cycle 2904 | 15 |
| bmb-text   | ✅ Cycle 2904 | 29 |

총 120 tests (5개 라이브러리). Native shared library 로드 필요 — `mvn test`는 해당 환경에서 실행.

---

## 변경 파일 (이번 세션)

**ecosystem (P0 수정)**:
- `ecosystem/bmb-text/src/lib.bmb`: str_reverse + str_trim 3곳
- `ecosystem/bmb-crypto/src/lib.bmb`: b64/b32 6곳
- `ecosystem/bmb-json/src/lib.bmb`: json_get + json_get_string + json_array_get 7곳

**scripts (신규)**:
- `scripts/check_backend_parity.py`
- `scripts/rebuild-runtime.sh`
- `scripts/check_export_string_safety.py`

**scripts (수정)**:
- `scripts/quick-check.sh`: Step 0a/0b/0c 추가
- `scripts/full-cycle.sh`: Step 0a/0b/0c 추가

**runtime (갱신)**:
- `bmb/runtime/libbmb_runtime.a`: 소스 기준 재빌드 (540→1695 symbols)
- `runtime/libbmb_runtime.a`: 동기화
- `bmb/runtime/bmb_runtime.o`: git 추적 제거 (빌드 산출물)

**Java 바인딩 (Cycle 2904 신규, 각 3파일 × 4라이브러리)**:
- `ecosystem/bmb-{json,compute,crypto,text}/bindings/java/pom.xml`
- `ecosystem/bmb-{json,compute,crypto,text}/bindings/java/src/main/java/io/bmb/*/BmbXxxLib.java`
- `ecosystem/bmb-{json,compute,crypto,text}/bindings/java/src/main/java/io/bmb/*/BmbXxx.java`
- `ecosystem/bmb-{json,compute,crypto,text}/bindings/java/src/test/java/io/bmb/*/BmbXxxTest.java`

**Cycle logs**:
- `claudedocs/cycle-logs/cycle-2901.md` ~ `cycle-2905.md`

---

## 다음 세션 우선순위

### Carry-Forward (Actionable)
- **없음** — autonomous actionables 소진.

### Structural Improvement Proposals
1. **bootstrap level**: `@export pub fn -> String`이 static literal 반환 시 컴파일러 자동 heap-copy (근본 수정).
2. **str_replace/str_replace_all 입력 passthrough 안전성** — no-match 시 입력 포인터 반환이 FFI double-free 위험 있는지 분석.
3. **libbmb_runtime.a git 추적 제거** — 소스에서 빌드 가능하므로 바이너리 커밋 불필요. `.gitignore`로 이동 + rebuild-runtime.sh를 CI 필수 단계로 격상.

### Pending Human Decisions
- **B축 재측정**: API key 확인 후 실행 가능. 예상 98.0% → 98.5%+. Stale 기한: 2026-08-13.
- **tier3-spawn-overhead**: ISSUE-20260512 Option A/B/C 선택.

### 다음 자율 작업 권장 (Cycle 2906+)
- **언어 갭 추가 해소** — 아직 미구현 BMB 언어 기능 탐색
- **str_replace/str_replace_all passthrough 분석** — FFI 안전성 확인
- **libbmb_runtime.a .gitignore 이동** (소규모 구조 개선)
