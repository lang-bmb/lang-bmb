# ISSUE-20260511 — `set obj.field[idx] = val` 파서 미지원

**Date**: 2026-05-11 (Cycle 2684)
**Drift**: C (AI-native 언어 갭)
**Severity**: medium (LLM 자연 패턴, 워크어라운드는 전체 재할당)

## 현상

```bmb
struct Stats { values: Array<f64> }

fn main() -> i64 = {
    let mut s = Stats { values: [1.5, 2.5, 3.5] };
    set s.values[0] = 99.0;  // ❌ parse error
    ...
};
```

파서 에러:
```
expected '=', '+=', '-=', '*=', or '/=' in set, got '[' at line N
  | set s.values[0] = 99.0;
  |             ^
```

## 원인

`bootstrap/compiler.bmb` `parse_set_field` (line 996):
- `set obj.field = val` 만 처리
- `set obj.field[idx] = val` 시 `[` 토큰 fall-through → assign op 기대 위치에서 실패

## 해결 방향

**Decision Framework 1순위**: 언어 스펙. AI-native 패턴이므로 지원해야 한다.

### Fix points (예상)

1. **`parse_set_field` (line 996)** — field 토큰 후 `[` 인 경우 새 분기:
   ```bmb
   if tok_kind(t_after_field) == TK_LBRACKET() {
       parse_set_field_index(src, ..., target_name, field_name)
   }
   ```
2. **AST 신규 노드**: `(set_field_index (var <name>) field_name <idx_ast> <val_ast>)`
3. **Lowering 추가** (이중 lowering 시스템 — Rule 5):
   - `lower_expr_sb` (recursive)
   - `step_expr` (iterative)
4. **MIR**: field-store + index 합성. 또는 `field-store-index` 신규 opcode.
5. **유사 케이스 검사**: `set obj.field1.field2 = val` (nested field set, 별도 갭)

## Workaround (현재)

전체 array 재할당:
```bmb
set s.values = [99.0, 1.2, 3.4];  // 동작
```

## Test scaffold

```bmb
// 동작해야 함: "99.000\n2.500\n3.500\n", exit 42
let mut s = Stats { values: [1.5, 2.5, 3.5] };
set s.values[0] = 99.0;
```

## 관련 작업

- M5-5d 인프라 (`~af` field suffix) 와 직교 — 본 이슈 해소 시 자동 dispatch 가능
- nested field set `set o.inner.values[0] = x` 도 함께 검토

## 우선순위

- ⏳ 다음 세션 (자율) — 2-3 cycles 예상
- M5 마일스톤 잔여 항목으로 분류
