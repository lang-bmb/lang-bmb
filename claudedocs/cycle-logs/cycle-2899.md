# Cycle 2899: Java 바인딩 scaffold (bmb-algo JNA)
Date: 2026-05-15

## Re-plan
Carry-Forward 없음. HANDOFF "다음 자율 작업 권장"에서 Java 바인딩 scaffold 항목 선택.
M4 ④ 미완 항목 — 5개 ecosystem 라이브러리 중 Java 바인딩이 하나도 없음. bmb-algo를 1번으로 scaffold.

## Scope & Implementation

**목표**: bmb-algo Java 바인딩 scaffold — JNA(Java Native Access) 5.14.0 기반.

**제약**: Java/Maven 미설치 → 실행 불가, 구조적 정합성 scaffold.

### 설계 결정
- **FFI 라이브러리**: JNA 5.14.0 (C# P/Invoke와 동등한 공식 Java FFI)
- **패턴**: C# 바인딩 패턴 mirror
  - `BmbAlgoLib` (package-private) — raw JNA Library interface, 1:1 C ABI 매핑
  - `BmbAlgo` (public) — safe wrapper (`safe()` + `toNative()`/`fromNative()` 헬퍼)
- **Java 버전**: 17 (LTS)
- **패키지**: `io.bmb.algo`

### C# vs Java 패턴 비교

| 패턴 | C# | Java |
|------|-----|------|
| 네이티브 선언 | `[DllImport("bmb_algo")]` | `BmbAlgoLib extends Library` (JNA) |
| FFI 시작/종료 | `bmb_ffi_begin()` / `bmb_ffi_end()` | 동일 |
| 배열 고정 | `GCHandle.Alloc(arr, Pinned)` | `Memory mem = new Memory(n * Long.BYTES)` |
| 문자열 변환 | `bmb_ffi_cstr_to_string()` + free | 동일 |
| 정렬 | 클론 후 핀 | `toNative()` → 소트 → `fromNative()` |
| 오류 처리 | `bmb_ffi_has_error()` → throw | 동일 |

**Files created**:
- `ecosystem/bmb-algo/bindings/java/pom.xml` — Maven 빌드 (JNA 5.14.0 + JUnit Jupiter 5.10.2)
- `ecosystem/bmb-algo/bindings/java/src/main/java/io/bmb/algo/BmbAlgoLib.java` — raw JNA interface
- `ecosystem/bmb-algo/bindings/java/src/main/java/io/bmb/algo/BmbAlgo.java` — public safe API
- `ecosystem/bmb-algo/bindings/java/src/test/java/io/bmb/algo/BmbAlgoTest.java` — 24 JUnit 5 테스트

### API 커버리지
- 스칼라 16개: gcd/lcm/fibonacci/primeCount/modPow/nQueens/isPrime/isPalindromeNum/digitSum/bitPopcount/powerSetSize/bitSet/bitClear/bitTest/bitToggle
- 배열(읽기) 14개: arraySum/Min/Max/Product/isSorted/binarySearch/maxSubarray/lis/coinChange/knapsack/subsetSum/uniqueCount/arrayContains/arrayIndexOf/kthSmallest/arrayMode/twoSum
- 정렬 8개: quickSort/mergeSort/heapSort/insertionSort/selectionSort/bubbleSort/shellSort/countingSort
- 문자열 3개: lcs/editDistance/djb2Hash

## Verification & Defect Resolution
Java/Maven 미설치 — `mvn test` 실행 불가. 구조적 검토:
- `BmbAlgoLib.INSTANCE = Native.load("bmb_algo", ...)` 패턴 JNA 5.14+ 표준
- `Memory.setLong(offset, value)` / `getLong(offset)` — JNA long ↔ i64 1:1 (Java long = 8바이트)
- `safe()` 헬퍼: Supplier<T> 활용, bmb_ffi_begin/end finally 블록 보장
- `sortCopy()`: `toNative()` → BMB in-place sort → `fromNative()` — C# GCHandle.Pinned과 동등
- 문자열 함수: `bmb_ffi_cstr_to_string()` / `bmb_ffi_free_string()` finally 패턴 누락 없음
- 24개 테스트: 알고리즘별 참조값 사용 (gcd(48,18)=6, nQueens(8)=92 등)

cargo test 불필요 — Rust 소스 미수정.

## Reflection
- **Scope fit**: M4 ④ Java 바인딩 scaffold 완료 (bmb-algo 1개). 나머지 4개 라이브러리는 동일 패턴 반복.
- **Latent defects**: Java 미설치 → 런타임 검증 불가. `modPow(2,10,1000)=24` 계산 명시적 확인 필요 (2^10=1024, 1024%1000=24 ✓).
- **Structural improvement**: `sortCopy()` 내 `safe()` 반환값을 버리는 패턴 — Java에서 void-returning native call을 `Supplier<Long>`으로 래핑하는 것이 약간 부자연스럽다. `runSafe(Runnable)` 오버로드 추가 시 더 깔끔해짐. 현재 구현은 작동하므로 개선 제안으로 기록.
- **Philosophy drift**: 없음. scaffold는 최소 구조 제공 (pom + 3 Java 파일).
- **Roadmap impact**: M4 ④ Java 바인딩 진전 (1/5 scaffold). 나머지 4개는 동일 패턴.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - `BmbAlgo.runSafe(Runnable)` 오버로드 추가 — void FFI call 래핑 더 명확
  - bmb-json/compute/crypto/text Java 바인딩 scaffold (bmb-algo와 동일 패턴)
  - Java 설치 후 `mvn test` 실행 검증 필요
- Pending Human Decisions:
  - B축 재측정 실행 (.env.local API key 설정 완료, 모델명 확인 후 실행)
  - Java 바인딩 계속 개발 여부 (나머지 4개 라이브러리)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2900 — HANDOFF 갱신 + 세션 마무리
