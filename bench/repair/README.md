# H-REPAIR 절제 (ADR-0078, 백로그 T1-45)

가설 H-REPAIR: 실패 진단의 반례와 수리 제안(`fix`)이 수리 반복을 줄인다. 같은 과제·모델·커밋에서 두 판을 비교한다.

| 판 | 집합 | 모델이 보는 도구 결과 |
| --- | --- | --- |
| (a) 전체 | `bench/m1b-write`(동결) | 지금 그대로: 진단 메시지 끝의 반례, `fix`(수정 제안·힌트) |
| (b) 절제 | `bench/m1b-write-norepair`(`gen.py`가 만든 동결 판) | 반례(`; counterexample: ..`, `; unconfirmed counterexample: ..`)와 모든 `fix`·`counterexample` 필드를 지운 것 |

두 판은 `set.json`의 `"tool_filter": "no-repair"` 하나만 다르다. 필터는 m1 하네스(`bench/m1/harness.py`의 `strip_repair`)가 모델에게 가는 모든 도구 결과에 건다. JSON이 아닌 줄과 지울 것이 없는 줄은 바이트 그대로다. 프롬프트도 같은 텍스트다("failures come with counterexamples"라는 문장이 남는다: 바꾸면 차이가 둘이 된다). 진단의 코드·위치·반례 앞의 문장은 남는다.

| 파일 | 내용 |
| --- | --- |
| `gen.py` | `bench/m1b-write-norepair`를 만든다(`--check`: 커밋된 판이 새로 만든 것과 같은지) |
| `compare.py` | 두 결과 파일의 비교 표와 H-REPAIR 판정 줄 |
| `test_gen.py` | 게이트: 생성 일치, 필터, 실패 증명 세기, 스크립트 에이전트, 판정 규칙 |

```
python3 bench/m1/harness.py --set bench/m1b-write run --agent openai:MODEL --out FULL.jsonl
python3 bench/m1/harness.py --set bench/m1b-write-norepair run --agent openai:MODEL --out NONE.jsonl
python3 bench/repair/compare.py FULL.jsonl NONE.jsonl
```

## 판정 규칙 (첫 실행 전에 등록, 2026-11-02)

- **지표:** 성공한 작성(`write`) 세션당 (거부된 커밋 `commits_rejected` + 실패한 증명 `proofs_failed`). `proofs_failed`는 커밋된 뒤 증명이 실패한 쓰기와 증명이 실패한 `verify` 호출의 수다(오류 진단이 있는 결과, m1 하네스 `proof_failed`). 탐색 과제는 쓰기가 없어 뺀다.
- **확정:** (a)의 지표가 (b)보다 40% 이상 적다((a) ≤ 0.6 × (b)). **반증:** 그렇지 않다. 두 결과는 같은 에이전트·컴파일러 커밋·조건이어야 하고(`compare.py`가 거부한다), 한쪽에 성공한 작성 세션이 없거나 (b)의 지표가 0이면 판정 불가.
- 모델은 qwen(ADR-0078). (a)는 같은 커밋의 `m1b-write` 실행(T0-13의 qwen 실행이 같은 커밋이면 그것)을 쓴다.
- 성공률 차이는 따로 보고한다. 1회 측정이라 ADR-0077의 최소 차이 D 이하의 성공률 차이는 방향을 주장하지 않는다.
