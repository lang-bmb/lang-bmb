---
id: DESIGN-M5-1-payload-enum
title: M5-1 Payload Enum 아키텍처 설계
type: design
priority: M5
status: decided
created: 2026-05-10
---

# M5-1: Payload Enum 아키텍처 설계

## 현황 (bootstrap/compiler.bmb 분석, Cycle 2622)

### 현재 enum 파이프라인

```
1. parse_enum_def → parse_enum_variants_to_registry
   - enum_reg에 "Color:Red,Green,Blue;" 형태로 저장
   - variant name만 기록, 페이로드 타입 정보 없음

2. parse_ident_or_call (line ~782)
   - Type::Variant → (enum_variant <Type> <Variant>)
   - Type::Method(args) → (call <Type_Method> args)  [M4-4 추가]

3. resolve_enum_variants_in_ast (line 2956)
   - (enum_variant <Name> <Variant>) → (int N)
   - 모든 variant = 정수 ordinal

4. LLVM codegen
   - enum value = i64 정수
```

### 핵심 제약

- `parse_enum_variants_to_registry` (line 2909-2922): variant name만 파싱, `(` 가 오면 skip
- 현재 레지스트리 포맷 `"EnumName:V1,V2,V3;"` — 페이로드 타입 표현 불가
- unit enum → i64 ordinal 전제로 codegen 전체 설계됨

---

## 목표 설계

### 1. Enum 레지스트리 포맷 확장

**현재**: `"Color:Red,Green,Blue;"`

**목표**: `"Color:Red,Green(i64),Blue;"` 또는 tagged 포맷
```
"Color:0:Red:unit,1:Green:i64,2:Blue:unit;"
```

필드: `ordinal:name:payload_type` (unit = payload 없음)

### 2. AST 노드 추가

현재:
```
(enum_variant <Name> <Variant>)          — unit variant 표현식
```

추가 필요:
```
(enum_construct <Name> <Variant> payload_ast)  — payload variant 표현식
```

### 3. 파서 변경

#### `parse_enum_variants_to_registry` (line 2909)

현재: variant name 뒤 `(` 있으면 skip
목표: `(type)` 파싱하여 레지스트리에 저장

```bmb
// 현재
fn parse_enum_variants_to_registry(src, pos, rsb, idx) -> i64 =
    // IDENT → push name → recurse
    // 그 외 → skip

// 목표
fn parse_enum_variants_to_registry_v2(src, pos, rsb, idx) -> i64 =
    // IDENT → push "ordinal:name:"
    // if next == TK_LPAREN → parse type name → push type → TK_RPAREN
    // else → push "unit"
    // recurse
```

#### `parse_ident_or_call` (line ~782, M4-4 이후)

현재:
```
Type::Variant    → (enum_variant <Type> <Variant>)    [unit]
Type::Method(x)  → (call <Type_Method> x)             [M4-4, static call]
```

목표:
```
Type::Variant     → (enum_variant <Type> <Variant>)       [unit, 동일]
Type::Variant(x)  → (enum_construct <Type> <Variant> x)   [payload construct]
                    단, 레지스트리에 payload variant로 등록된 경우
                    미등록이면 (call <Type_Variant> x) 유지 [static call fallback]
```

**주의**: M4-4와의 충돌 — 현재 `Type::X(args)` 는 항상 static call로 처리됨.
레지스트리 조회가 파서보다 나중에 실행되므로 파서 시점에 구분 불가.
→ 해결 방법: **2단계 해석** (파서에서는 tagged AST 노드로 기록, post-parse phase에서 레지스트리 참조)

### 4. 표현 레이어 — Tagged Union

#### 메모리 레이아웃

```
struct EnumValue {
    discriminant: i64,  // variant ordinal
    payload: i64,       // 0 for unit, value or ptr for payload
}
```

LLVM IR:
```llvm
%EnumValue = type { i64, i64 }

; unit variant 생성
%e = alloca %EnumValue
%d = getelementptr %EnumValue, ptr %e, i32 0, i32 0
store i64 0, ptr %d  ; ordinal

; payload variant 생성
%e = alloca %EnumValue
%d = getelementptr %EnumValue, ptr %e, i32 0, i32 0
store i64 1, ptr %d  ; ordinal
%p = getelementptr %EnumValue, ptr %e, i32 0, i32 1
store i64 42, ptr %p  ; payload (i64)
```

#### 하위 호환성 문제

- 현재 unit enum = `i64` 하나
- 새 표현 = `{i64, i64}` 구조체
- `match` 패턴: 현재 `(int N) == scrutinee` → 새로 discriminant 비교로 변경 필요
- **모든 기존 enum 사용 코드 영향** → 신규 기능이므로 기존 회귀 주의

### 5. 패턴 매칭 확장

현재 match arm:
```
(enum_variant <Name> <Variant>) → resolve → (int N) → i64 비교
```

목표:
```
(enum_variant <Name> <Variant>)     → discriminant 비교 (unit)
(enum_pattern <Name> <Variant> var) → discriminant 비교 + payload 추출
```

LLVM IR:
```llvm
; match arm 패턴 확인
%disc = load i64, ptr (GEP discriminant field)
%match = icmp eq i64 %disc, N
br i1 %match, label %arm_body, label %next_arm

; payload 추출 (Some(v) 같은 경우)
%pay_ptr = GEP payload field
%v = load i64, ptr %pay_ptr
```

---

## 구현 순서 (M5-1 사이클 계획)

| 단계 | 작업 | 예상 cycles |
|------|------|-------------|
| M5-1a | 레지스트리 포맷 확장 + payload 타입 저장 | 1-2 |
| M5-1b | `(enum_construct)` AST 노드 + codegen (alloca+store) | 2-3 |
| M5-1c | 기존 unit enum을 새 표현으로 마이그레이션 | 1-2 |
| M5-1d | 패턴 매칭 payload 추출 | 2-3 |
| M5-1e | 테스트 + 3-Stage bootstrap 검증 | 1-2 |
| **합계** | | **7-12 사이클** |

---

## M4-4 사이드 이펙트 해결 방안

Cycle 2620에서 `Type::Variant(x)` → `(call <Type_Variant> x)` 가 된 이슈:

**단기 (현재)**: 레지스트리 없이 static call로 처리 → payload enum constructor 불가
**M5-1 완성 후**: 2단계 해석으로 레지스트리 기반 구분 → `(enum_construct)` vs `(call)`

M5-1 전까지는 payload enum을 사용하지 않는 것을 권장 (CLAUDE.md Rule 2에 명시됨).

---

## 결정 사항 (2026-05-10 확정)

1. **하위 호환성**: ✅ unit enum도 `{i64, i64}` 로 **전체 마이그레이션** — 이중 코드젠 경로 금지
2. **LLVM 표현**: ✅ `%EnumValue = type { i64, i64 }` **고정 2-word**, 스택 할당 (heap alloc 없음)
3. **Result<T,E>** 지원: ✅ M5-1 범위 외 — i64 단일 페이로드만. 가변 페이로드 타입은 **M5-2로 defer**

---

## 참고 — 현재 동작하는 workaround

payload enum이 필요한 경우 M5-1 전까지:

```bmb
// 불가
enum Option { None, Some(i64) }
let x = Option::Some(42);

// 가능 (workaround, 권장하지 않음)
fn Option_Some(v: i64) -> i64 = v;  // tag 무시
fn Option_None() -> i64 = -1;       // sentinel
let x = Option::Some(42);  // M4-4로 Option_Some(42) 호출
```
