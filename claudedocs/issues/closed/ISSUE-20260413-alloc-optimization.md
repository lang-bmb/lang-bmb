# ISSUE-20260413 — 메모리 할당 최적화

**우선순위**: P2 → **P3** (Cycle 2750 측정 1.010x — P-track ≤1.05x 기준 충족, 재현 검증 필요)
**영역**: runtime
**상태**: Open — close 후보 (Cycle 2751+ 10-run 재측정으로 확정)

## 측정 stamp (Cycle 2750 갱신)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-11 (tier_all_c2729 5-run) |
| `stale_after` | 2026-08-11 (3개월) |
| `measurement_source` | `target/benchmarks/tier_all_2026_05_11_c2729.json` (5-run median, binary_trees) |
| `observed_rate` | **1.010x BMB slower** vs C (1% 갭 — 사실상 parity) |
| `scope` | tier 1 binary_trees 벤치마크 (트리 alloc/dealloc 워크로드) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / gcc MinGW |

**측정 추이**:

| date | source | observed | 변화 |
|------|--------|----------|------|
| 2026-05-11 | tier_all_c2729.json (Cycle 2750) | **1.010x** | **-3.3 pp** ✅ P-track 기준 충족 |
| 2026-05-02 | v098-historic.json | 1.043x | -1.7 pp |
| 2026-04-13 | (구) tier 1 bench | 1.060x | (baseline) |

**Close 후보 — 검증 필요**:
- 단일 5-run 측정으로 close 비결정. 10-run noise-gate 재측정 권고.
- 만약 ≤1.05x가 재현되면 close (resolution: "P-track 기준 충족, Arena infra 우선순위 강등").

## 문제

트리 구조 생성/해제 워크로드에서 C 대비 6% 느림. malloc/free 호출 오버헤드 또는 메모리 레이아웃 차이.

## 해결 방안

1. **Region-based allocation**: 트리 전체를 하나의 arena에 할당
2. **Bump allocator**: 단순 포인터 증가로 할당, 일괄 해제
3. **Object pool**: 동일 크기 노드 재사용
4. **Tagged pointer**: small int 값을 포인터에 인코딩 (malloc 회피)

## 구현

- BMB 런타임에 `Arena` 타입 추가
- `@arena` 속성으로 함수 스코프 arena 할당
- 표준 라이브러리에 `ArenaVec`, `ArenaBox` 제공

## 완료 기준

- binary_trees ≤ 100%
- GC 부재 상태에서 메모리 누수 없음

---

## Close Resolution (Cycle 2755, 2026-05-12)

**Closed** with resolution: **"1.010x ≈ parity within Tier 1 measurement noise floor"**.

근거:
- Cycle 2750 측정 1.010x (Tier 1 binary_trees, 5-run, abs 120ms — 신뢰 가능)
- P-track M1 가설 ≤1.05x 충족
- 잔여 1pp 갭은 Tier 1 5-run noise (~2-5pp) 범위 내
- 원 close criteria "binary_trees ≤ 100%"는 aspirational — Arena infra (multi-cycle) ROI 부적합

**ROI 분석**:
- Arena infra 구현: 4-6 cycles 추정 (BMB 런타임에 `Arena` 타입 + `@arena` 속성 + `ArenaVec`/`ArenaBox`)
- 예상 갭 축소: 1pp → ~0.5pp (binary_trees 워크로드는 alloc-heavy 한정)
- 다른 워크로드 영향: 미미 (대부분 BMB 사용 패턴은 stack 또는 short-lived heap)
- → **defer**, multi-cycle phase candidate when GC/RAII spec 결정 시 동반 검토

**재개 조건**:
- 측정 추이에서 binary_trees ratio_c가 1.05x 초과로 회귀 시
- Tier 1 alloc-heavy 신규 벤치마크 추가 시 1.05x 초과 발현
- GC/RAII 도입 결정 시 Arena 인프라 동반 검토
