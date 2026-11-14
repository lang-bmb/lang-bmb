# 포트의 차등 시험 사례

`scripts/difftest.py run tests/port/cases.json`이 읽는다(백로그 4D-15, 포팅 지도 `dev-docs/measurements/selfhost-port-map.md` §3.2). 사례마다 stage0 쪽(`a`)과 포트 쪽(`b`)을 같은 입력으로 돌려 stdout·stderr·종료 코드·남긴 파일을 바이트로 견준다. 게이트 작업 "port difftest (4D-15)"가 매번 돌린다.

| 파일 | 내용 |
| --- | --- |
| `cases.json` | 사례와 빌드 목록 |
| `sha256_files.bmb` | 인자의 파일마다 `std::sha256` 값을 `sha256sum`의 줄 모양으로 |
| `json_roundtrip.bmb` | 표준 입력의 줄마다 `std::json`으로 읽어 공백 없이 다시 쓴다 |
| `json_ref.rs` | 같은 일을 stage0의 `json.rs`로(`#[path]`로 그대로 컴파일) |
| `json_edge.txt` | JSON의 경계 입력(이스케이프, 수, 중첩, 받지 않는 꼴) |

포팅이 시작되면(`4D-02`) 구성 요소마다 사례를 더한다: `a`는 `{bmb}`(stage0), `b`는 `{build:stage1}`.
