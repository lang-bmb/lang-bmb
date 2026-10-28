# H-COMMUTE: 동시 편집의 병합 (백로그 T1-12)

> 2026-10-28. 컴파일러는 2N-01b(`62751306`) 이후 바뀌지 않은 `target/debug/bmb`. 생성: 아래 각 절의 명령. 가설과 교환 관계의 정의는 `dev-docs/reviews/collaboration-2026-10-28.md` §4·§7.

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
