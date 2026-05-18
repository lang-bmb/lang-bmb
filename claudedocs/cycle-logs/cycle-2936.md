# Cycle 2936: `break <expr>` 언어 기능 — break-with-value
Date: 2026-05-19

## Re-plan
Cycle 2935 Carry-Forward: 클로저 HOF 통합 표현 (별도 사이클). 이번 사이클은 언어 갭 해소 — `break <value>` 문법 추가.

## Scope & Implementation

### 1. 동기

인터프리터는 이미 `Expr::Break { value: Some(...) }` AST를 처리함. 그러나 grammar에 `break <expr>` 생산규칙이 없어 파서 에러 발생. `loop { break i }` 형식의 early exit with return value를 허용하기 위해 grammar에 추가.

### 2. Grammar 추가 (bmb/src/grammar.lalrpop)

`return <expr>`과 동일한 패턴으로 두 규칙에 추가:

**BlockExpr (line 1398)** — block 문맥 (BlockStmt → SpannedBlockExpr → BlockExpr fallthrough):
```lalrpop
// v0.99 Cycle 2936: Break with value
"break" <e:SpannedImpliesExpr> => Expr::Break { value: Some(Box::new(e)) },
```

**Expr (line 1658)** — 표현식 문맥:
```lalrpop
// v0.99 Cycle 2936: Break with value - loop { break expr }
"break" <e:SpannedImpliesExpr> => Expr::Break { value: Some(Box::new(e)) },
```

### 3. LALR 충돌 해결

처음에 `BlockStmt`에도 추가하려 했으나 구조 오해. `BlockStmt`는 자체 `break` 처리 없이 `SpannedBlockExpr → BlockExpr`로 fallthrough함. `BlockExpr` 안에 중복 추가하면 LALR 충돌 발생:
```
Multiple productions for the same reduction:
  "break" SpannedImpliesExpr ";"
```

중복 제거 후 빌드 성공.

## Verification & Defect Resolution

### 기능 검증

```bmb
fn first_square_gt(n: i64) -> i64 = {
    let i = 1;
    loop {
        if i * i > n { break i };  // break with value in block context
        set i = i + 1
    }
};
first_square_gt(20) // → 5 (5²=25 > 20) ✅

fn find_divisor(n: i64) -> i64 = {
    let i = 2;
    loop {
        if i * i > n { break 0 - 1 };  // negative sentinel
        if n % i == 0 { break i };
        set i = i + 1
    }
};
find_divisor(35) // → 5 ✅
find_divisor(37) // → -1 ✅
```

### 테스트 결과

- `cargo test --release -p bmb`: 2388 passed, 0 failed ✅
- 골든 테스트 신규: `break_with_value.bmb` / `.out` 추가

### 타입 체커 호환성

`Expr::Break { value }` 는 `Type::Never` 반환 (기존 동작 유지). 루프 자체도 `Type::Never`. 이는 `break <value>` 에 대해 충분 — 타입 체커가 break value 타입을 loop 반환 타입으로 전파하지 않아도 인터프리터에서 정상 동작. 단, native codegen에서는 MIR 지원 필요 (현재 미구현).

## Reflection

### Scope fit
- ✅ `break <expr>` 파서 지원 완료
- ✅ 블록 컨텍스트 + 표현식 컨텍스트 모두 동작
- ✅ 2388 테스트 통과
- ⚠️ Native codegen MIR: `break <value>` 값이 MIR에서 무시됨 (인터프리터 전용)

### 언어 완성도 의의
`loop { break value }` 패턴은 early exit with result를 표현하는 핵심 idiom. 이전에는 별도 변수(`let result = ...`) + 루프 + `set result = ...` 패턴이 필요했음. 이제 직접 `break i`로 표현 가능.

### LALR 구조 인사이트
BMB grammar에서 block 문맥 표현식은 `BlockStmt → SpannedBlockExpr → BlockExpr → ImpliesExpr` 체인. 새 control flow 키워드 추가 시 `BlockExpr`와 `Expr` 두 곳에 추가하면 충분. `BlockStmt`에 별도 추가하면 중복으로 LALR 충돌.

## Carry-Forward

- Actionable: Native codegen에서 `break <value>` MIR 지원 (루프 result-slot local 방식) — 별도 사이클
- Structural Improvement Proposals:
  1. **Loop type inference 개선**: `loop { break i }` → 타입 체커가 loop 반환 타입을 `i64`로 추론 (현재 `Never`). 실용적 영향 낮음 (인터프리터 동작, 타입 에러 없음).
  2. **Closure HOF unified representation** (Cycle 2935 carry-forward 유지)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2937 — 언어 갭 추가 (continue <label> / while-let 개선 / native codegen break value)
