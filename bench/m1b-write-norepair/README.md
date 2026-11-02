# m1b-write-norepair (H-REPAIR 절제 판, 백로그 T1-45)

`bench/repair/gen.py`가 `bench/m1b-write`에서 만든 판이다(동결). 직접 고치지 않고 생성기로 다시 만든다. m1b-write와 다른 것은 `set.json`의 `"tool_filter": "no-repair"` 하나다: m1 하네스가 모델에게 가는 모든 도구 결과에서 진단 메시지 끝의 반례(`; counterexample: ..`, `; unconfirmed counterexample: ..`), 모든 `fix`(수정 제안과 힌트), 모든 `counterexample` 필드를 지운다. 프롬프트·씨앗·과제·변이체·판정·한도는 m1b-write와 바이트 단위로 같다. 판정 규칙은 `bench/repair/README.md`.
