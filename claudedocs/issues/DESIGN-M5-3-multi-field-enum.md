# DESIGN-M5-3: Multi-Field Enum Variant
Date: 2026-05-10

## Goal

`enum Node { Leaf(i64), Branch(i64, i64) }` — variant당 다중 필드 지원.

## Current State (M5-1/2)

- 모든 variant: `calloc(2, 8)` — word 0 = tag, word 1 = payload (i64 1개)
- Match pattern: `Type::Variant(v)` — 단일 바인딩만

## Required Changes

### 1. Parser

**Construction**: `Node::Branch(20, 30)` → `(enum_val <Node> <Branch> 20 30)`
- `parse_match_arms` 내 `TK_LPAREN()` 분기: 다중 인자 파싱
- `lower_enum_val_sb`: n-field 지원

**Pattern**: `Node::Branch(a, b)` → multi-binding
- `parse_match_arms` 내 payload_bind 패턴: `,` 구분 다중 이름

### 2. Enum Registry

현재: `"Option:None,Some[i64];"` → 변경: `"Node:Leaf[i64],Branch[i64,i64];"`
- `parse_enum_variants_to_registry`: `,` 구분 다중 타입
- `variant_field_count(registry, type_name, variant_name)` 헬퍼 추가

### 3. LLVM Representation

현재: `calloc(2, 8)` → 2-word
제안: `calloc(1 + max_fields, 8)` → (1 + N)-word
- word 0: tag
- word 1..N: fields (i64 각각)

**주의**: 모든 variant에 동일 크기 할당 필요 (최대 필드 수 기준)
→ `max_variant_fields(registry, type_name)` 헬퍼 필요

### 4. Match Pattern Extraction

현재: `resolve_payload_extracts` → 단일 GEP+load
확장: N-field 반복 GEP+load, 각 bind_name에 매핑

## Estimated Effort

- Parser: ~2 edits
- Registry: ~2 edits  
- `lower_enum_val_sb`: ~1 edit (n-field loop)
- `resolve_payload_extracts`: ~1 edit (n-field loop)
- `step_expr` enum_val: 자동 처리 (기존 위임 패턴)
- Golden tests: 2-3개

**총 예상**: 3-4 cycles

## Test Cases

```bmb
enum Node { Leaf(i64), Branch(i64, i64) }
// sum_node(Leaf(10)) = 10
// sum_node(Branch(20, 30)) = 50
// Expected: 60
```

## Known Risks

- `calloc` 크기 결정: 각 enum 타입의 최대 필드 수를 컴파일 타임에 계산
- 혼합 필드 (1-field + 2-field variant): 항상 2-word 할당 → 1-field variant의 word 2 = 미사용
