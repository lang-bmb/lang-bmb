# Cycle 2904: Java scaffold batch (bmb-json/compute/crypto/text)
Date: 2026-05-17

## Re-plan
Carry-Forward 없음. HANDOFF "Pending Human Decision: Java 바인딩 계속 개발 여부" → Cycle 2899의 bmb-algo scaffold 선례를 바탕으로 나머지 4개 ecosystem 라이브러리 Java scaffold 작성.

## Scope & Implementation

**목표**: bmb-algo (Cycle 2899)와 동일한 JNA 패턴으로 4개 라이브러리 Java scaffold 생성.

**생성 파일** (각 라이브러리: pom.xml + Lib interface + wrapper + test):

### bmb-json (12 @export 함수)
- `BmbJsonLib.java` — raw JNA interface
- `BmbJson.java` — high-level wrapper (validate, stringify, type, count, get, getString, getNumber, getBool, hasKey, objectLen, arrayLen, arrayGet)
- `BmbJsonTest.java` — **25 tests**

### bmb-compute (33 @export 함수)
- `BmbComputeLib.java` — raw JNA interface (Pointer 타입 for i64* 배열 파라미터)
- `BmbCompute.java` — high-level wrapper (scalar 11종 + RNG 4종 + 통계 8종 + 2배열 3종 + 출력배열 7종)
- `BmbComputeTest.java` — **27 tests**

### bmb-crypto (14 @export 함수)
- `BmbCryptoLib.java` — raw JNA interface
- `BmbCrypto.java` — high-level wrapper (sha256/md5/crc32/adler32/fletcher16/xor_checksum/hmac_sha256/base64/base32/hex/rot13)
- `BmbCryptoTest.java` — **15 tests**

### bmb-text (23 @export 함수)
- `BmbTextLib.java` — raw JNA interface
- `BmbText.java` — high-level wrapper (kmpSearch/find/rfind/count/contains/startsWith/endsWith/findByte/countByte/hamming/isPalindrome/tokenCount/wordCount/len/charAt/compare/reverse/replace/replaceAll/toUpper/toLower/trim/repeat)
- `BmbTextTest.java` — **29 tests**

**총 96 tests** (4 라이브러리 합산).

### bmb-compute 설계 특이사항
`bmb-compute`의 `arr: i64` 파라미터는 BMB raw 포인터 인코딩. Java/JNA에서 `Pointer` (JNA `Memory`) 로 매핑. 출력 배열 함수 (`cumsum`, `vecAdd` 등)는 caller-allocated `Memory` 버퍼로 결과를 받아 Java `long[]`로 반환.

## Defect Resolution (STEP 4 발견 → 즉시 수정)

**bmb-json P0 재발 발견**: Cycle 2901 스캔이 `bmb_json_type`만 확인하고 나머지 @export String-반환 함수를 누락.

수정 7곳 (`ecosystem/bmb-json/src/lib.bmb`):
- `bmb_json_get`: 2곳 `""` → `str_repeat("", 1)`
- `bmb_json_get_string`: 3곳 `""` → `str_repeat("", 1)`
- `bmb_json_array_get`: 2곳 `""` → `str_repeat("", 1)`

## Verification
`cargo test --release`: **2388/2388 PASS** ✓

## Reflection
- **Scope fit**: 4개 라이브러리 완성. bmb-algo 패턴 일관성 유지.
- **Latent defects**: Cycle 2901 스캔 방법론 한계 노출 — `@export pub fn -> String` 전수 스캔 시 해당 함수 내부의 모든 `""` 반환 경로를 검토해야 했는데, `bmb_json_type`만 이미 수정됐다는 확인에 그침.
- **bmb-json 스캔 완결**: 이제 bmb-json의 모든 P0 static literal 반환 패치 완료.
- **Philosophy drift**: 없음.
- **Roadmap impact**: M4-6 Java 바인딩 4종 추가. bmb-algo 포함 총 5개 ecosystem 라이브러리 Java JNA scaffold 완성.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - `@export pub fn -> String` 전수 스캔 스크립트 개선 — 함수 내 `""` 경로 추출 (단순 `@export` 라인 인접 확인만으로는 부족)
  - bmb-algo Java scaffold처럼 C# scaffold도 동일 패턴 검토
- Pending Human Decisions:
  - B축 재측정 실행
- Roadmap Revisions: None
- Next Recommendation: Cycle 2905 — `@export -> String` 전수 스캔 자동화 스크립트 개선 또는 조기 종료 평가
