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
