# Cycle 2950: bool_operators 비트 연산 개선 + 다중 쿼리 problem.md 2종
Date: 2026-05-19

## Re-plan

Cycle 2949 Carry-Forward → 잔여 언어 갭 탐색.

발견: `bool_operators` 패턴이 `a & b`(bitwise AND)와 `a | b`(bitwise OR)도 발동하지만
제안이 boolean(`and`/`or`)만 안내 → bitwise 의도 시 틀린 제안.
92_bit_counting(popcount)이 `n & 1` 사용 시 `n band 1`이 필요한데 잘못된 힌트 제공 가능성.

## Scope & Implementation

### Fix 1: bool_operators 제안 개선

트리거 `"Unrecognized token \`&\`"` 는 두 가지 원인:
- `&&` (boolean AND) → 올바른 fix: `and`
- `&` (bitwise AND) → 올바른 fix: `band`

기존 제안: `or`/`and`만 언급 → bitwise 케이스 미커버.

수정된 제안:
```
"BMB does not use '|', '||', '&', '&&' operators.
For BOOLEAN operators: 'a || b' → 'a or b',  'a && b' → 'a and b'
For BITWISE operators: 'a | b' → 'a bor b',  'a & b' → 'a band b'
Note: BMB uses 'band'/'bor'/'bxor' for bitwise — NOT &/|/^."
```

### Fix 2: unknown_function 제안 확장

기존: println/print/read_int/vec_* 목록만 포함.
수정: read_line, abs, i64_min, i64_max, f64_sqrt, str_concat, str_len, str_substr, str_to_int 추가.

### Fix 3: 92_bit_counting problem.md

t-first 다중 쿼리 + BMB 비트 연산자 안내 + popcount 구현 스케치 추가.
기존: 단순 설명만, multi-query 구조 미명시.

### Fix 4: 84_accumulator_pattern problem.md

t-first 다중 쿼리 구조 + 초기값 안내 + i64_min/i64_max 사용법 추가.

### Fix 5: 85_registry_pattern problem.md

병렬 배열(keys vec + vals vec + size) 기반 구현 접근법 + 변수 인수 읽기 순서 명시.

### New tests: test_bool_operators_bitwise_band, test_bool_operators_bitwise_bor

`&` 오류 시 제안에 `band` 포함 확인, `|` 오류 시 `bor` 포함 확인.

## Verification & Defect Resolution

```
cargo test --release -p bmb --test diagnostics_test
  diagnostics_test: 22/22 PASSED  (was 20, +2 new: bitwise tests)
```

## Reflection

### Scope fit
- ✅ bool_operators bitwise 케이스 보완 → 92_bit_counting 등 B루프 탈출 개선
- ✅ unknown_function 더 완전한 builtin 목록
- ✅ 3개 problem.md 개선 (다중 쿼리 + 병렬 배열)

### 누적 수정 현황 (Cycles 2945-2950)
- 에러 패턴: 6개 신규 + 2개 개선 (bool_operators, unknown_function 제안 확장)
- problem.md: 17개 파일 수정
- diagnostics 테스트: 13 → 22 (+9)

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — 6 cycles분 fix 효과 검증 (외부 API 필요)
  2. **51_bracket_match `||` 지원** — BMB 언어 자체에 `||` 추가 검토
  3. **inttoptr UB (P3)** — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2951 → 추가 problem.md 탐색 (80-100 범위) or 언어 갭 분석
