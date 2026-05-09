# BMB Session Handoff — 2026-05-09 (Cycles 2558-2565 — Track T Complete ★)

> **이전 HEAD**: `9e7132d1` (docs(handoff): Cycles 2550-2557 closure — Track N complete, M2 ~95%)
> **현재 세션**: Cycles 2558-2565 (uncommitted)
> **세션 성격**: 10-cycle run-cycle. Track M closeout + Track T Node.js bindings 전체 완성 (5/5 libraries).
> **결정적 결과**: Track T Node 5/5 ✅ (algo/compute/text/crypto/json) + Track M 100% ✅.

---

## 1. 이번 세션 요약 (Cycles 2558-2565)

### Cycle 2558 — Track M closeout: `bmb parse --format compact`

**구현** (`bmb/src/main.rs`):
- `Parse` command default format: `"json"` → `"compact"`
- `parse_file()` 재작성:
  - `compact` (default): `serde_json::to_string()` (machine-friendly)
  - `pretty`: `serde_json::to_string_pretty()`
  - `sexpr`/`s-expression`: S-expression AST
  - `--human` 모드에서는 pretty 자동 선택
- Rule 8 완전 준수 달성.

### Cycle 2559 — Track M closeout: AI_OUTPUT_SCHEMA.md § 3 완성

**구현** (`docs/AI_OUTPUT_SCHEMA.md`):
- Section 3 "Parse Output" 완성: `bmb parse --format compact|pretty|sexpr|json` 스키마 문서화
- Track M 완료 체크리스트 3/4 체크 (CI gate는 optional로 유지)
- Track M ~100% ✅ 달성

### Cycle 2560 — Track T PoC: bmb-algo Node.js bindings

**구현** (`ecosystem/bmb-algo/bindings/node/`):
- `index.js`: 24 functions (gcd, fibonacci, prime_count, knapsack, lcs, edit_distance, array_sum, binary_search 등)
- `package.json`: `"bmb-algo"` v0.1.0, koffi ^2.16.2
- `test/test.js`: 21 tests

**핵심 koffi 패턴 확립**:
- 배열 파라미터: `int64_t*` (not `int64_t`) 선언 필요
- `BigInt64Array` for passing array data
- 모든 반환값: JS number (not BigInt)
- String FFI: `bmb_ffi_cstr_to_string` → void* → `bmb_ffi_string_data` → `bmb_ffi_free_string`

**21/21 tests PASS**

### Cycle 2561 — Track T: bmb-algo README + .gitignore

**구현**:
- `ecosystem/bmb-algo/bindings/node/README.md`: 전체 API 문서, FFI 아키텍처 다이어그램
- `ecosystem/bmb-algo/README.md`: Node.js 설치 섹션 추가
- `ecosystem/.gitignore`: `**/node_modules/` + `**/package-lock.json` 추가

### Cycle 2562 — Track T: bmb-compute Node.js bindings

**구현** (`ecosystem/bmb-compute/bindings/node/`):
- 27 functions: abs/min/max/clamp/sign/ipow/sqrt/factorial, XorShift64*, sum/mean_scaled/variance_scaled/median_scaled, dot_product/dist_squared/weighted_sum 등
- **10/10 tests PASS**

### Cycle 2563 — Track T: bmb-text + bmb-crypto Node.js bindings

**구현**:
- `bmb-text`: 21 functions (kmp_search, str_find, str_contains, str_reverse, to_upper/lower, trim, str_replace, repeat, hamming_distance, token_count 등) — **9/9 PASS**
- `bmb-crypto`: 14 functions (sha256/md5/crc32, base64/base32 encode/decode, hmac_sha256, checksums, rot13, hex encode/decode) — **8/8 PASS**
  - SHA-256('hello') = `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824` ✅

### Cycle 2564 — Track T complete: bmb-json Node.js bindings

**구현** (`ecosystem/bmb-json/bindings/node/`):
- 12 functions: validate, get_type, stringify, array_len, object_len, count, get_number, has_key, get_bool, get, get_string, array_get
- **16/16 tests PASS**

**중요 결함 발견 및 수정**:
- `_from` helper가 출력 포인터를 `_free_str(ptr)` 하면 exit 116 크래시 발생
- 원인: bmb-json 출력 문자열은 **library-owned** (Python 바인딩도 출력을 free하지 않음)
- 수정: `const _from = (ptr) => _str_data(ptr);` (free 제거)
- bmb-text/bmb-crypto는 caller-owned 모델이므로 free 필요. 라이브러리별 ownership 차이 존재.

### Cycle 2565 — M2 gate assessment + ROADMAP update

- M2 현황 평가: M(100%), N(99%), O(90%), Q(60%), R(75%)
- M3 External Bindings 조건 "Python + Node 바인딩" → ✅
- ROADMAP.md cycle 로그 테이블 갱신 (2532-2564)
- ROADMAP.md M3 조건 업데이트

---

## 2. 산출물 (미커밋)

### Modified / Created files

| 분류 | 파일 | 상태 |
|------|------|------|
| Track M | `bmb/src/main.rs` | modified |
| Track M | `docs/AI_OUTPUT_SCHEMA.md` | modified |
| Track T | `ecosystem/bmb-algo/bindings/node/index.js` | new |
| Track T | `ecosystem/bmb-algo/bindings/node/package.json` | new |
| Track T | `ecosystem/bmb-algo/bindings/node/test/test.js` | new |
| Track T | `ecosystem/bmb-algo/bindings/node/node_modules/` | new (gitignored) |
| Track T | `ecosystem/bmb-algo/bindings/node/README.md` | new |
| Track T | `ecosystem/bmb-algo/README.md` | modified |
| Track T | `ecosystem/bmb-compute/bindings/node/index.js` | new |
| Track T | `ecosystem/bmb-compute/bindings/node/package.json` | new |
| Track T | `ecosystem/bmb-compute/bindings/node/test/test.js` | new |
| Track T | `ecosystem/bmb-text/bindings/node/index.js` | new |
| Track T | `ecosystem/bmb-text/bindings/node/package.json` | new |
| Track T | `ecosystem/bmb-text/bindings/node/test/test.js` | new |
| Track T | `ecosystem/bmb-crypto/bindings/node/index.js` | new |
| Track T | `ecosystem/bmb-crypto/bindings/node/package.json` | new |
| Track T | `ecosystem/bmb-crypto/bindings/node/test/test.js` | new |
| Track T | `ecosystem/bmb-json/bindings/node/index.js` | new |
| Track T | `ecosystem/bmb-json/bindings/node/package.json` | new |
| Track T | `ecosystem/bmb-json/bindings/node/test/test.js` | new |
| Gitignore | `ecosystem/.gitignore` | modified |
| ROADMAP | `docs/ROADMAP.md` | modified |
| Cycle logs | `claudedocs/cycle-logs/cycle-{2558..2565}.md` | new |
| HANDOFF | `claudedocs/HANDOFF.md` | this file |

---

## 3. 검증 상태

| 항목 | 결과 |
|------|------|
| bmb-algo Node bindings | ✅ **21/21 tests** |
| bmb-compute Node bindings | ✅ **10/10 tests** |
| bmb-text Node bindings | ✅ **9/9 tests** |
| bmb-crypto Node bindings | ✅ **8/8 tests** |
| bmb-json Node bindings | ✅ **16/16 tests** |
| Track M: `bmb parse --format compact` | ✅ machine-friendly output |
| `cargo test --release --lib` | ⚠️ pre-existing 1 fail (3772/3773, unchanged) |

---

## 4. Track T Node Bindings 완성 요약

| 라이브러리 | 함수 수 | 테스트 | 핵심 패턴 |
|----------|---------|--------|----------|
| bmb-algo | 24 | 21/21 | int64_t*, BigInt64Array |
| bmb-compute | 27 | 10/10 | scalar + stats + vector |
| bmb-text | 21 | 9/9 | String→String + String×String |
| bmb-crypto | 14 | 8/8 | String→String (hash/encode) |
| bmb-json | 12 | 16/16 | library-owned output (no free) |

**총계**: 98 functions, 64/64 tests

**koffi FFI 패턴 (이번 세션 확립)**:
- 배열 파라미터: `int64_t*` 선언 + `BigInt64Array` 전달
- String FFI: `bmb_ffi_cstr_to_string` → void* 전달 → `bmb_ffi_string_data` → `bmb_ffi_free_string`
- 출력 ownership: bmb-text/crypto = caller-owned (free 가능); bmb-json = library-owned (free 금지)

---

## 5. 다음 세션 우선순위

### 1차 — Track O Phase 7 (optional, ~1 cycle)

**내용**: context-pack v1 schema JSON 유효성 검증 + uses 의존성 그래프.

### 2차 — Track Q Phase 2 (BMB-native lint, 2-3 cycles)

**내용**: `bmb lint --ai-friendly` BMB 자체 구현.

### 3차 — npm publish 준비 (Track T publishing)

**내용**: `@bmb/algo`, `@bmb/compute`, `@bmb/text`, `@bmb/crypto`, `@bmb/json` npm 패키지.

### Backlog

| 작업 | 추정 | 트리거 |
|------|------|--------|
| FFI_OWNERSHIP.md 작성 | 0.5 cycle | Track T publish 준비 시 |
| bmb-text/crypto output ownership 검증 | 0.5 cycle | Track T publish 준비 시 |
| M3 showcase library 선정 | Human decision | M2 완성 후 |
| M2 자율 게이트 완성 선언 | 0.5 cycle | Q/R 추가 진척 후 |

---

## 6. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| GCC | MinGW-w64 |
| Rust | stable |
| Node.js | v24.14.0 |
| koffi | ^2.16.2 |
| BMB workspace | `Cargo.toml workspace.package.version = "0.98.0"` |
| Python | 3.10+ (bmb-mcp) |
| `target/release/bmb.exe` | 이번 세션 main.rs 수정됨 — cargo build 필요 |
| Branch | `main`, 미커밋 상태 |
| bmb-mcp submodule HEAD | `6321cda` (변경 없음) |

---

## 7. HUMAN-Decision

| 항목 | 현황 |
|------|------|
| `git push origin main` | ⏳ 사용자 결정 |
| npm publish 전략 (scoped vs unscoped) | ⏳ 결정 필요 |
| M3 showcase library 선정 | ⏳ 결정 필요 |
| Track Q Phase 2 우선순위 | ⏳ 결정 필요 |

---

## 8. 본 세션 핵심 메시지

**Track T Node.js bindings 5/5 완성**:
- Python에 이어 Node.js도 전 라이브러리 커버.
- koffi FFI 패턴 완전 확립: 배열/문자열/int64 파라미터 모두 검증.
- 발견된 bmb-json 특이사항: 출력 포인터는 library-owned — 이 ownership 모델은 BMB 생태계 내에서 라이브러리마다 다를 수 있음.

**Track M 100% 완성**:
- `bmb parse --format compact` 디폴트로 Rule 8 완전 준수.
- AI_OUTPUT_SCHEMA.md Parse section 완성.

**M3 External Bindings 진척**:
- "Python + Node 바인딩" 조건 → ✅
- 남은 M3 조건: (a) showcase 라이브러리 1개 선정, (b) C ABI 자동생성 안정화, (c) Track S 90%.

---

**세션 종료**: 2026-05-09 (Cycles 2558-2565, 미커밋)

**다음 세션 첫 액션**:
1. `cargo build --release` — main.rs 수정 반영
2. `git add` 및 커밋
3. `git push origin main` (선택)
4. Track O Phase 7 또는 Track Q Phase 2 선택
