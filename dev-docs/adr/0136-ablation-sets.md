# ADR-0136: H-UNDECIDE·H-PROBE·H-HISTORY의 절제 판 동결

- 상태: 채택(자율 채택, AUTONOMY §4 "결정 기록": 등록된 가설의 과제 집합 동결. 판정 기준은 바꾸지 않는다)
- 날짜: 2026-11-14
- 관련: ADR-0108(H-UNDECIDE), ADR-0109(H-PROBE), ADR-0110(H-HISTORY), ADR-0021(동결), ADR-0070(측정 조건), 백로그 T0-36·T0-37

## 맥락

- 세 ADR은 각자의 비교를 실행 전에 등록했다: `brb-write`·`bvb-write`에 그 한 가지만 더한 판 대 원판, 같은 커밋.
- 기능은 플래그 뒤에 있고(`bmb mcp --undecided-info`, `--probe-tool`, `set.json`의 `history`), 하네스 연결은 T0-36에서 끝났다. 남은 것은 판의 동결이었다.

## 결정

1. **판 여섯.** `bench/ablate/gen.py`가 만든다: `bvb-write-undecided`·`-probe`·`-stateless`, `brb-write-undecided`·`-probe`·`-stateless`.
2. **원판과 `set.json`의 한 키만 다르다.** `mcp_flags`에 플래그 하나를 더하거나 `"history": "stateless"`를 더한다. 프롬프트·씨앗·과제·숨은 판정은 원판의 바이트 그대로다.
3. **프롬프트는 탐침 도구를 말하지 않는다.** 모델이 보는 것은 서버의 도구 목록과 설명이다(ADR-0109 결정 4: 플래그 하나만 더한 판). 프롬프트에 안내를 더하면 변수가 둘이 된다.
4. **BRB의 Rust 팔.** 판이 없다(BMB 팔의 변형만 있다). H-HISTORY의 Rust 팔은 `harness.py --arm rust --history stateless`이고, 나머지 둘은 BMB 서버의 기능이라 Rust 팔은 원판의 것을 쓴다.
5. **게이트.** `bench ablate`: 커밋된 판이 생성기의 출력과 같음, 원판과 한 키만 다름, 판마다 서버의 도구 목록(탐침 판만 `probe` 하나 더), 대본 기준·약한 에이전트가 판마다 통과·실패(BVB 2과제, BRB 2과제), Rust 팔의 `--history stateless`.

## 결과

- 측정 머신은 판을 받는 즉시 돌릴 수 있다. 실행 순서와 횟수는 각 ADR 그대로다(sonnet 1회, qwen 순차 반복).
- 판정은 T0-37이다. H-PROBE가 반증이면 일괄 탐침의 제거 태스크를 도출한다(ADR-0120).
- 대본 에이전트는 프롬프트도 도구 설명도 읽지 않으므로, 게이트가 보이는 것은 판이 하네스를 지나간다는 것까지다. 플래그의 효과는 모델 실행만 잰다.
