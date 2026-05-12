# ISSUE-20260413 — HashMap 해시 함수 최적화

**우선순위**: **P3** (Cycle 2767 재측정: 1.020x, 노이즈 범위 — 갭 자체가 measurement noise였음)
**영역**: stdlib, runtime
**상태**: **Closed** (Cycle 2768) — 실측 1.020x ≈ parity, P-track 기준 1.05x 내부. cycle 2750 1.040x = measurement noise (advisor 가설 우월). compiler fix ROI 부정 (5-7 cycles → 0.2pp).

## 측정 stamp (Cycle 2750 갱신)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-11 (tier_all_c2729 5-run) |
| `stale_after` | 2026-08-11 (3개월) |
| `measurement_source` | `target/benchmarks/tier_all_2026_05_11_c2729.json` (5-run median) |
| `observed_rate` | **1.040x BMB slower** vs C hash_table (4% 갭) |
| `scope` | tier 1 hash_table 벤치마크 |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / gcc MinGW |

**측정 추이**:

| date | source | observed | 변화 |
|------|--------|----------|------|
| 2026-05-11 | tier_all_c2729.json (Cycle 2750) | **1.040x** | +1.2 pp (노이즈 범위) |
| 2026-05-02 | v098-historic.json | 1.027x | -8.3 pp |
| 2026-04-13 | (구) tier 1 bench | 1.110x | (baseline) |

## 문제

BMB `HashMap<K, V>` 구현이 C의 hash_table 구현 대비 11% 느림. 해시 함수 선택, 컬리전 해결, 메모리 레이아웃이 원인 후보.

## 근본 원인 분석 (Cycle 2766 진단 완료)

### Diagnosis 1 — bench와 stdlib 분리

`ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` 는 **자체 구현** (stdlib `core::hashmap` 미사용):
- `hash_i64`, `hm_insert`, `hm_get`, `hm_remove` 모두 bench 내부 정의
- `*i64` raw pointer 기반 manual layout (3 i64 words per entry)
- 같은 hash function (Fibonacci hash with constant `0x517cc1b727220a95`)
- 같은 capacity (131072), 같은 open-addressing + linear probing

→ **알고리즘 차이 없음**. C `static inline int64_t hashmap_get(...)` 과 BMB `fn hm_get(...) = ...` 모두 동일 로직.

### Diagnosis 2 — IR-level smoking gun

BMB IR (`--emit-ir`):
```
define private noundef i64 @hm_insert(ptr noundef %m, ...) inlinehint nosync ...
define private noundef i64 @hm_get(ptr noundef %m, ...) noinline nosync ... memory(read) ...
define private noundef i64 @hm_remove(ptr noundef %m, ...) inlinehint nosync ...
```

**`hm_get`만 `noinline` + `memory(read)`**. 다른 두 함수는 `inlinehint`.

### Diagnosis 3 — 원인 코드

`bmb/src/mir/optimize.rs:6874` `should_no_inline_for_licm`:
- Cycle 2532에서 추가 (`v0.99 MemoryEffectAnalysis-driven noinline pass`)
- 조건: read_only + 비-memory_free + 비-alwaysinline + 비-recursive + ≥10 MIR 명령 + in-loop callee
- 목적: read-only fns에 `noinline + memory(read)` 부착 → LLVM LICM이 invariant call hoist 가능

**의도된 이득**: `json_parse`의 `count_array`/`validate_json` (loop-invariant 입력, 동일 결과) → LICM hoist → 1.12x → 1.0x 회복 (Cycle 2532)

**의도되지 않은 손실**: `hm_get`도 read-only + in-loop + ≥10 insts → noinline 적용. 그러나 `benchmark_lookup` hot loop에서 매 iteration `key` 인자가 변동 → LICM hoist 불가 → **순수 호출 overhead 손실**.

### Diagnosis 4 — Rule 6 영향

- Rust compiler `bmb/src/mir/optimize.rs` 에 pass 존재
- `bootstrap/compiler.bmb` 에 동일 pass **없음** (`grep noinline bootstrap/compiler.bmb` 결과 0)
- Rule 7 (코드젠 백엔드 parity) 위반 가능성 — 동일 BMB 소스가 Rust vs bootstrap 컴파일러에서 다른 IR
- Rule 6 (Rust 동결): 새 기능은 bootstrap 추가. 이 pass는 Rust 전용 잔존 기능, bootstrap port 필요

### Cycle 2767 측정 결과 (가설 검증)

#### 분기 ① bootstrap-built 측정 — **불가**

`D:/data/lang-bmb/bootstrap/compiler.exe` 로 hash_table bench 빌드 시도 → **STATUS_STACK_OVERFLOW** (returncode 3221225725 = 0xC00000FD). 부트스트랩 파서가 deep AST nesting을 처리 못함 (250 LOC bench, deep nested `let _s1 ... let _s2 ...`).

→ 별도 carry-forward: bootstrap parser stack overflow는 separate ISSUE.

#### A/B test — @inline annotation으로 우회 검증

Rust compiler로 두 변종 빌드 + 직접 측정 (10-run min):

| 변종 | min | median | p25-p75 | C 대비 ratio (median) |
|------|-----|--------|---------|-----------|
| BMB orig (`hm_get` 자동 noinline) | 81.5 | 82.2 | 81.7-82.3 | **1.020x** |
| BMB `@inline` on `hm_get` | 80.5 | 82.1 | 81.5-82.4 | **1.018x** |
| C clang -O3 | 79.8 | 80.6 | 80.0-81.4 | 1.000x baseline |

@inline 효과: median 0.1ms 개선 (0.1%), p25 0.2ms 개선, p75 동일. **측정 노이즈 내부**.

#### 결론

- **갭 1.040x → 1.020x 정정**: Cycle 2750 1.040x = noise. 실측 = 1.018-1.020x (≈ parity)
- **noinline pass 영향 미미**: 0.2pp 미만 (advisor 가설 우월: "갭 자체가 measurement noise일 가능성")
- **bootstrap port ROI 부정적**: 5-7 cycles compiler work → 0.2pp 개선 — ROI 명백히 부정
- **C side comparison fairness**: C `hashmap_get/insert/remove`는 `inline` 표기 없으나 `static` 으로 clang 자동 inline. BMB @inline은 explicit annotation → BMB에 @inline 추가 시 비대칭 ("BMB가 C에 없는 hint 사용") — **bench 변경 회피** (Principle 2 정합)

#### Status

- 우선순위 P1 → **P3** 강등
- close 후보 (실측 갭 < P-track 기준 1.05x)
- carry-forward: 부트스트랩 parser stack overflow 별도 ISSUE

## 해결 방안

### 1단계: 프로파일링
- C 버전과 BMB 버전의 LLVM IR 비교 (`--emit-ir`)
- `perf stat` 또는 Windows VTune로 캐시 미스/분기 실패율 측정

### 2단계: 해시 함수 교체
- **FxHash** (Rust 컴파일러 채택): 정수 키에 매우 빠름, 암호학적 안전성 불필요
- **SipHash-1-3**: DoS 저항성 필요 시 (외부 입력)
- 선택 기준: 벤치마크 워크로드(정수 키)는 FxHash 우선

### 3단계: 오픈 어드레싱 + Robin Hood
- Rust `hashbrown`/Swisstable 영감
- 캐시 친화적 linear probing + Robin Hood hashing
- SIMD probing (AVX2) — 단, LLVM 자동 벡터화 의존

### 4단계: 제네릭 Specialization
- `HashMap<i64, V>` 같은 기본 타입에 대한 fast-path
- monomorphization 시 해시 함수 인라인

## 구현 단계

1. [ ] 현재 HashMap 소스 분석: `stdlib/collections/*.bmb` 또는 내장
2. [ ] 해시 함수 벤치마크 (FNV vs FxHash vs Identity)
3. [ ] 오픈 어드레싱 재구현
4. [ ] `hash_table` 벤치마크 IR 수준 비교
5. [ ] 회귀 테스트: 기존 HashMap 사용 골든 테스트 전체 통과

## 완료 기준

- `hash_table` 벤치마크 **BMB ≤ C** (목표 ≤ 100%)
- 기타 HashMap 사용 벤치마크 회귀 없음
- 메모리 사용량 증가 ≤ 10%
