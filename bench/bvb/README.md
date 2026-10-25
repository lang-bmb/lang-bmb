# BVB: BMB-Verus Bench (ADR-0034 §2, 백로그 2A-08·2A-09)

공개 Verus 과제(Verus-Bench, Vericoding) 가운데 M1 언어(ADR-0016)로 옮길 수 있는 45개를 BMB 과제로 이식하고, 같은 과제를 원본 Verus 툴체인에서도 같은 규칙으로 풀게 해 같은 모델의 1회 성공률을 직접 비교한다. 출처·라이선스·선정 기록은 `SOURCES.md`.

| 파일 | 내용 |
| --- | --- |
| `prompt.md` | 에이전트 시스템 프롬프트(동결). `bench/m1/prompt.md`와 바이트 단위로 같다 |
| `seed/` | 시작 작업 공간(동결). 단위 `given` 하나: 원본이 명세에 쓰는 재귀·트리거용 명세 함수(규칙 G1) 11개 |
| `tasks/` | 과제 45개(동결): Verus-Bench 30(`vb-*`), Vericoding 15(`vc-*`). 모두 작성 과제 |
| `MANIFEST` | 동결 파일의 SHA-256. 다르면 두 하네스 모두 실행을 거부한다 |
| `port/`, `port.py` | 과제 원천(사람이 읽는 형식). `port.py build`가 `tasks/`를 만들고 `port.py check`가 어긋남을 잡는다 |
| `sources.py`, `licenses/` | 원본 고정 커밋, 선정 기록(모든 원본 과제의 결정과 사유), 출처 확인 |
| `verus/` | 원본 Verus 비교군 하네스(2A-09): 설치, 프롬프트, 판정, 비교 표 |

```
python3 bench/m1/harness.py --set bench/bvb manifest                       # 동결 확인
python3 bench/m1/harness.py --set bench/bvb run --agent oracle --jobs 4    # BMB 기준 해법: 전부 성공(게이트)
python3 bench/m1/harness.py --set bench/bvb run --agent weak --jobs 4      # BMB 약한 대조군: 전부 실패(게이트)
bench/bvb/verus/install.sh [--dry-run]                                     # 고정 Verus 릴리스 설치
python3 bench/bvb/verus/harness.py run --agent oracle --jobs 4             # Verus 원본 해법: 전부 성공(CI bvb-verus)
python3 bench/bvb/verus/harness.py run --agent weak --jobs 4               # Verus 약한 대조군: 전부 실패(CI bvb-verus)
python3 bench/bvb/verus/harness.py compare BMB.jsonl VERUS.jsonl           # 과제별 BMB·Verus 나란히
```

## 과제 형식

`m1b` 작성 과제와 같은 키(`prompt`, `signature`, `accept`, `reference`, `weak_reference`)에 다음을 더한다. `bench/m1/harness.py`는 `m1`·`m1b`의 판정을 바꾸지 않고 이 키를 읽는다.

| 키 | 뜻 |
| --- | --- |
| `source` | 원본 벤치마크, 원본 과제(`origin`, `path`), 적용한 이식 규칙 |
| `accept.specs`의 `ref_*` 보조 술어 | 숨긴 기준 계약이 쓰는 보조 술어. 판정용 정의의 `#[uses]`에 모두 들어간다(전에는 `ref_pre`·`ref_post`만) |
| `frozen` | 숨긴 계약이 이름으로 부르는 `given` 정의. 판정 때 정의 해시가 씨앗과 같아야 한다(`given_intact`). 바꿔서 명세 검사를 통과시키는 답을 막는다 |
| `verus` | 원본 Verus 쪽: 같은 설명에 원본 시그니처를 붙인 `prompt`, `signature`, `target`, 원본 명세(`specs`·`pre`·`post`, 보조 함수는 `bvb_ref_*`로 이름만 바꿈), 검증된 해법 `reference`와 그 출처 `reference_origin`, G1 과제의 `given` |

두 언어의 프롬프트는 같은 설명(`description`)을 쓰고 끝의 시그니처만 다르다. BMB 쪽은 "Create unit `tasks`" 한 줄이 앞에 붙는다. G1 과제는 두 쪽 모두 주어진 명세 함수의 코드를 프롬프트에 보여 준다.

## 이식 규칙

| 규칙 | 원본(Verus) | BMB |
| --- | --- | --- |
| T1 | `&Vec<T>`, `Vec<T>`(읽기만), `&[T]`; 길이를 바꾸지 않는 `&mut Vec<T>` | `&[T']`; `&mut [T']` |
| T2 | `i8`·`i16`·`i32` / `u8`·`u16`·`u32` / 원소 `usize` | `i64` / `u64` / `u64`. 원본 타입의 범위가 계산이나 명세에 영향을 주면(넘침, 입력 영역) 그 범위를 숨긴 `requires`와 설명에 넣는다. 영향이 없으면 넣지 않는다(64비트 전체에서 맞아야 하므로 과제가 약해지지 않는다) |
| T3 | `exists\|x\| P` | `!(forall\|x\| !P)` (M1에 `exists` 없음) |
| T4 | `Seq` 위의 재귀 명세(`drop_last`/`last`) | 접두 길이 `n`에 대한 재귀 명세 함수 |
| T5 | `int` 명세 함수 | 같은 식의 `i64` 명세 함수(명세 산술은 무한 정수, ADR-0017 §2) |
| T6 | 그 자체로 항상 참인 절(`exists\|i\| (범위 ==> ..)`) | 뜻(`true`)대로 뺀다. Verus 쪽 숨긴 계약에서도 뺀다(`vb-misc-findmax`) |
| T7 | `int`의 `/`·`%`(유클리드) | 음수 피제수에서는 `(n % d + d) % d`와 그것으로 맞춘 몫으로 옮긴다(BMB는 0 쪽 자름) |
| G1 | 원본 명세가 재귀 명세 함수이거나, 양화사 트리거가 산술식이라 이름 붙은 술어가 필요한 경우 | 그 함수를 원본처럼 과제에 **준다**(BMB `given::f`, Verus 시작 파일). 두 계약을 이름이 다른 재귀 함수로 쓰면 함의 증명에 귀납이 필요해 명세 강도 검사가 의미를 잃기 때문이다. 원본 Verus-Bench·Vericoding 과제도 이 함수를 주었다 |

숨긴 계약은 원본 명세의 번역이다. 다만 양화사의 트리거는 배열 원소 항(`a[i]`)에 둔다(Verus 쪽도 같고, 이 때문에 원본 해법 하나의 트리거 표기를 바꿨다: `vb-mbpp-113`, `reference_origin: upstream-triggers`). 원본 해법 둘(`vb-misc-reverse`, `vb-misc-map`)은 고정 Verus 버전의 `&mut` 사후 조건 문법(`final(v)`)으로 옮겼다(`upstream-migrated`). 원본 해법이 없는 Vericoding 과제 5개의 Verus 해법은 BVB에서 썼다(`bvb`).

## 판정

- **BMB:** ADR-0021 §2 그대로(작업 공간 전체 증명, 시그니처, 숨긴 계약 함의) + G1 과제의 `given_intact`.
- **Verus(`verus/harness.py`):** 같은 생각을 Verus로 옮겼다. (1) 마지막 파일이 `verus --no-cheating`으로 검증된다(`assume`·`admit`·`external_body`·`assume_specification` 거부, BMB M1에 탈출구가 없는 것과 맞춘다). (2) 대상 함수의 시그니처가 같다(`pub`, 공백, 결과 이름 무시). (3) 파일 끝에 `fn bvb_check(..) requires 원본 pre ensures 원본 post { target(..) }`를 붙여 다시 검증된다. G1 과제는 주어진 함수가 그대로 있어야 한다.
- **약한 대조군:** BMB는 과제마다 쓴 약한 명세, Verus는 원본 해법에서 대상 함수의 `ensures`를 지운 것(자동). 둘 다 검증은 되고 명세 검사에서 떨어져야 한다.
- **세션 규칙(공정성):** 과제당 한 세션, 재시도 없음, 도구 호출 40회, 같은 모델 설정(`bench/m1/anthropic_agent.py`의 effort·`max_tokens`). Verus 에이전트의 도구는 `read_file`·`write_file`·`verify` 셋이고 BMB는 MCP 도구다. 둘 다 파일 시스템·셸이 없다. 결과 파일의 `task_set`은 BMB `bvb`, Verus `bvb-verus`이고 `compare`는 두 파일의 MANIFEST가 같을 때만 표를 만든다.

## 실제 모델 비교(사용자 머신, 유료)

```
bench/bvb/verus/install.sh                       # 고정 Verus 0.2026.10.04.426d8b0 + Rust 1.98.1 (약 500MB)
python3 bench/m1/harness.py --set bench/bvb run --agent anthropic:MODEL --out bench/bvb/results/$(date +%F)-MODEL.jsonl
python3 bench/bvb/verus/harness.py run --agent anthropic:MODEL --out bench/bvb/results/$(date +%F)-MODEL-verus.jsonl
python3 bench/bvb/verus/harness.py compare bench/bvb/results/DATE-MODEL.jsonl bench/bvb/results/DATE-MODEL-verus.jsonl
```

## 알려진 한계

- **명세 강도 검사는 트리거에 민감하다(두 언어 모두).** 에이전트의 양화사가 자기 술어 적용(`p(a[i])`)에만 트리거를 두면, 뜻이 같아도 숨긴 계약과의 함의를 해법기가 찾지 못할 수 있다. 숨긴 계약은 원소 항 트리거로 이 위험을 줄였지만 없애지는 못한다. 실제 모델 결과에서 `spec_accepted`만 실패한 과제는 이 가능성을 확인한다.
- **BMB 완전성 문제(발견, 2A-08 중):** `i64` 배열 원소를 인자로 받는 명세 함수는 양화사 안에서 펼쳐지지 않는다. 예: `forall|i| i < a.len() ==> small(#[trigger] a[i])`에서 `forall|i| .. ==> a[i] < 5`를 증명하지 못한다(`u64`는 된다). 원인은 `verify.rs`의 `fresh_elems`가 원소 범위 공리를 부호 없는 원소에만 넣고(1B-30), 명세 함수의 정의 공리는 인자 범위를 가드로 갖기 때문이다(ADR-0017 §4). BVB의 숨긴 계약과 기준 해법은 이 꼴을 피했지만 에이전트는 이 꼴을 자연스럽게 쓴다. 실제 측정 전에 고칠 후보다.
- 선정 편향은 `SOURCES.md` "선택 편향 공개".
