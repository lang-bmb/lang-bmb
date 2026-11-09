# ADR-0096: 기본 증명 문맥은 본문으로 닫지 않는다 (r2)

- 상태: 채택 (2026-11-09, 사람 승인 "권장안 채택". 백로그 2R-11)
- 날짜: 2026-11-09
- 관련: ADR-0095 결정 1(이 ADR이 대체), ADR-0019(정의 해시), ADR-0032·0043(증분 키), ADR-0059(정체성 해시의 국소성), `dev-docs/health/2R.md`, `dev-docs/measurements/syntax-uses/default-context-2026-11-08.jsonl`

## 맥락

ADR-0095 결정 1은 정의의 기본 증명 문맥을 r2c로 정했다: 자기 계약·불변식·`assert`와 피호출자의 `requires`·`ensures`가 부르는 `spec fn`의 **닫힘**(그 `spec fn`들의 본문이 부르는 것까지). 근거(2R-04a)는 m3 생성 작업 공간과 m1b 시드였다.

구현(2R-11)에서 2R-04a가 재지 않은 표준 라이브러리를 쟀다. 라이브러리는 비트 수준 술어를 일부러 다른 술어 뒤에 숨긴다(`crc32::byte`를 `byte_result` 뒤에, `sha256`·`base64`도 같은 꼴). 닫힘은 그것을 펼쳐 네 정의(`base64::encode`, `crc32::table`·`update_with`, `sha256::compress`)가 예산을 넘어 실패했고 라이브러리 증명 비용이 +384%였다.

## 결정

1. **기본 증명 문맥(r2)은 정의 자신의 절·루프 불변식·`assert`·`proof` 코드와 피호출자의 `requires`·`ensures`가 이름으로 부르는 `spec fn`이다.** 그 `spec fn`의 본문이 부르는 것은 들어가지 않는다(전이 없음). `spec fn` 자신의 문맥은 그 절만이다(본문은 아니다).
2. **`#[uses(..)]`는 그 밖의 사실을 더할 때만 쓴다.** 기본 문맥이 이미 담은 항목은 W0916이 알리고 `bmb fix`가 지운다(ADR-0095의 "도구가 지운다" 그대로).
3. **불투명성은 한 단계다.** 술어 `p`의 본문이 부르는 `q`는 `p`를 부르는 정의에게 이름일 뿐이다. 펼치려면 `#[uses(q)]`를 적는다.
4. **정체성 해시는 적힌 `#[uses]` 목록을 담는다(구문 그대로).** 기본 문맥은 피호출자의 계약에 달리므로 해시에 넣으면 피호출자 계약이 호출자의 정체성을 바꾼다(ADR-0059 국소성). 증분 키(ADR-0032·0043)는 `verify::context_of`로 실제 문맥을 쓴다.

ADR-0095의 나머지 결정(2~6)은 그대로다.

## 근거(검토한 대안)

| 안 | 판정 |
| --- | --- |
| **r2, 본문으로 닫지 않음(채택)** | 라이브러리 +0.58%, `tests/verify/pass` +0.19%, `tests/run` +4.39%, 판정 변화 0(불투명성을 시험하던 실패 코퍼스 둘은 본문 한 단계 아래로 옮김). r2c의 부분집합이고, 손으로 적던 `#[uses]`와 같은 뜻(전이 없음)이다. |
| r2c, 닫힘(ADR-0095 문구) | 라이브러리 4정의 실패, +384%. 의도된 불투명을 되살리려면 새 표면(불투명 표시)이 필요하다 |
| r2c + 불투명 표시(`#[opaque]` 등) | 표시를 빠뜨리면 증명 비용이 조용히 폭증한다(실패가 멀리서 난다). 기본이 불투명인 원칙(DESIGN §2 "정의는 기본 불투명")과 반대 방향이고, 동결 전 표면을 늘린다 |

원자료는 `dev-docs/measurements/syntax-uses/default-context-2026-11-08.jsonl`.

## 결과

- 구현은 그대로다: `verify::default_contexts`, `Tables::context`, `verify::context_of`(증명·증분 키·`weaken`), 맵의 `uses` 간선, W0916(`lint::default_uses`). 시험 `tests/verify/pass/default_context`, `tests/verify/fail/contracts`·`bit_vector`, `compiler/tests/fix_cmd.rs`의 `uses_entries_of_the_default_context_are_removed`.
- 본문 두 단계 아래를 펼쳐야 하는 증명은 `#[uses]`를 적는다(`bmb uses`가 채운다).
- ADR-0095의 상태는 "결정 1은 ADR-0096으로 대체됨"으로 바뀐다.
