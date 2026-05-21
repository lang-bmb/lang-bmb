# Cycle 3007: 상태 점검 + GPUStack B축 재측정 준비
Date: 2026-05-21

## Re-plan
Plan valid. M3-4 PyPI ✅ 완료 (이전 세션). 사용자 승인:
- "api-key test 승인(GPUStack)": GPUStack API 키 테스트 진행 허가
- "publish 승인": M3-4 PyPI publish 확인

이번 10-사이클 범위:
1. 3007: 상태 점검 (현재)
2. 3008: v0.100 버전 선언 + CHANGELOG
3. 3009: GPUStack 파일럿 테스트 (3 failing problems: 01/30/86)
4. 3010: Full GPUStack B-axis run (100 × 3 = 300 calls)
5. 3011: 결과 분석 + 실패 문제 fix
6. 3012: 추가 문제 수정 + 재검증
7. 3013: 언어 기능 개선 (결과 기반)
8. 3014: ISSUE triage
9. 3015: ROADMAP M4 업데이트
10. 3016: 세션 wrap-up (HANDOFF + commit)

## Scope & Implementation

### 상태 확인
- `cargo test --release`: ✅ 2390 passed (bmb) + 23 (unit) — 0 failed
- GPUStack 연결: ✅ `http://172.30.1.53:8080/v1`, model=`qwen3.6-35b-a3b`
- bmb-ai-bench dry-run: ✅ 100 problems, api 연결 확인

### pyproject.toml 버그 수정
`pip install -e .` 실행 시 setuptools 패키지 자동 탐색 오류 (results/, problems/, protocol/ 디렉토리 포함):
```
Multiple top-level packages discovered in a flat-layout
```
수정: `[tool.setuptools.packages.find] include = ["bmb_ai_bench*"]` 추가

### 3 failing problems 현황 분석
| 문제 | 마지막 실패 | 원인 가설 |
|------|-----------|---------|
| 01_binary_search | 99.7% 측정 시 1 fail | 루프 종료 조건 `set lo = hi + 1` 미준수 |
| 30_contract_chain | 1 fail | `pre x >= 0 and limit >= 0` 두 조건 동시 필요 |
| 86_heap_sort | 1 fail | 이름은 "heap sort"지만 bubble sort 구현 필요 |

모든 문제 BMB Notes에 완전한 예제가 있음. 실패는 LLM 비결정성 가능성.

## Verification & Defect Resolution
- bmb-ai-bench doctor: ✅ ALL OK (bmb 0.98.0, LLVM, gcc, rustc, python)
- pyproject.toml 수정: ✅ (packages.find 추가)
- GPUStack 연결: ✅ qwen3.6-35b-a3b 사용 가능

## Reflection
- **Scope fit**: 상태 점검 완료. 환경 준비됨.
- **Latent defects**: pyproject.toml 패키지 탐색 버그 (즉시 수정됨).
- **Philosophy drift**: 없음.
- **Roadmap impact**: GPUStack B축 재측정 가능 상태. M3 완료로 v0.100 선언 적기.

## Carry-Forward
- Actionable: v0.100 버전 선언 (Cycle 3008)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: v0.100 버전 선언 → GPUStack 파일럿 테스트 순서
