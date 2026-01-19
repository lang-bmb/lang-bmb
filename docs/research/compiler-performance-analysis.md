# C/Rust 컴파일러가 의도적으로 포기하는 1~10% 성능의 기술적 분석

현대 C/Rust 컴파일러(GCC, Clang/LLVM, rustc)는 안전성, 호환성, 정확성을 보장하기 위해 **이론적 최적 성능 대비 1~10% 수준의 성능을 의도적으로 포기**합니다. 본 연구는 이러한 성능 저하 영역을 정량적으로 분석하고, 각 트레이드오프의 기술적 근거와 극복 방안을 제시합니다. 핵심 발견으로, Rust의 borrow checker는 **런타임 비용이 0**이며, 경계 검사는 CPU 분기 예측 덕분에 실제 비용이 **0.5~3%** 수준에 불과합니다. 반면 정수 오버플로우 검사(-ftrapv)는 최대 **10~500배** 성능 저하를, `-ffast-math` 미사용은 **10~40%** 성능 손실을 야기합니다. PGO와 LTO를 조합하면 대부분의 애플리케이션에서 **20~40%** 성능을 회복할 수 있습니다.

---

## 방어 코드 오버헤드의 실제 비용

### 경계 검사는 생각보다 저렴하다

Rust의 배열/슬라이스 경계 검사(bounds checking)는 메모리 안전성의 핵심이지만, 실제 성능 비용은 예상보다 낮습니다. ACM ASE '22 논문에 따르면 전체 런타임 검사가 **1.96배 오버헤드**를 유발하며, 이 중 경계 검사가 **52.7%**를 차지합니다. 그러나 이는 마이크로벤치마크 결과이며, 실제 프로덕션 환경에서는 양상이 다릅니다.

Readyset 데이터베이스의 실측 연구가 이를 증명합니다. 복잡한 대규모 애플리케이션에서 단일 쿼리당 약 **1,227번**의 경계 검사가 발생했지만, 이를 576개로 줄인 후에도 **측정 가능한 성능 향상이 없었습니다**. 결론적으로 현대 CPU의 분기 예측기가 충분히 우수하여 예측 가능한 bounds check의 실제 비용은 **0에 가깝습니다**. The Rust Performance Book에서도 경계 검사의 일반적인 비용을 **0.5~3%**로 추정합니다.

C++의 `std::vector::at()` vs `operator[]` 비교에서도 최적화 빌드(-O3)에서 성능 차이는 **~5%** 수준이며, 대부분의 워크로드에서 무시할 수 있습니다. 단, 오디오 처리같이 초당 수백만 번 접근하는 경우에만 체감됩니다.

### 정수 오버플로우 검사는 비용이 극단적이다

반면 정수 오버플로우 검사는 극단적인 성능 비용을 유발합니다. Daniel Lemire의 2020년 벤치마크에서 `-ftrapv` 옵션 사용 시:

- **GCC**: 배열 합산에서 **10배 이상** 성능 저하
- **Clang 곱셈 연산**: **537배** 성능 저하 (매우 비효율적 구현)
- **벡터화 비활성화 시**: 약 **43%** 오버헤드

GCC는 오버플로우 검사를 위해 `__addvdi3` 같은 out-of-line 함수 호출을 사용하여 SIMD 벡터화와 인라인 최적화를 완전히 방해합니다. Rust는 이를 해결하기 위해 **debug 모드에서만 오버플로우 패닉**을 발생시키고, release 모드에서는 기본적으로 검사를 비활성화합니다(RFC 560).

### 스택 보호와 PIE의 합리적인 비용

`-fstack-protector-strong`은 커널 코드 크기를 **2.4%** 증가시키고 함수의 **20.5%**를 커버하지만, 런타임 오버헤드는 함수당 몇 개의 추가 명령어(load, compare, conditional jump)로 **1~5%** 수준입니다. Chrome OS는 10개월간 사용 후 문제없음을 보고했습니다.

Position Independent Executable(PIE) 오버헤드는 아키텍처에 따라 크게 다릅니다:

| 아키텍처 | 평균 오버헤드 | 최대 오버헤드 |
|---------|-------------|-------------|
| x86 (32비트) | **10%** | 26% |
| x86_64 (64비트) | **2~3%** | 5% |

x86_64에서 오버헤드가 낮은 이유는 PC-relative 주소 지정 지원과 레지스터 수가 2배(16개)이기 때문입니다.

---

## Rust 메모리 안전성의 진짜 비용

### Borrow checker는 런타임에 아무것도 하지 않는다

**Borrow checker의 런타임 오버헤드는 정확히 0입니다.** 이는 100% 컴파일 타임에 작동하며, 바이너리에는 관련 코드가 전혀 포함되지 않습니다. `rustc_borrowck` crate는 MIR(Mid-level IR) 단계에서 데이터플로우 분석을 수행하며, ownership, borrowing, lifetime 검사는 모두 컴파일 시점에 완료됩니다.

| 기능 | 검사 시점 | 런타임 비용 |
|------|----------|-----------|
| Ownership/Move | 컴파일 타임 | **0** |
| Borrowing rules | 컴파일 타임 | **0** |
| Lifetime | 컴파일 타임 | **0** |

### Reference counting의 측정 가능한 비용

`Rc<T>`와 `Arc<T>`는 런타임 비용이 존재하는 몇 안 되는 Rust 안전성 기능입니다. 실제 벤치마크에서:

| 데이터 규모 | Arc/Mutex | Rc/RefCell | 배율 |
|-----------|-----------|------------|------|
| 1,000 항목 | 70ns/lock | 7ns/lock | **~10x** |
| 1,000,000 항목 | 16ns/lock | 5ns/lock | **~3x** |

작은 데이터셋(캐시 적중)에서 Rc가 Arc보다 약 **10배 빠르며**, 큰 데이터셋에서도 **3배** 이상 빠릅니다. Arc의 추가 비용은 `lock add` 또는 `lock xadd` 같은 atomic 명령어와 CPU 캐시 라인 동기화에서 발생합니다.

흥미롭게도 GNU libstdc++의 `shared_ptr`은 **조건부 atomic** 연산을 수행합니다. 단일 스레드 프로그램에서는 비원자적 연산을 사용하여 Rust Arc보다 빠르지만, `pthread_create`를 참조하는 순간 둘의 성능이 비슷해집니다.

### Zero-cost abstraction이 실제로 작동하는 증거

FLAC 디코더 최적화 사례(ruudvanasseldonk.com)는 Rust의 zero-cost abstraction을 증명합니다. 고수준 iterator 체인 코드가 생성한 어셈블리를 분석한 결과:

- 12개의 `movslq` (로드 + 확장)
- 12개의 `imul` (곱셈)  
- 11개의 `add` (덧셈)
- **모든 iterator 오버헤드 완전 제거**
- **모든 bounds check 제거**

저자의 결론: "All overhead is gone completely... I could not have written this better myself."

단, `RefCell`의 런타임 borrow checking은 예외입니다. `borrow_mut()` 호출은 **12 CPU cycles**가 필요하며, Cell이나 unsafe static mut의 **6 cycles** 대비 고정 오버헤드가 존재합니다.

---

## Pointer aliasing이 최적화를 파괴하는 방식

### 왜 컴파일러는 보수적으로 가정해야 하는가

Pointer aliasing은 두 개 이상의 포인터가 동일한 메모리 위치를 참조할 때 발생합니다. 컴파일러는 별도 컴파일 단위 간 aliasing을 정적으로 분석할 수 없으므로, **안전을 위해 aliasing이 발생할 수 있다고 가정**해야 합니다.

```c
void example(float *a, float *b, float *c, int i) {
    a[i] = a[i] + c[i];
    b[i] = b[i] + c[i];  // c[i]를 다시 로드해야 함
}
```

`a`와 `c`가 동일 메모리를 가리킬 수 있으므로, 컴파일러는 `c[i]`를 **두 번 로드**해야 합니다. 이로 인해 레지스터 캐싱, 루프 벡터화, 명령어 재배치가 모두 불가능해집니다.

### restrict 키워드의 극적인 효과

C99의 `restrict` 키워드는 "이 포인터가 가리키는 객체는 해당 스코프 내에서 오직 이 포인터를 통해서만 접근된다"고 컴파일러에 약속합니다. 벤치마크 결과:

| 테스트 케이스 | restrict 미사용 | restrict 사용 | 성능 향상 |
|-------------|---------------|--------------|----------|
| NVIDIA CPU 예제 | 3.13ms | 1.05ms | **3x (200%)** |
| TI C2000 DSP | 3,618 cycles | 1,209 cycles | **3x (200%)** |
| NVIDIA Kepler GPU | 47.6μs | 22.5μs | **2.1x** |

restrict가 활성화하는 최적화에는 루프 벡터화(SIMD), 레지스터 캐싱, 소프트웨어 파이프라이닝, GPU read-only 캐시 사용이 포함됩니다. Fortran이 C보다 수치 연산에서 빠른 이유 중 하나는 **기본적으로 no-aliasing을 가정**하기 때문입니다.

단, restrict 위반 시 컴파일러가 잘못된 코드를 생성하여 **silent corruption**이 발생할 수 있습니다. CERT C Coding Standard(EXP43-C)는 restrict 오용을 소프트웨어 버그의 주요 원인으로 지목합니다.

### Rust noalias의 험난한 역사

Rust의 `&mut`는 해당 값에 대한 유일한 가변 참조임을 **언어 수준에서 보장**하므로, 컴파일러가 LLVM IR 생성 시 `noalias` attribute를 추가합니다. 그러나 LLVM 버그로 인해 이 최적화는 **5년간 비활성화**되어야 했습니다:

| 시기 | 이벤트 |
|-----|-------|
| 2015-2016 | LLVM이 unwinding과 noalias 조합에서 miscompilation 발생 |
| 2016-02 | `&mut`에서 noalias **비활성화** |
| 2021-03 | LLVM 12에서 **noalias 재활성화** |
| 2021-03-23 | 새로운 miscompilation 버그 발견 (ring 라이브러리) |
| 현재 | LLVM 12+에서 기본 활성화, `!Unpin` 타입은 제외 |

이 사례는 안전성과 최적화 사이의 긴장을 잘 보여줍니다. PR #31545에 따르면 noalias 비활성화로 인한 성능 손실은 "매우 작은" 수준이었지만, 이는 Rust가 포기해야 했던 최적화 기회입니다.

---

## 언어 제약과 컴파일러 보수성의 비용

### Exception handling의 숨겨진 비용

C++의 "zero-cost" 예외는 **try 블록 진입 시에만** zero-cost입니다. 실제 예외 발생 시 stack unwinding, RTTI, side table 검색으로 인해 극단적인 비용이 발생합니다:

| 시나리오 | 시간 (-O2) |
|---------|-----------|
| 예외 throw/catch | **~1,100ns** |
| 에러 코드 반환 | ~11ns |
| Expected<T> 패턴 | ~57ns |

예외는 에러 코드 대비 **~100배 느리며**, Expected<T> 패턴 사용 시 **20배 이상 빠릅니다**. `.eh_frame` 테이블은 일반적으로 바이너리 크기의 **1~2%**를 차지합니다. Rust의 `panic!`도 유사한 stack unwinding 비용이 있지만, `Result<T, E>`는 컴파일 타임 체크로 런타임 오버헤드가 최소입니다.

### Virtual dispatch의 6배 비용

가상 함수 호출은 CRTP(정적 디스패치) 대비 **~6배 느립니다**:

- 가상 함수: **9개 명령어**/호출 (vtable 포인터 역참조, 간접 분기)
- CRTP: **4개 명령어**/호출

Devirtualization은 평균 **0.8%** 성능 향상을 제공하지만, 별도 컴파일 유닛의 동적 타입은 devirtualize가 불가능합니다. Clang Binary-Trees 벤치마크에서는 devirtualization 실패로 GCC 대비 **60%** 느린 결과가 나왔습니다.

### IEEE 754 준수의 10~40% 비용

`-ffast-math` 플래그는 IEEE 754 준수를 포기하고 연산 재배열, FMA 자동 생성, SIMD 벡터화 기회 증가를 활성화합니다. 수치 연산 집중 코드에서 **10~40% 성능 향상**이 가능하며, 일부 코드에서는 **2배 이상** 빨라집니다.

단, `-ffast-math` 사용 시 `isnan()` 체크가 항상 false로 최적화되고, Kahan summation 같은 수치 안정화 기법이 완전히 무효화됩니다. 게임/그래픽에서는 권장되지만, 과학 계산에서는 IEEE 754 준수가 필수입니다.

### Auto-vectorization 실패의 정량적 분석

TSVC 벤치마크에서 LLVM-19와 GCC-14 비교 결과, 151개 루프 중 **~50개**에서 자동 벡터화가 실패했습니다. 주요 실패 원인:

- Loop-carried dependency (반복 간 데이터 의존성)
- Pointer aliasing 가능성
- Non-unit stride access (비연속 메모리 접근)
- 조건문/분기문 존재
- 사이드 이펙트 가능 함수 호출

컴파일러마다 강점이 다릅니다. GCC는 mask 연산 벡터화에서 우수하고(s331 테스트에서 **4.5x** 향상), LLVM은 일부 루프에서 **3배 이상** 성능 차이를 보입니다.

---

## 성능 회복을 위한 극복 방안

### PGO와 LTO의 조합이 핵심이다

Profile-Guided Optimization(PGO)은 런타임 프로파일링 데이터를 사용하여 컴파일러의 최적화 결정을 개선합니다:

| 프로젝트 | PGO 효과 |
|---------|---------|
| Chrome | 14.8% 빠른 새 탭 로드, 16.8% 빠른 시작 |
| Clang 컴파일러 | **~20%** 컴파일 속도 향상 |
| 일반적 범위 | **10~30%** 성능 향상 |

Link-Time Optimization(LTO)은 링크 시점에 전체 프로그램 분석을 수행합니다. ThinLTO는 Full LTO와 유사한 성능 향상(평균 **2.86%**)을 제공하면서 빌드 시간은 비LTO 빌드와 유사합니다. Rust에서 최대 성능을 위한 `Cargo.toml` 설정:

```toml
[profile.release]
opt-level = 3
lto = "fat"           # Full LTO
codegen-units = 1     # 최대 최적화
panic = "abort"       # unwinding 제거
```

### 안전한 unsafe 사용 패턴

`get_unchecked()`를 통한 bounds check 제거는 신중하게 사용해야 합니다:

```rust
fn safe_unchecked_access(arr: &[u8], idx: usize) -> Option<u8> {
    if idx < arr.len() {
        // SAFETY: idx가 arr.len()보다 작음을 확인했으므로 안전
        unsafe { Some(*arr.get_unchecked(idx)) }
    } else {
        None
    }
}
```

Readyset은 37개 인덱싱 연산을 `get_unchecked`로 교체하여 bounds check 횟수를 1,000,000회에서 293회로 줄였지만, **측정 가능한 성능 향상은 없었습니다**. 이는 대부분의 경우 bounds check 비용이 미미함을 재확인합니다. 주요 이점은 auto-vectorization이 가능해지는 경우입니다.

### SIMD의 극적인 효과

수동 SIMD intrinsics는 auto-vectorization 대비 훨씬 높은 성능을 제공합니다. Hex 인코딩 벤치마크에서:

| 접근법 | 처리량 |
|-------|-------|
| Fallback (스칼라) | 612 MB/s |
| SSE4.1 intrinsics | **14,279 MB/s** |
| **개선율** | **~23배** |

Rust의 `std::arch` intrinsics는 런타임 디스패치와 결합하여 안전하게 사용할 수 있습니다:

```rust
fn process(data: &mut [f32]) {
    if is_x86_feature_detected!("avx2") {
        unsafe { process_avx2(data) }
    } else {
        process_fallback(data)
    }
}
```

`portable-simd`(nightly)는 플랫폼 독립적인 SIMD 프로그래밍을 제공하며, SIMD가 없는 플랫폼에서는 자동으로 스칼라 코드로 폴백합니다.

---

## Conclusion: 본질적 한계와 미래 전망

컴파일러가 포기하는 성능은 크게 **회복 가능한 영역**과 **본질적 한계**로 나뉩니다.

**회복 가능한 영역**: PGO+LTO 조합으로 **20~40%** 성능 회복이 가능하며, 수동 SIMD로 특정 루프에서 **수십 배** 향상을 달성할 수 있습니다. Bounds check는 iterator 패턴으로 대부분 제거되고, `-ffast-math`는 정확성이 불필요한 경우 **10~40%** 성능을 제공합니다.

**본질적 한계**: 완전한 alias analysis는 결정 불가능 문제이며, 최적 레지스터 할당과 명령어 스케줄링은 NP-완전/NP-하드 문제입니다. 동적 링크 경계를 넘는 최적화와 완벽한 분기 예측은 런타임 동작에 의존합니다.

**미래 전망**: MLGO(Machine Learning Guided Optimization)가 LLVM에 통합되어 inlining-for-size에서 최대 **7%** 코드 크기 감소를 달성했습니다. Polyhedral optimization은 MLIR을 통해 딥러닝 컴파일러에서 활용되고 있습니다. 그러나 halting problem 관련 한계는 이론적으로 해결 불가능합니다.

핵심 통찰은 **1~10% 성능 손실이 대부분의 애플리케이션에서 합리적인 트레이드오프**라는 점입니다. Rust는 C++와 **5~10% 이내** 성능 차이를 보이면서 메모리 안전성과 동시성 안전성을 컴파일 타임에 보장합니다. 성능 크리티컬 영역에서는 targeted 최적화(PGO, LTO, SIMD, 제한적 unsafe)를 통해 이론적 최적에 근접할 수 있습니다.