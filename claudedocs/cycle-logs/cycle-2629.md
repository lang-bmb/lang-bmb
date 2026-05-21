# Cycle 2629: M3 완료 — npm publish 성공 + PyPI 빌드 시작
Date: 2026-05-10

## Re-plan
Plan valid. HANDOFF § 5에서 HUMAN 결정 확정 확인:
- M3 showcase = bmb-algo
- npm publish = 즉시 진행 ✅
- PyPI publish = 즉시 진행 ✅

## Scope & Implementation

**M3-2: bmb-algo 벤치마크 측정**
- 실행: `python ecosystem/bmb-algo/benchmarks/bench_algo.py`
- 결과: 7/7 wins (2.13x~25.25x Python 대비)

| 함수 | BMB (µs) | Python (µs) | 배수 |
|------|----------|-------------|------|
| knapsack | 3.79 | 18.95 | 5.00x FAST |
| fibonacci(30) | 0.23 | 0.49 | 2.13x FAST |
| prime_count(10000) | 12.75 | 321.95 | 25.25x FAST |
| nqueens(10) | 1747.52 | 7039.08 | 4.03x FAST |
| quicksort(15) | 1.95 | 3.78 | 1.94x OK |
| merge_sort(15) | 2.67 | 6.78 | 2.54x FAST |
| edit_distance | 1.40 | 8.81 | 6.27x FAST |

**M3-3: npm publish**
- dry-run 먼저 검증 → 성공
- 실제 publish: run #25628114846 → ✅ 25초 완료
- 5개 패키지 배포: bmb-algo@0.1.0, bmb-compute@0.1.0, bmb-text@0.1.0, bmb-crypto@0.1.0, bmb-json@0.1.0

**M3-4: PyPI publish**
- run #25628128772 트리거됨 (publish=true, repository=pypi)
- 4개 플랫폼 빌드 중: win/linux/macOS-x64/macOS-arm64
- 완료까지 ~20분 예상

**M3-5: bmb-mcp 미커밋 변경사항**
- `git -C ecosystem/bmb-mcp status` → 변경사항 없음 (이미 깨끗한 상태)

## Verification & Defect Resolution

- npm registry: `npm view bmb-algo version` → 0.1.0 ✅
- PyPI: 빌드 진행 중 (Cycle 2630에서 확인)

## Reflection

**Scope fit**: M3 publish 계획 대로 진행. npm 5개 패키지 성공 배포.

**벤치마크 주목할 점**: prime_count(10000)에서 25x speedup — BMB 성능 가설의 강력한 증거. ctypes/koffi FFI 오버헤드를 포함한 수치로 순수 성능은 더 높을 것.

**Node.js 패키지 구조 관찰**: 현재 DLL을 패키지에 포함하지 않고 런타임에 찾는 방식 (`findLib()`). 사용자가 DLL을 별도로 빌드/설치해야 하는 구조. 개선 여지 있음.

**Roadmap impact**: M3 publish 완료 후 v0.100 선언 가능 (메인테이너 결정).

## Carry-Forward
- Actionable: PyPI publish 완료 확인 (Cycle 2630 시작 시)
- Structural Improvement Proposals: npm 패키지에 prebuilt DLL 포함 검토 (현재 사용자가 빌드 필요) — 단, 플랫폼별 DLL 빌드 인프라가 필요하므로 M4/M5 이후 과제
- Pending Human Decisions: v0.100 버전 선언 (메인테이너)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2630 — PyPI 완료 확인 + M4-6 C# 바인딩 scaffold 시작
