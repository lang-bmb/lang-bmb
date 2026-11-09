# BRB-HARD: 테스트가 놓치기 쉬운 결함 (H-ORACLE, 백로그 T0-08)

BRB(`../brb`)의 새 판이다. 가설 H-ORACLE은 "증명 통과는 테스트 통과보다 숨은 결함을 덜 놓친다"이다. BRB에서는 판정할 수 없었다(T0-07, T0-10, T0-11). 두 모델 모두 Rust 쪽에서 "테스트 통과 & 숨은 실패"가 0이었고, 과제가 쉬웠다. 이 판은 결함이 드문 입력에 숨는 과제로 그 조건을 만든다. 중복 값, 용량을 꼭 채우는 물건, 빈 입력, 표현 폭 끝의 중간값이 그런 입력이다.

이 디렉터리는 과제(동결)·하네스·CI 검사까지다. 실제 모델 실행은 아직 없다.

## 출처 (ADR-0103 결정 4)

과제 12개는 모두 **QuixBugs**에서 옮겼다(D. Lin, J. Koppel, A. Chen, A. Solar-Lezama, "QuixBugs: A Multi-Lingual Program Repair Benchmark Set Based on the Quixey Challenge", SPLASH Companion 2017; https://github.com/jkoppel/QuixBugs, MIT 라이선스, 사본 `QUIXBUGS-LICENSE`).

- QuixBugs의 프로그램마다 사람이 찾아 문서화한 한 줄 결함과 그 수정본이 있다.
- 옮긴 방식은 셋이다.
  - 문제: 프로그램의 설명문을 BRB의 타입(`i64`·`u64`·`usize`·`bool`, `&[i64]`·`&mut [i64]`)으로 다시 썼다.
  - 결함: 문서화된 결함을 과제의 Rust 변이체 0번으로 다시 구현했다.
  - 대조군: 같은 결함 계열을 BMB·Rust 약한 대조군에 썼다.
- 원본 Python 코드는 가져오지 않았다.
- 과제 파일 첫 줄 주석에 원본 프로그램 이름과 그 결함을 적었다.

작성자가 고른 것은 셋이다. 원본 40개 가운데 BRB 타입으로 옮길 수 있고 결함이 드문 입력에 숨는 12개를 골랐다. 정의역의 크기 상한도 정했다. h12는 결함 계열을 하나 더했다(표현 폭: 곱셈 공식의 중간값 넘침).

| 과제 | QuixBugs | 문서화된 결함 | 숨기 쉬운 입력 |
| --- | --- | --- | --- |
| h01 first_in_sorted | `find_first_in_sorted` | `lo <= hi`(끝 너머 읽기) | x가 모든 원소보다 큼, 중복 |
| h02 kth_smallest | `kth` | 위쪽으로 내려갈 때 `k`를 줄이지 않음 | 중복, 피벗 위의 답 |
| h03 lis_length | `lis` | `longest = length + 1`(최댓값 아님) | 뒤에서 짧은 수열을 늘림, 중복 |
| h04 next_permutation | `next_permutation` | 교환 상대의 비교 방향 | 중복, 마지막 순열 |
| h05 count_change | `possible_change` | 동전 종류가 떨어진 경우의 기저 없음 | 빈 동전, 합 0 |
| h06 knapsack | `knapsack` | `weight < j`(꼭 맞는 물건 제외) | 용량을 꼭 채우는 물건 |
| h07 edit_distance | `levenshtein` | 일치에도 비용 1 | 긴 공통 부분 |
| h08 common_substring | `lcs_length` | `dp[i-1][j]`(대각선 아님) | 긴 공통 구간 |
| h09 count_primes | `sieve` | `any`를 `all` 자리에 | 소수의 제곱 |
| h10 counting_sort | `bucketsort` | 출력 고리가 입력을 셈 | 중복, 최댓값 k-1 |
| h11 balanced | `is_valid_parenthesization` | 끝의 깊이 검사 없음 | 닫히지 않은 괄호 |
| h12 binomial | `pascal` | 행마다 한 칸 짧음 | 행의 마지막 칸, n이 60대(넘침) |

## 세션 조건

BRB와 같다. 판정기(`bench/brb/judge.py`), 도구, 한도(도구 호출 40, 세션 모델 출력 64,000 토큰), Rust 샌드박스, 세션 뒤 자기 검사(BMB `verify`, Rust `cargo test`)를 그대로 쓴다. 하네스는 BRB의 동결 하네스를 라이브러리로 불러 이 판의 과제로 돈다.

BMB 팔은 정식 쓰기 표면(ADR-0101)으로 돈다. 그래서 `bench/brb-write`의 `bmb mcp` 플래그를 쓰고, 그 판의 MANIFEST를 확인한다.

BMB 프롬프트(`prompt-bmb.md`)는 쓰기 판의 프롬프트에서 한 규칙만 다르다. 정수 `as` 변환을 허용한다(값이 그 타입에 들어감을 증명해야 함). h05·h06·h10은 원소 값을 배열 색인으로 써야 하는데, M1 규칙에는 그 변환이 없다. Rust 프롬프트는 BRB 그대로다.

기준 해법(BMB)은 지금 컴파일러가 받는 언어로 썼다(`return` 등). 기준 해법은 게이트에서 판정기·도구 사슬을 확인할 뿐 에이전트에게 보이지 않는다.

## 판정 (실행 전 등록, 2026-11-09)

1. **H-ORACLE 판정 줄은 BRB의 `oracle_verdict` 그대로다.** 기준은 자기 검사를 통과한 과제 가운데 숨은 결함이 있는 비율이다.
   - BMB 비율 ≥ Rust 비율이면 **반증**, BMB 비율 < Rust 비율이면 **지지**다.
   - Rust 쪽 "통과 & 숨은 실패"가 0이면 **판정 불가**다.
2. **반복(ADR-0103 결정 2).** 첫 qwen 실행에서 다음 중 하나이면 같은 커밋·조건에서 3회까지 반복하고, ADR-0077 §3의 구간으로 판정한다.
   - 판정 줄이 "판정 불가"다.
   - 두 비율의 차이가 0.30 안이다.
   - 반복 여부는 첫 실행의 이 두 수로만 정하고, 결과의 방향으로 정하지 않는다.
3. **주장의 기준 모델은 sonnet이다(ADR-0103 결정 3).** sonnet은 같은 커밋에서 한 번 돈다. 집합 전체의 판정은 sonnet의 판정 줄이고, qwen은 같은 표에 나란히 보고한다.
4. **변별력 확인.** 이 판이 변별 과제라는 주장은 sonnet Rust 팔의 "통과 & 숨은 실패"가 1 이상일 때만 성립한다. 0이면 결과에 "이 집합도 변별 실패"로 적는다. 그 경우 과제를 고쳐 다시 돌리지 않는다(사후 조정 금지).
5. G5·G8 줄(BRB 요약)도 함께 보고하지만 이 판의 목적 판정이 아니다.

## 파일

| 파일 | 내용 |
| --- | --- |
| `tasks/hNN_*.py` | 과제 12개(동결). BRB 과제 형식 그대로: 문장·정의역·시그니처, Python 모델, 케이스 생성기(과제당 316~2,437개, 합계 6,845개), 기준 해법과 약한 대조군(BMB·Rust), Rust 변이체 4개(0번이 QuixBugs의 결함) |
| `prompt-bmb.md` | BMB 시스템 프롬프트(동결, 위 한 규칙만 쓰기 판과 다름) |
| `harness.py` | 세션 실행·판정·요약(동결) |
| `QUIXBUGS-LICENSE` | 출처의 라이선스 |
| `MANIFEST` | 위 파일들과 이 README, BRB·`brb-write`의 MANIFEST의 SHA-256. 다르면 하네스가 실행을 거부한다 |
| `test_brb_hard.py` | 게이트 |

```
python3 bench/brb-hard/harness.py manifest                                   # 동결 확인
python3 bench/brb-hard/harness.py run --arm bmb  --agent oracle --jobs 4     # 기준 해법: 결함 0 (CI)
python3 bench/brb-hard/harness.py run --arm rust --agent oracle --proptest off --jobs 4
python3 bench/brb-hard/harness.py run --arm bmb  --agent weak --jobs 4       # 약한 대조군: 자기 검사 통과, 숨은 판정이 결함 검출 (CI)
python3 bench/brb-hard/harness.py run --arm rust --agent weak --proptest off --jobs 4
python3 bench/brb-hard/harness.py adequacy --jobs 4                          # 숨은 케이스가 Rust 변이체 48개를 모두 죽인다 (CI)
python3 bench/brb-hard/harness.py run --arm bmb --agent openai:qwen3.8-27b --jobs 2 --sessions S --out B.jsonl
python3 bench/brb-hard/harness.py run --arm rust --agent openai:qwen3.8-27b --jobs 2 --sessions S --out R.jsonl
python3 bench/brb-hard/harness.py summarize B.jsonl R.jsonl                  # 표, G5·H-ORACLE 줄
```
