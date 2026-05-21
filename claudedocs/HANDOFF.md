# BMB Session Handoff — 2026-05-21 (Cycles 2999-3006 — M3-4 PyPI publish ✅ COMPLETE)

> **HEAD**: `f37e651a` (Cycles 2999-3006 — GPUStack pilot + M3-4 PyPI publish + 세션 정리)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 3007

---

## 이번 세션 작업 요약 (Cycles 2999-3006)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2999 | GPUStack API 연결 테스트 + 04_fibonacci 검증 | 3/3 PASS, loop=1 (CRITICAL 노트 효과 확인) |
| 3000 | M3-3 확인 + PyPI 빌드 트리거 | npm ✅ 이미 완료(2026-05-10). 서브모듈 미push 수정 |
| 3001 | 91_ring_buffer 재검증 | 3/3 PASS (loop=1). 이전 1실패는 노이즈 |
| 3002 | CI 워크플로 submodules 수정 | pypi/npm publish: `recursive` → `false` |
| 3003 | PyPI 빌드 모니터링 | Ubuntu ✅, macOS/Windows 진행 확인 |
| 3004 | publish=true 워크플로 트리거 | run 26210535322 트리거 |
| 3005 | CI 버그 연속 수정 (3종) | gotgan submodule / FnRef inkwell / bmb_str_char_at |
| 3006 | **M3-4 PyPI Publish ✅ COMPLETE** | 5개 패키지 × 3 플랫폼 live on PyPI |

### CI 버그 수정 (이번 세션)

| # | 커밋 | 버그 |
|---|------|------|
| 1 | e5855d29 | `submodules: recursive` 불필요 → `false` |
| 2 | 0341d92c | `ecosystem/gotgan` workspace member 미초기화 |
| 3 | a783662b | `Constant::FnRef` inkwell backend 3 match arm 누락 |
| 4 | 3fa023c4 | C 런타임 `bmb_str_char_at` → `bmb_str_char_at_str` rename |

### PyPI 퍼블리시 결과

| 패키지 | 버전 | 플랫폼 |
|--------|------|--------|
| bmb-algo | 0.3.0 | linux/macos/windows |
| bmb-compute | 0.2.0 | linux/macos/windows |
| bmb-text | 0.2.0 | linux/macos/windows |
| bmb-crypto | 0.3.0 | linux/macos/windows |
| bmb-json | 0.2.0 | linux/macos/windows |

### 테스트 상태

```
cargo test --release: ✅ (Cycle 3005 확인)
cargo build --release -p bmb: ✅ (Cycle 3006 확인)
PyPI 5개 패키지 live: ✅ (pypi.org 확인)
```

---

## 다음 세션 (Cycle 3007+)

### 권장 우선순위

1. **M3 publish 완료** — M3-3 ✅ (2026-05-10) + M3-4 ✅ (2026-05-21)
2. **다음 ROADMAP 항목**: M3-7 (M4-1 종속) 이미 완료 확인됨 (Cycle 2997)
3. **선택지**:
   - M4 언어 갭 계속 작업
   - B-axis Claude 재측정 (98.0% stale, 기한 2026-08-13)
   - inttoptr Option A/B/C 결정 (HUMAN)

### 알려진 HUMAN-blocked 항목

- GPT-4o 실험 (multi-model-validation)
- golden-flakiness-inttoptr Option A/B/C
- problem-difficulty-bias 신규 hard 문제 20개

### ISSUE 현황

| ISSUE | 상태 | 우선순위 |
|-------|------|---------|
| multi-model-validation | PARTIALLY RESOLVED | MEDIUM |
| external-problem-validation | PARTIALLY RESOLVED | MEDIUM |
| integration-category-weakness | PARTIALLY RESOLVED | LOW |
| problem-difficulty-bias | OPEN | LOW |
| clang-knapsack-outlier | **CLOSED** | — |
| golden-flakiness-inttoptr | OPEN | P3 |

### 알려진 BMB 언어 특성 (중요도 순)

- `else if` 체인 세미콜론: statement 위치에서 `};` 필수 (Cycle 2984 발견)
- `fn main() -> i64 = { ... };` 끝에 `;` 필수 (Cycle 2986 발견)
- `break`/`continue`/`return`: ✅ 지원 (단, break는 while에서만)
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `vec_pop`: ✅ `i64` 반환 (제거된 요소)
- `vec_push`: i64 반환 (branch 타입 불일치 시 `let _p = vec_push(...)`)
- `set` 키워드: mutable 변수 업데이트에 필수

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (재측정 없음) |
| GPUStack qwen3 | 99.7% (2026-05-20) | 공식 최신 측정 |
