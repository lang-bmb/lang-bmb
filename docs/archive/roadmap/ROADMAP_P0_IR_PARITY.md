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
| **hash_table** | 111% | HashMap 구현 오버헤드 | ✅ Pure BMB HashMap 구현 완료 | P0-B 재측정 필요 |
| **sorting** | 110% | 재귀 오버헤드 | ✅ TailRecursiveToLoop + alwaysinline 완료 | 재측정 필요 |
| **lexer** | 109% | byte_at 호출 + if-else | IfElseToSwitch 적용됨, byte_at 인라인 필요 | P0-D |
| **fasta** | 108% | 문자열 빌더 오버헤드 | ✅ sb_with_capacity 구현 완료 | P0-E 재측정 필요 |
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

## P0-B: HashMap 최적화 (hash_table 111% → ~105%) ✅ 구현 완료

### 문제 분석

**C 구현:**
```c
typedef struct Entry { int64_t key, value, state; } Entry;  // 24바이트
Entry* table = calloc(TABLE_SIZE, sizeof(Entry));            // 직접 포인터 연산
uint64_t hash = key * 0x517cc1b727220a95ULL;                // 인라인 해시
table[idx].value = val;                                      // 직접 메모리 접근
```

**이전 BMB 구현:**
```
%m = call hm_new()              // 런타임 HashMap 생성
%_r = call hm_insert(%m, %k, %v)  // 함수 호출 (인라인 불가)
%v = call hm_get(%m, %k)        // 함수 호출 (인라인 불가)
```

**근본 원인:**
1. **런타임 HashMap 외부 호출**: 모든 연산이 C 런타임으로 가서 LLVM이 인라인 불가
2. **범용 구현**: BMB HashMap은 다양한 타입 지원, C는 벤치마크 전용

### 해결책: Pure BMB HashMap (v0.51.45)

BMB 프리미티브만으로 HashMap 구현:
- `load_i64`, `store_i64`: 직접 메모리 접근
- `calloc`, `free`: 메모리 할당
- `band`, `bor`, `bxor`, `>>`: 해시 및 마스킹

**새 BMB 구현:**
```bmb
fn hash_i64(key: i64) -> i64 = {
    let h = key * 5871781006564002453;  // 0x517cc1b727220a95
    h bxor (h >> 32)
};

fn hm_insert_loop(m: i64, key: i64, value: i64, idx: i64, mask: i64) -> i64 =
    if idx > mask { 0 }
    else {
        let e = entry_ptr(m, idx);
        let state = entry_state(e);
        if state == 0 or state == 2 {
            let _s = set_entry(e, key, value, 1);
            0
        } else if state == 1 and entry_key(e) == key {
            let old = entry_value(e);
            let _u = store_i64(e + 8, value);
            old
        } else {
            hm_insert_loop(m, key, value, (idx + 1) band mask, mask)
        }
    };
```

**MIR 최적화 확인:**
```
fn hash_i64(key: i64) -> i64 @alwaysinline @memory(none) {
  %h = mul %key, 5871781006564002453
  %_t1 = shr %h, 32
  %_t2 = bxor %h, %_t1
  ret %_t2
}

fn hm_insert_loop(...) {
entry:
  goto loop_header_10
loop_header_10:
  %idx_loop = phi [%idx, entry], [%_t14, else_7]
  ...  // TailRecursiveToLoop 변환 완료
}
```

### 구현 파일

- `ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/hashmap_pure.bmb`: 라이브러리
- `ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main_pure.bmb`: 벤치마크

### 검증

인터프리터로 실행 결과 확인 (원본과 동일):
```
95259    <- 삽입 후 엔트리 수
100000   <- 검색 성공 수
46445    <- 삭제 후 엔트리 수
46445    <- 최종 출력
```

### 남은 작업

- LLVM 빌드 환경 정상화 후 네이티브 성능 측정
- C 대비 111% → 목표 105% 달성 여부 확인

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

## P0-D: ~~직접 바이트 접근~~ ✅ 완료 (v0.51.44)

### 상태: ✅ 이미 구현됨

**LLVM codegen**에서 `byte_at`와 `char_at`이 이미 인라인됨:
- GEP로 BmbString 구조체에서 data pointer 접근
- GEP로 data 배열 인덱싱
- i8 로드 후 i64로 zext

### 벤치마크 개선 (v0.51.44)

`ord(char_at(s, i))` → `s.byte_at(i)` 변경으로 MIR 단순화:

```
Before:
  %_t3 = call char_at(%s, %pos)
  %_t4 = tail call ord(%_t3)

After:
  %_t3 = tail call byte_at(%s, %pos)
```

적용된 벤치마크:
- lexer: `peek` 함수
- brainfuck: `get_char` 함수

### 남은 작업

- LLVM 빌드 환경 정상화 후 실제 성능 측정

---

## P0-E: StringBuilder 최적화 (fasta 108% → ~100%) 🔧 진행 중

### 문제 분석 (v0.51.44 분석 완료)

**C 구현:**
```c
char line[LINE_WIDTH + 1];  // 스택 고정 버퍼, 할당 오버헤드 0
line[pos++] = char;          // 직접 메모리 쓰기
puts(line);                  // 배치 출력
```

**BMB 구현 (이전):**
```
fn print_repeat_lines(...) {
  ...
  %_t2 = call sb_new()           // 매 라인마다 힙 할당 (기본 64바이트)
  ...
  %_t5 = call sb_push_char(...)  // 문자당 함수 호출
  ...
  %_t8 = call sb_build(%sb)      // 문자열 생성
}
```

**근본 원인:**
1. **매 라인 sb_new() 할당**: 60자 출력마다 새 StringBuilder 할당
2. **문자당 함수 호출**: sb_push_char가 인라인되지 않음
3. **동적 버퍼 성장**: 고정 크기가 아니라 realloc 가능성

**긍정적 사항:**
- `iub_prob`, `iub_char_code` 등 15-case if-else가 switch로 변환됨:
```
switch %idx, [0 -> then_0, 1 -> then_3, ..., 13 -> then_39], else_40
```

### 해결책: sb_with_capacity 구현 (v0.51.45) ✅

**런타임 함수 추가:**
```c
// bmb_runtime.c
int64_t bmb_sb_with_capacity(int64_t capacity) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    sb->cap = capacity > 0 ? capacity : 64;
    sb->len = 0;
    sb->data = (char*)malloc(sb->cap);
    sb->data[0] = '\0';
    return (int64_t)sb;
}
```

**벤치마크 수정:**
```bmb
fn print_repeat_lines(alu_str: String, k: i64, remaining: i64) -> i64 =
    if remaining <= 0 { k }
    else {
        let sb = sb_with_capacity(61);  // v0.51.45: pre-allocate 61 bytes (60 + null)
        ...
    };
```

**MIR 확인:**
```
%sb = call sb_with_capacity(61)  // 용량 힌트 전달
```

### 남은 최적화

| 옵션 | 접근 방식 | 난이도 | 효과 |
|------|----------|--------|------|
| ~~A~~ | ~~`sb_with_capacity(60)` 런타임 함수 추가~~ | ~~낮음~~ | ✅ 완료 |
| B | 고정 크기 배열 타입 추가 `Array<u8, 60>` | 높음 | C와 동등 |
| C | sb_push_char LLVM 인라인 | 중간 | 함수 호출 제거 |

### 남은 작업

- LLVM 빌드 환경 정상화 후 네이티브 성능 측정
- 108% → 목표 100% 달성 여부 확인

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

## 🚧 현재 차단 요소: LLVM 빌드 환경

### 문제

MSYS2 환경에서 빌드 시 llvm-sys가 MSYS2의 llvm-config를 사용하여
MSVC와 호환되지 않는 헤더 경로를 주입:

```
llvm-config --cflags → -IC:/msys64/ucrt64/include
```

MSVC는 MSYS2 stdlib.h를 파싱하지 못함:
```
C:/msys64/ucrt64/include\stdlib.h: error C2085: '_Exit': not in formal parameter list
```

### 해결책

1. **Windows CMD에서 빌드**: MSYS2 없는 환경에서 cargo build
2. **LLVM 개발 패키지 설치**: MSVC용 LLVM 설치 (llvm-config 포함)
3. **llvm-sys 패치**: MSYS2 경로 필터링

### 영향

- 모든 벤치마크 재측정 불가
- LLVM 네이티브 컴파일 불가
- MIR 분석으로만 최적화 효과 검증 가능

---

> 이 문서는 P0 IR Parity 달성까지의 집중 로드맵입니다.
> 달성 후 부트스트랩 및 기타 기능 작업을 재개합니다.
