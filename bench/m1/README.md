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
python3 bench/m1/harness.py summarize results/*.jsonl  # ADR-0021 §4 지표 표(파일마다 한 열)
```

하네스는 과제 집합 디렉터리를 `--set DIR`로 받는다(기본은 이 디렉터리). 두 번째 집합 `m1b`와 모델 비교 절차는 `bench/m1b/README.md`(ADR-0025). `--jobs N`은 세션을 N개씩 병렬로 돌린다(CI용, 기본 1).

작성 과제의 성공 = 작업 공간 전체 증명 + 요구 시그니처 + **명세 강도 검사**(숨긴 `ref_pre ⇒ 에이전트 requires`, `에이전트 ensures ⇒ ref_post`를 판정용 정의로 증명). 탐색 과제의 정답은 맵에서 계산한다. 주어진 계약 과제(`given`, `bench/impossible`, 2A-13)는 과제의 `setup`을 시드에 커밋한 작업 공간에서 시작하고, 하네스가 `report` 도구를 더하며, 게이트 통과·`bmb weaken`(시작 대 끝)·보고로 판정한다. 판정의 `bmb` 호출은 모두 세션 작업 공간의 증명 캐시(`.bmb-cache/`, `bmb mcp`가 쓰는 것)를 거친다. 캐시는 출력 바이트를 바꾸지 않고 증명된 의무만 해법기 입력 해시로 저장하므로 판정은 같다. 세션은 프로세스마다 한 번 씨앗으로 데운 캐시의 사본에서 시작한다(INF-08).

## 실제 모델 실행(사용자 머신)

실제 모델 측정(백로그 1E-10)은 유료 API를 쓰므로 사용자 머신에서 실행한다. 이 리포의 클라우드 세션은 코드와 실행 가능한 검증만 맡는다.

```
# 1. 도구: Rust(rust-toolchain.toml이 고정), Z3 4.15.4, LLVM 18
./scripts/setup.sh                       # z3(pip)·LLVM 확인/설치. macOS면 z3·llvm@18을 직접 설치
cargo build                              # target/debug/bmb
cargo run -q -- doctor                   # z3·llvm 버전 확인(둘 다 "ok"여야 함)

# 2. 기준 해법으로 하네스 점검(무료)
python3 bench/m1/harness.py run --agent oracle --expect all-pass

# 3. 실제 모델(유료)
pip install anthropic
export ANTHROPIC_API_KEY=...             # 또는 `ant auth login`
python3 bench/m1/harness.py run --agent anthropic:claude-opus-5-5 \
    --out bench/m1/results/$(date +%F)-claude-opus-5-5.jsonl
```

- 과제당 세션은 한 번이다. 실패해도 다시 돌리지 않는다(ADR-0021 §3).
- `--tasks w01-clamp`로 한 과제만 먼저 돌려 비용과 동작을 확인할 수 있다. 이렇게 돌린 결과는 보고에 섞지 않는다.
- 결과 파일을 커밋하면 요약(1E-06)은 세션에서 작성한다.
