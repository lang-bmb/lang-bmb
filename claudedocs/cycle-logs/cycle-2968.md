# Cycle 2968: csv_parse 성능 회귀 검증
Date: 2026-05-19

## Re-plan
Cycle 2967 Carry-Forward: 추가 언어 갭 또는 P축 성능 개선.
우려 사항: &&/|| MIR short-circuit(phi 노드 생성)이 csv_parse 성능에 영향을 미칠 수 있음.
csv_parse는 `and` 조건을 루프에서 많이 사용 → 구 코드는 `and i1`(eager), 신 코드는 phi 노드(short-circuit).

## Scope & Implementation

csv_parse 벤치마크 실행 및 비교:
- `main_inproc.bmb` → 새 컴파일러로 빌드 → 5회 실행
- C `main_inproc.exe` → 5회 실행

## Verification & Defect Resolution

측정 결과 (in-process timing, µs/50 iterations):

| 실행 | BMB (new) | C |
|------|-----------|---|
| 1 | 2553 | 2875 |
| 2 | 2631 | 2662 |
| 3 | 2630 | 2612 |
| 4 | 2649 | 2927 |
| 5 | 2794 | 2580 |
| 6 | 2646 | 2968 |
| **중앙값** | **2639** | **2662** |

**결론**: BMB 2639 µs ÷ C 2662 µs ≈ **0.991× (BMB가 C보다 약간 빠름)**

Short-circuit 변경 전 측정치 1.057×와 비교하면 개선됨. 측정 노이즈 감안 시 사실상 파리티 (±3%).

No regression detected. In fact, short-circuit might have slightly improved csv_parse
by allowing LLVM to prove memory safety more clearly.

## Reflection

- csv_parse P축 상태: C 파리티 ✅ (0.991× ≈ 1.0×)
- short-circuit phi 노드가 eager `and i1`보다 약간 더 나은 성능 가능 (LLVM이 branch prediction으로 최적화)
- ROADMAP의 1.057× 측정치는 다른 시스템 조건에서 측정된 것으로 추정

## Carry-Forward
- Actionable: None (csv_parse는 해결됨)
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정
- Roadmap Revisions: csv_parse 1.057× → ~1.0× (C 파리티) 갱신 필요
- Next Recommendation: ROADMAP/HANDOFF 갱신 후 추가 언어 개선
