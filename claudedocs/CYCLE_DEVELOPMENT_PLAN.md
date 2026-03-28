# BMB 반복 사이클 개발 계획

> **목적**: 언어 개발/완성도 검증. C/Rust 추월 성능 유지. 한계 발견 시 언어 스펙/컴파일러 변경 포함.
> **기준일**: 2026-03-29 | **현재 버전**: v0.97.2

---

## 1. 현재 상태 — 정직한 평가

### 1.1 성능 현황

```
벤치마크 38개 중:
  BMB > C (FASTER):  13개 (34%)   ← 증명됨
  BMB ≈ C (OK):       8개 (21%)   ← 허용 범위
  BMB < C (SLOWER):  11개 (29%)   ← 해결 필요
  BMB FAIL:           1개 (3%)    ← brainfuck 실행 실패
  기타:               5개 (13%)
```

### 1.2 계약→성능 파이프라인

| 항목 | 상태 | 증거 |
|------|------|------|
| EXISTENTIAL 7/7 | ✅ 달성 | llvm.assume, check elimination, 43%/32% gap |
| purity_opt | ✅ 2.88x FASTER | Phase Ordering: MIR CSE → LLVM inline |
| 실세계 적용 범위 | ⚠️ 좁음 | 재귀 순수함수 + 같은 파일에서만 발현 |
| 별도 컴파일 단위 | ❌ 미증명 | LTO-free 시나리오 미테스트 |

### 1.3 언어 완성도

| 카테고리 | 구현 | 미구현 | 영향 |
|----------|------|--------|------|
| 타입 시스템 | i64, f64, bool, String, *T, &T | **제네릭 타입 인스턴스화** | 모든 컬렉션, Option<T>, Result<T,E> 불가 |
| 데이터 구조 | 고정 배열 [T; N], 구조체, 열거형 | **Vec<T>, HashMap<K,V>** | 동적 자료구조 불가 |
| 메모리 모델 | 스택, malloc/free, *T 포인터 | **제네릭 소유권, 수명** | 안전한 동적 메모리 불가 |
| 제어 흐름 | if, while, for, match, return | **for-in 이터레이터, async/await** | 컬렉션 순회 패턴 부재 |
| 다형성 | 구조체 메서드, 트레이트(파싱) | **dyn Trait, 트레이트 객체** | 런타임 다형성 불가 |
| 매크로 | 없음 | **전부** | 코드 생성/DSL 불가 |

### 1.4 에코시스템

| 항목 | 수치 | 실질 |
|------|------|------|
| 패키지 수 | 103 | **~15개만 실제 코드 (15%)** |
| stdlib 함수 | 384 | 모두 단형적 (i64 전용) |
| Python 바인딩 | 5개 DLL | 동작 확인됨 |
| IDE | VS Code + LSP | 기본 동작 (hover/completion) |
| CI/CD | action-bmb | 3 플랫폼 |

---

## 2. 핵심 문제 정의

### 문제 1: 성능 SLOWER 벤치마크 (11개)

BMB의 존재 이유가 성능이다. SLOWER 벤치마크는 존재해서는 안 된다.

| 벤치마크 | 비율 | 원인 분석 | 필요한 변경 수준 |
|----------|------|----------|-----------------|
| fannkuch 2.12x | 미분석 | 알고리즘/codegen 분석 필요 | 최적화 패스 또는 codegen |
| fibonacci 1.44x | 미분석 | TCO 미동작? | 최적화 패스 |
| http_parse 2.29x | 미분석 | 문자열 처리 오버헤드? | codegen 또는 런타임 |
| json_serialize 1.46x | 미분석 | 문자열 연결 비용? | 런타임 또는 언어 스펙 |
| simd_sum 1.50x | 미분석 | SIMD 벡터화 미지원 | codegen |
| mandelbrot 1.20x | 미분석 | f64 연산 패턴? | codegen |
| fasta 1.20x | 미분석 | 출력 버퍼링? | 런타임 |
| pointer_chase 1.20x | 미분석 | inttoptr? | codegen |
| memory_copy 1.25x | 미분석 | memcpy intrinsic? | codegen |
| binary_trees 1.09x | 미분석 | 동적 할당 패턴? | 언어 스펙 |
| purity_opt 1.25x | 분석완료 | 벤치마크 러너의 측정값 (ad-hoc 2.88x와 상이) | 벤치마크 방법론 |

### 문제 2: 제네릭 부재

모든 표준 라이브러리와 패키지가 단형적. `Vec<T>`, `Option<T>`, `Result<T,E>`, `HashMap<K,V>` 불가능.
- **영향**: 102개 패키지 중 ~30%가 제네릭 부재로 제한됨
- **영향**: 실세계 프로그램 작성 불가 (사실상 알고리즘 벤치마크 전용 언어)

### 문제 3: 계약 우위 범위 협소

@pure CSE 우위가 "재귀 순수함수 + 같은 파일"에서만 발현.
- 현대 C 컴파일러도 noinline 함수의 순수성 분석 가능
- 별도 컴파일 단위(LTO-free)에서의 우위는 미증명

---

## 3. 개발 사이클 계획

### Phase 1: SLOWER 벤치마크 전수 분석 + 수정 (Cycles 161-180)

> **목표**: SLOWER 11개 → 0개. 모든 벤치마크에서 BMB ≥ C.
> **원칙**: 원인이 codegen이면 codegen 수정. 언어 스펙이면 스펙 변경. Workaround 금지.

```
Cycle 161-163: SLOWER 11개 벤치마크 전수 IR 비교 분석
  - 각 벤치마크: BMB IR vs C IR 비교
  - inttoptr 수, 벡터화 비율, 명령어 수, 루프 구조 비교
  - 원인 분류: codegen / 최적화 패스 / 런타임 / 언어 스펙

Cycle 164-168: 원인별 수정 (codegen + 최적화 패스)
  - inttoptr 제거 (codegen)
  - 벡터화 패턴 개선 (MIR→IR)
  - memcpy/memmove intrinsic 추가 (codegen)
  - SIMD 힌트 생성 (codegen)

Cycle 169-172: 원인별 수정 (런타임 + 언어 스펙)
  - 문자열 처리 최적화 (StringBuilder 개선)
  - 출력 버퍼링 (fwrite batch)
  - 필요시 언어 구문 추가

Cycle 173-175: 재측정 + 검증
  - 전체 38개 벤치마크 재측정
  - SLOWER → OK 또는 FASTER 확인
  - 회귀 확인 (기존 FASTER가 유지되는지)

Cycle 176-180: brainfuck FAIL 수정 + 벤치마크 스위트 강화
  - brainfuck 실행 실패 분석 + 수정
  - 새 벤치마크 추가 (계약 가치 시나리오)
  - 벤치마크 공정성 감사
```

**Phase 1 완료 기준**:
- SLOWER 벤치마크: 0개
- FAIL 벤치마크: 0개
- 전체: FASTER 또는 OK만 존재
- 테스트: 전체 통과, 회귀 0건

---

### Phase 2: 별도 컴파일 단위 계약 최적화 증명 (Cycles 181-195)

> **목표**: LTO-free 시나리오에서 @pure/@noinline의 절대적 우위 증명.
> **원칙**: 실세계 시나리오 (라이브러리 호출)에서 계약이 성능에 기여함을 측정으로 증명.

```
Cycle 181-185: 별도 .o 파일 벤치마크 인프라
  - BMB: @pure @noinline 함수를 별도 .bmb 파일로 분리
  - C: 동일 함수를 별도 .c 파일로 분리 (LTO 비활성)
  - 빌드: bmb build lib.bmb + bmb build main.bmb --link lib.o
  - 측정: BMB (memory(none) CSE) vs C (body 볼 수 없어 CSE 불가)

Cycle 186-190: 계약 최적화 시나리오 확대
  - bounds_check: 별도 컴파일 단위에서 사전조건 전파
  - divzero_check: 별도 컴파일 단위에서 사전조건 전파
  - purity: 별도 컴파일 단위에서 CSE/LICM
  - noalias: 소유권 계약에서 포인터 비별칭 전파

Cycle 191-195: 결과 문서화 + EXISTENTIAL 확장
  - 별도 컴파일 단위 벤치마크 결과 정리
  - "계약이 성능에 기여한 벤치마크" 목록 업데이트
  - ROADMAP에 결과 반영
```

**Phase 2 완료 기준**:
- 별도 컴파일 단위에서 BMB > C 증명 (최소 3개 시나리오)
- "계약→성능 벤치마크" 수: 현재 1개 → 최소 4개
- 모든 결과에 출력 동일성 + 측정 방법론 문서화

---

### Phase 3: 제네릭 타입 시스템 기반 구축 (Cycles 196-220)

> **목표**: `Vec<T>`, `Option<T>`, `Result<T,E>` 동작.
> **원칙**: 성능 저하 없는 제네릭 (단형화). Workaround 금지.
> **수준**: 언어 스펙 변경 (Decision Framework Level 1)

```
Cycle 196-200: 제네릭 인스턴스화 설계 + 프로토타입
  - 현재: 제네릭 파싱 됨 (skip_generic_params), 인스턴스화 없음
  - 목표: fn foo<T>(x: T) → foo_i64(x: i64), foo_f64(x: f64) 단형화
  - 설계: 타입 파라미터 → 구체 타입 매핑 → 함수/구조체 복제
  - Rust 컴파일러(types/) 수정 필요

Cycle 201-205: 제네릭 함수 단형화 구현
  - types/generics.rs: 제네릭 함수 인스턴스화
  - MIR 레벨에서 타입 파라미터 치환
  - 테스트: fn id<T>(x: T) -> T = x; → id_i64, id_f64 생성

Cycle 206-210: 제네릭 구조체 단형화 구현
  - struct Vec<T> { data: *T, len: i64, cap: i64 }
  - → Vec_i64 { data: *i64, len: i64, cap: i64 }
  - 메서드: impl Vec<T> → impl Vec_i64

Cycle 211-215: Option<T>, Result<T,E> 구현
  - stdlib core/option → 제네릭 Option<T>
  - stdlib core/result → 제네릭 Result<T, E>
  - 패턴 매칭: match opt { Some(x) => ..., None => ... }

Cycle 216-220: Vec<T> 구현 + 벤치마크
  - 힙 할당 + 동적 크기 조정 (grow/shrink)
  - push, pop, get, set, len, iter
  - binary_trees 벤치마크 Vec<T>로 재작성
  - 성능: C의 동적 배열과 동등 (단형화 → 제로 오버헤드)
```

**Phase 3 완료 기준**:
- `fn foo<T>(x: T) -> T` 동작 (함수 제네릭)
- `struct Pair<A, B>` 동작 (구조체 제네릭)
- `Option<T>`, `Result<T, E>` stdlib에서 사용 가능
- `Vec<T>` 기본 동작 (push, pop, get, len)
- binary_trees 벤치마크 동작 + C 동등 성능
- 단형화로 인한 런타임 오버헤드: 0

---

### Phase 4: 에코시스템 실질화 + v0.98 (Cycles 221-240)

> **목표**: 스텁 패키지 → 실제 코드. 에코시스템 완성도 15% → 50%.
> **원칙**: 제네릭 기반 패키지만 인정. 단형적 스텁 정리.

```
Cycle 221-225: stdlib 제네릭 전환
  - core/option: Option<T> 전환
  - core/result: Result<T, E> 전환
  - collections: Vec<T> 추가
  - array: 제네릭 배열 함수

Cycle 226-230: 핵심 패키지 실질화
  - bmb-hashmap → HashMap<K, V> (해시 + 제네릭)
  - bmb-algorithms → 제네릭 정렬/검색
  - bmb-collections → Vec<T>, Deque<T>, Stack<T>
  - 각 패키지 최소 200 LOC + 테스트

Cycle 231-235: 벤치마크 확장
  - binary_trees: Vec<T> 사용
  - k-nucleotide: HashMap<String, i64> 사용
  - reverse-complement: Vec<u8> 사용
  - 이전 FAIL/BLOCKED 벤치마크 실행 가능

Cycle 236-240: v0.98 릴리즈 준비
  - 전체 벤치마크 재측정
  - ROADMAP v0.98 업데이트
  - 버전 범프
  - 릴리즈 노트
```

**Phase 4 완료 기준**:
- stdlib에서 Option<T>, Result<T,E>, Vec<T> 사용 가능
- 실질 패키지: 15개 → 30개 이상
- FAIL/BLOCKED 벤치마크: 0개
- 전체 벤치마크: SLOWER 0개, FASTER 50%+

---

## 4. 성공 판정 기준

### Phase 1 (Cycles 161-180) — 성능 완전성
```
✅ SLOWER 벤치마크: 0개
✅ FAIL 벤치마크: 0개
✅ 전체 테스트 통과
✅ 기존 FASTER 유지 (회귀 0건)
```

### Phase 2 (Cycles 181-195) — 계약 우위 실증
```
✅ 별도 컴파일 단위에서 BMB > C: 3+ 시나리오
✅ "계약→성능 벤치마크": 4+ 개
✅ 측정 방법론 문서화
```

### Phase 3 (Cycles 196-220) — 제네릭 완성
```
✅ fn<T>, struct<T>, impl<T> 동작
✅ Option<T>, Result<T,E>, Vec<T> 구현
✅ binary_trees 벤치마크 실행 + C 동등 성능
✅ 단형화 오버헤드: 0
```

### Phase 4 (Cycles 221-240) — 에코시스템 실질화
```
✅ 실질 패키지: 30+ 개
✅ BLOCKED 벤치마크: 0개
✅ 전체 SLOWER: 0개, FASTER: 50%+
```

---

## 5. Decision Framework 적용 원칙

문제 발견 시 반드시 **위에서부터** 검토:

| 순위 | 수준 | 해당 Phase |
|------|------|-----------|
| 1 | **언어 스펙** | Phase 3 (제네릭), Phase 1 (필요시 구문 추가) |
| 2 | **컴파일러 구조** | Phase 3 (타입 시스템), Phase 1 (MIR 구조 변경) |
| 3 | **최적화 패스** | Phase 1 (벡터화, CSE), Phase 2 (계약 전파) |
| 4 | **코드 생성** | Phase 1 (inttoptr, SIMD, memcpy) |
| 5 | **런타임** | Phase 1 (문자열, 출력 버퍼링) |

**낮은 수준에서 해결하려는 유혹을 경계하라.**
성능 문제가 Level 4에서 해결되면 그것은 workaround일 수 있다. Level 1에서 해결해야 하는지 먼저 검토.

---

## 6. 리스크

| 리스크 | 확률 | 영향 | 대응 |
|--------|------|------|------|
| 제네릭 단형화가 컴파일 시간 폭발 | 중 | 고 | 지연 인스턴스화, 캐싱 |
| 제네릭이 부트스트랩 깨뜨림 | 고 | 고 | 3-Stage 검증 매 사이클 |
| SLOWER 벤치마크 일부 LLVM 한계 | 중 | 중 | IR 분석 후 LLVM 한계 증명 시 OK 판정 |
| Vec<T> 구현이 GC 필요 | 저 | 고 | Rust 스타일 소유권 (이미 설계됨) |
| 기존 FASTER 벤치마크 회귀 | 중 | 고 | 매 Phase 끝 전수 재측정 |

---

## 7. 타임라인 (추정)

```
Phase 1: 20 cycles  ← SLOWER 제거 (codegen/최적화 중심)
Phase 2: 15 cycles  ← 계약 우위 증명 (벤치마크/인프라)
Phase 3: 25 cycles  ← 제네릭 (언어 스펙 변경, 가장 대규모)
Phase 4: 20 cycles  ← 에코시스템 (패키지 실질화)
──────────────────
Total:   80 cycles  → v0.98 릴리즈
```

---

## 8. 즉시 실행 가능한 첫 번째 사이클 (Cycle 161)

```
목표: SLOWER 11개 벤치마크 전수 IR 비교 분석
방법:
  1. 각 SLOWER 벤치마크 빌드 (BMB + C Clang)
  2. BMB IR vs C IR 비교: inttoptr, 벡터화, 명령어 수
  3. 원인 분류 테이블 작성
  4. 수정 우선순위 결정

산출물: SLOWER 원인 분석 테이블 (벤치마크 × 원인 × 필요한 변경 수준)
```
