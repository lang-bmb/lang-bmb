# T2-10: 충족 가능성 검사(공허 검사 W0905, 사전조건 충족 W0901)의 비용 측정 도구

결과는 `results.md`, 결정은 ADR-0072.

- `cost_driver.rs`: 측정용 시험 하나(`#[ignore]`). `compiler/tests/zz_cost.rs`로 복사해 빌드하고 복사본은 지운다(게이트의 시험이 아니다). `COST_CHECK=vac|pre`, `COST_FAIL=1`이면 실패 코퍼스, `COST_ONLY=부분문자열`이면 그 프로그램만.
- `z3log.py`: `z3` 대리. 명령을 그대로 넘기고 `(check-sat)`마다 `(get-info :rlimit)`를 물어 `Z3_LOG`에 한 행, `Z3_QDIR`에 질의 텍스트를 남긴다. 진짜 z3는 `Z3_REAL`(기본 `/usr/local/bin/z3`).
- `measure.sh OUT vac|pre`: 위 둘로 한 번 잰다(저장소 뿌리에서).
- `summarize.py OUT [fail]`: 한도·답별 질의 수와 rlimit 합, 미결정·반박 수.
- `ablate.py OUT vac|pre`: 미결정 질의의 원인 귀속(단언 한 종류씩 빼고 다시 풂).
- `sx.py`: S-식 읽기·쓰기.
