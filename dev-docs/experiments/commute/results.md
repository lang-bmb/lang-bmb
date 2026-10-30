# H-COMMUTE: 동시 편집의 병합 (백로그 T1-12)

> 2026-10-28. 컴파일러는 2N-01b(`62751306`) 이후 바뀌지 않은 `target/debug/bmb`. 생성: 아래 각 절의 명령. 가설과 교환 관계의 정의는 `dev-docs/reviews/collaboration-2026-10-28.md` §4·§7.
> 2026-10-29: 의미 충돌 집합(T1-19)과 ADR-0065 판정(T1-27)의 재측정은 아래 "재측정" 절. 그 절이 첫 재생의 판정 규칙과 거짓 충돌 수치를 새 정답으로 다시 본다. 이 앞의 절은 첫 재생 그대로다.
> 2026-10-30: 충돌 쌍의 겹친 정의 재증명(T1-29, ADR-0065 결정 4)은 끝의 "비교환 쌍의 해소" 절.
> 2026-10-30: 겹친 읽기가 큰 편집에서 정제 면제와 재증명의 비용(T1-31)은 끝의 "겹친 읽기가 큰 편집" 절.

## 요약

| 설정 | 쓴 쌍 | 텍스트 충돌(BMB) | 정의 단위 쓰기 충돌 | git 깨끗한 병합 중 증명 실패 | 독립 판정 | 독립 판정의 증명 실패(건전성 위반) | 충돌 판정인데 병합이 증명됨 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| contract-heavy, 각자 커밋 1개 | 40 | 0 | 0 | 0/40 | 40 | 0 | 0 |
| contract-heavy, 각자 커밋 10개 | 38 | 9 | 10 | 0/29 | 20 | 0 | 8 |
| body-heavy, 각자 커밋 10개 | 35 | 9 | 8 | 0/26 | 17 | 0 | 10 |
| contract-heavy, 각자 커밋 40개 | 3 | 3 | 3 | 0/0 | 0 | 0 | 0 |

## 해석

- **건전성(가설의 첫 절):** 독립으로 판정한 77쌍을 정의 단위로 합친 결과가 모두 증명되었다. 반증되지 않았다. 다만 아래 셋째 항목 때문에 이 결과는 약하다.
- **거짓 충돌(가설의 둘째 절):** 정의 단위 쓰기 충돌(10, 8)은 텍스트 충돌(9, 9)과 같은 수준이다. "텍스트 병합보다 거짓 충돌이 적다"는 이 데이터로 지지되지 않는다. 이력의 편집 단위가 정의 하나여서 두 기준이 같은 곳에서 부딪친다.
- **git의 거짓 병합이 한 번도 나오지 않았다(0/95).** BCB 이력은 한 줄의 순차 이력에서 나온 편집이라, 따로 적용한 두 편집이 서로의 계약을 어기는 경우를 거의 만들지 않는다. 그래서 이 실험은 교환 관계가 의미 충돌을 잡는 힘을 재지 못했다. 의미 충돌을 일부러 만드는 생성기(한쪽은 계약을 강화하고 다른 쪽은 옛 계약에 기대는 호출을 더함)가 필요하다(백로그 T1-19).
- **교환 판정은 보수적이다.** 충돌로 판정한 37쌍 중 18쌍은 합쳐도 증명되었다. 계약 변경의 정제 면제(2D-19)가 없고, 읽기 집합을 L2 간선 전체로 잡기 때문이다.
- **각자 커밋 40개는 쓸 수 없다.** 37쌍 중 34쌍에서 B가 A의 편집(추가된 잎, 바뀐 계약)에 기대어 혼자서는 적용되거나 증명되지 않았다. 순차 이력을 쪼개 동시 작업을 흉내 내는 방식의 한계다.

## 프로필 `contract-heavy`, 정의 1000개, 각자 커밋 1개

생성: `python3 dev-docs/experiments/commute/run.py --profile contract-heavy --span 1 --every 25`. 기준 커밋 c에서 A = 커밋 c+1..c+1, B = 커밋 c+2..c+2를 각자 적용한 쌍.

| 항목 | 값 |
| --- | ---: |
| 표본 쌍 | 40 |
| 건너뜀(B가 A에 기댐) | 0 |
| 쓴 쌍 | 40 |
| 텍스트 충돌(BMB 단위 파일, `git merge-file`) | 0 |
| 텍스트 충돌(Rust 쌍둥이) | 0 |
| 깨끗한 텍스트 병합 중 증명 실패(거짓 병합) | 0 / 40 |
| 정의 단위 쓰기 충돌 | 0 |
| 정의 단위 병합 중 증명 실패 | 0 / 40 |
| 교환 판정: 독립 | 40 |
| **독립 판정인데 병합이 증명 실패(건전성 위반)** | 0 |
| 충돌 판정인데 정의 단위 병합이 증명됨(보수적 판정) | 0 |
| 충돌 판정인데 텍스트로는 깨끗이 병합 | 0 |

| k | A 편집 | B 편집 | 쓰기 A·B | 독립 | 텍스트 충돌 BMB·Rust | 텍스트 병합 증명 | 정의 충돌 | 정의 병합 증명 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 3 | 3 | 3·3 | True | 0·0 | True | False | True |
| 25 | 3 | 1 | 4·1 | True | 0·0 | True | False | True |
| 50 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 75 | 1 | 1 | 1·2 | True | 0·0 | True | False | True |
| 100 | 2 | 1 | 3·1 | True | 0·0 | True | False | True |
| 125 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 150 | 1 | 3 | 1·3 | True | 0·0 | True | False | True |
| 175 | 2 | 3 | 2·3 | True | 0·0 | True | False | True |
| 200 | 1 | 3 | 1·3 | True | 0·0 | True | False | True |
| 225 | 1 | 2 | 1·2 | True | 0·0 | True | False | True |
| 250 | 1 | 1 | 2·1 | True | 0·0 | True | False | True |
| 275 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 300 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 325 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 350 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 375 | 2 | 1 | 2·2 | True | 0·0 | True | False | True |
| 400 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 425 | 1 | 2 | 1·3 | True | 0·0 | True | False | True |
| 450 | 1 | 1 | 2·1 | True | 0·0 | True | False | True |
| 475 | 1 | 3 | 1·4 | True | 0·0 | True | False | True |
| 500 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 525 | 3 | 1 | 4·1 | True | 0·0 | True | False | True |
| 550 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 575 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 600 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 625 | 1 | 3 | 1·4 | True | 0·0 | True | False | True |
| 650 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 675 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 700 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 725 | 1 | 2 | 2·2 | True | 0·0 | True | False | True |
| 750 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 775 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |
| 800 | 1 | 1 | 2·1 | True | 0·0 | True | False | True |
| 825 | 3 | 1 | 4·1 | True | 0·0 | True | False | True |
| 850 | 2 | 3 | 3·3 | True | 0·0 | True | False | True |
| 875 | 3 | 1 | 5·1 | True | 0·0 | True | False | True |
| 900 | 1 | 2 | 2·3 | True | 0·0 | True | False | True |
| 925 | 3 | 1 | 4·2 | True | 0·0 | True | False | True |
| 950 | 1 | 1 | 1·1 | True | 0·0 | True | False | True |
| 975 | 2 | 1 | 2·1 | True | 0·0 | True | False | True |

## 프로필 `contract-heavy`, 정의 1000개, 각자 커밋 10개

생성: `python3 dev-docs/experiments/commute/run.py --profile contract-heavy --span 10 --every 25`. 기준 커밋 c에서 A = 커밋 c+1..c+10, B = 커밋 c+11..c+20를 각자 적용한 쌍.

| 항목 | 값 |
| --- | ---: |
| 표본 쌍 | 40 |
| 건너뜀(B가 A에 기댐) | 2 |
| 쓴 쌍 | 38 |
| 텍스트 충돌(BMB 단위 파일, `git merge-file`) | 9 |
| 텍스트 충돌(Rust 쌍둥이) | 9 |
| 깨끗한 텍스트 병합 중 증명 실패(거짓 병합) | 0 / 29 |
| 정의 단위 쓰기 충돌 | 10 |
| 정의 단위 병합 중 증명 실패 | 0 / 28 |
| 교환 판정: 독립 | 20 |
| **독립 판정인데 병합이 증명 실패(건전성 위반)** | 0 |
| 충돌 판정인데 정의 단위 병합이 증명됨(보수적 판정) | 8 |
| 충돌 판정인데 텍스트로는 깨끗이 병합 | 10 |

| k | A 편집 | B 편집 | 쓰기 A·B | 독립 | 텍스트 충돌 BMB·Rust | 텍스트 병합 증명 | 정의 충돌 | 정의 병합 증명 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 23 | 13 | 25·14 | False | 2·2 | None | True | None |
| 25 | 16 | 17 | 19·16 | True | 0·0 | True | False | True |
| 50 | 16 | 15 | 20·15 | True | 0·0 | True | False | True |
| 75 | 13 | 12 | 15·13 | True | 0·0 | True | False | True |
| 100 | 16 | 19 | 17·23 | False | 0·0 | True | False | True |
| 125 | 15 | 15 | 16·18 | False | 2·2 | None | True | None |
| 150 | 22 | 17 | 24·19 | False | 1·1 | None | True | None |
| 175 | 19 | 14 | 21·15 | False | 0·0 | True | False | True |
| 200 | 14 | 13 | — | — | — | — | — | B does not apply alone: KeyError |
| 225 | 15 | 12 | 18·13 | True | 0·0 | True | False | True |
| 250 | 15 | 19 | 17·19 | False | 0·0 | True | False | True |
| 275 | 13 | 15 | 16·18 | True | 0·0 | True | False | True |
| 300 | 14 | 16 | 14·19 | False | 0·0 | True | False | True |
| 325 | 15 | 17 | — | — | — | — | — | B does not apply alone: KeyError |
| 350 | 14 | 15 | 16·18 | True | 0·0 | True | False | True |
| 375 | 18 | 14 | 21·16 | True | 0·0 | True | False | True |
| 400 | 16 | 13 | 17·16 | True | 0·0 | True | False | True |
| 425 | 15 | 14 | 16·15 | True | 0·0 | True | False | True |
| 450 | 14 | 13 | 15·15 | False | 1·2 | None | True | None |
| 475 | 16 | 15 | 18·16 | False | 1·2 | None | True | None |
| 500 | 13 | 14 | 14·15 | False | 0·0 | True | True | None |
| 525 | 22 | 19 | 26·19 | False | 0·0 | True | False | True |
| 550 | 17 | 14 | 18·14 | False | 1·1 | None | False | True |
| 575 | 16 | 15 | 17·14 | True | 0·0 | True | False | True |
| 600 | 16 | 15 | 18·16 | True | 0·0 | True | False | True |
| 625 | 15 | 14 | 17·16 | True | 0·0 | True | False | True |
| 650 | 14 | 16 | 14·16 | False | 0·0 | True | False | True |
| 675 | 13 | 17 | 13·19 | True | 0·0 | True | False | True |
| 700 | 12 | 13 | 13·10 | True | 0·0 | True | False | True |
| 725 | 22 | 13 | 29·13 | False | 0·0 | True | True | None |
| 750 | 15 | 20 | 16·23 | True | 1·1 | None | False | True |
| 775 | 14 | 15 | 16·14 | False | 1·1 | None | True | None |
| 800 | 11 | 16 | 12·15 | True | 0·0 | True | False | True |
| 825 | 15 | 19 | 16·24 | True | 0·0 | True | False | True |
| 850 | 18 | 13 | 21·15 | False | 1·1 | None | True | None |
| 875 | 15 | 14 | 18·15 | False | 0·0 | True | False | True |
| 900 | 14 | 12 | 18·15 | True | 0·0 | True | False | True |
| 925 | 17 | 20 | 21·22 | True | 0·0 | True | False | True |
| 950 | 10 | 15 | 8·17 | True | 0·0 | True | False | True |
| 975 | 13 | 13 | 14·13 | False | 0·0 | True | True | None |

## 프로필 `body-heavy`, 정의 1000개, 각자 커밋 10개

생성: `python3 dev-docs/experiments/commute/run.py --profile body-heavy --span 10 --every 25`. 기준 커밋 c에서 A = 커밋 c+1..c+10, B = 커밋 c+11..c+20를 각자 적용한 쌍.

| 항목 | 값 |
| --- | ---: |
| 표본 쌍 | 40 |
| 건너뜀(B가 A에 기댐) | 5 |
| 쓴 쌍 | 35 |
| 텍스트 충돌(BMB 단위 파일, `git merge-file`) | 9 |
| 텍스트 충돌(Rust 쌍둥이) | 9 |
| 깨끗한 텍스트 병합 중 증명 실패(거짓 병합) | 0 / 26 |
| 정의 단위 쓰기 충돌 | 8 |
| 정의 단위 병합 중 증명 실패 | 0 / 27 |
| 교환 판정: 독립 | 17 |
| **독립 판정인데 병합이 증명 실패(건전성 위반)** | 0 |
| 충돌 판정인데 정의 단위 병합이 증명됨(보수적 판정) | 10 |
| 충돌 판정인데 텍스트로는 깨끗이 병합 | 9 |

| k | A 편집 | B 편집 | 쓰기 A·B | 독립 | 텍스트 충돌 BMB·Rust | 텍스트 병합 증명 | 정의 충돌 | 정의 병합 증명 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 17 | 14 | 18·14 | False | 1·1 | None | True | None |
| 25 | 15 | 12 | 16·12 | True | 0·0 | True | False | True |
| 50 | 15 | 14 | 14·15 | False | 0·0 | True | False | True |
| 75 | 12 | 19 | 14·20 | True | 0·0 | True | False | True |
| 100 | 16 | 16 | — | — | — | — | — | B does not apply alone: KeyError |
| 125 | 14 | 15 | 13·16 | False | 0·0 | True | False | True |
| 150 | 17 | 12 | 18·11 | True | 0·0 | True | False | True |
| 175 | 14 | 14 | 14·14 | True | 0·0 | True | False | True |
| 200 | 19 | 17 | 19·19 | False | 0·0 | True | False | True |
| 225 | 12 | 12 | 13·12 | True | 0·0 | True | False | True |
| 250 | 14 | 15 | — | — | — | — | — | B alone does not prove |
| 275 | 15 | 15 | — | — | — | — | — | B alone does not prove |
| 300 | 16 | 15 | 16·17 | False | 0·0 | True | False | True |
| 325 | 17 | 18 | 17·19 | False | 0·0 | True | False | True |
| 350 | 17 | 16 | 18·16 | True | 0·0 | True | False | True |
| 375 | 21 | 16 | 23·16 | True | 0·0 | True | False | True |
| 400 | 15 | 15 | 14·17 | True | 0·0 | True | False | True |
| 425 | 17 | 21 | 17·23 | False | 1·1 | None | True | None |
| 450 | 14 | 16 | 15·15 | False | 0·0 | True | False | True |
| 475 | 17 | 11 | 18·12 | False | 0·0 | True | False | True |
| 500 | 14 | 18 | 15·19 | False | 1·1 | None | True | None |
| 525 | 13 | 16 | — | — | — | — | — | B alone does not prove |
| 550 | 17 | 15 | 18·16 | False | 2·2 | None | True | None |
| 575 | 17 | 17 | 19·17 | False | 0·0 | True | False | True |
| 600 | 17 | 16 | 16·17 | True | 0·0 | True | False | True |
| 625 | 16 | 20 | 17·22 | False | 0·0 | True | False | True |
| 650 | 12 | 13 | 13·13 | False | 1·1 | None | True | None |
| 675 | 15 | 16 | 15·17 | True | 0·0 | True | False | True |
| 700 | 18 | 20 | — | — | — | — | — | B does not apply alone: KeyError |
| 725 | 13 | 15 | 13·16 | True | 0·0 | True | False | True |
| 750 | 17 | 14 | 18·14 | False | 1·1 | None | True | None |
| 775 | 14 | 17 | 14·18 | True | 0·0 | True | False | True |
| 800 | 14 | 13 | 15·13 | True | 0·0 | True | False | True |
| 825 | 14 | 18 | 16·23 | False | 1·1 | None | True | None |
| 850 | 18 | 13 | 19·14 | True | 0·0 | True | False | True |
| 875 | 20 | 14 | 21·13 | True | 0·0 | True | False | True |
| 900 | 15 | 20 | 15·20 | False | 1·1 | None | True | None |
| 925 | 13 | 14 | 15·14 | True | 0·0 | True | False | True |
| 950 | 21 | 16 | 23·17 | True | 0·0 | True | False | True |
| 975 | 15 | 12 | 15·13 | False | 1·1 | None | False | True |

## 프로필 `contract-heavy`, 정의 1000개, 각자 커밋 40개

생성: `python3 dev-docs/experiments/commute/run.py --profile contract-heavy --span 40 --every 25`. 기준 커밋 c에서 A = 커밋 c+1..c+40, B = 커밋 c+41..c+80를 각자 적용한 쌍.

| 항목 | 값 |
| --- | ---: |
| 표본 쌍 | 37 |
| 건너뜀(B가 A에 기댐) | 34 |
| 쓴 쌍 | 3 |
| 텍스트 충돌(BMB 단위 파일, `git merge-file`) | 3 |
| 텍스트 충돌(Rust 쌍둥이) | 3 |
| 깨끗한 텍스트 병합 중 증명 실패(거짓 병합) | 0 / 0 |
| 정의 단위 쓰기 충돌 | 3 |
| 정의 단위 병합 중 증명 실패 | 0 / 0 |
| 교환 판정: 독립 | 0 |
| **독립 판정인데 병합이 증명 실패(건전성 위반)** | 0 |
| 충돌 판정인데 정의 단위 병합이 증명됨(보수적 판정) | 0 |
| 충돌 판정인데 텍스트로는 깨끗이 병합 | 0 |

| k | A 편집 | B 편집 | 쓰기 A·B | 독립 | 텍스트 충돌 BMB·Rust | 텍스트 병합 증명 | 정의 충돌 | 정의 병합 증명 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 70 | 61 | — | — | — | — | — | B alone does not prove |
| 25 | 64 | 59 | 64·67 | False | 5·6 | None | True | None |
| 50 | 60 | 66 | — | — | — | — | — | B does not apply alone: KeyError |
| 75 | 57 | 68 | — | — | — | — | — | B does not apply alone: KeyError |
| 100 | 65 | 73 | — | — | — | — | — | B does not apply alone: KeyError |
| 125 | 68 | 64 | — | — | — | — | — | B does not apply alone: KeyError |
| 150 | 71 | 58 | — | — | — | — | — | B alone does not prove |
| 175 | 59 | 56 | — | — | — | — | — | B does not apply alone: KeyError |
| 200 | 55 | 60 | — | — | — | — | — | B does not apply alone: KeyError |
| 225 | 61 | 56 | — | — | — | — | — | B does not apply alone: KeyError |
| 250 | 61 | 57 | — | — | — | — | — | B alone does not prove |
| 275 | 58 | 60 | — | — | — | — | — | B does not apply alone: KeyError |
| 300 | 61 | 63 | — | — | — | — | — | B does not apply alone: KeyError |
| 325 | 63 | 65 | — | — | — | — | — | B does not apply alone: KeyError |
| 350 | 64 | 58 | — | — | — | — | — | B alone does not prove |
| 375 | 62 | 58 | — | — | — | — | — | B does not apply alone: KeyError |
| 400 | 58 | 60 | — | — | — | — | — | B does not apply alone: KeyError |
| 425 | 56 | 60 | 57·64 | False | 3·4 | None | True | None |
| 450 | 59 | 58 | — | — | — | — | — | B does not apply alone: KeyError |
| 475 | 59 | 72 | — | — | — | — | — | B does not apply alone: KeyError |
| 500 | 64 | 66 | 71·63 | False | 1·2 | None | True | None |
| 525 | 75 | 61 | — | — | — | — | — | B does not apply alone: KeyError |
| 550 | 61 | 63 | — | — | — | — | — | B alone does not prove |
| 575 | 62 | 62 | — | — | — | — | — | B does not apply alone: KeyError |
| 600 | 62 | 61 | — | — | — | — | — | B alone does not prove |
| 625 | 58 | 66 | — | — | — | — | — | B does not apply alone: KeyError |
| 650 | 60 | 67 | — | — | — | — | — | B does not apply alone: KeyError |
| 675 | 60 | 67 | — | — | — | — | — | B does not apply alone: KeyError |
| 700 | 63 | 67 | — | — | — | — | — | B does not apply alone: KeyError |
| 725 | 69 | 61 | — | — | — | — | — | B does not apply alone: KeyError |
| 750 | 65 | 57 | — | — | — | — | — | B does not apply alone: KeyError |
| 775 | 57 | 66 | — | — | — | — | — | B does not apply alone: KeyError |
| 800 | 60 | 62 | — | — | — | — | — | B does not apply alone: KeyError |
| 825 | 65 | 58 | — | — | — | — | — | B does not apply alone: KeyError |
| 850 | 60 | 57 | — | — | — | — | — | B does not apply alone: KeyError |
| 875 | 56 | 65 | — | — | — | — | — | B alone does not prove |
| 900 | 61 | 57 | — | — | — | — | — | B does not apply alone: KeyError |

## 재측정: 의미 충돌 집합과 정제 면제 (백로그 T1-19·T1-27, 2026-10-29)

> 컴파일러: 이 커밋의 `target/debug/bmb`(`bmb reads`, `bmb weaken --readers`, `weaken`의 같은-`id` 지름길 수정). 판정 하네스 `judge.py`, 의미 충돌 생성기 `semantic.py`(동결: `MANIFEST`), BCB 쌍 `run.py`. 위의 첫 재생 절은 그대로 둔다(컴파일러 `62751306`).

## 판정(judge.py)

세 판(기준 O, 편집 A, 편집 B)만 보고 판정한다. 병합은 증명하지 않는다.

- **쓰기 집합:** 자기 텍스트 행(`bmb reads`의 `self`: 다른 정의를 경로로 적은 한정 정규형)이 바뀐 정의·더한 정의·지운 정의, 선언이 바뀐 타입. ADR-0065 초안의 `id` 쓰기 집합은 비교용으로만 센다(`ww_id`).
- **읽기 집합:** `bmb reads`가 내는 증분 증명 키의 행(자기 텍스트, 피호출자 계약, 펼친 `spec fn` 텍스트, 선언, 타입). 편집 X가 다시 증명하는 정의는 O와 X 사이에 키 행이 바뀐 정의다. 다만 바뀐 행이 모두 그 정의에 대해 면제되는 피호출자 계약뿐이면 다시 증명하지 않는다. X의 읽기 집합은 X가 다시 증명하는 정의들의 행이다.
- **충돌:** 두 쓰기 집합이 겹치거나, 한쪽이 다시 증명하는 정의가 다른 쪽이 바꾼 행을 읽고 그 행이 면제되지 않을 때.
- **면제:** `callee` 행(실행 함수의 계약)만 면제될 수 있다. `undecided`와 질의가 없는 경우는 충돌이다. 변형 넷:

| 변형 | 면제 조건 |
| --- | --- |
| `none` | 면제 없음 |
| `own` | `bmb weaken O X`가 `same`·`refined`·`compatible`(두 판 각자의 `#[uses]`로 펼침, 2D-19 그대로) |
| `reader` | `bmb weaken --readers`(읽는 쪽 증명의 `#[uses]`만 펼침)가 `same`·`refined`·`compatible`. ADR-0065 초안 §3 |
| `world` | `reader`이고, 질의가 펼친 술어(`revealed`) 중 읽는 쪽의 편집이 쓴 것이 없음. 이 측정이 찾은 수정(아래 해석 2) |

`old`는 첫 재생의 규칙(`bmb hash`의 본문·계약 해시가 바뀐 정의, L2 간선, 계약 변경은 모두 충돌)을 지금 컴파일러로 돌린 것이다. ADR-0059 이후 이 해시는 참조하는 정의의 `id`를 담아 전이적이므로, 첫 재생 때보다 훨씬 보수적이다(아래 BCB 표의 `old` 열). 첫 재생의 수치(충돌 판정 37쌍 중 18쌍이 병합 가능)는 위 절에 그대로 있다.

**정답:** 정의 단위 병합(기준 위에 A와 B의 정의 연산을 모두 적용)의 증명. 두 편집이 같은 정의를 썼으면 병합이 없으므로 충돌이 정답이다. 의미 충돌 집합은 기준·A·B가 각자 증명되는 쌍만 쓴다(192쌍 모두 해당).

## 요약

| 집합 | 쌍 | 병합 실패(정답 충돌) | git 깨끗한 병합 중 증명 실패(거짓 병합) | git 거짓 충돌 | `world` 교환 | `world` 위반 | `world` 거짓 충돌 | `reader`(초안) 위반 | `own` 위반 | `old` 위반 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 의미 충돌 집합(12가족 × 16) | 192 | 58 | 57 / 191 | 0 | 78 | **0** | 56 | **6** | **10** | **3** |
| BCB contract-heavy, 각자 커밋 1개 | 40 | 0 | 0 / 40 | 0 | 40 | 0 | 0 | 0 | — | 0 |
| BCB contract-heavy, 각자 커밋 10개 | 38 | 10 | 0 / 29 | 2 | 20 | 0 | 8 | 0 | — | 0 |
| BCB body-heavy, 각자 커밋 10개 | 35 | 8 | 0 / 26 | 1 | 26 | 0 | 1 | 0 | — | 0 |

- "병합 실패"는 정의 단위 병합이 증명되지 않거나(의미 충돌) 같은 정의를 둘 다 쓴 쌍(쓰기 충돌)이다. BCB의 병합 실패는 모두 쓰기 충돌이다(의미 충돌 0, 첫 재생과 같음).
- "git 거짓 충돌"은 `git merge-file`이 충돌했는데 정의 단위 병합이 증명된 쌍이다.
- BCB에서 `own`은 돌리지 않았다(`bmb weaken`이 정의 1,000개 프로그램 전체를 비교해 쌍당 수십 초가 들고, 의미 충돌 집합이 그 결함을 이미 보인다).

## 해석

1. **건전성(가설 첫 절, ADR-0065 반증 조건):** 수정한 관계(`world`)는 305쌍에서 위반이 0이다. 의미 충돌 58쌍을 모두 충돌로 판정했다(58/58). 그러나 **초안대로의 관계(`reader`)는 위반 6을 냈다.** ADR-0065의 반증 조항에 해당하므로 원인을 찾아 고쳤다.
2. **위반의 원인(`refine_vs_spec` 6쌍):** A는 `f`의 `requires q(x)`를 `requires a(x)`로 바꿨다. 기준의 `q`는 `x >= q0`, `a`는 `x >= a`(`a <= q0`)이므로 `q`를 펼치는 증명에게는 정제다. B는 `q`의 본문을 `x >= qb`(`qb < a`)로 넓히고, `q`와 `a`를 펼쳐 `f(v)`를 부르는 호출자를 더했다(`qb <= v < a`). 면제 질의는 O와 A의 판으로 묻는다. 그래서 `q`의 옛 본문으로 정제라고 답한다. 그런데 호출자는 B의 판에 있고 B의 `q`를 본다. 질의가 읽은 것(펼친 `q`의 본문)이 읽기 집합에 없었던 것이 원인이다. 수정: 질의가 펼친 술어(`--readers`의 `revealed`) 중 읽는 쪽 편집의 쓰기 집합에 든 것이 있으면 면제하지 않는다. ADR-0065 §3에 반영했다.
3. **정의 자신의 `#[uses]`로 면제하면(`own`) 위반 10(`opaque_rewrite`).** A가 `requires pos(x)`를 `f`가 펼치는 같은 뜻의 `alt(x)`로 바꿨고, B의 호출자는 `pos`만 펼치거나 아무것도 펼치지 않는다. 2D-19가 예고한 결함이 실제 병합 실패로 나왔다. 읽는 쪽의 공개 집합으로 판정하면(`reader`·`world`) 10쌍 모두 충돌이고, 둘 다 펼치는 호출자 6쌍은 교환(모두 병합 증명)이다.
4. **`id` 쓰기 집합(ADR-0065 §1 초안)은 양쪽으로 틀린다.**
   - 전이적이라 거짓 쓰기 충돌을 만든다: 자기 텍스트로는 겹치지 않는데 `id`로는 겹치는 쌍이 의미 충돌 집합 53쌍, BCB 각자 커밋 10개 51쌍(27 + 24), 1개 1쌍이다.
   - 이름이 없어서 편집을 놓친다: `alt`가 `pos`와 본문이 같으면 `f`의 `id`가 그대로다. `old`의 위반 3은 이 경우다(`opaque_rewrite`에서 `ha = h`인 쌍). 같은 원인으로 `bmb weaken`도 그런 정의를 질의 없이 `same`이라 했다. 지름길을 `id`와 경로로 한정한 계약 텍스트가 모두 같을 때로 좁혀 고쳤다(`tests/weaken/readers`의 `quarter`).
   - ADR-0065 §1을 자기 텍스트 기준으로 고쳤다. 이름 바꾸기를 쓰기 집합에서 빼는 것은 ADR-0063(이름 없는 질의 기호) 뒤다.
5. **거짓 충돌(가설 둘째 절):** 첫 재생은 각자 커밋 10개의 두 설정에서 충돌로 본 36쌍 중 18쌍이 병합 가능했다. 그때의 정답은 전이적 쓰기 집합이 겹친 쌍을 병합하지 않고 충돌로 쳤다. 같은 표본(73쌍)에서 이번 정답으로 보면, `world`는 충돌 27쌍 중 9쌍이 거짓이다. 나머지 18쌍은 같은 정의를 둘 다 쓴 쌍이다. 같은 규칙을 지금 해시로 돌린 `old`는 69쌍 중 51쌍이 거짓이다. 남은 거짓 충돌 9쌍은 정제가 아닌 계약 변경(BCB의 `_ok` 절 빼기)이나 `spec fn` 본문 변경을 상대가 읽는 경우다. 정제 판정으로는 원리상 풀 수 없고, ADR-0065 §4대로 겹친 정의만 다시 증명해야 풀린다(재증명 비용은 다음 태스크).
   - **가설의 둘째 절("텍스트 병합보다 거짓 충돌이 적다")은 반박되었다.** `git merge-file`의 거짓 충돌은 BCB 3쌍, 의미 충돌 집합 0쌍이다. `world`는 9쌍과 56쌍이다.
   - 대신 git은 의미 충돌을 거의 다 놓친다. 의미 충돌 집합에서 병합이 실패하는 58쌍 중 57쌍을 깨끗이 병합했다(유일한 텍스트 충돌은 `remove_vs_call` 1쌍). 첫 재생은 이 비교를 할 수 없었다.
   - 즉 교환 관계는 git보다 보수적이지만 건전하고, git은 덜 충돌하지만 건전하지 않다. 의미 충돌 집합의 거짓 충돌 56쌍은 정제가 아닌 계약 변경이나 펼친 `spec fn` 본문 변경을 상대가 읽는 쌍이다. 생성기가 일부러 만든 꼴이다.
6. **정제 면제의 효과:** 면제 없음(`none`)에서 `world`로 가면 거짓 충돌이 의미 충돌 집합 99 → 56, BCB 14 → 9로 준다. 건전성은 그대로다. 면제로 교환이 된 쌍은 `pre_weaken`·`post_strengthen` 32쌍 전부(합친 결과 모두 증명), `pre_strengthen`에서 더한 절이 기존 절에 함축되는 경우, `opaque_rewrite`에서 호출자가 두 술어를 다 펼치는 경우다.
7. **비용:**
   - 의미 충돌 집합(정의 100여 개): 면제 질의 행 138개(쌍당 0.72), 해법기 의무 244개. `weaken --readers` 128회에 CPU 24.9초(회당 0.19초). 같은 쌍들의 병합 재증명(따뜻한 캐시)은 192회 22.3초다. 이 크기에서 면제 질의는 재증명보다 싸지 않다.
   - BCB(정의 1,000개): 각자 커밋 10개 두 설정에서 질의 행 642개, 의무 1,585개, `weaken --readers` 125회 CPU 258.7초(회당 2.1초). `bmb reads`는 회당 약 1.1초다.
   - 회당 해법기 의무는 약 13개인데 회당 CPU는 프로그램 크기를 따라 늘었다(정의 100여 개 0.19초, 1,000개 2.1초). 두 판 전체를 검사·정렬하는 몫으로 보이지만 나눠 재지 않았다.
   - 이 측정에서 관계의 이득은 CPU가 아니다. 병합을 만들거나 증명하지 않고 판정한다는 것과, 그 판정이 건전하다는 것이다.
8. **생성기의 한계:**
   - 의미 충돌 집합의 시나리오는 가족마다 정의 3~9개의 작은 꼴을 배경 BCB 작업 공간(정의 100개)에 끼운 것이다. 실제 에이전트 편집의 분포가 아니다.
   - 가족별 비율은 생성기가 정한다. 그래서 "몇 %가 의미 충돌인가"는 이 집합에서 읽을 수 없다. 읽을 수 있는 것은 판정의 건전성과 검출력, 그리고 git과의 차이다.
   - 동결 전에 `pre_strengthen`의 인자 추첨을 한 번 바꿨다. 처음 추첨에서는 16쌍 모두 병합이 증명되어 그 가족이 의미 충돌을 만들지 못했다. 바꾼 판이 동결 판이다.

## 의미 충돌 집합(T1-19): 시드 1, 가족당 16쌍, 배경 정의 100개

생성: `python3 dev-docs/experiments/commute/semantic.py gen P.jsonl --seed 1 --per-family 16 --defs 100`, 측정: `semantic.py run P.jsonl --work W`.

쌍 192개 중 기준·A·B가 각자 증명된 쌍 192개(나머지는 표본에서 뺌: 없음).

| 가족 | 쌍 | 병합 증명 | 병합 실패 | git 충돌 | git 거짓 병합 | old 위반·거짓 충돌 | none 위반·거짓 충돌 | own 위반·거짓 충돌 | reader 위반·거짓 충돌 | world 위반·거짓 충돌 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `body_vs_caller` | 16 | 16 | 0 | 0 | 0 | 0·0 | 0·0 | 0·0 | 0·0 | 0·0 |
| `independent` | 16 | 16 | 0 | 0 | 0 | 0·0 | 0·0 | 0·0 | 0·0 | 0·0 |
| `opaque_rewrite` | 16 | 6 | 10 | 0 | 10 | 3·5 | 0·6 | 10·0 | 0·0 | 0·0 |
| `post_strengthen` | 16 | 16 | 0 | 0 | 0 | 0·16 | 0·16 | 0·0 | 0·0 | 0·0 |
| `post_weaken` | 16 | 10 | 6 | 0 | 6 | 0·10 | 0·10 | 0·10 | 0·10 | 0·10 |
| `pre_narrow` | 16 | 12 | 4 | 0 | 4 | 0·12 | 0·12 | 0·12 | 0·12 | 0·12 |
| `pre_strengthen` | 16 | 10 | 6 | 0 | 6 | 0·10 | 0·10 | 0·5 | 0·5 | 0·5 |
| `pre_weaken` | 16 | 16 | 0 | 0 | 0 | 0·16 | 0·16 | 0·0 | 0·0 | 0·0 |
| `refine_vs_spec` | 16 | 10 | 6 | 0 | 6 | 0·10 | 0·10 | 0·10 | 6·0 | 0·10 |
| `remove_vs_call` | 16 | 0 | 16 | 1 | 15 | 0·0 | 0·0 | 0·0 | 0·0 | 0·0 |
| `spec_body` | 16 | 12 | 4 | 0 | 4 | 0·12 | 0·9 | 0·9 | 0·9 | 0·9 |
| `two_weakenings` | 16 | 10 | 6 | 0 | 6 | 0·10 | 0·10 | 0·10 | 0·10 | 0·10 |
| **합** | 192 | 134 | 58 | 1 | 57 | 3·101 | 0·99 | 10·56 | 6·46 | 0·56 |

열 `v 위반·거짓 충돌`: 판정 v가 교환 가능으로 본 쌍 중 정의 단위 병합이 증명되지 않은 수(건전성 위반) · 충돌로 본 쌍 중 병합이 증명된 수(거짓 충돌). 판정:

- `old`: 첫 재생(T1-12)의 규칙(`bmb hash`의 본문·계약 해시와 L2 간선)을 지금 컴파일러로. ADR-0059 뒤로 해시가 전이적이다
- `none`: 키, 면제 없음
- `own`: 키 + 정의 자신의 `#[uses]`로 면제
- `reader`: 키 + 읽는 쪽 `#[uses]`로 면제(ADR-0065 초안)
- `world`: 키 + 읽는 쪽 면제 + 질의가 펼친 술어를 상대가 안 씀

| 항목 | 값 |
| --- | ---: |
| 병합이 실패하는 쌍(의미 충돌 또는 쓰기 충돌) | 58 |
| 그중 git이 깨끗이 병합(거짓 병합) | 57 / 58 |
| 그중 `old`가 충돌로 판정(검출) | 55 / 58 |
| 그중 `none`가 충돌로 판정(검출) | 58 / 58 |
| 그중 `own`가 충돌로 판정(검출) | 48 / 58 |
| 그중 `reader`가 충돌로 판정(검출) | 52 / 58 |
| 그중 `world`가 충돌로 판정(검출) | 58 / 58 |
| 면제 질의(읽는 쪽): 행 · 해법기 의무 | 138 · 244 |
| CPU `hash`: 호출 · 초 | 576 · 25.1 |
| CPU `map`: 호출 · 초 | 384 · 10.0 |
| CPU `reads`: 호출 · 초 | 576 · 48.0 |
| CPU `verify_merge`: 호출 · 초 | 192 · 22.3 |
| CPU `verify_text`: 호출 · 초 | 191 · 18.5 |
| CPU `weaken_own`: 호출 · 초 | 128 · 26.9 |
| CPU `weaken_reader`: 호출 · 초 | 128 · 24.9 |
| `id` 쓰기 집합이면 생기는 쓰기 충돌(자기 텍스트로는 없음) | 53 |

| k | 가족 | 매개변수 | 병합 증명 | git 충돌·증명 | old | none | own | reader | world | 면제 질의 |
| ---: | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: |
| 0 | `body_vs_caller` | `{"c":778,"n":974}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 1 | `body_vs_caller` | `{"c":321,"n":829}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 2 | `body_vs_caller` | `{"c":483,"n":698}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 3 | `body_vs_caller` | `{"c":528,"n":953}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 4 | `body_vs_caller` | `{"c":446,"n":767}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 5 | `body_vs_caller` | `{"c":594,"n":768}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 6 | `body_vs_caller` | `{"c":607,"n":942}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 7 | `body_vs_caller` | `{"c":590,"n":628}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 8 | `body_vs_caller` | `{"c":287,"n":347}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 9 | `body_vs_caller` | `{"c":149,"n":708}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 10 | `body_vs_caller` | `{"c":647,"n":804}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 11 | `body_vs_caller` | `{"c":399,"n":927}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 12 | `body_vs_caller` | `{"c":610,"n":890}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 13 | `body_vs_caller` | `{"c":592,"n":782}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 14 | `body_vs_caller` | `{"c":170,"n":209}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 15 | `body_vs_caller` | `{"c":888,"n":894}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 16 | `independent` | `{"e0":109,"e1":698,"h":597,"k":399}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 17 | `independent` | `{"e0":322,"e1":633,"h":75,"k":21}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 18 | `independent` | `{"e0":177,"e1":218,"h":695,"k":511}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 19 | `independent` | `{"e0":231,"e1":366,"h":737,"k":286}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 20 | `independent` | `{"e0":438,"e1":897,"h":792,"k":168}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 21 | `independent` | `{"e0":441,"e1":727,"h":345,"k":287}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 22 | `independent` | `{"e0":144,"e1":748,"h":774,"k":268}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 23 | `independent` | `{"e0":301,"e1":576,"h":783,"k":669}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 24 | `independent` | `{"e0":464,"e1":868,"h":825,"k":806}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 25 | `independent` | `{"e0":245,"e1":343,"h":103,"k":85}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 26 | `independent` | `{"e0":319,"e1":866,"h":319,"k":49}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 27 | `independent` | `{"e0":166,"e1":360,"h":465,"k":341}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 28 | `independent` | `{"e0":368,"e1":720,"h":774,"k":551}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 29 | `independent` | `{"e0":316,"e1":707,"h":691,"k":505}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 30 | `independent` | `{"e0":289,"e1":661,"h":107,"k":77}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 31 | `independent` | `{"e0":430,"e1":430,"h":436,"k":397}` | True | 0·True | 교환 | 교환 | 교환 | 교환 | 교환 | 0 |
| 32 | `opaque_rewrite` | `{"h":648,"ha":725,"reveal":"none"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 33 | `opaque_rewrite` | `{"h":620,"ha":847,"reveal":"pos"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 34 | `opaque_rewrite` | `{"h":834,"ha":911,"reveal":"both"}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 35 | `opaque_rewrite` | `{"h":572,"ha":572,"reveal":"none"}` | False | 0·False | 교환 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 36 | `opaque_rewrite` | `{"h":101,"ha":513,"reveal":"pos"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 37 | `opaque_rewrite` | `{"h":657,"ha":923,"reveal":"both"}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 38 | `opaque_rewrite` | `{"h":871,"ha":913,"reveal":"both"}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 39 | `opaque_rewrite` | `{"h":507,"ha":567,"reveal":"pos"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 40 | `opaque_rewrite` | `{"h":638,"ha":638,"reveal":"pos"}` | False | 0·False | 교환 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 41 | `opaque_rewrite` | `{"h":843,"ha":954,"reveal":"pos"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 42 | `opaque_rewrite` | `{"h":447,"ha":598,"reveal":"both"}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 43 | `opaque_rewrite` | `{"h":773,"ha":902,"reveal":"pos"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 44 | `opaque_rewrite` | `{"h":461,"ha":461,"reveal":"none"}` | False | 0·False | 교환 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 45 | `opaque_rewrite` | `{"h":687,"ha":687,"reveal":"both"}` | True | 0·True | 교환 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 46 | `opaque_rewrite` | `{"h":386,"ha":438,"reveal":"pos"}` | False | 0·False | 충돌 | 충돌 | 교환 | 충돌 | 충돌 | 1 |
| 47 | `opaque_rewrite` | `{"h":239,"ha":376,"reveal":"both"}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 48 | `post_strengthen` | `{"c":272,"n":823,"t":129}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 49 | `post_strengthen` | `{"c":382,"n":685,"t":67}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 50 | `post_strengthen` | `{"c":778,"n":884,"t":618}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 51 | `post_strengthen` | `{"c":508,"n":832,"t":428}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 52 | `post_strengthen` | `{"c":617,"n":819,"t":524}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 53 | `post_strengthen` | `{"c":286,"n":725,"t":214}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 54 | `post_strengthen` | `{"c":716,"n":992,"t":589}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 55 | `post_strengthen` | `{"c":672,"n":778,"t":517}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 56 | `post_strengthen` | `{"c":223,"n":709,"t":108}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 57 | `post_strengthen` | `{"c":421,"n":479,"t":209}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 58 | `post_strengthen` | `{"c":724,"n":769,"t":598}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 59 | `post_strengthen` | `{"c":638,"n":766,"t":616}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 60 | `post_strengthen` | `{"c":344,"n":957,"t":295}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 61 | `post_strengthen` | `{"c":426,"n":426,"t":343}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 62 | `post_strengthen` | `{"c":193,"n":394,"t":136}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 63 | `post_strengthen` | `{"c":680,"n":942,"t":612}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 64 | `post_weaken` | `{"b_changes_caller":true,"e0":170,"e1":368,"n":186,"reveal_hi":true}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 65 | `post_weaken` | `{"b_changes_caller":false,"e0":372,"e1":552,"n":789,"reveal_hi":false}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 66 | `post_weaken` | `{"b_changes_caller":false,"e0":282,"e1":520,"n":813,"reveal_hi":false}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 67 | `post_weaken` | `{"b_changes_caller":true,"e0":366,"e1":518,"n":721,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 68 | `post_weaken` | `{"b_changes_caller":true,"e0":224,"e1":406,"n":251,"reveal_hi":true}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 69 | `post_weaken` | `{"b_changes_caller":true,"e0":430,"e1":638,"n":951,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 70 | `post_weaken` | `{"b_changes_caller":true,"e0":104,"e1":221,"n":407,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 71 | `post_weaken` | `{"b_changes_caller":false,"e0":332,"e1":412,"n":932,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 72 | `post_weaken` | `{"b_changes_caller":true,"e0":123,"e1":266,"n":588,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 73 | `post_weaken` | `{"b_changes_caller":false,"e0":213,"e1":402,"n":403,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 74 | `post_weaken` | `{"b_changes_caller":false,"e0":408,"e1":645,"n":460,"reveal_hi":true}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 75 | `post_weaken` | `{"b_changes_caller":false,"e0":418,"e1":821,"n":800,"reveal_hi":true}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 76 | `post_weaken` | `{"b_changes_caller":true,"e0":455,"e1":456,"n":788,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 77 | `post_weaken` | `{"b_changes_caller":true,"e0":392,"e1":570,"n":656,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 78 | `post_weaken` | `{"b_changes_caller":true,"e0":224,"e1":334,"n":637,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 79 | `post_weaken` | `{"b_changes_caller":true,"e0":434,"e1":837,"n":938,"reveal_hi":true}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 80 | `pre_narrow` | `{"b_changes_caller":false,"h0":682,"ha":557,"v":393}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 81 | `pre_narrow` | `{"b_changes_caller":true,"h0":284,"ha":276,"v":137}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 82 | `pre_narrow` | `{"b_changes_caller":true,"h0":667,"ha":383,"v":214}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 83 | `pre_narrow` | `{"b_changes_caller":false,"h0":332,"ha":236,"v":105}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 84 | `pre_narrow` | `{"b_changes_caller":true,"h0":475,"ha":451,"v":188}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 85 | `pre_narrow` | `{"b_changes_caller":true,"h0":250,"ha":242,"v":64}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 86 | `pre_narrow` | `{"b_changes_caller":false,"h0":805,"ha":39,"v":36}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 87 | `pre_narrow` | `{"b_changes_caller":true,"h0":366,"ha":219,"v":350}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 88 | `pre_narrow` | `{"b_changes_caller":true,"h0":560,"ha":559,"v":377}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 89 | `pre_narrow` | `{"b_changes_caller":true,"h0":180,"ha":77,"v":103}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 90 | `pre_narrow` | `{"b_changes_caller":true,"h0":825,"ha":367,"v":272}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 91 | `pre_narrow` | `{"b_changes_caller":false,"h0":751,"ha":512,"v":440}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 92 | `pre_narrow` | `{"b_changes_caller":true,"h0":145,"ha":104,"v":120}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 93 | `pre_narrow` | `{"b_changes_caller":false,"h0":226,"ha":53,"v":181}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 94 | `pre_narrow` | `{"b_changes_caller":true,"h0":767,"ha":690,"v":71}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 95 | `pre_narrow` | `{"b_changes_caller":true,"h0":197,"ha":146,"v":116}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 96 | `pre_strengthen` | `{"h0":622,"hc":649,"v":583}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 97 | `pre_strengthen` | `{"h0":125,"hc":86,"v":73}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 98 | `pre_strengthen` | `{"h0":543,"hc":546,"v":489}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 99 | `pre_strengthen` | `{"h0":395,"hc":391,"v":365}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 100 | `pre_strengthen` | `{"h0":326,"hc":290,"v":323}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 101 | `pre_strengthen` | `{"h0":344,"hc":304,"v":342}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 102 | `pre_strengthen` | `{"h0":547,"hc":534,"v":510}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 103 | `pre_strengthen` | `{"h0":599,"hc":582,"v":579}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 104 | `pre_strengthen` | `{"h0":415,"hc":375,"v":410}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 105 | `pre_strengthen` | `{"h0":885,"hc":853,"v":884}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 106 | `pre_strengthen` | `{"h0":193,"hc":187,"v":159}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 107 | `pre_strengthen` | `{"h0":99,"hc":67,"v":75}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 108 | `pre_strengthen` | `{"h0":586,"hc":563,"v":579}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 1 |
| 109 | `pre_strengthen` | `{"h0":762,"hc":793,"v":747}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 110 | `pre_strengthen` | `{"h0":421,"hc":427,"v":394}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 111 | `pre_strengthen` | `{"h0":808,"hc":835,"v":783}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 112 | `pre_weaken` | `{"h0":623,"k":40,"opaque":false,"v":438}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 113 | `pre_weaken` | `{"h0":633,"k":306,"opaque":false,"v":40}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 114 | `pre_weaken` | `{"h0":892,"k":568,"opaque":true,"v":441}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 115 | `pre_weaken` | `{"h0":843,"k":595,"opaque":true,"v":211}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 116 | `pre_weaken` | `{"h0":283,"k":0,"opaque":true,"v":260}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 117 | `pre_weaken` | `{"h0":660,"k":511,"opaque":false,"v":445}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 118 | `pre_weaken` | `{"h0":657,"k":106,"opaque":false,"v":404}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 119 | `pre_weaken` | `{"h0":260,"k":120,"opaque":false,"v":259}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 120 | `pre_weaken` | `{"h0":412,"k":169,"opaque":false,"v":74}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 121 | `pre_weaken` | `{"h0":192,"k":36,"opaque":true,"v":55}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 122 | `pre_weaken` | `{"h0":674,"k":446,"opaque":false,"v":380}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 123 | `pre_weaken` | `{"h0":480,"k":437,"opaque":true,"v":360}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 124 | `pre_weaken` | `{"h0":378,"k":32,"opaque":false,"v":227}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 125 | `pre_weaken` | `{"h0":778,"k":665,"opaque":false,"v":642}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 126 | `pre_weaken` | `{"h0":885,"k":391,"opaque":true,"v":312}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 127 | `pre_weaken` | `{"h0":376,"k":244,"opaque":false,"v":32}` | True | 0·True | 충돌 | 충돌 | 교환 | 교환 | 교환 | 1 |
| 128 | `refine_vs_spec` | `{"a":255,"q0":349,"qb":133,"v":227}` | False | 0·False | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 129 | `refine_vs_spec` | `{"a":57,"q0":408,"qb":36,"v":52}` | False | 0·False | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 130 | `refine_vs_spec` | `{"a":35,"q0":204,"qb":31,"v":85}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 131 | `refine_vs_spec` | `{"a":77,"q0":83,"qb":9,"v":71}` | False | 0·False | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 132 | `refine_vs_spec` | `{"a":35,"q0":403,"qb":13,"v":214}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 133 | `refine_vs_spec` | `{"a":166,"q0":261,"qb":32,"v":244}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 134 | `refine_vs_spec` | `{"a":22,"q0":313,"qb":3,"v":168}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 135 | `refine_vs_spec` | `{"a":159,"q0":382,"qb":40,"v":115}` | False | 0·False | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 136 | `refine_vs_spec` | `{"a":61,"q0":141,"qb":9,"v":92}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 137 | `refine_vs_spec` | `{"a":24,"q0":66,"qb":19,"v":24}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 138 | `refine_vs_spec` | `{"a":109,"q0":495,"qb":38,"v":299}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 139 | `refine_vs_spec` | `{"a":118,"q0":165,"qb":84,"v":137}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 140 | `refine_vs_spec` | `{"a":50,"q0":256,"qb":34,"v":222}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 141 | `refine_vs_spec` | `{"a":205,"q0":224,"qb":182,"v":191}` | False | 0·False | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 142 | `refine_vs_spec` | `{"a":82,"q0":242,"qb":67,"v":95}` | True | 0·True | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 143 | `refine_vs_spec` | `{"a":392,"q0":419,"qb":42,"v":93}` | False | 0·False | 충돌 | 충돌 | 충돌 | 교환 | 충돌 | 1 |
| 144 | `remove_vs_call` | `{"h":511}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 145 | `remove_vs_call` | `{"h":283}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 146 | `remove_vs_call` | `{"h":639}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 147 | `remove_vs_call` | `{"h":465}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 148 | `remove_vs_call` | `{"h":263}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 149 | `remove_vs_call` | `{"h":794}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 150 | `remove_vs_call` | `{"h":835}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 151 | `remove_vs_call` | `{"h":724}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 152 | `remove_vs_call` | `{"h":33}` | False | 1·None | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 153 | `remove_vs_call` | `{"h":33}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 154 | `remove_vs_call` | `{"h":891}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 155 | `remove_vs_call` | `{"h":646}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 156 | `remove_vs_call` | `{"h":91}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 157 | `remove_vs_call` | `{"h":465}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 158 | `remove_vs_call` | `{"h":599}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 159 | `remove_vs_call` | `{"h":340}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 160 | `spec_body` | `{"c0":636,"ca":562,"n":718,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 161 | `spec_body` | `{"c0":301,"ca":291,"n":943,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 162 | `spec_body` | `{"c0":440,"ca":257,"n":503,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 163 | `spec_body` | `{"c0":543,"ca":298,"n":943,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 164 | `spec_body` | `{"c0":579,"ca":543,"n":719,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 165 | `spec_body` | `{"c0":793,"ca":875,"n":865,"through":false}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 166 | `spec_body` | `{"c0":535,"ca":880,"n":768,"through":false}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 167 | `spec_body` | `{"c0":375,"ca":178,"n":797,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 168 | `spec_body` | `{"c0":613,"ca":326,"n":746,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 169 | `spec_body` | `{"c0":404,"ca":827,"n":698,"through":false}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 170 | `spec_body` | `{"c0":665,"ca":695,"n":908,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 171 | `spec_body` | `{"c0":135,"ca":199,"n":704,"through":true}` | True | 0·True | 충돌 | 교환 | 교환 | 교환 | 교환 | 0 |
| 172 | `spec_body` | `{"c0":101,"ca":412,"n":703,"through":true}` | True | 0·True | 충돌 | 교환 | 교환 | 교환 | 교환 | 0 |
| 173 | `spec_body` | `{"c0":682,"ca":835,"n":701,"through":false}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 174 | `spec_body` | `{"c0":550,"ca":618,"n":808,"through":false}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 0 |
| 175 | `spec_body` | `{"c0":748,"ca":248,"n":934,"through":true}` | True | 0·True | 충돌 | 교환 | 교환 | 교환 | 교환 | 0 |
| 176 | `two_weakenings` | `{"ga":520,"hb":932,"kb":558}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 177 | `two_weakenings` | `{"ga":362,"hb":791,"kb":347}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 178 | `two_weakenings` | `{"ga":417,"hb":710,"kb":200}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 179 | `two_weakenings` | `{"ga":426,"hb":864,"kb":560}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 180 | `two_weakenings` | `{"ga":357,"hb":728,"kb":287}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 181 | `two_weakenings` | `{"ga":310,"hb":688,"kb":317}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 182 | `two_weakenings` | `{"ga":355,"hb":500,"kb":165}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 183 | `two_weakenings` | `{"ga":187,"hb":674,"kb":505}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 184 | `two_weakenings` | `{"ga":332,"hb":705,"kb":475}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 185 | `two_weakenings` | `{"ga":278,"hb":623,"kb":260}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 186 | `two_weakenings` | `{"ga":257,"hb":352,"kb":190}` | False | 0·False | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 187 | `two_weakenings` | `{"ga":560,"hb":836,"kb":237}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 188 | `two_weakenings` | `{"ga":261,"hb":711,"kb":379}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 189 | `two_weakenings` | `{"ga":468,"hb":827,"kb":262}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 190 | `two_weakenings` | `{"ga":312,"hb":912,"kb":577}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |
| 191 | `two_weakenings` | `{"ga":179,"hb":539,"kb":283}` | True | 0·True | 충돌 | 충돌 | 충돌 | 충돌 | 충돌 | 2 |

## BCB 쌍의 ADR-0065 판정(T1-27): 프로필 `contract-heavy`, 각자 커밋 1개

생성: `python3 dev-docs/experiments/commute/run.py --span 1 --every 25 --variants none,reader,world`. 표본은 첫 재생과 같다(같은 이력, 같은 기준 커밋).

정답은 정의 단위 병합의 증명이다. 두 편집이 같은 정의를 썼으면(자기 텍스트 기준) 병합이 없으므로 충돌이 정답이다. 쓴 쌍 40개 중 같은 정의를 쓴 쌍 0개, 병합이 증명된 쌍 40개, 실패 0개. 첫 재생은 전이적 쓰기 집합(`bmb hash`)이 겹친 쌍을 병합하지 않았다. 그중 자기 텍스트로는 겹치지 않는 1쌍을 이번에 병합했다.

| 판정 | 교환 | 충돌 | 건전성 위반(교환인데 병합 실패) | 거짓 충돌(충돌인데 병합 증명) |
| --- | ---: | ---: | ---: | ---: |
| `old` | 39 | 1 | 0 | 1 |
| `none` | 40 | 0 | 0 | 0 |
| `reader` | 40 | 0 | 0 | 0 |
| `world` | 40 | 0 | 0 | 0 |

면제 질의(읽는 쪽 판): 행 56개, 해법기 의무 147개. `id` 쓰기 집합(ADR-0065 §1 초안)이면 생기는 쓰기 충돌(자기 텍스트로는 없음): 1쌍. CPU(호출 수 · 초): `hash` 120 · 54.6, `reads` 120 · 133.6, `weaken_reader` 25 · 47.0.

| k | 쓰기 겹침(자기·id) | 병합 증명 | old | none | reader | world | 면제 질의 |
| ---: | --- | --- | --- | --- | --- | --- | ---: |
| 0 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 25 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 50 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 75 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 2 |
| 100 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 125 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 150 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 9 |
| 175 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 2 |
| 200 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 225 | 0·5 | True | 충돌 | 교환 | 교환 | 교환 | 0 |
| 250 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 275 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 300 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 325 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 350 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 375 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 400 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 6 |
| 425 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 5 |
| 450 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 475 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 500 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 525 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 550 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 575 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 600 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 625 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 650 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 675 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 700 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 725 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 2 |
| 750 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 775 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 800 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 825 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 2 |
| 850 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 875 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 900 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 0 |
| 925 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 950 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 975 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 2 |

## BCB 쌍의 ADR-0065 판정(T1-27): 프로필 `contract-heavy`, 각자 커밋 10개

생성: `python3 dev-docs/experiments/commute/run.py --span 10 --every 25 --variants none,reader,world`. 표본은 첫 재생과 같다(같은 이력, 같은 기준 커밋).

정답은 정의 단위 병합의 증명이다. 두 편집이 같은 정의를 썼으면(자기 텍스트 기준) 병합이 없으므로 충돌이 정답이다. 쓴 쌍 38개 중 같은 정의를 쓴 쌍 10개, 병합이 증명된 쌍 28개, 실패 0개. 첫 재생은 전이적 쓰기 집합(`bmb hash`)이 겹친 쌍을 병합하지 않았다. 그중 자기 텍스트로는 겹치지 않는 27쌍을 이번에 병합했다.

| 판정 | 교환 | 충돌 | 건전성 위반(교환인데 병합 실패) | 거짓 충돌(충돌인데 병합 증명) |
| --- | ---: | ---: | ---: | ---: |
| `old` | 1 | 37 | 0 | 27 |
| `none` | 18 | 20 | 0 | 10 |
| `reader` | 20 | 18 | 0 | 8 |
| `world` | 20 | 18 | 0 | 8 |

면제 질의(읽는 쪽 판): 행 474개, 해법기 의무 1182개. `id` 쓰기 집합(ADR-0065 §1 초안)이면 생기는 쓰기 충돌(자기 텍스트로는 없음): 27쌍. CPU(호출 수 · 초): `hash` 114 · 52.8, `reads` 114 · 133.8, `weaken_reader` 74 · 159.1.

| k | 쓰기 겹침(자기·id) | 병합 증명 | old | none | reader | world | 면제 질의 |
| ---: | --- | --- | --- | --- | --- | --- | ---: |
| 0 | 2·32 | None | 충돌 | 충돌 | 충돌 | 충돌 | 12 |
| 25 | 0·27 | True | 충돌 | 교환 | 교환 | 교환 | 9 |
| 50 | 0·2 | True | 충돌 | 교환 | 교환 | 교환 | 16 |
| 75 | 0·17 | True | 충돌 | 교환 | 교환 | 교환 | 9 |
| 100 | 0·7 | True | 충돌 | 충돌 | 충돌 | 충돌 | 12 |
| 125 | 1·29 | None | 충돌 | 충돌 | 충돌 | 충돌 | 13 |
| 150 | 1·30 | None | 충돌 | 충돌 | 충돌 | 충돌 | 22 |
| 175 | 0·2 | True | 충돌 | 교환 | 교환 | 교환 | 12 |
| 225 | 0·26 | True | 충돌 | 교환 | 교환 | 교환 | 12 |
| 250 | 0·19 | True | 충돌 | 충돌 | 충돌 | 충돌 | 23 |
| 275 | 0·1 | True | 충돌 | 교환 | 교환 | 교환 | 21 |
| 300 | 0·17 | True | 충돌 | 충돌 | 교환 | 교환 | 12 |
| 350 | 0·2 | True | 충돌 | 교환 | 교환 | 교환 | 11 |
| 375 | 0·12 | True | 충돌 | 교환 | 교환 | 교환 | 15 |
| 400 | 0·5 | True | 충돌 | 교환 | 교환 | 교환 | 23 |
| 425 | 0·22 | True | 충돌 | 교환 | 교환 | 교환 | 20 |
| 450 | 1·12 | None | 충돌 | 충돌 | 충돌 | 충돌 | 17 |
| 475 | 1·12 | None | 충돌 | 충돌 | 충돌 | 충돌 | 10 |
| 500 | 1·35 | None | 충돌 | 충돌 | 충돌 | 충돌 | 13 |
| 525 | 0·23 | True | 충돌 | 충돌 | 교환 | 교환 | 20 |
| 550 | 0·18 | True | 충돌 | 충돌 | 충돌 | 충돌 | 12 |
| 575 | 0·6 | True | 충돌 | 교환 | 교환 | 교환 | 11 |
| 600 | 0·12 | True | 충돌 | 교환 | 교환 | 교환 | 11 |
| 625 | 0·17 | True | 충돌 | 충돌 | 충돌 | 충돌 | 11 |
| 650 | 0·11 | True | 충돌 | 충돌 | 충돌 | 충돌 | 17 |
| 675 | 0·23 | True | 충돌 | 충돌 | 충돌 | 충돌 | 11 |
| 700 | 0·1 | True | 충돌 | 교환 | 교환 | 교환 | 8 |
| 725 | 1·9 | None | 충돌 | 충돌 | 충돌 | 충돌 | 10 |
| 750 | 0·1 | True | 충돌 | 교환 | 교환 | 교환 | 13 |
| 775 | 1·4 | None | 충돌 | 충돌 | 충돌 | 충돌 | 9 |
| 800 | 0·7 | True | 충돌 | 교환 | 교환 | 교환 | 3 |
| 825 | 0·2 | True | 충돌 | 교환 | 교환 | 교환 | 11 |
| 850 | 1·18 | None | 충돌 | 충돌 | 충돌 | 충돌 | 12 |
| 875 | 0·4 | True | 충돌 | 충돌 | 충돌 | 충돌 | 6 |
| 900 | 0·1 | True | 충돌 | 교환 | 교환 | 교환 | 8 |
| 925 | 0·10 | True | 충돌 | 충돌 | 충돌 | 충돌 | 11 |
| 950 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 3 |
| 975 | 1·1 | None | 충돌 | 충돌 | 충돌 | 충돌 | 5 |

## BCB 쌍의 ADR-0065 판정(T1-27): 프로필 `body-heavy`, 각자 커밋 10개

생성: `python3 dev-docs/experiments/commute/run.py --profile body-heavy --span 10 --every 25 --variants none,reader,world`. 표본은 첫 재생과 같다(같은 이력, 같은 기준 커밋).

정답은 정의 단위 병합의 증명이다. 두 편집이 같은 정의를 썼으면(자기 텍스트 기준) 병합이 없으므로 충돌이 정답이다. 쓴 쌍 35개 중 같은 정의를 쓴 쌍 8개, 병합이 증명된 쌍 27개, 실패 0개. 첫 재생은 전이적 쓰기 집합(`bmb hash`)이 겹친 쌍을 병합하지 않았다. 그중 자기 텍스트로는 겹치지 않는 24쌍을 이번에 병합했다.

| 판정 | 교환 | 충돌 | 건전성 위반(교환인데 병합 실패) | 거짓 충돌(충돌인데 병합 증명) |
| --- | ---: | ---: | ---: | ---: |
| `old` | 3 | 32 | 0 | 24 |
| `none` | 23 | 12 | 0 | 4 |
| `reader` | 26 | 9 | 0 | 1 |
| `world` | 26 | 9 | 0 | 1 |

면제 질의(읽는 쪽 판): 행 168개, 해법기 의무 403개. `id` 쓰기 집합(ADR-0065 §1 초안)이면 생기는 쓰기 충돌(자기 텍스트로는 없음): 24쌍. CPU(호출 수 · 초): `hash` 105 · 47.0, `reads` 105 · 112.7, `weaken_reader` 51 · 99.6.

| k | 쓰기 겹침(자기·id) | 병합 증명 | old | none | reader | world | 면제 질의 |
| ---: | --- | --- | --- | --- | --- | --- | ---: |
| 0 | 1·20 | None | 충돌 | 충돌 | 충돌 | 충돌 | 4 |
| 25 | 0·23 | True | 충돌 | 충돌 | 충돌 | 충돌 | 6 |
| 50 | 0·5 | True | 충돌 | 교환 | 교환 | 교환 | 4 |
| 75 | 0·10 | True | 충돌 | 교환 | 교환 | 교환 | 7 |
| 125 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 150 | 0·16 | True | 충돌 | 교환 | 교환 | 교환 | 3 |
| 175 | 0·7 | True | 충돌 | 교환 | 교환 | 교환 | 3 |
| 200 | 0·26 | True | 충돌 | 충돌 | 교환 | 교환 | 6 |
| 225 | 0·7 | True | 충돌 | 교환 | 교환 | 교환 | 2 |
| 300 | 0·12 | True | 충돌 | 교환 | 교환 | 교환 | 4 |
| 325 | 0·6 | True | 충돌 | 교환 | 교환 | 교환 | 4 |
| 350 | 0·7 | True | 충돌 | 교환 | 교환 | 교환 | 4 |
| 375 | 0·3 | True | 충돌 | 교환 | 교환 | 교환 | 7 |
| 400 | 0·2 | True | 충돌 | 교환 | 교환 | 교환 | 3 |
| 425 | 1·25 | None | 충돌 | 충돌 | 충돌 | 충돌 | 3 |
| 450 | 0·8 | True | 충돌 | 교환 | 교환 | 교환 | 6 |
| 475 | 0·3 | True | 충돌 | 충돌 | 교환 | 교환 | 6 |
| 500 | 1·8 | None | 충돌 | 충돌 | 충돌 | 충돌 | 6 |
| 550 | 3·15 | None | 충돌 | 충돌 | 충돌 | 충돌 | 6 |
| 575 | 0·16 | True | 충돌 | 교환 | 교환 | 교환 | 0 |
| 600 | 0·10 | True | 충돌 | 교환 | 교환 | 교환 | 7 |
| 625 | 0·15 | True | 충돌 | 교환 | 교환 | 교환 | 7 |
| 650 | 1·9 | None | 충돌 | 충돌 | 충돌 | 충돌 | 7 |
| 675 | 0·11 | True | 충돌 | 충돌 | 교환 | 교환 | 7 |
| 725 | 0·6 | True | 충돌 | 교환 | 교환 | 교환 | 2 |
| 750 | 1·5 | None | 충돌 | 충돌 | 충돌 | 충돌 | 6 |
| 775 | 0·2 | True | 충돌 | 교환 | 교환 | 교환 | 6 |
| 800 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 10 |
| 825 | 1·3 | None | 충돌 | 충돌 | 충돌 | 충돌 | 6 |
| 850 | 0·0 | True | 교환 | 교환 | 교환 | 교환 | 1 |
| 875 | 0·12 | True | 충돌 | 교환 | 교환 | 교환 | 6 |
| 900 | 1·15 | None | 충돌 | 충돌 | 충돌 | 충돌 | 5 |
| 925 | 0·25 | True | 충돌 | 교환 | 교환 | 교환 | 2 |
| 950 | 0·3 | True | 충돌 | 교환 | 교환 | 교환 | 7 |
| 975 | 0·1 | True | 충돌 | 교환 | 교환 | 교환 | 4 |

## 비교환 쌍의 해소: 겹친 정의만 재증명 (백로그 T1-29, 2026-10-30)

> 컴파일러: 이 커밋의 `target/debug/bmb`(`bmb verify --def`를 더함). 하네스: `judge.py`의 `resolve`·`report_resolve`를 `semantic.py run`과 `run.py`가 판정 뒤에 부른다. 의미 충돌 쌍 집합은 그대로다(`MANIFEST`의 쌍 해시 불변, 하네스 세 파일의 해시만 갱신). 같은 명령으로 다시 돌린 판정(T1-27)의 수치는 위 "재측정" 절과 모두 같았다(CPU 초만 다름).

### 단계(ADR-0065 결정 4)

- **대상:** 판정이 충돌로 본 쌍 중 두 쓰기 집합이 겹치지 않는 쌍. 겹치면 병합이 없으므로 충돌로 끝난다.
- **절차:** 정의 단위 병합 M(기준 위에 A와 B의 정의 연산)을 만들어 검사하고, 충돌 원소의 읽는 정의(상대가 바꾼 행을 면제 없이 읽는, 다시 증명할 정의) 중 M에 남은 것만 `bmb verify --def`로 증명한다. M이 검사를 통과하고 그 정의가 모두 증명되면 병합 가능(해소)이다.
- **나머지 정의를 증명하지 않는 근거:** M에서 그 정의의 행은 한쪽 판의 것이거나, 상대가 바꾼 행이 그 정의에 대해 면제다. 교환 판정과 같은 근거다.
- **`verify --def`:** `verify::verify_only`(3A-04)를 명령줄에 낸 것이다. 프로그램 전체를 검사하고, 이름 댄 정의만 증명해 전체 증명과 같은 상태 줄을 낸다. 종료 코드는 그 정의들만 따른다(`tests/cli.rs`의 `verify_def_proves_only_the_named_definitions`: 캐시 없음·차가운 캐시·따뜻한 캐시에서 상태 줄이 전체 증명과 바이트 동일).
- **판정 둘에 붙였다:** `world`(ADR-0065, T1-27이 고친 관계)와 `none`(면제 없음).
- **정답과 위반:** 정답은 T1-27과 같다(정의 단위 병합 전체의 증명). 해소했는데 병합이 증명되지 않으면 위반이다. 결정 4와 반증 조항에 해당한다.
- **비용:** 같은 쌍에서 겹친 정의의 재증명(캐시 없음)과 병합 전체의 증명(캐시 없음)을 잰다. CPU는 자식 프로세스의 사용자+시스템 시간이다. 의미 충돌 집합에서는 병합을 따뜻한 캐시(기준·A·B를 먼저 증명한 증분 캐시, 집합 전체가 함께 씀)로도 증명했다. 병합 전체의 차가운 증명과 따뜻한 증명은 단계를 돌린 모든 쌍에서 판정이 같았다.

### 요약

| 집합 | 쌍 | `world` 충돌 | 그중 쓰기 겹침 | 재증명한 쌍 | 해소 | **위반** | 병합 실패 중 최종 충돌 | 거짓 충돌 전 → 후 | 재증명 CPU초 | 병합 전체 CPU초(캐시 없음) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| 의미 충돌 집합(12가족 × 16) | 192 | 114 | 0 | 114 | 56 | **0** | 58 / 58 | 56 → 0 | 6.9 | 207.6 |
| BCB contract-heavy, 각자 커밋 1개 | 40 | 0 | 0 | 0 | 0 | **0** | 0 / 0 | 0 → 0 | 0 | 0 |
| BCB contract-heavy, 각자 커밋 10개 | 38 | 18 | 10 | 8 | 8 | **0** | 10 / 10 | 8 → 0 | 3.0 | 367.0 |
| BCB body-heavy, 각자 커밋 10개 | 35 | 9 | 8 | 1 | 1 | **0** | 8 / 8 | 1 → 0 | 0.4 | 34.2 |

- "병합 실패 중 최종 충돌": 정의 단위 병합이 증명되지 않거나 없는 쌍 중 단계 뒤에도 충돌로 남은 수. BCB의 병합 실패는 모두 쓰기 겹침이라 단계가 돌지 않는다.
- BCB 각자 커밋 1개(`run.py --span 1 --every 25`) 40쌍은 두 판정 모두 교환이라 단계가 돌지 않았다.
- 면제 없는 판정(`none`)에 단계를 붙여도 결과가 같다: 의미 충돌 집합 충돌 157쌍 중 해소 99, 위반 0, 거짓 충돌 99 → 0. BCB는 각자 커밋 10개 두 설정에서 10 → 0, 4 → 0, 위반 0.

### 해석

1. **건전성(결정 4, 반증 조항):** 해소한 쌍 65개(`world`)와 99 + 14개(`none`)의 병합이 모두 증명되었다. 위반 0이다. 판정이 교환으로 본 쌍의 위반 0(T1-27)도 그대로다.
2. **거짓 충돌:** 단계 뒤에 남은 거짓 충돌은 0이다(BCB 9 → 0, 의미 충돌 집합 56 → 0). 구성상 그렇다: 병합 전체가 증명되면 그 안의 어느 정의도 증명되고, `verify --def`의 상태는 전체 증명과 같다. 그래서 이 수치가 재는 것은 단계의 건전성(위반)과 비용이고, 거짓 충돌 0은 그 결과다.
3. **진짜 충돌은 모두 남았다(58/58).** 의미 충돌 42쌍은 겹친 정의의 재증명이 실패했다. `remove_vs_call` 16쌍은 병합이 검사를 통과하지 못했다(A가 지운 정의를 B가 부름). 판정이 고른 읽는 정의는 쌍마다 하나였고(114쌍 모두), M에서 사라진 읽는 정의는 없었다.
4. **비용:** 겹친 정의의 재증명은 같은 쌍의 병합 전체 증명(캐시 없음)의 CPU 3.3%(의미 충돌 집합, 정의 100여 개), 0.8%(BCB contract-heavy), 1.1%(BCB body-heavy)다. 증명한 정의는 쌍당 1~2개, 의무는 병합 전체의 0.1~0.5%다. 재증명 CPU에는 병합 전체의 검사가 들어 있다. 정의 1,000개에서 단계 한 번은 0.3~0.5초다. 따뜻한 증분 캐시로 병합을 증명하면 의미 충돌 집합 114쌍에서 다시 인코딩한 정의 196개, CPU 14.9초다. 재증명 단계는 정의 98개, 6.9초다. 증분 캐시는 키가 기준·A·B 어느 판과도 다른 정의(양쪽의 변경을 함께 읽는 정의)를 모두 다시 인코딩하고, 단계는 면제되지 않은 읽기만 다시 증명한다.
5. **면제 질의와 재증명의 비용이 뒤집혔다.** 단계를 붙이면 면제의 유무가 판정의 질을 바꾸지 않는다(두 판정 모두 위반 0, 거짓 충돌 0). 차이는 비용뿐이다.
   - 의미 충돌 집합: `none` + 재증명은 9.3초다. `world` + 재증명은 재증명 6.9초에 면제 질의(`weaken --readers`) 26.8초를 더한다.
   - BCB 각자 커밋 10개 두 설정: `none` + 재증명 5.2초(3.7 + 1.5), `world` + 재증명은 3.4초(3.0 + 0.4)에 면제 질의 272.4초(168.5 + 103.9)다.
   - 즉 이 측정에서 정제 면제(결정 3)는 재증명 몇 초를 아끼려고 수십~수백 초의 질의를 쓴다. 면제의 남은 이득은 병합을 만들지 않고 판정한다는 것이다. 결정 3을 유지할지, 재증명을 기본으로 두고 면제를 선택적으로 둘지는 JUDGMENT §10으로 따로 판단한다(백로그 T1-31). T1-30(읽는 쪽 판의 면제)이 노린 거짓 충돌 10쌍(`refine_vs_spec`)은 이 단계로 이미 풀린다.
6. **한계:**
   - 겹친 정의가 적은 집합이다. 의미 충돌 집합은 생성기가 정의 3~9개의 꼴을 끼운 것이고, BCB 쌍은 읽는 정의가 쌍당 1~2개였다. 겹친 읽기가 큰 편집(많이 불리는 계약의 변경)에서의 비율은 재지 않았다.
   - 단계는 병합 M 전체를 검사한다. 검사와 증명의 몫은 나눠 재지 않았다.
   - 위반 0은 이 네 설정(쌍 305개, `world`로 단계를 돌린 쌍 123개)에서의 관찰이다. 근거는 관계의 논증과 `verify --def`가 전체 증명과 같은 상태를 낸다는 시험이다.

### 의미 충돌 집합: 시드 1, 가족당 16쌍, 배경 정의 100개

생성: `python3 dev-docs/experiments/commute/semantic.py run P.jsonl --work W`(보고의 마지막 절). `P.jsonl`은 `semantic.py gen`의 동결 판(`MANIFEST`).

| 판정 | 충돌 판정 | 그중 쓰기 겹침(병합 없음) | 겹친 정의를 재증명한 쌍 | 해소(병합 가능) | **위반(해소인데 병합 실패)** | 병합 실패 중 최종 충돌 | 거짓 충돌: 단계 전 → 후 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `none` + 재증명 | 157 | 0 | 157 | 99 | 0 | 58 / 58 | 99 → 0 |
| `world` + 재증명 | 114 | 0 | 114 | 56 | 0 | 58 / 58 | 56 → 0 |

| 판정 | 재증명한 쌍 | 겹친 정의 재증명: 정의 · 의무 · CPU초 | 병합 전체(캐시 없음): 정의 · 의무 · CPU초 | CPU 비율 | 의무 비율 | 면제 질의 CPU초(이 쌍들) |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| `none` | 157 | 141 · 282 · 9.3 | 10466 · 55634 · 298.5 | 0.031 | 0.005 | 0.0 |
| `world` | 114 | 98 · 208 · 6.9 | 7279 · 38673 · 207.6 | 0.033 | 0.005 | 18.5 |

같은 `world` 쌍 114개의 병합을 따뜻한 캐시(기준·A·B를 증명한 증분 캐시)로 증명하면 다시 인코딩한 정의 196개, CPU 14.9초다.

| 가족 | 쌍 | 병합 실패 | `world` 충돌 | 쓰기 겹침 | 해소 | 위반 | 거짓 충돌 전 → 후 | 재증명 정의 · 의무 | 병합 전체 정의 · 의무 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| `body_vs_caller` | 16 | 0 | 0 | 0 | 0 | 0 | 0 → 0 | 0 · 0 | 0 · 0 |
| `independent` | 16 | 0 | 0 | 0 | 0 | 0 | 0 → 0 | 0 · 0 | 0 · 0 |
| `opaque_rewrite` | 16 | 10 | 10 | 0 | 0 | 0 | 0 → 0 | 10 · 10 | 740 · 3930 |
| `post_strengthen` | 16 | 0 | 0 | 0 | 0 | 0 | 0 → 0 | 0 · 0 | 0 · 0 |
| `post_weaken` | 16 | 6 | 16 | 0 | 10 | 0 | 10 → 0 | 16 · 32 | 1184 · 6304 |
| `pre_narrow` | 16 | 4 | 16 | 0 | 12 | 0 | 12 → 0 | 16 · 16 | 1184 · 6288 |
| `pre_strengthen` | 16 | 6 | 11 | 0 | 5 | 0 | 5 → 0 | 11 · 44 | 825 · 4389 |
| `pre_weaken` | 16 | 0 | 0 | 0 | 0 | 0 | 0 → 0 | 0 · 0 | 0 · 0 |
| `refine_vs_spec` | 16 | 6 | 16 | 0 | 10 | 0 | 10 → 0 | 16 · 16 | 1184 · 6288 |
| `remove_vs_call` | 16 | 16 | 16 | 0 | 0 | 0 | 0 → 0 | 0 · 0 | 0 · 0 |
| `spec_body` | 16 | 4 | 13 | 0 | 9 | 0 | 9 → 0 | 13 · 26 | 962 · 5122 |
| `two_weakenings` | 16 | 6 | 16 | 0 | 10 | 0 | 10 → 0 | 16 · 64 | 1200 · 6352 |

### BCB 쌍: 프로필 `contract-heavy`, 각자 커밋 10개

생성: `python3 dev-docs/experiments/commute/run.py --span 10 --every 25 --variants none,reader,world`(보고의 마지막 절). 표본은 위 T1-27 절과 같다.

| 판정 | 충돌 판정 | 그중 쓰기 겹침(병합 없음) | 겹친 정의를 재증명한 쌍 | 해소(병합 가능) | **위반(해소인데 병합 실패)** | 병합 실패 중 최종 충돌 | 거짓 충돌: 단계 전 → 후 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `none` + 재증명 | 20 | 10 | 10 | 10 | 0 | 10 / 10 | 10 → 0 |
| `world` + 재증명 | 18 | 10 | 8 | 8 | 0 | 10 / 10 | 8 → 0 |

| 판정 | 재증명한 쌍 | 겹친 정의 재증명: 정의 · 의무 · CPU초 | 병합 전체(캐시 없음): 정의 · 의무 · CPU초 | CPU 비율 | 의무 비율 | 면제 질의 CPU초(이 쌍들) |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| `none` | 10 | 15 · 63 · 3.7 | 8311 · 42586 · 460.1 | 0.008 | 0.001 | 0.0 |
| `world` | 8 | 9 · 42 · 3.0 | 6692 · 34191 · 367.0 | 0.008 | 0.001 | 38.0 |

| k | 병합 증명 | `none` 충돌 원소 · 재증명 정의 · 의무 · 해소 | `world` 충돌 원소 · 재증명 정의 · 의무 · 해소 | 병합 전체 정의 · 의무 | CPU초: 재증명 none · world · 병합 전체 |
| ---: | --- | --- | --- | --- | --- |
| 100 | True | 2 · 2 · 16 · 해소 | 1 · 1 · 10 · 해소 | 760 · 4032 | 0.55 · 0.44 · 38.7 |
| 250 | True | 2 · 2 · 7 · 해소 | 1 · 1 · 4 · 해소 | 783 · 4110 | 0.36 · 0.34 · 41.0 |
| 300 | True | 1 · 1 · 3 · 해소 | 교환 | 794 · 4145 | 0.31 · 0.00 · 43.1 |
| 525 | True | 1 · 1 · 3 · 해소 | 교환 | 825 · 4250 | 0.35 · 0.00 · 50.0 |
| 550 | True | 1 · 1 · 3 · 해소 | 1 · 1 · 3 · 해소 | 834 · 4278 | 0.32 · 0.39 · 49.9 |
| 625 | True | 4 · 3 · 12 · 해소 | 2 · 2 · 9 · 해소 | 838 · 4291 | 0.41 · 0.51 · 48.1 |
| 650 | True | 1 · 1 · 3 · 해소 | 1 · 1 · 3 · 해소 | 845 · 4299 | 0.29 · 0.31 · 47.6 |
| 675 | True | 2 · 2 · 9 · 해소 | 1 · 1 · 6 · 해소 | 850 · 4312 | 0.36 · 0.34 · 48.2 |
| 875 | True | 2 · 1 · 4 · 해소 | 2 · 1 · 4 · 해소 | 886 · 4412 | 0.35 · 0.32 · 45.8 |
| 925 | True | 1 · 1 · 3 · 해소 | 1 · 1 · 3 · 해소 | 896 · 4457 | 0.40 · 0.32 · 47.8 |

### BCB 쌍: 프로필 `body-heavy`, 각자 커밋 10개

생성: `python3 dev-docs/experiments/commute/run.py --profile body-heavy --span 10 --every 25 --variants none,reader,world`(보고의 마지막 절). 표본은 위 T1-27 절과 같다.

| 판정 | 충돌 판정 | 그중 쓰기 겹침(병합 없음) | 겹친 정의를 재증명한 쌍 | 해소(병합 가능) | **위반(해소인데 병합 실패)** | 병합 실패 중 최종 충돌 | 거짓 충돌: 단계 전 → 후 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `none` + 재증명 | 12 | 8 | 4 | 4 | 0 | 8 / 8 | 4 → 0 |
| `world` + 재증명 | 9 | 8 | 1 | 1 | 0 | 8 / 8 | 1 → 0 |

| 판정 | 재증명한 쌍 | 겹친 정의 재증명: 정의 · 의무 · CPU초 | 병합 전체(캐시 없음): 정의 · 의무 · CPU초 | CPU 비율 | 의무 비율 | 면제 질의 CPU초(이 쌍들) |
| --- | ---: | --- | --- | ---: | ---: | ---: |
| `none` | 4 | 4 · 27 · 1.5 | 3118 · 16388 · 148.8 | 0.010 | 0.002 | 0.0 |
| `world` | 1 | 1 · 6 · 0.4 | 748 · 3988 · 34.2 | 0.011 | 0.002 | 3.9 |

| k | 병합 증명 | `none` 충돌 원소 · 재증명 정의 · 의무 · 해소 | `world` 충돌 원소 · 재증명 정의 · 의무 · 해소 | 병합 전체 정의 · 의무 | CPU초: 재증명 none · world · 병합 전체 |
| ---: | --- | --- | --- | --- | --- |
| 25 | True | 1 · 1 · 6 · 해소 | 1 · 1 · 6 · 해소 | 748 · 3988 | 0.34 · 0.37 · 34.2 |
| 200 | True | 1 · 1 · 9 · 해소 | 교환 | 768 · 4076 | 0.34 · 0.00 · 35.5 |
| 475 | True | 1 · 1 · 9 · 해소 | 교환 | 799 · 4160 | 0.44 · 0.00 · 36.6 |
| 675 | True | 1 · 1 · 3 · 해소 | 교환 | 803 · 4164 | 0.36 · 0.00 · 42.6 |

## 겹친 읽기가 큰 편집: 정제 면제와 재증명의 비용 (백로그 T1-31, 2026-10-30)

> 컴파일러: 이 커밋의 `target/debug/bmb`(T1-29 뒤 컴파일러 변경은 T2-05의 반례 탐침뿐). 하네스: 새 생성기·측정기 `hot.py`, 판정은 `judge.py`. 쌍 집합은 `hot.py gen`의 동결 판이다(`MANIFEST`의 `pairs/hot-s1-d1000.jsonl`). 측정 뒤 `judge.py`에 `decide`(권장 판정)·`resolve`의 캐시 인자·질의 생략(면제 변형을 묻지 않을 때)을 더했다. `hot.py`가 쓰는 경로(`world`를 함께 물음)는 바뀌지 않는다. 의미 충돌 집합 앞 48쌍(가족 셋)을 고치기 전후의 `judge.py`로 다시 돌려 판정 행이 같음을 확인했다(CPU 초 제외).

### 왜 이 집합인가

T1-29의 집합은 겹친 읽기가 쌍당 1~2개 정의였다. 그래서 "많이 불리는 계약이 바뀌어 상대가 그 호출자를 모두 다시 증명해야 하는 경우에도 재증명이 싼가, 면제가 이기는가"를 재지 못했다(T1-29 해석 6). 이 집합은 겹침을 일부러 크게 만든다.

- **꼴:** 계약 f를 호출자 N개가 부른다(배경: BCB 작업 공간 정의 1,000개, `semantic.py`와 같은 생성기). A는 f의 계약을, B는 N개 호출자 모두를 건드린다. 그래서 두 편집이 모두 N개 호출자를 다시 증명하고, 겹친 읽기가 N개 정의다.
- **가족:**
  - `hot_sweep`: A는 f의 `requires` 절 하나를 뺀다(정제). B는 모든 호출자의 인자를 바꾼다. 병합이 증명된다.
  - `hot_two`: A는 f를 위처럼 정제하고, B는 g의 `ensures`에 절을 더한다(정제). 모든 호출자가 f와 g를 함께 부른다. 병합이 증명된다.
  - `hot_break`(음성 대조): A는 f에 모든 기준 호출자가 지키는 `requires` 절을 더한다. B가 바꾼 인자 중 하나가 그 절을 어긴다. 병합이 실패한다.
- **축:** N ∈ {1, 4, 16, 64}. 호출자의 `#[uses]`가 모두 같은 `shared`(면제 질의가 편집마다 (피호출자, 공개 집합) 한 행)와 호출자마다 자기 `spec fn`을 하나 더 펼치는 `distinct`(N행). `shared`가 면제에 가장 유리한 꼴이다.
- **쌍마다 재는 것:** 판정(`none`, `world`)과 결정 4의 재증명(캐시 없음, T1-29와 같음), 병합 전체의 차가운 증명, 그리고 **빌드 게이트**: 기준·A·B를 증명한 따뜻한 증분 캐시로 병합을 증명할 때 다시 인코딩하는 정의와 CPU(ADR-0009: 병합된 판도 바이너리 전에 게이트를 지난다). 또 그 캐시의 사본에서 `none`의 읽는 정의를 먼저 `--cache`로 증명한 뒤의 게이트.
- **CPU:** 자식 프로세스의 사용자+시스템 시간이다(`judge.Bmb`). 공유 머신의 부하는 들어가지 않는다.

### 요약

| 구분 | 쌍 | 판정 | 위반 | 병합 실패 중 최종 충돌 | 거짓 충돌(단계 뒤) | 면제 질의 CPU초 | 재증명 CPU초(정의 · 의무) | 게이트: 다시 인코딩한 정의 · CPU초 |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- | --- |
| 병합이 증명되는 쌍(`hot_sweep`·`hot_two`) | 16 | `world`: 16쌍 모두 교환(재증명 없음) | 0 | — | 0 | 50.5 | 0 | 356 · 17.9 |
| 같은 쌍 | 16 | `none`: 16쌍 모두 충돌 → 재증명으로 해소 | 0 | — | 0 | 0 | 7.0(339 · 679) | 356 · 17.9. 재증명을 캐시로 하면 17 · 재증명 포함 22.4 |
| 병합이 실패하는 쌍(`hot_break`) | 8 | 두 판정 모두 충돌, 재증명 실패 | 0 | 8 / 8 | 0 | 17.6(`world`) | `none` 3.1, `world` 3.0 | 178 · 8.9 |

- 24쌍 모두 기준·A·B가 각자 증명되었다. 쌍당 `bmb reads` 3회(두 판정에 공통)는 2.4~3.1초다.
- 병합 전체의 차가운 증명은 24쌍에 687.5초다. `none`의 재증명(10.1초)은 그 1.5%다.

### N과 공개 집합별 CPU초(가족 둘, 병합이 증명되는 쌍)

| N | `hot_sweep` shared: 질의 · 재증명 | `hot_sweep` distinct: 질의 · 재증명 | `hot_two` shared: 질의 · 재증명 | `hot_two` distinct: 질의 · 재증명 | 게이트 다시 인코딩(가족마다) |
| ---: | --- | --- | --- | --- | ---: |
| 1 | 1.4 · 0.3 | 1.4 · 0.2 | 2.7 · 0.2 | 3.0 · 0.3 | 2 |
| 4 | 1.5 · 0.2 | 1.5 · 0.2 | 2.6 · 0.3 | 3.2 · 0.3 | 5 |
| 16 | 1.4 · 0.2 | 2.5 · 0.3 | 2.9 · 0.5 | 4.9 · 0.5 | 17 |
| 64 | 1.4 · 0.4 | 5.7 · 0.5 | 3.1 · 1.3 | 11.2 · 1.3 | 65 |

"질의"는 `world`의 면제 질의(`weaken --readers`, 두 편집 합), "재증명"은 `none`의 겹친 정의 재증명(`verify --def`, 캐시 없음, 병합 전체 검사 포함)이다. `world`는 이 쌍들에서 재증명이 없다.

### 해석

1. **판정의 질은 여기서도 같다.** 두 판정 모두 위반 0, 병합 실패 8/8을 충돌로 남겼고, 단계 뒤 거짓 충돌 0이다. `world`는 병합이 증명되는 16쌍을 재증명 없이 교환으로 판정했고(면제가 의도대로 동작), `none`은 같은 16쌍을 N개 정의의 재증명으로 해소했다.
2. **겹친 읽기가 커도 면제 질의가 더 비쌌다.** 병합이 증명되는 16쌍에서 질의 50.5초 대 재증명 7.0초(7.2배)다. 면제에 가장 유리한 칸(`shared`, N=64)에서도 1.4 대 0.4, 3.1 대 1.3이다.
   - 질의는 편집마다 고정 비용이 크다(정의 1,000개에서 약 1.4초. 코드 경로상 두 판과 둘을 정렬한 판을 각각 검사하는 몫이고, 나눠 재지 않았다). `distinct`에서는 행마다 약 0.07초가 붙는다(한 정의를 공개 집합마다 따로 비교하는 회차).
   - 재증명은 병합 검사 약 0.2초에, 읽는 정의 하나에 약 0.003~0.017초가 붙는다(이 집합의 호출자는 의무 1~3개).
   - 외삽(측정 아님): `shared`의 기울기를 직선으로 늘리면 판정만의 비용은 N이 수백일 때 뒤집힐 수 있다. 그러나 다음 항목 때문에 전체 비용은 뒤집히지 않는다.
3. **면제는 빌드 게이트의 일을 덜지 않는다.** 증분 키는 피호출자 계약을 담으므로(ADR-0043) 병합의 게이트는 `world`가 교환으로 본 쌍에서도 N개 호출자를 모두 다시 인코딩했다(N=64에서 65개, 16쌍에 356개). 면제는 판정의 근거일 뿐 증명 캐시에 들어가지 않기 때문이다.
   - 반대로 결정 4의 재증명을 게이트와 같은 캐시로 하면, 게이트가 다시 인코딩하는 정의가 356 → 17개(쌍마다 1~2개)가 된다. 재증명과 게이트를 합한 CPU는 22.4초로 게이트만(17.9초)보다 4.5초 많다. 이 차이는 프로그램 전체 검사를 한 번 더 하는 몫이다(쌍당 약 0.28초).
   - 즉 판정 + 게이트의 총비용은 `none` + 캐시 재증명이 게이트 + 0.28초/쌍, `world`가 게이트 + 질의(1.4~11.2초/쌍)다.
4. **음성 대조:** `hot_break`에서 두 판정 모두 충돌이고, N개 중 하나가 어긴 재증명이 실패했다. 재증명을 캐시로 해도 실패한 정의는 캐시에 남지 않아 게이트가 다시 다룬다(쌍마다 2개).
5. **한계:**
   - 호출자는 생성기가 만든 작은 정의다(의무 1~3개, BCB의 읽는 정의는 평균 약 4.7개). 읽는 정의가 크면 재증명의 기울기가 커진다. 그래도 항목 3의 게이트 비교는 크기와 무관하다: 게이트는 면제와 상관없이 그 정의들을 증명한다.
   - 면제 질의의 고정 비용(두 판과 정렬한 판의 검사)은 지금 구현의 것이다. 이것을 줄여도 항목 3은 그대로다.
   - N은 64까지만 쟀다.
### 쌍별 결과

생성: `python3 dev-docs/experiments/commute/hot.py gen P.jsonl`(동결: `MANIFEST`), 측정: `hot.py run P.jsonl --work W --jobs 2 --rows R.jsonl`(보고 전체).

쌍 24개 중 기준·A·B가 각자 증명된 쌍 24개(표본 제외: 없음).

| k | 가족 | 호출자 `#[uses]` | N | 병합 증명 | `none`: 판정 · 재증명 정의 · 의무 · 해소 | `world`: 판정 · 면제 질의 행 · 재증명 정의 · 해소 | CPU초: 읽기 집합(`reads` 3회) | 면제 질의 | 재증명 `none` | 재증명 `world` | 병합 전체(캐시 없음) | 게이트(따뜻한 캐시): 정의 · 초 | 재증명 뒤 게이트: 정의 · 초(재증명 · 게이트) |
| ---: | --- | --- | ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| 0 | `hot_break` | shared | 1 | False | 충돌 · 1 · 2 · 실패 | 충돌 · 1 · 1 · 실패 | 2.8 | 1.5 | 0.2 | 0.2 | 27.3 | 2 · 1.0 | 2 · 1.2(0.3 · 0.9) |
| 1 | `hot_break` | shared | 4 | False | 충돌 · 4 · 8 · 실패 | 충돌 · 1 · 4 · 실패 | 2.8 | 1.5 | 0.3 | 0.3 | 26.9 | 5 · 1.2 | 2 · 1.2(0.3 · 0.8) |
| 2 | `hot_break` | shared | 16 | False | 충돌 · 16 · 32 · 실패 | 충돌 · 1 · 16 · 실패 | 2.5 | 1.3 | 0.3 | 0.3 | 27.9 | 17 · 1.0 | 2 · 1.3(0.4 · 0.9) |
| 3 | `hot_break` | shared | 64 | False | 충돌 · 64 · 128 · 실패 | 충돌 · 1 · 64 · 실패 | 2.5 | 1.5 | 0.7 | 0.6 | 27.9 | 65 · 1.3 | 2 · 1.7(0.8 · 0.9) |
| 4 | `hot_break` | distinct | 1 | False | 충돌 · 1 · 2 · 실패 | 충돌 · 1 · 1 · 실패 | 2.7 | 1.4 | 0.2 | 0.2 | 28.1 | 2 · 0.9 | 2 · 1.3(0.3 · 1.0) |
| 5 | `hot_break` | distinct | 4 | False | 충돌 · 4 · 8 · 실패 | 충돌 · 4 · 4 · 실패 | 2.6 | 1.5 | 0.2 | 0.2 | 28.6 | 5 · 1.0 | 2 · 1.2(0.3 · 0.9) |
| 6 | `hot_break` | distinct | 16 | False | 충돌 · 16 · 32 · 실패 | 충돌 · 16 · 16 · 실패 | 2.8 | 2.6 | 0.4 | 0.4 | 32.5 | 17 · 1.1 | 2 · 1.3(0.4 · 0.9) |
| 7 | `hot_break` | distinct | 64 | False | 충돌 · 64 · 128 · 실패 | 충돌 · 64 · 64 · 실패 | 2.9 | 6.3 | 0.8 | 0.8 | 34.9 | 65 · 1.4 | 2 · 1.8(0.8 · 1.0) |
| 8 | `hot_sweep` | shared | 1 | True | 충돌 · 1 · 1 · 해소 | 교환 · 1 · 0 · — | 2.5 | 1.4 | 0.3 | 0.0 | 26.7 | 2 · 0.9 | 1 · 1.1(0.3 · 0.9) |
| 9 | `hot_sweep` | shared | 4 | True | 충돌 · 4 · 4 · 해소 | 교환 · 1 · 0 · — | 2.5 | 1.5 | 0.2 | 0.0 | 26.1 | 5 · 0.9 | 1 · 1.2(0.3 · 0.9) |
| 10 | `hot_sweep` | shared | 16 | True | 충돌 · 16 · 16 · 해소 | 교환 · 1 · 0 · — | 2.6 | 1.4 | 0.2 | 0.0 | 26.4 | 17 · 0.9 | 1 · 1.1(0.3 · 0.9) |
| 11 | `hot_sweep` | shared | 64 | True | 충돌 · 63 · 63 · 해소 | 교환 · 1 · 0 · — | 2.6 | 1.4 | 0.4 | 0.0 | 27.1 | 65 · 1.0 | 2 · 1.3(0.4 · 0.9) |
| 12 | `hot_sweep` | distinct | 1 | True | 충돌 · 1 · 1 · 해소 | 교환 · 1 · 0 · — | 2.4 | 1.4 | 0.2 | 0.0 | 25.9 | 2 · 0.9 | 1 · 1.2(0.3 · 0.9) |
| 13 | `hot_sweep` | distinct | 4 | True | 충돌 · 4 · 4 · 해소 | 교환 · 4 · 0 · — | 2.6 | 1.5 | 0.2 | 0.0 | 26.3 | 5 · 1.0 | 1 · 1.1(0.3 · 0.8) |
| 14 | `hot_sweep` | distinct | 16 | True | 충돌 · 16 · 16 · 해소 | 교환 · 16 · 0 · — | 2.5 | 2.5 | 0.3 | 0.0 | 27.6 | 17 · 0.9 | 1 · 1.3(0.3 · 0.9) |
| 15 | `hot_sweep` | distinct | 64 | True | 충돌 · 64 · 64 · 해소 | 교환 · 64 · 0 · — | 2.9 | 5.7 | 0.5 | 0.0 | 31.5 | 65 · 1.1 | 1 · 1.5(0.5 · 1.0) |
| 16 | `hot_two` | shared | 1 | True | 충돌 · 1 · 3 · 해소 | 교환 · 2 · 0 · — | 2.7 | 2.7 | 0.2 | 0.0 | 26.7 | 2 · 0.9 | 1 · 1.2(0.3 · 0.9) |
| 17 | `hot_two` | shared | 4 | True | 충돌 · 4 · 12 · 해소 | 교환 · 2 · 0 · — | 2.6 | 2.6 | 0.3 | 0.0 | 31.5 | 5 · 1.0 | 1 · 1.2(0.4 · 0.9) |
| 18 | `hot_two` | shared | 16 | True | 충돌 · 16 · 48 · 해소 | 교환 · 2 · 0 · — | 2.9 | 2.9 | 0.5 | 0.0 | 29.5 | 17 · 1.2 | 1 · 1.5(0.5 · 1.0) |
| 19 | `hot_two` | shared | 64 | True | 충돌 · 64 · 192 · 해소 | 교환 · 2 · 0 · — | 3.1 | 3.1 | 1.3 | 0.0 | 31.1 | 65 · 2.0 | 1 · 2.4(1.3 · 1.1) |
| 20 | `hot_two` | distinct | 1 | True | 충돌 · 1 · 3 · 해소 | 교환 · 2 · 0 · — | 2.8 | 3.0 | 0.3 | 0.0 | 27.1 | 2 · 1.1 | 1 · 1.3(0.3 · 1.0) |
| 21 | `hot_two` | distinct | 4 | True | 충돌 · 4 · 12 · 해소 | 교환 · 8 · 0 · — | 2.7 | 3.2 | 0.3 | 0.0 | 28.0 | 5 · 1.0 | 1 · 1.2(0.3 · 0.9) |
| 22 | `hot_two` | distinct | 16 | True | 충돌 · 16 · 48 · 해소 | 교환 · 32 · 0 · — | 2.6 | 4.9 | 0.5 | 0.0 | 27.8 | 17 · 1.2 | 1 · 1.5(0.5 · 1.0) |
| 23 | `hot_two` | distinct | 64 | True | 충돌 · 64 · 192 · 해소 | 교환 · 128 · 0 · — | 3.0 | 11.2 | 1.3 | 0.0 | 33.8 | 65 · 1.8 | 1 · 2.3(1.3 · 1.0) |

| 판정 | 교환 | 충돌 | 해소 | **위반** | 병합 실패 중 최종 충돌 | 거짓 충돌(단계 뒤) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `none` + 재증명 | 0 | 24 | 16 | 0 | 8 / 8 | 0 |
| `world` + 재증명 | 16 | 8 | 0 | 0 | 8 / 8 | 0 |
