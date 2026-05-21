# BMB Session Handoff — 2026-05-21 (Cycles 2995-2997 — IR 분석 + 문서 정리)

> **HEAD**: `7696bdf6` (Cycles 2995-2998 — IR 분석 + 문서 정리)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2999

---

## 이번 세션 작업 요약 (Cycles 2995-2997)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2995 | csv_parse IR 분석 — LLVM 파리티 확인 | `byte_at` → `getelementptr i8 + load i8` LLVM 파리티. `load_u8(ptr)` 전환 불필요 판정. ROADMAP 갱신 |
| 2996 | rebuild-bootstrap-exe.sh --check-only CI 연결 분석 | `*.exe` gitignore로 CI 적용 불가. ROADMAP P4 아이템 CLOSED |
| 2997 | M3-7 annotation + ISSUE triage | b_baseline JSON에 supersedes 필드 추가. clang-knapsack-outlier closed/ 이동. M3-7 ✅ |

### csv_parse IR 분석 결과 (Cycle 2995)

**핵심 발견**:
- `byte_at` → `getelementptr inbounds i8 + load i8` — 추가 함수 호출 없음
- BmbString ptr 필드 루프 밖 호이스팅 확인 (`bb_while_body_1.lr.ph`에서 단 1회 load)
- `skip_ws`, `parse_quoted_field`, `parse_unquoted_field` 전부 `parse_csv`에 완전 인라인
- C는 필드 내용을 출력 버퍼에 복사하는 반면 BMB는 길이 카운트만 (BMB가 더 경량)
- csv_parse 1.06× = 측정 노이즈 (Tier 3 기준 BMB 0.820× — BMB faster)

**결론**: `load_u8(ptr)` 전환 불필요. ROADMAP §5 항목 CLOSED.

### CI --check-only 분석 결과 (Cycle 2996)

- `*.exe` gitignore → CI에서 prebuilt binary 없음
- `--check-only`는 항상 exit 1 → CI 블로킹
- `bootstrap-benchmark.yml`이 3-Stage 빌드 이미 커버
- ROADMAP P4 아이템 CLOSED

### M3-7 annotation (Cycle 2997)

`claudedocs/measurements/b_baseline_2026-05-13_c2810.json`에 `supersedes` 필드 추가:
- 2026-03-26 비공식 90.9% 측정 supersede 명시
- M3 자율 작업 완전 소진

### 테스트 상태

```
코드 변경 없음 (IR 분석 + 문서 갱신만)
cargo test --release: 6260 tests ✅ (이전 세션 Cycle 2987 확인)
```

---

## 다음 세션 (Cycle 2999+)

### 권장 우선순위

1. **자율 작업 소진 상태** — 모두 HUMAN-blocked
2. **선택지**:
   - B-axis re-measurement (Claude — ANTHROPIC_API_KEY 필요)
   - GPUStack 04_fibonacci CRITICAL 노트 효과 검증 (GPUSTACK_API_KEY 필요)
   - npm/PyPI publish (HUMAN dispatch)
   - inttoptr Option A/B/C 결정 (HUMAN)
   - problem-difficulty-bias 신규 hard 문제 추가 (HUMAN 설계)

### 알려진 HUMAN-blocked 항목

- GPT-4o 실험 (multi-model-validation 완결용)
- npm/PyPI publish (M3 잔여)
- golden-flakiness-inttoptr Option A/B/C 결정
- problem-difficulty-bias 신규 hard 문제 20개
- GPUStack 재측정 (GPUSTACK_API_KEY 재설정 필요)
- B-axis Claude 재측정 (ANTHROPIC_API_KEY 재설정 필요)

### ISSUE 현황 (2026-05-21 기준)

| ISSUE | 상태 | 우선순위 |
|-------|------|---------|
| multi-model-validation | PARTIALLY RESOLVED | MEDIUM |
| external-problem-validation | PARTIALLY RESOLVED | MEDIUM |
| integration-category-weakness | PARTIALLY RESOLVED | LOW |
| problem-difficulty-bias | OPEN | LOW |
| clang-knapsack-outlier | **CLOSED** (Cycle 2992) | — |
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
| GPUStack (qwen3.6-35b-a3b) | **99.7%** 3-run 299/300 (2026-05-20) | ✅ 목표 달성 |

### P-axis 상태 (Cycle 2995 갱신)

| 구분 | 결과 | 비고 |
|------|------|------|
| csv_parse | 1.06× | 측정 노이즈 — IR LLVM 파리티 확인 |
| 그 외 6개 | BMB faster | 0.15×~1.00× |

`byte_at` → `load_u8(ptr)` 전환 불필요 (CLOSED Cycle 2995).
