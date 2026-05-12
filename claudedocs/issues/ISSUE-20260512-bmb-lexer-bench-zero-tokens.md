# ISSUE-20260512 — Tier 3 lexer bench (BMB) returns 0 tokens for all categories

## 핵심 메타

**우선순위**: P2 (measurement fairness — Tier 3 lexer 1.000x 측정 부적절 fair 가능성)
**영역**: ecosystem/benchmark-bmb / bench correctness
**상태**: Open — 별도 cycle (단발성 디버깅 가능)

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2765) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | `ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` 직접 실행 |
| `observed_rate` | small + large source 모두 모든 token category 0 출력 |
| `scope` | `lexer` bench BMB 측만 (C 측 정상) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 |

## 문제

`ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` 의 `count_tokens(src)` 호출이 모든 입력에 대해 0을 반환:

```
$ ./main.exe
Lexer Benchmark
Small source:
  Identifiers: 0
  Numbers: 0
  Keywords: 0
  Operators: 0
  Punctuation: 0

Large source (1000x):
  Total tokens: 0
Done.
```

C 측 동일 입력에서 정상 출력:
```
Identifiers: 20
Numbers: 9
Keywords: 12
Strings: 1
Operators: 17
Punctuation: 29
Comments: 1
```

## 진단 (Cycle 2765, partial)

`count_tokens_loop` 는 tail-recursive로 각 토큰을 인식하며 카운터를 증가. 모든 카운터가 0인 결과 → **루프가 즉시 tok_eof 분기로 종료** 가설:

```bmb
fn count_tokens(src: String) -> i64 =
    count_tokens_loop(src, 0, 0, 0, 0, 0, 0, 0, 0);

fn count_tokens_loop(...) -> i64 = {
    let result = next_token(src, pos);
    let tok = result.0;
    ...
    if tok == tok_eof() { encoded } else { recurse }
};
```

후보 root cause:
1. **`next_token` 즉시 tok_eof() 반환**: `is_alpha`/`is_digit`/`is_punct_char` 등 인식 fn이 모두 0 반환. `byte_at`/`peek` 결과 인식 실패?
2. **String 입력 손상**: `test_source()` 의 `+` 연산이 빈 문자열을 반환?
3. **부트스트랩 컴파일러 회귀**: M5-5* 기능 추가 도중 lexer-style 패턴 회귀 (확률 낮음 — 다른 bench는 정상)

## 영향 평가

| 영역 | 영향 |
|------|------|
| Tier 3 lexer 측정 | ⚠️ **BMB가 실제로는 "0 work" 상태**일 가능성 — 1.000x parity 측정 부적절 fair |
| 다른 bench | ✅ 영향 없음 (lexer 단독 패턴) |
| 부트스트랩 | ✅ 영향 없음 (`bootstrap/lexer.bmb` 는 별도 코드, 정상 작동) |
| M1 P-track 가설 | 부분 영향 — Tier 3 1/7 bench 측정 의문, 나머지 6 정상 |

## 해결 방안

### 1단계: 진단
- [ ] `count_tokens(test_source())` 단독 호출 + intermediate value 출력
- [ ] `next_token(test_source(), 0)` 직접 호출 → 첫 토큰 반환 확인
- [ ] `is_alpha(102) == 1` ('f') 확인
- [ ] `test_source().len() > 0` 확인
- [ ] `byte_at(test_source(), 0)` 결과 확인

### 2단계: fix
- 진단 결과에 따라 fix path 결정
- 부트스트랩 컴파일러 회귀라면 별도 isolation 테스트로 재현 + 해결

### 3단계: 회귀 방지
- 골든 테스트 추가 (lexer count_tokens correctness)
- bench output verification (BMB ↔ C 출력 diff 자동 검사) 추가 검토

## 완료 기준

- [ ] BMB lexer bench가 C와 동일한 token counts 출력
- [ ] count_tokens correctness 골든 테스트 추가
- [ ] Tier 3 lexer 재측정 (BMB가 실제 work 수행 후 1.000x 유지 확인)

## 메타

- 관련 ISSUE: `ISSUE-20260512-tier3-spawn-overhead-methodology.md` (Cycle 2765 발견)
- 인용 cycle: cycle-2765.md (workload amplification POC 도중 발견)
