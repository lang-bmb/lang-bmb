# brb-write-stateless (H-HISTORY, 백로그 T0-37)

`bench/ablate/gen.py`가 `bench/brb-write`에서 만든 판이다(동결). 직접 고치지 않고 생성기로 다시 만든다. brb-write와 `set.json`의 한 가지만 다르다: `"history": "stateless"`(수리 차례마다 고정 문맥·현재 상태·시도마다 한 줄). 프롬프트·씨앗·과제·판정은 그대로다. 비교와 반증 조건은 ADR-0110에 실행 전에 등록돼 있다(Rust 팔은 `harness.py --history stateless`).
