# Z3와 배열에 중첩된 데이터타입 (백로그 2O-03c)

`Vec`을 거친 재귀 열거형(`enum Tree { Leaf(i64), Many(Vec<Tree>) }`)은 증명에서 원소 배열이 그 데이터타입인 필드를 가진다(`(Array Int |Tree|)`, Z3의 중첩 데이터타입). 2O-03c를 구현하며 만족 가능한 질의 하나가 끝나지 않는 것을 보았다.

- `hang.smt2`: 증명기가 낸 질의 그대로(`tests/verify/fail/vec_recursion.bmb`의 `two`, `+` 넘침 의무). Z3 4.15.4(`scripts/toolchain.env`)가 20초 안에 답하지 않고 `rlimit 5000000`에서도 멈추지 않는다(증명기는 `-T 360`까지 기다린다).
- `ok.smt2`: 같은 질의에서 데이터타입 선언만 서곡(`bmb_abs`·`bmb_tdiv`·`bmb_trem` 정의) 앞으로 옮긴 것. 즉시 `sat`.

| 질의 | 결과 | 시간 |
| --- | --- | --- |
| `hang.smt2` | 답 없음 | 20초 제한에 걸림 |
| `ok.smt2` | `sat` | 0.03초 |

정의 하나씩 빼 보면 `bmb_tdiv`·`bmb_trem`(정수 `div`를 쓰는 매크로)이 있을 때만 걸리고, 데이터타입을 그 앞에 두면 걸리지 않는다. 시드 1~4에서도 같다.

조치(`verify::needed_decls`): 모든 질의에서 데이터타입 선언을 서곡 앞에 둔다. 데이터타입은 서곡을 쓰지 않으므로 의미는 같다. 증분 키의 질의 머리(`query_header`)는 서곡을 계속 포함한다.

재현: `z3 hang.smt2`(걸림), `z3 ok.smt2`(`sat`).
