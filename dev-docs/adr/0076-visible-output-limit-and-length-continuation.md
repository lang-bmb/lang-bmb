# ADR-0076: 세션 출력 한도는 보이는 출력만 세고, 출력 한도에서 끊긴 응답은 같은 차례에서 한 번 이어 쓴다

- 상태: 채택 (사람 승인 2026-11-01 "권장안 채택, 승인사항 승인": 백로그 T0-16의 권장안 (나)). ADR-0066 §3(측정 조건)과 ADR-0070(같은 조건)을 보완한다
- 날짜: 2026-11-01
- 관련: ADR-0021 §3(세션 규칙), ADR-0034 §2(공정성), ADR-0066, ADR-0070, 백로그 T0-16, T0-27, T1-40

## 맥락

- T0-10(qwen3.8-27b)에서 세션이 출력 길이로 끝난 경우가 많았다(`dev-docs/measurements/t0-10.md` §3.2·§3.4, `bench/failures.py`). `length`는 m1 1·m1b 1·BVB 6·Verus 1이고, 그중 일부는 첫 응답이 출력 32,768토큰을 모두 추론에 쓰고 도구 호출 없이 끝났다(커밋 0).
- BRB의 BMB 팔(중간 집계)에서는 미제출 18 중 11이 세션 출력 한도 64,000에서 끝났다. 몇 응답이 각각 10,000~21,000토큰을 추론에 썼다. 첫 응답 `length`는 7이다.
- 한도는 추론 모델 이전에 정한 것이고, API가 세는 출력 토큰에는 추론이 들어 있다. 그래서 추론 모델에서는 측정이 언어나 도구가 아니라 토큰 예산을 쟀다. qwen의 BRB 2/24(sonnet 21/24)는 이 조건에 섞여 있다(utility §2.7, 검토 `dev-docs/reviews/hypotheses-2026-11-01.md`의 H-SMALL).

## 결정

1. **보이는 출력.** 모델 어댑터(`bench/m1/model_api.py`의 `OpenAIAgent`, `bench/m1/anthropic_agent.py`의 `AnthropicAgent`)는 응답마다 보이는 출력을 센다. 보이는 출력은 텍스트와 각 도구 호출의 이름·인자이고, 바이트 수를 4로 나눠 올림한다(`model_api.visible_tokens`). 결과 행의 `model_usage.visible_output_tokens`가 그 합이다. API가 추론 토큰을 따로 알려 주면(`completion_tokens_details.reasoning_tokens`) `model_usage.reasoning_tokens`에 기록하되 판정·한도에는 쓰지 않는다. `output_tokens`(API가 센 출력 전체)는 그대로 기록한다.
2. **세션 출력 한도.** BRB(`bench/brb/harness.py`)와 BRB-encap의 세션 한도 64,000(`SESSION_OUTPUT_TOKENS`)은 보이는 출력의 합으로 확인한다(`brb.session_output`). 값은 그대로다.
3. **출력 한도에서 끊긴 응답.** 응답이 출력 한도로 끝나면(OpenAI `finish_reason: "length"`, Anthropic `stop_reason: "max_tokens"`) 어댑터는 같은 차례에서 한 번 더 요청한다. 끊긴 응답의 텍스트는 대화에 남기고, 그 안의 도구 호출은 버린다(인자가 잘렸을 수 있고, Anthropic에서는 결과가 없는 `tool_use`를 보낼 수 없다). 그 뒤에 고정된 하네스 문구(`model_api.CONTINUE_NOTE`)를 사용자 차례로 붙인다. 보이는 텍스트가 없으면 문구는 마지막 사용자 차례에 덧붙인다(역할이 번갈아야 한다). 이어 쓴 응답도 출력 한도로 끝나면 세션은 지금처럼 그 이유(`length`·`max_tokens`)로 끝난다. 한 차례에 한 번이고, 도구 호출이 있는 응답이 오면 다음 차례는 다시 한 번 이어 쓸 수 있다.
4. **같은 규칙.** 두 공급자, 모든 집합(m1 계열, BVB와 Verus 팔, BRB와 그 변형, BRB-encap)에 같은 규칙이다. 응답당 출력 한도(`MAX_TOKENS`: qwen 32,768, Anthropic 16,000)와 도구 호출 40회는 그대로다.
5. **측정 조건.** 어댑터와 BRB 하네스가 바뀌므로 새 조건이다(ADR-0070 (c)). BRB 하네스는 BRB MANIFEST에 들어 있어 다시 동결했고, 그것을 담는 `brb-write`·`brb-diff`·`brb-ast`·`brb-encap`의 MANIFEST도 다시 썼다(과제·프롬프트·판정은 그대로, ADR-0070 "계기 코드를 고치면 MANIFEST를 다시 쓴다"). 기록된 결과(T0-10, T0-11)는 그대로 두고 이 조건과 섞어 보고하지 않는다. 이 조건의 첫 정식 측정(T0-13)에서 비교할 원판(m1, m1b, BVB·Verus, BRB 두 팔)을 같은 커밋에서 모델마다 한 번씩 다시 돈다. 그 뒤 비교(T0-14·T0-15·T0-17 등)는 이 행들을 기준으로 한다.

## 근거

- **원인과 귀속 층:** 측정 조건(프로세스). 한도가 추론 토큰을 세어, 같은 한도가 추론 모델과 그렇지 않은 모델에 다른 뜻이 된다(S1 공정성, ADR-0034 §2).
- **정석:** 추론 모델 평가는 추론 예산을 출력 예산과 따로 두거나 보이는 출력만 센다. 추론 길이를 API로 고정하는 방법(T0-16 후보 (다))은 공급자마다 다르고 qwen 게이트웨이에서 확인되지 않아 버렸다.
- **바이트/4로 세는 이유:** 공급자가 추론 토큰을 따로 알려 주는지에 기대지 않고, 모델이 실제로 보이게 낸 것만으로 두 모델에 같은 규칙을 준다. 하네스는 이미 읽기 양을 같은 방식으로 센다(`harness.py`의 질의당 토큰, ceil(bytes/4)).
- **한 번만 이어 쓰는 이유:** 끊긴 첫 응답이 세션 전체를 끝내던 것을 막되, 끝없는 추론이 세션을 무한히 늘리지 않게 한다. 도구 호출 한도 40회는 이어 쓴 요청을 세지 않는다(도구 호출이 아니다). 요청 수는 `model_usage.requests`에 남는다.
- **기각한 것:** (가) 지금 조건 그대로(T0-16 후보 (가)). qwen 결과가 계속 토큰 예산을 잰다. 세션 한도를 없애는 것은 ADR-0034 §2가 두 팔에 같은 예산을 요구하므로 하지 않는다.

## 결과

- 시험: `bench/m1/test_model_api.py`의 `test_length_is_continued_once_per_turn`·`test_reasoning_tokens_are_recorded_apart`, `bench/m1/test_anthropic_agent.py`의 `test_max_tokens_is_continued_once_per_turn`·`test_max_tokens_with_nothing_visible_joins_the_last_user_turn`, `bench/brb/test_brb.py`의 `test_output_token_cap`·`test_reasoning_does_not_count_toward_the_cap`.
- 결과 행: `model_usage`에 `visible_output_tokens`(언제나)·`reasoning_tokens`(API가 줄 때)가 더해지고, 에이전트 설정(`agent_settings`)에 `"length": "continue once per turn"`이 붙는다. BRB 결과 머리의 `limits`에 `"session_output_counts": "visible (ADR-0076)"`.
- 같은 커밋에서 BRB BMB 팔의 커밋 계수(`commits_ok`·`commits_rejected`, T1-40)도 들어갔다. BRB 재동결을 한 번으로 하기 위해서다.
- 되돌리기: 어댑터의 이어 쓰기 분기를 지우고 `session_output`이 `output_tokens`를 돌려주면 이전 조건이다.
