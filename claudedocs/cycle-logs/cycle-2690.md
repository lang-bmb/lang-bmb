# Cycle 2690: set field-index 파서 설계 + 기본 i64 동작
Date: 2026-05-11

## Re-plan
Carry-Forward (Cycle 2689): set obj.field[idx] = val 파서 확장 (ISSUE-20260511).
Trigger 없음. Phase 1 시작.

## Scope & Implementation

### 설계 결정: AST 차원 desugar (신규 노드 0개)

`set obj.field[idx] = val` → `(set_index (field (var <obj>) field) idx val)`

**왜 desugar?**
- `set_index`는 base expr를 `EX`로 평가 → field-access MIR 발행 → ptr 반환
- `field-access`는 Array<X> field일 때 dest를 `str_ptr` / `f64_ptr`로 자동 마크
- `store_ptr`는 marker 기반 `store i64` vs `store double` 자동 선택
- 이중 lowering 회피 (Rule 5 자동 만족 — 신규 AST 노드 없음)

### 구현

`bootstrap/compiler.bmb`:
- `parse_set_field` (line 996) — field ident 뒤가 `[`이면 `parse_set_field_index`로 분기
- `parse_set_field_index` (신규) — `(set_index (field (var <target>) field) idx val)` 생성
- compound assign (`+=`) 도 `(binop + (index (field obj fname) idx) val)` 로 생성

### 검증 케이스

`test_golden_set_field_index_basic.bmb`:
```bmb
struct Stats { values: Array<i64> }
fn main() -> i64 = {
    let mut s = Stats { values: [10, 20, 30] };
    set s.values[1] = 99;
    let v = s.values[1];
    if v == 99 { 42 } else { 0 }
};
```

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Stage 1 빌드 | ✅ 18.9s |
| basic i64 array set | ✅ exit 42 |
| compile/link | ✅ 3ms / 244ms |

결함: 없음.

## Reflection

- AST desugar 전략 검증 — M5-5e (nested) 무구현 통과 패턴과 동일 직교성
- field-access + set_index 기존 인프라가 모든 marker propagation 자연 처리
- 신규 AST/MIR 노드 없음 → 이중 lowering 점검 불필요 (Rule 5 우회)
- Drift C 언어 갭 1개 해소 (LLM 자연 패턴)

## Carry-Forward
- Actionable: f64/String/compound assign 변형 검증 (Cycle 2691)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음 — Phase 1 단축 가능 (4 → 2-3 cycles)
- Next Recommendation: Array<f64>, Array<String>, nested struct, `+=` 변형 + 골든 등록
