# BMB Binding Roadmap — Dogfooding 기반 언어 완성 전략

> **목표**: 바인딩 라이브러리 개발을 통해 BMB의 한계를 발견하고 해결하여 언어 완성도를 높인다.
> BMB의 한계가 발견되면 **언어/스펙/컴파일러를 개선**하는 것이지, 포기하는 것이 아니다.
>
> **이중 목적**: (1) BMB 언어 성숙 (2) "가장 빠르고 안전한 언어" 존재가치 증명

---

## 현재 상태: 솔직한 평가

### ✅ 작동하는 것 (2026-03-22 업데이트)
- `@export` 어트리뷰트 → 함수 외부 노출 ✅
- `--shared` → .dll/.so 빌드 ✅
- Python ctypes 바인딩 → **5개 라이브러리, 100개 @export, 111 Python 테스트** ✅
- 309 벤치마크 빌드, 16+ FASTER vs C ✅
- **bmb-algo**: 41 algorithms (knapsack 90.7x, nqueens 181.6x vs Pure Python) ✅
- **bmb-crypto**: 11 functions (SHA-256, MD5, CRC32, HMAC, Base64/32, Adler-32) ✅
- **bmb-text**: 20 functions (KMP, find, replace, case, trim, palindrome) ✅
- **bmb-json**: 8 functions (validate, stringify, get, array) ✅
- **bmb-compute**: 20 functions (math, statistics, random, vector) ✅
- **Bootstrap**: @export codegen (dllexport) 지원 추가 ✅

### ✅ 해결된 프로덕션 차단 요소

| 문제 | 심각도 | 해결 상태 | BMB 개선 내용 |
|------|--------|----------|--------------|
| **에러 핸들링: exit(1)** | CRITICAL | ✅ 해결 | `bmb_panic` → setjmp/longjmp + FFI 에러코드 |
| **String FFI 부재** | HIGH | ✅ 해결 | `bmb_ffi_cstr_to_string` / `bmb_ffi_string_data` API |
| **스레드 안전성** | HIGH | ✅ 해결 | TLS 전역 상태 (`__thread` / `__declspec(thread)`) |
| **계약 런타임 검증** | MEDIUM | ✅ 해결 | @export 함수에 pre-condition 런타임 체크 자동 삽입 |
| **심볼 필터링** | MEDIUM | ⚠️ 부분 | 런타임 심볼 충돌 발견 시 이름 변경으로 대응 |

### ❌ 남은 작업
| 문제 | 심각도 | 상태 |
|------|--------|------|
| Bootstrap parser @export | MEDIUM | codegen만 완료, parser/lowering 미완 |
| 크로스 플랫폼 빌드 | MEDIUM | Windows만 검증, Linux/macOS 미검증 |
| PyPI 패키지 배포 | LOW | setup.py 생성, wheel 빌드 미완 |
| 심볼 필터링 (.def) | LOW | 미착수 |

### 핵심 인사이트

> 이 문제들은 **바인딩의 문제가 아니라 BMB 언어의 문제**다.
> 해결하면 BMB 자체가 성숙해진다.

---

## 단기 로드맵 (3개월) — Foundation

### Sprint 1: FFI 안전성 (4-6주)

**목표**: `@export` 함수가 호스트 프로세스를 죽이지 않는다

| 작업 | BMB 개선 | 바인딩 효과 |
|------|---------|------------|
| **1-1. bmb_panic → setjmp/longjmp** | 런타임 에러 핸들링 아키텍처 | 라이브러리가 에러를 반환 |
| **1-2. @export 함수 pre 런타임 체크** | 계약의 런타임 안전망 | 잘못된 입력 → -1 반환 (crash 아님) |
| **1-3. BmbString C API** | 문자열 FFI 레이어 | Python/JS에서 문자열 함수 호출 가능 |
| **1-4. TLS 전역 상태** | 스레드 안전 런타임 | 멀티스레드 호스트에서 안전 사용 |

```
구현 순서:
1-1 → 1-2 (에러가 안전해야 체크를 넣을 수 있다)
1-3 (독립적, 병렬 가능)
1-4 (독립적, 규모가 큼)
```

### Sprint 2: 경쟁력 있는 라이브러리 1개 (4-6주)

**목표**: `pip install bmb-algo` → 실제 사용 가능 → 벤치마크 증명

| 작업 | 상세 |
|------|------|
| **2-1. bmb-algo 프로덕션화** | Sprint 1 결과 적용, 에러 핸들링 + 문자열 FFI |
| **2-2. 벤치마크 자동화** | CI에서 매 커밋 성능 측정 (2% 임계값) |
| **2-3. PyPI 패키지** | setup.py + manylinux wheel + 문서 |
| **2-4. 경쟁 벤치마크** | vs scipy.optimize / vs networkx 비교 |

**완료 기준**:
```python
# 이것이 실제로 작동하고, 경쟁 라이브러리보다 빨라야 함
pip install bmb-algo
import bmb_algo
result = bmb_algo.knapsack(weights=[...], values=[...], capacity=1000)
# Error handling: 잘못된 입력 → ValueError, 프로세스 죽지 않음
```

### Sprint 3: 크로스 플랫폼 (2-4주)

| 작업 | 상세 |
|------|------|
| **3-1. Linux x64** | WSL/Docker 빌드 검증 |
| **3-2. macOS ARM64** | Apple Silicon 지원 |
| **3-3. CI/CD** | GitHub Actions: build + test + benchmark |
| **3-4. manylinux wheel** | PyPI 배포용 사전 빌드 바이너리 |

---

## 중기 로드맵 (6개월) — Growth

### Sprint 4: bmb-crypto (4-6주)

**왜**: 암호학 라이브러리는 성능이 곧 가치이고, BMB의 강점(순수 계산, 제로 오버헤드)이 직접 적용됨

| 함수 | 기존 패키지 | 상태 |
|------|------------|------|
| SHA-256 | bmb-sha256 (372 LOC, 9 tests) | ✅ 완성 |
| MD5 | bmb-md5 (428 LOC) | ✅ 완성 |
| Base64 | bmb-base64 (379 LOC) | ✅ 완성 |
| Base32 | bmb-base32 (437 LOC) | ✅ 완성 |
| CRC32 | bmb-crc32 (145 LOC) | ⚠️ 소규모 |
| HMAC-SHA256 | 미구현 | ❌ 신규 |

```python
pip install bmb-crypto
import bmb_crypto
digest = bmb_crypto.sha256(b"hello world")
encoded = bmb_crypto.base64_encode(b"binary data")
```

**BMB 발견 예상 한계**: 바이트 배열 처리, u8 배열 ↔ i64 변환 효율

### Sprint 5: bmb-json (4-6주)

**왜**: JSON 파싱은 모든 개발자의 일상. stdlib에 이미 파서 있음.

| 작업 | 상세 |
|------|------|
| stdlib json 고도화 | f64 파싱, 유니코드 이스케이프, 스트리밍 |
| Node.js WASM 바인딩 | WASM 백엔드 활용 → npm 패키지 |
| Python 바인딩 | ctypes → .dll |
| 벤치마크 | vs simdjson, vs ujson, vs json (stdlib) |

**BMB 발견 예상 한계**: f64 정밀도, 문자열 이스케이프 성능, WASM 코드 사이즈

### Sprint 6: bmb-text (4-6주)

**왜**: 문자열 처리는 BMB의 입증된 강점 (string_match 0.90x)

| 함수 | 기반 |
|------|------|
| KMP 문자열 검색 | bmb-string-algo (329 LOC) |
| 패턴 매칭 | bmb-memchr (545 LOC) |
| Trie 기반 멀티패턴 | bmb-trie (300 LOC) |
| 토크나이저 | bmb-tokenizer (199 LOC) |

---

## 장기 로드맵 (12개월) — Maturity

### Sprint 7: 제네릭 타입 시스템 (8-12주)

**BMB 언어 수준 변경**: gotgan-packages의 50+ 패키지가 제네릭 부재로 제한됨

| 작업 | 효과 |
|------|------|
| `Vec<T>` | 모든 데이터 구조의 기반 |
| `HashMap<K,V>` | 현재 bmb-hashmap은 String 키만 |
| `Option<T>`, `Result<T,E>` | 타입 안전 에러 핸들링 |

**이것이 해결되면**: gotgan-packages의 절반이 제네릭으로 재작성 가능 → 실질적 라이브러리 생태계

### Sprint 8: Native Ptr 타입 시스템 (6-8주)

**BMB 컴파일러 수준 변경**: inttoptr 5,638개 제거

- 부트스트랩 IR에서 모든 포인터를 native ptr로 전환
- 성능 10-15% 향상 예상 (LLVM alias analysis 개선)
- tree_depth 벤치마크 FAIL → PASS

### Sprint 9: 공개 + 커뮤니티 (4-6주)

| 작업 | 상세 |
|------|------|
| HN/Reddit 발표 | 벤치마크 결과 + bmb-algo 소개 |
| 블로그 3편 | "Why BMB beats C", "Contract-driven optimization", "Building a Python lib in BMB" |
| Good First Issue 20개 | 커뮤니티 기여 유도 |
| v1.0 Go/No-Go | Stars 100+, Contributors 5+ |

---

## 수요 기반 라이브러리 우선순위

```
1. bmb-algo     ← Python DP/그래프 (6.8x faster, 경쟁자 부재)
2. bmb-crypto   ← SHA256/Base64 (성능 = 가치, 패키지 이미 존재)
3. bmb-json     ← 모든 개발자 사용 (stdlib 이미 구현)
4. bmb-text     ← 문자열 처리 (입증된 강점)
5. bmb-compute  ← 수치 계산 (numpy 보완)
```

---

## BMB 언어 개선 트래커

바인딩 개발 과정에서 발견될 BMB 한계와 해결 계획:

| 발견된 한계 | 발견 시점 | 해결 방법 | 우선순위 |
|------------|----------|----------|---------|
| exit(1) panic | Sprint 1 | setjmp/longjmp + 에러코드 | ★★★ |
| String FFI 부재 | Sprint 1 | cstr_to_bmb / bmb_to_cstr API | ★★★ |
| 전역 상태 비안전 | Sprint 1 | TLS 전환 | ★★★ |
| 제네릭 부재 | Sprint 7 | 타입 파라미터 구현 | ★★ |
| inttoptr 과다 | Sprint 8 | native ptr 타입 시스템 | ★★ |
| f64 정밀도 | Sprint 5 | f64 파싱/직렬화 개선 | ★ |
| WASM 코드 사이즈 | Sprint 5 | 데드 코드 제거 최적화 | ★ |

**원칙**: 한계를 발견할 때마다 이 표에 추가하고, BMB 자체를 개선한다.

---

## 성공 지표

### 단기 (3개월)
- [ ] `pip install bmb-algo` 작동
- [ ] Python에서 에러 시 프로세스 안 죽음
- [ ] Linux + Windows + macOS 빌드
- [ ] 벤치마크: bmb-algo > scipy (knapsack)

### 중기 (6개월)
- [ ] `pip install bmb-crypto` 작동
- [ ] `npm install bmb-json` 작동 (WASM)
- [ ] PyPI 다운로드 1,000+/월
- [ ] GitHub Stars 100+

### 장기 (12개월)
- [ ] 제네릭 타입 시스템
- [ ] v1.0 Go/No-Go 게이트 통과
- [ ] 외부 기여자 5+
- [ ] HN frontpage 1회+
