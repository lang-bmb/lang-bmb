# Benchmarks Game 프로그램 (백로그 T3-01)

[Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)의 과제를 BMB로 쓴 것이다. 각 프로그램은 라이브러리(`lib/`)와 함께 전부 증명된다(게이트 시험 `game_programs_are_proved`). 실행에 수십 초가 걸려 게이트는 실행하지 않는다.

| 프로그램 | 인자 | 출력 | 증명된 것 | 메모 |
| --- | --- | --- | --- | --- |
| `binarytrees.bmb` | n = 21 | `binarytrees.out` | 패닉·넘침 없음, 재귀의 종료 | 노드 수 `check`의 합은 `wrapping_add`다. "깊이 d 완전 트리의 노드 수 = 2^(d+1) - 1"을 계약으로 쓰려면 귀납 보조 정리가 필요하다(ADR-0062 제안). Rust 제출의 `+`도 릴리스 빌드에서 같은 값을 낸다 |
| `fannkuchredux.bmb` | n = 12 | `fannkuchredux.out` | 패닉·넘침 없음, 모든 인덱스 범위(순열 원소 < 12, 카운터 1..=i+1의 불변식) | 뒤집기 반복의 종료(topswops)는 감소 측정 하나로 증명되지 않는 정리라 연료 1000으로 묶는다(n = 12의 상한은 Knuth의 F(13) = 233). 체크섬은 `wrapping_add`·`wrapping_sub` |

```
target/debug/bmb build bench/game/binarytrees.bmb lib/bytes.bmb -o bt && ./bt | diff - bench/game/binarytrees.out
target/debug/bmb build bench/game/fannkuchredux.bmb lib/bytes.bmb -o fk && ./fk | diff - bench/game/fannkuchredux.out
```

## 상태

- 두 출력은 각 알고리즘이 정하는 값이고, 이 과제들의 알려진 결과(fannkuch-redux 12: `3968050`, `Pfannkuchen(12) = 65`)와 같다. 공식 기대 출력 파일과의 바이트 대조, 공식 Rust 제출과의 시간·크기 비교는 아직 하지 않았다. 공식 소스(salsa.debian.org, benchmarksgame-team.pages.debian.net)가 이 환경의 네트워크 정책에 막혀 있다(T3-01 🚫).
- 단일 스레드다. 공식 Rust 제출 대부분은 여러 스레드를 쓴다. BMB에는 아직 스레드가 없으므로 비교는 단일 스레드 제출 또는 `taskset`으로 한 코어에 묶은 실행과 한다.
