# ISSUE-20260512 — Tier 3 lexer bench (BMB) returns 0 tokens for all categories

## 핵심 메타

**우선순위**: P2 → **RESOLVED**
**영역**: ecosystem/benchmark-bmb / bench correctness
**상태**: **RESOLVED (Cycle 2788, 2026-05-13)**

## 해결 요약

Cycle 2788에서 `lexer/bmb/main.bmb` 전면 재작성으로 해결. 근본 원인은 0-token이 아닌 다수 버그였음:

1. **`is_keyword_at` false positive**: "return"(len=6)과 "result"(len=6) 모두 매칭 → 2개 identifier 누락
2. **단일 i64 packing 오버플로**: op/punct 카운트가 10^1 간격에서 overflow (op=17→19로 잘못 추출)
3. **Strings/Comments 미추적**: `count_tokens_loop`에 `new_str`/`new_comment` 업데이트 누락
4. **`main()` 출력 누락**: Strings, Comments 미출력
5. **`count_tokens` 반환 타입**: 단일 i64 → (i64, i64) 튜플로 변경
6. **"return" vs "result" 구분**: 3번째 문자 체크 추가 (`peek(src, start+2)==116`)

## 검증 결과

```
$ scripts/verify_bench_outputs.py --tier all --epsilon 1e-6
real_world/lexer    PASS
```

BMB 출력 = C 출력 정확히 일치:
- Small: Identifiers:20, Numbers:9, Keywords:12, Strings:1, Operators:17, Punctuation:29, Comments:1
- Large(100x): Total tokens:8900

## 측정 stamp (초기)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2765 발견), 2026-05-13 (Cycle 2788 해결) |
| `original_issue` | small + large source 모두 모든 token category 0 출력 |

## 메타

- 관련 ISSUE: `ISSUE-20260512-bench-output-fairness-survey.md`
- 인용 cycle: cycle-2765.md (발견), cycle-2788 (해결)
