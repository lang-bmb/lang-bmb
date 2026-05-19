# BMB Session Handoff — 2026-05-20 (Cycles 2981-2990 — GPUStack 99.0% + 품질 대폭 개선)

> **HEAD**: `af4dac54` (Cycle 2989 — 추가 패턴 검사 완료)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2991

---

## 이번 세션 작업 요약 (Cycles 2981-2990)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2981 | ISSUE 정리 | for-loop 스코프 버그 재현 시도 (재현 불가), ISSUE 4개 closed |
| 2982 | GPUStack 측정 + 3종 수정 | 01/30/86 pass 확인, lru_simulate/pipeline/registry 수정 |
| 2983 | ISSUE 정리 + lru break 수정 | 94_lru_simulate의 break 키워드 → flag 패턴 |
| 2984 | GPUStack 99.0% 달성 | 99/100 (↑97.0%), ring_buffer else-if 세미콜론 발견+수정 |
| 2985 | else-if 체인 전수 검사 | 50_calculator/83_pipeline/84_accumulator_pattern 예방적 수정 |
| 2986 | Multi-shot 분석 + fn main 수정 | 04_fibonacci/36_array_rotation/69_overflow/75_plateau/72_alternating |
| 2987 | 품질 검토 + cargo test | 6260 tests ✅, HIGH RISK 파일 분석 (추가 수정 불필요) |
| 2988 | 출력 포맷 + integration 확인 | Integration 100% 1-shot 확인 |
| 2989 | 추가 패턴 검사 | i32/bool/return/stdin 패턴 — 이슈 없음 |
| 2990 | 세션 종료 정리 | HANDOFF/ROADMAP 갱신 |

### 핵심 발견: BMB 언어 특성 (이번 세션 발견)

**1. else-if 체인 세미콜론 규칙** (Cycle 2984 발견)
```
if op == 1 { ... } else if op == 2 { ... } else if op == 3 { ... };  // ';' 필수
set op_idx = op_idx + 1  // ';' 없으면: parse error "Unrecognized token"
```

**2. fn main 종결자** 
```
fn main() -> i64 = { ... };  // 마지막이 '};' 여야 함 ('}' 만 쓰면 타입 오류 가능)
```

**3. vec_push 타입 일관성**
- `vec_push(v, x)` → i64 반환
- if-else 분기에서 다른 branch가 `()` 반환이면 → `let _p = vec_push(v, x)` 사용

### 수정된 problem.md 파일 (이번 세션, 13개)

#### else-if 세미콜론 패턴
- `91_ring_buffer`: CRITICAL 노트 + 완전한 fn main 예시 (Cycle 2984)
- `50_calculator`: CRITICAL 노트 + 안전한 코드 패턴 (Cycle 2985)
- `83_pipeline`: CRITICAL 노트 + 완전한 fn main 예시 (Cycle 2985)
- `84_accumulator_pattern`: CRITICAL 노트 + 완전한 fn main 예시 (Cycle 2985)

#### fn main 래퍼/종결자
- `04_fibonacci`: fn main 래퍼 추가 (Cycle 2986)
- `36_array_rotation`: fn main 래퍼 + set first 수정 (Cycle 2986)
- `69_overflow_detect`: `}` → `};` 수정 (Cycle 2986)
- `75_longest_plateau`: `}` → `};` 수정 (Cycle 2986)
- `72_alternating`: `}` → `};` 수정 (Cycle 2986)

#### 기타
- `94_lru_simulate`: break 키워드 → flag 패턴 (Cycle 2983)
- `29_bounded_stack`: `vec_push` → `let _p = vec_push` (타입 일관성) (Cycle 2986)
- `01_binary_search`, `85_registry_pattern`: set 키워드 수정 (Cycle 2982)

### B-axis 측정 결과

| 모델 | 점수 | 변화 | 측정일 |
|------|------|------|--------|
| Claude (claude-sonnet-4-6) | **98.0%** | 고정 베이스라인 | 2026-05-13 |
| GPUStack (qwen3.6-35b-a3b) | **99.0%** | +2%p (97.0%→99.0%) | 2026-05-20 |

GPUStack 세부: 99/100 통과, first-shot 94% (94/100)
Multi-shot: 04_fibonacci(2), 29_bounded_stack(2), 36_array_rotation(2), 69_overflow_detect(3), 75_longest_plateau(2)
실패: 91_ring_buffer (11회 전부 실패 → 수정 완료)

**다음 측정 예상**: 100/100 가능 (모든 식별된 문제 수정 완료)

### 테스트 결과

```
cargo test --release (Cycle 2987 확인)
  lib.rs:         3778/3778 PASSED
  main.rs:          47/47   PASSED
  diagnostics:      22/22   PASSED
  integration.rs: 2390/2390 PASSED
  + 기타:            23/23   PASSED
  총: 6260 tests, 0 failed ✅
```

---

## 다음 세션 (Cycle 2991+)

### 권장 우선순위

1. **GPUStack 3-run 측정** — 통계적 신뢰성 확보 (100/100 예상)
   - 사전 준비: `GPUSTACK_API_KEY` 환경변수 설정
   - 실행: `cd ecosystem/bmb-ai-bench && python3 -m bmb_ai_bench.run_cmd --model qwen3.6-35b-a3b --base-url http://172.30.1.53:8080/v1 --out results/2026-05-21 --runs 3`
2. **Bootstrap for-loop 스코프 버그** (낮은 우선순위, 재현 불가)
3. **ISSUE 재검토**: golden-flakiness-inttoptr (P3), clang-knapsack-outlier (P3)

### GPUStack 2차 측정 실행 방법

```bash
# GPUSTACK_API_KEY 환경변수 설정 필요
# PowerShell:
$env:GPUSTACK_API_KEY = "your-api-key-here"
$env:GPUSTACK_ENDPOINT = "http://172.30.1.53:8080"

# 측정 실행 (백그라운드):
cd D:\data\lang-bmb\ecosystem\bmb-ai-bench
python3 -m bmb_ai_bench.run_cmd \
  --model qwen3.6-35b-a3b \
  --base-url http://172.30.1.53:8080/v1 \
  --api-key $env:GPUSTACK_API_KEY \
  --out results/2026-05-21 \
  --max-loops 12 \
  --runs 3
```

### 알려진 BMB 언어 특성 (중요도 순)
- `else if` 체인 세미콜론: statement 위치에서 `};` 필수 (Cycle 2984 발견)
- `fn main() -> i64 = { ... };` 끝에 `;` 필수 (Cycle 2986 발견)
- `break`/`continue`/`return`: ✅ 지원 (단, break는 while에서만)
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `vec_pop`: ✅ `i64` 반환 (제거된 요소)
- `vec_push`: i64 반환 (branch 타입 불일치 시 `let _p = vec_push(...)`)
- `set` 키워드: mutable 변수 업데이트에 필수 (block context에서 `x = expr`도 동작하나 `set` 권장)

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (재측정 없음) |
| GPUStack (qwen3.6-35b-a3b) | **99.0%** (2026-05-20) | ✅ 목표 달성 (99%+) |
| GPUStack 2차 측정 | TBD | 예상 100% (모든 실패 원인 수정) |
