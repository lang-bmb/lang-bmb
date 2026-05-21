## Cycle 2792 완료: or/and 단락 평가 (Short-Circuit) 부트스트랩 fix

### 개발 범위

`bootstrap/compiler.bmb`에서 `or`/`and` 연산자의 단락 평가 시맨틱 위반 수정.

**문제**: `a or b or c`가 eager `or i1` 캐스케이드로 낮춰져 오른쪽 피연산자가 항상 평가됨.

**수정 목표**: `a or b` → `if a { true } else { b }` 패턴으로 낮추기 (branch+phi 사용).

### 구현

3개 위치 수정 — 두 낮춤 경로 + 순수식 판별 함수:

**1. `is_pure_expr` (line ~5669)**
```bmb
// or/and를 impure로 표시 → lower_if_sb가 select 경로 대신 branch 경로 사용
if op == "/" or op == "%" or op == "or" or op == "and" { false }
```

**2. `lower_binop_sb` (recursive 경로, line ~5627)**
```bmb
if op == "or" { lower_if_branch_sb(left_ast, "(bool true)", right_ast, ...) }
else if op == "and" { lower_if_branch_sb(left_ast, right_ast, "(bool false)", ...) }
else { /* 기존 eager 경로 */ }
```

**3. `step_binop_start` (iterative 경로, line ~4194)**
```bmb
if op == "or" {
    let cont = make_work("IT", "(bool true)", right, ...);
    let do_cond = make_work3("EX", left, "", "");
    make_step(...)
} else if op == "and" {
    let cont = make_work("IT", right, "(bool false)", ...);
    ...
}
```

### 검증

**단락 평가 행위 확인**:
- `true or expensive(42)` → expensive 미호출, 출력: `0` ✅
- `false or expensive(42)` → expensive 호출, 출력: `42\n0` ✅
- `false and expensive(42)` → expensive 미호출, 출력: `0` ✅

**MIR 옵티마이저 관찰**: 순수 피연산자의 경우 `lower_if_branch_sb`가 `branch+phi`를 방출하지만,
bootstrap의 `ifs_fn_lines` 옵티마이저(~line 12588)가 `branch+pure_empty_blocks+phi` → `select`로 폴딩.
이는 순수 표현식에 대해 의미상 동등하며, 의도된 최적화임.

### 현재 상태

| 항목 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6211 PASS |
| 골든 테스트 (parser/selfhost/lexer/codegen) | ✅ 4/5 PASS |
| `error_test` FAIL | ⚠️ 기존 문제 (6/10, JSON vs 인간 포맷 불일치) — 이번 변경과 무관 |
| Stage 2 부트스트랩 | ✅ IR 생성 성공 |
| Stage 3 Fixed Point (S2==S3) | ✅ diff 없음 |

### 미비/결함/개선

| 유형 | 내용 | 심각도 |
|------|------|--------|
| 개선 | 순수 or-chain에서 switch/jump table 생성: MIR 옵티마이저의 `select` 폴딩이 LLVM switch 생성 방해. 별도 사이클 필요 | P3 |
| 결함 | `error_test` 기존 6/10 실패: JSON 포맷 출력을 기대하는 코드가 "Type error"/"Parser error" 문자열 검색 (기존 문제) | P3 |

### 후속 단계

1. error_test 포맷 정합 수정 (P3, 별도 사이클)
2. 순수 or-chain의 switch/jump table 생성 최적화 (P3)
3. D5-A GitHub Actions verify workflow (HUMAN)
