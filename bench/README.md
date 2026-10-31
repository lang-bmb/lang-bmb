# 벤치

| 디렉터리 | 벤치 | 모델 |
| --- | --- | --- |
| `m1`, `m1b` | 작성·탐색 1회 성공률, BSB 판별률(ADR-0021, 0025, 0035) | 필요 |
| `bvb` | Verus 이식 과제, 원본 Verus와 비교(ADR-0038) | 필요 |
| `impossible` | BMB-Impossible: 불가능 계약에서 정직 보고 대 게이트 우회(H-GAME, 2A-13) | 필요(증명서·대조군은 불필요) |
| `brb` | BMB+증명 대 Rust+테스트(G5, H-ORACLE, G8) | 필요 |
| `brb-encap` | 자료 구조 수정: 타입 불변식 대 private + 테스트(H-ENCAP, T0-09) | 필요(기준·약한·게이밍 해법은 불필요) |
| `bnb` | 맵 도구 대 파일 도구(Rust 쌍둥이) | 오라클 팔은 불필요 |
| `bcb` | 누적 검증 비용(ADR-0042) | 불필요 |
| `m3` | 생성 코드베이스 | 불필요 |
| `baseline` | 성능 기준선(안전 Rust 대비) | 불필요 |

## 실제 모델 측정: 명령 하나 (`run_models.py`, 백로그 INF-14)

모델이 필요한 동결 과제 집합 여섯(m1, m1b, bvb, bvb-verus, brb-bmb, brb-rust)을 정해진 순서로 기존 하네스에 돌리고, 모든 요약과 지표별 채움 여부를 한 보고서로 낸다. 각 하네스가 자기 MANIFEST를 확인하고 1회·재시도 없음 규칙(ADR-0021)을 지킨다.

```
python3 bench/run_models.py --dry-run                     # 대본 기준 해법으로 끝까지(비용 0). 게이트는 --limit 2
python3 bench/run_models.py --agent openai:qwen3.8-27b \
    --price-in 0 --price-out 0 --budget 0 --report report.md        # 기본 모델(ADR-0066, 로컬)
python3 bench/run_models.py --agent anthropic:claude-sonnet-5-5 \
    --price-in P --price-out Q --budget USD --report report.md      # 프런티어 교차 비교
python3 bench/run_models.py --agent ... --sets brb-bmb,brb-rust   # 일부만, 또는 예산으로 멈춘 뒤 나머지
```

- **모델 환경(ADR-0066).** 에이전트 종류는 `openai:MODEL`(Chat Completions)과 `anthropic:MODEL`(Messages API)이고, 주소 기본값은 이 환경의 게이트웨이 `https://token.iyulab.com`(`/openai/v1`, `/anthropic/v1`)다. `BMB_OPENAI_BASE_URL`·`BMB_ANTHROPIC_BASE_URL`로 바꾼다. 자격 증명은 환경이 넣고 저장소에는 키가 없다(`bench/m1/model_api.py`). 기본은 `qwen3.8-27b`, `claude-sonnet-5-5`는 같은 집합·같은 커밋의 교차 비교에만 쓴다. 각 하네스의 `run --agent`도 같은 문자열을 받는다.

- **가격은 인자다.** 가격이 바뀌므로 제공자의 가격표에서 백만 토큰당 USD를 넣는다.
- **예산 장치.** 집합이 끝날 때마다 기록된 토큰으로 지출을 계산하고, 다음 집합을 시작하기 전에 (지금까지 세션당 평균 비용 × 그 집합의 세션 수)로 예상한다. 지출 + 예상이 예산을 넘으면 그 집합부터 시작하지 않는다. 집합을 중간에 끊지 않으므로 한 집합만큼 넘을 수 있다. 첫 집합(m1, 15세션)에는 예상이 없다.
- **중단과 재개(INF-16).** 하네스는 세션이 끝날 때마다 결과 행을 덧붙이고(첫 줄은 실행을 밝히는 `partial` 머리), 집합이 끝나면 과제 순서대로 다시 써서 요약 행을 붙인다. 같은 인자로 다시 실행하면 이 커밋·모델에서 끝난 집합은 다시 돌지 않고 보고에 넣으며, 반쯤 돈 집합은 행이 없는 과제부터 잇는다. 행이 쓰인 과제는 다시 돌지 않으므로 재시도가 아니다. 이어 돈 횟수는 요약의 `resumes`에 남는다.
- **시작 전 거부.** 커밋되지 않은 트리(`+dirty` 결과는 보고에 쓰지 않는다), MANIFEST 불일치, bvb-verus를 골랐는데 Verus 없음(`bench/bvb/verus/install.sh`). 같은 날짜·모델의 다른 커밋 결과 파일이 이미 있으면 덮어쓰지 않는다. 미커밋 판정은 결과 파일(`bench/*/results/`)을 세지 않는다.
- **결과 위치.** `bench/<집합>/results/<날짜>-<모델>[-verus|-bmb|-rust].jsonl`. 보고서(1E-06)는 이 파일들로 쓴다.
- **한 번에 채우는 지표.** G2(m1·m1b 작성 1회 성공률), G3(m1b BSB 판별률), H-SPEC 보조(BVB 명세 강도), H-FAMILIAR(BVB BMB 대 Verus, Verus 필요), G5·H-ORACLE·G8(BRB). 결과 행에 최종 파일이 남아(BRB `final_sources`) 나중에 정한 지표도 다시 돌리지 않고 계산한다.
- **차례별 문맥 기록(선택, 백로그 2D-25).** `BMB_CONTEXT_LOG=1`이면 결과 행에 요청별 입력·출력 토큰(`model_usage.request_input_tokens`·`request_output_tokens`)과 호출별 도구·결과 바이트(`calls`)가 더해진다. 기본은 꺼져 결과 행이 그대로이고, 모델이 보는 것은 켜도 같다. 분해 도구는 `dev-docs/experiments/context-size/`.
- 게이트(`bench run_models (dry run)`)는 Verus 없이 돌아 H-FAMILIAR를 "건너뜀"으로 보고하고 나머지 줄이 모두 채워지는지 확인한다.
