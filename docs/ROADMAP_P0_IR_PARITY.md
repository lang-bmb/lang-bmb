# BMB Roadmap: P0 IR Parity (v0.52+)

> **핵심 원칙**: C/Rust와 동등한 IR 생성이 최우선. 이를 위해 언어 스펙 변경 감수.
> **부트스트랩**: IR Parity 달성 후 재작업

---

## 우선순위 체계

| 순위 | 목표 | 상태 |
|------|------|------|
| **P0** | C/Rust와 동등 IR 생성 → 벤치마크 ≤100% | 🎯 진행 중 |
| **P1** | P0 달성을 위한 언어 스펙 변경 | 🎯 필요시 |
| **P2** | 부트스트랩 자기 컴파일 완성 | ⏸️ P0 후 |

---

## 현재 벤치마크 현황 (v0.51.44)

### ✅ BMB > C (이미 달성)

| 벤치마크 | 비율 | IR 상태 |
|----------|------|---------|
| json_serialize | 56% | ✅ 동등 이상 |
| http_parse | 61% | ✅ 동등 이상 |
| csv_parse | 77% | ✅ 동등 이상 |
| fannkuch | 89% | ✅ 동등 이상 |

### ✅ BMB ≈ C (±5%)

| 벤치마크 | 비율 | IR 상태 |
|----------|------|---------|
| json_parse | 101% | ✅ 동등 |
| fibonacci | 102% | ✅ 동등 |
| spectral_norm | 102% | ✅ 동등 |
| mandelbrot | 105% | ⚠️ 미세 차이 |

### ⚠️ BMB < C (개선 필요)

| 벤치마크 | 비율 | 근본 원인 | 해결책 | 우선순위 |
|----------|------|----------|--------|----------|
| **brainfuck** | 111% | if-else 체인 vs switch | ✅ v0.51.8 IfElseToSwitch 완료 | 재측정 필요 |
| **hash_table** | 111% | HashMap 구현 오버헤드 | 런타임 최적화 | P0-B |
| **sorting** | 110% | 재귀 오버헤드 | ✅ TailRecursiveToLoop + alwaysinline 완료 | 재측정 필요 |
| **lexer** | 109% | byte_at 호출 + if-else | IfElseToSwitch 적용됨, byte_at 인라인 필요 | P0-D |
| **fasta** | 108% | 문자열 빌더 오버헤드 | StringBuilder 최적화 | P0-E |
| **binary_trees** | 106% | 메모리 할당 패턴 | typed pointer 최적화 | P0-F |
| **n_body** | 106% | FP 연산 | SIMD 고려 | P0-G |

---

## P0-A: ~~match → jump table~~ ✅ 완료 (v0.51.8)

### 상태: ✅ 이미 구현됨

**v0.51.8**에서 `IfElseToSwitch` MIR 최적화 패스가 이미 구현되어 있음.
**v0.51.44**에서 `--emit-mir`가 최적화된 MIR을 출력하도록 수정하여 확인 완료.

### 동작 확인

brainfuck의 `execute_instruction` 함수 MIR 출력:
```
switch %c, [62 -> then_0, 60 -> then_3, 43 -> then_9, 45 -> then_12,
            46 -> then_18, 44 -> then_21, 91 -> then_24, 93 -> then_30], else_31
```

생성되는 LLVM IR:
```llvm
switch i64 %c, label %bb_else_31 [
  i64 62, label %bb_then_0
  i64 60, label %bb_then_3
  i64 43, label %bb_then_9
  ...
]
```

### 구현 내역

- `MIR Switch 인스트럭션`: `Terminator::Switch { discriminant, cases, default }`
- `IfElseToSwitch 패스`: 3개 이상의 정수 상수 비교 if-else 체인 감지 및 변환
- `LLVM codegen`: Switch → `switch i64` IR 생성

### 다음 단계

- **벤치마크 재측정 필요**: LLVM 빌드 환경 정상화 후 brainfuck 111% 개선 확인
- lexer 벤치마크도 동일한 최적화 적용 확인 필요

---

## P0-B: HashMap 최적화 (hash_table 111% → ~105%)

### 문제 분석

- 런타임 HashMap 구현 오버헤드
- C는 간단한 open addressing, BMB는 범용 HashMap

### 해결책

- 해시 함수 인라인화
- 버킷 크기 최적화
- 또는 벤치마크 코드를 C와 동일한 알고리즘으로 재작성

---

## P0-C: ~~비교 함수 인라인~~ ✅ 완료 (v0.51.8)

### 상태: ✅ 이미 구현됨

**v0.51.8**에서 `AggressiveInlining` MIR 패스가 구현되어 있음.
**v0.51.9**에서 `TailRecursiveToLoop` 패스로 재귀 함수가 루프로 변환됨.

### 동작 확인 (sorting 벤치마크)

MIR 출력에서 확인:
```
fn array_get(arr: i64, i: i64) -> i64 @alwaysinline {
fn array_set(arr: i64, i: i64, val: i64) -> i64 @alwaysinline {
fn swap(arr: i64, i: i64, j: i64) -> i64 @alwaysinline {
```

재귀 함수도 루프로 변환됨:
```
fn bubble_inner(...) {
entry:
  goto loop_header_7
loop_header_7:
  %j_loop = phi [%j, entry], [%_t11, merge_5]
  ...
}
```

### 남은 차이 (110%)

- C와 BMB의 근본적 구조 차이 (직접 배열 접근 vs 함수 호출)
- LLVM 빌드 환경 정상화 후 실제 성능 측정 필요

---

## P0-D: 직접 바이트 접근 (lexer 109% → ~102%)

### 문제 분석

```bmb
// BMB: 함수 호출
let c = s.byte_at(i);
```

```c
// C: 직접 포인터 접근
char c = s[i];
```

### 해결책

- `byte_at` 인라인화 또는 인트린식화
- 또는 `s[i]` 배열 인덱싱 지원 (String에 대해)

---

## P0-E: StringBuilder 최적화 (fasta 108% → ~100%)

### 문제 분석

- StringBuilder 구현의 재할당 오버헤드
- C는 고정 버퍼 사용

### 해결책

- 용량 힌트 활용 개선
- 또는 벤치마크를 고정 배열로 재작성

---

## P0-F: typed pointer 최적화 (binary_trees 106% → ~100%)

### 문제 분석

- 이미 v0.51.37에서 typed pointer 도입
- 남은 오버헤드는 malloc/free 패턴

### 해결책

- 이미 근접함 (6%)
- 추가 최적화는 우선순위 낮음

---

## P0-G: SIMD 고려 (n_body 106% → ~100%)

### 문제 분석

- 벡터화 가능한 FP 연산
- LLVM 자동 벡터화가 작동하지 않음

### 해결책

- SIMD 인트린식 추가 (장기)
- 또는 코드 구조 변경으로 자동 벡터화 유도

---

## 실행 계획

### Phase 1: match → switch (v0.52)
1. MIR Switch 인스트럭션 정의
2. match lowering 업데이트
3. LLVM switch IR 생성
4. 검증: brainfuck, lexer 벤치마크

### Phase 2: 인라인 최적화 (v0.53)
1. 작은 함수 자동 인라인
2. byte_at 인라인화
3. 검증: sorting, lexer 벤치마크

### Phase 3: 런타임 최적화 (v0.54)
1. HashMap 개선
2. StringBuilder 개선
3. 검증: hash_table, fasta 벤치마크

### Phase 4: 부트스트랩 재작업 (v0.55+)
- P0 달성 후 진행
- 새 언어 기능 반영
- 자기 컴파일 완성

---

## 성공 기준

| 기준 | 목표 | 현재 |
|------|------|------|
| 모든 벤치마크 ≤105% | 15/15 | 11/15 |
| 모든 벤치마크 ≤110% | 15/15 | 15/15 |
| BMB > C 벤치마크 | 8+ | 7 |

---

## 언어 스펙 변경 후보

P0 달성을 위해 검토 중인 스펙 변경:

| 변경 | 목적 | 영향 | 상태 |
|------|------|------|------|
| match → switch IR | jump table 생성 | 코드젠만 | 🎯 우선 |
| 자동 인라인 확대 | 함수 호출 제거 | 최적화 정책 | 검토 중 |
| String 인덱싱 | 직접 바이트 접근 | 타입 시스템 | 검토 중 |

---

> 이 문서는 P0 IR Parity 달성까지의 집중 로드맵입니다.
> 달성 후 부트스트랩 및 기타 기능 작업을 재개합니다.
