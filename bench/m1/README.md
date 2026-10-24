# M1 측정 (ADR-0021)

에이전트가 파일 없이 MCP 도구만으로 BMB 과제를 푸는지 잰다.

| 파일 | 내용 |
| --- | --- |
| `prompt.md` | 에이전트 시스템 프롬프트(동결) |
| `seed/` | 시작 작업 공간(동결). 정의 23개 |
| `tasks/` | 과제 15개(동결): 작성 10, 탐색 5. 작성 과제는 숨긴 기준 계약과 기준 해법을 가진다 |
| `MANIFEST` | 동결 파일의 SHA-256. 다르면 하네스가 실행을 거부한다 |
| `harness.py` | 세션 실행·판정·지표 |
| `anthropic_agent.py` | 실제 모델 어댑터(유료, 사람 승인 필요) |

```
python3 bench/m1/harness.py manifest                  # 동결 확인
python3 bench/m1/harness.py run --agent oracle        # 기준 해법: 전부 성공해야 함(CI)
python3 bench/m1/harness.py run --agent weak          # 약한 명세: 전부 실패해야 함(CI)
python3 bench/m1/harness.py run --agent anthropic:claude-opus-5-5 --out results/DATE-MODEL.jsonl
```

작성 과제의 성공 = 작업 공간 전체 증명 + 요구 시그니처 + **명세 강도 검사**(숨긴 `ref_pre ⇒ 에이전트 requires`, `에이전트 ensures ⇒ ref_post`를 판정용 정의로 증명). 탐색 과제의 정답은 맵에서 계산한다.
