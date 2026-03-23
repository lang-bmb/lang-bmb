# BMB AI-Native Proof — 설계 문서

> **목표**: BMB가 "성능 + AI 친화적"임을 재현 가능한 실험으로 증명한다.

---

## 1. 핵심 가설

### 주 가설 (H1) — 계약 효과 분리

**BMB의 계약 시스템은 AI 코드 생성의 피드백 루프를 단축한다.**

검증 방법: **BMB+계약 vs BMB-계약** (동일 언어, 계약 유무만 차이)
- 이것이 가장 깨끗한 검증. 언어 차이/학습 데이터/에러 메시지 변수를 제거.

### 부 가설 (H2) — 크로스 언어 비교

**BMB는 Rust/Python 대비 AI 코드 생성에서 더 적은 반복으로 정확한 코드를 생성하고, 성능은 C와 동등하다.**

검증 방법: **BMB vs Rust vs Python** (동일 문제, 동일 프롬프트)
- 이것은 복합 변수(학습 데이터, 에러 메시지 품질, 언어 성숙도) 포함. 보조 증거.

### 측정 지표

| 지표 | 정의 | 성공 기준 | 최소 효과 크기 |
|------|------|----------|--------------|
| **Loop Count** | 코딩→빌드→에러→수정 반복 횟수 | BMB < 비교군 | ≥ 25% 감소 |
| **Final Correctness** | 최종 코드가 모든 테스트 통과 | BMB ≥ 비교군 | ≥ 동등 |
| **Performance Ratio** | 최종 바이너리 실행 시간 / C 베이스라인 | BMB ≤ 1.05x | 5% 이내 |

**각 지표는 독립적으로 보고.** 복합 스코어는 보조 지표로만 사용하며, 가중치 민감도 분석 포함.

### 루프 유형 분류

루프를 단순 카운트가 아닌 **유형별로 분류**하여 계약의 기여도를 직접 측정:

| 유형 | 정의 | 예시 |
|------|------|------|
| **A. 계약/타입 위반** | 컴파일타임에 계약 또는 타입 시스템이 잡은 에러 | `pre idx < len` 위반 |
| **B. 문법 에러** | 파서 수준 에러 | 잘못된 키워드, 괄호 불일치 |
| **C. 의미 에러 (빌드)** | 컴파일 가능하나 의미적 오류 | 타입 불일치, 미정의 함수 |
| **D. 테스트 실패** | 빌드 성공, 런타임 정답 불일치 | 경계 조건 미처리 |

**핵심 비교**: H1에서 BMB+계약의 Type A 루프가 BMB-계약의 Type D 루프로 "이동"하는지 확인.
→ 계약이 런타임 에러를 컴파일타임으로 끌어올리는 효과를 직접 측정.

---

## 2. 타겟 오디언스

| 대상 | 관심사 | 제공할 증거 |
|------|--------|-----------|
| **개발자 커뮤니티** (HN/Reddit) | "왜 BMB를 써야 하나?" | 루프 카운트 비교 차트, 성능 테이블, 재현 스크립트 |
| **실무 엔지니어/기업** | "프로덕션에 쓸 수 있나?" | E2E 파이프라인 검증, 바인딩 성능, 에러 안전성 증명 |

---

## 3. 실험 설계

### 3.1 사전 등록 (Pre-registration)

실험 실행 전 아래를 git commit으로 동결:
- 문제 세트 (problems/)
- 프롬프트 템플릿
- 성공 기준 및 최소 효과 크기
- 통계 검정 방법
- BMB 문법 레퍼런스 (버전 해시)

커밋 메시지: `pre-register: ai-native-proof protocol v1`
→ 이후 프로토콜 변경은 별도 커밋으로 기록하고 리포트에 명시.

### 3.2 실험 구조

```
실험 1 (H1): 계약 효과 분리 — PRIMARY
┌─────────────────────────────────────────────┐
│  동일 30문제 × BMB+계약 vs BMB-계약         │
│  × 3회 반복                                 │
│  = 180 실험 단위                            │
│  통계: Wilcoxon signed-rank (쌍체 비교)     │
└─────────────────────────────────────────────┘

실험 2 (H2): 크로스 언어 비교 — SECONDARY
┌─────────────────────────────────────────────┐
│  동일 30문제 × BMB vs Rust vs Python        │
│  × 3회 반복                                 │
│  = 270 실험 단위                            │
│  통계: Friedman test (반복측정 비모수)       │
└─────────────────────────────────────────────┘
```

### 3.3 E2E 파이프라인

```
자연어 문제 설명
    ↓ LLM (Claude)
코드 생성 (1st attempt)
    ↓
빌드 (BMB: 계약 검증 포함)
    ↓ 실패 시
에러 메시지 (정규화 + 원본 양쪽 기록) → LLM에 피드백 → 재생성
    ↓ 성공 시
테스트 스위트 실행 (정답 검증)
    ↓ 실패 시
테스트 결과 → LLM에 피드백 → 재생성
    ↓ 전체 통과
성능 측정 (BMB: opt -O2, C: -O2, Rust: --release)
    ↓
결과 기록 (JSON) — 루프 유형별 분류 포함
```

### 3.4 루프 규칙

- **최대 루프 횟수**: 10회 (10회 내 미해결 = 실패, loop_count = 11로 기록)
- **루프 1회 = 에러 피드백 → 재생성 1회** (유형 A/B/C/D 태그)
- **빌드 성공 + 테스트 전체 통과 = 완료**
- **1회 시도에 성공 = Loop Count 1**

### 3.5 실험 조건

| 항목 | 값 |
|------|---|
| LLM | Claude (Opus 4.6) — Phase 1, 향후 확장 가능 |
| 비교 조건 | H1: BMB+계약, BMB-계약 / H2: BMB, Rust, Python |
| 온도 | 0 |
| **반복 횟수** | **각 조건 3회** (비결정성 대응, 분포 보고) |
| 프롬프트 | 언어 간 동일 템플릿 (언어명만 교체) |
| C 베이스라인 | 성능 기준만 (AI 생성 아님, 레퍼런스 구현 사용) |

---

## 4. 프롬프트 설계 (공정성)

### 4.1 기본 템플릿

```
# 문제: {problem_name}
# 언어: {lang}

{problem_description}

## 요구사항
- {functional_requirements}
- {performance_constraints}  (BMB/Rust/C만)

## 테스트 케이스
{test_cases_preview}

위 요구사항을 만족하는 {lang} 코드를 작성하세요.
```

### 4.2 에러 피드백 템플릿

**정규화 형식** (모든 언어 동일):
```
빌드/테스트 실패.

[에러 유형]: {compile_error | test_failure}
[메시지]: {normalized_message}
[위치]: {file}:{line}:{col}

원본 컴파일러 출력:
{raw_compiler_output}

위 에러를 수정하세요.
```

→ 정규화 형식으로 LLM에 전달, 원본도 함께 기록 (에러 메시지 품질 변수 통제).

### 4.3 공정성 원칙

1. **언어별 힌트 없음** — BMB에 "계약을 쓰세요" 같은 유도 금지
2. **동일 프롬프트 템플릿** — 언어명만 교체
3. **에러 피드백 정규화** — 모든 언어 동일 형식으로 전달
4. **BMB 학습 데이터 부재 보정**:
   - BMB 프롬프트에 LANGUAGE_REFERENCE.md 발췌 첨부 (버전 동결, 해시 기록)
   - **프레이밍**: "BMB + 문서" vs "Rust + 사전학습" — 이 비대칭을 명시
   - H1(BMB vs BMB)에서는 이 변수가 제거됨 — 이것이 H1이 primary인 이유

### 4.4 다중 비교 보정

- **주 비교** (pre-registered): H1의 BMB+계약 vs BMB-계약 Loop Count
- **부 비교** (exploratory): H2의 3언어 비교 — Bonferroni 보정 적용 (alpha = 0.05/3)

---

## 5. 문제 세트

### Phase 1 (30문제)

#### 카테고리 A: 알고리즘 (10문제) — 언어 중립

| # | 문제 | 핵심 측정 | 출처 |
|---|------|----------|------|
| 1 | binary_search | 기본 정확도, 경계 조건 | LeetCode #704 |
| 2 | quicksort | 배열 조작, 재귀 | Classic |
| 3 | knapsack_dp | DP, 2D 배열 | Classic |
| 4 | fibonacci_memo | 메모이제이션 | Classic |
| 5 | prime_sieve | 반복문, 비트 연산 | Eratosthenes |
| 6 | merge_sort | 분할정복, 안정 정렬 | Classic |
| 7 | dijkstra | 그래프, 우선순위 큐 | Classic |
| 8 | longest_common_subseq | DP, 문자열 | LeetCode #1143 |
| 9 | topological_sort | 그래프, BFS/DFS | Classic |
| 10 | huffman_encoding | 트리, 빈도 분석 | Classic |

#### 카테고리 B: 시스템 프로그래밍 (10문제) — 메모리/소유권 중심

| # | 문제 | 핵심 측정 | 비고 |
|---|------|----------|------|
| 11 | stack | LIFO, 동적 배열 | 기본 자료구조 |
| 12 | queue | FIFO, 순환 버퍼 | 인덱스 순환 |
| 13 | ring_buffer | 고정 크기, 덮어쓰기 | 경계 조건 |
| 14 | bitset | 비트 조작 | 저수준 연산 |
| 15 | simple_allocator | 메모리 풀 | 할당/해제 |
| 16 | hash_table | 해싱, 충돌 처리 | 복합 구조 |
| 17 | string_builder | 동적 문자열 | 재할당 |
| 18 | matrix_ops | 행렬 연산 | 2D 인덱싱 |
| 19 | bit_reader | 비트 스트림 파싱 | 비트 경계 |
| 20 | fixed_point_math | 정수 기반 소수 연산 | 오버플로 |

#### 카테고리 C: 경계 조건 집중 (10문제) — 계약이 유리할 수 있는 문제

> **주의**: 이 카테고리는 계약이 유리하도록 *의도적으로 설계*됨.
> 결과는 카테고리별 분리 보고하며, 전체 집계에서의 가중치를 명시.

| # | 문제 | 계약이 잡을 수 있는 에러 | 비고 |
|---|------|----------------------|------|
| 21 | bounded_array | `pre idx < len` — OOB 접근 | 배열 경계 |
| 22 | safe_division | `pre divisor != 0` — 0 나눗셈 | 산술 안전 |
| 23 | matrix_multiply | `pre a.cols == b.rows` — 차원 불일치 | 차원 계약 |
| 24 | checked_cast | `pre val >= MIN and val <= MAX` — 오버플로 | 타입 변환 |
| 25 | sorted_merge | `post is_sorted(result)` — 정렬 보장 | 사후 조건 |
| 26 | range_sum | `pre left <= right and right < len` — 범위 검증 | 구간 쿼리 |
| 27 | safe_sqrt | `pre x >= 0` — 음수 입력 | 수학 함수 |
| 28 | utf8_byte_at | `pre idx < byte_len` — 바이트 경계 | 문자열 처리 |
| 29 | circular_index | `pre capacity > 0` — 모듈로 0 방지 | 순환 접근 |
| 30 | clamped_lerp | `pre t >= 0.0 and t <= 1.0` — 보간 범위 | 수치 연산 |

각 문제에 15-20개 테스트 케이스 (정상 + 경계 + 에러 유발 입력).

### Phase 2 (60문제로 확장) — Phase 1 분석 후

- FFI/바인딩 시나리오 (10문제)
- 실세계 문제: JSON 파서, CSV 처리 등 (10문제)
- Phase 1 결과에서 발견된 약점 보강 (10문제)

### 문제 사전 검증 (Step 0)

실험 실행 전 **모든 30문제에 대해**:
1. BMB 정답 솔루션이 컴파일+통과하는지 수동 검증
2. Rust 정답 솔루션이 컴파일+통과하는지 수동 검증
3. C 베이스라인이 컴파일+통과하는지 수동 검증
4. BMB 현재 기능으로 풀 수 없는 문제는 제외하고 대체 문제로 교체

---

## 6. 성능 측정 방법론

| 항목 | 조건 |
|------|------|
| BMB | `--release` + `opt -O2` |
| C 베이스라인 | `clang -O2` (동일 LLVM 백엔드) |
| Rust | `cargo build --release` |
| Python | CPython (정확도/루프 비교 전용, 성능은 참고치) |
| 반복 | 각 벤치마크 최소 10회, median 사용 |
| 통계 | 95% 신뢰구간, IQR 아웃라이어 제거 |
| 환경 | 동일 머신, CPU frequency 고정, 격리 실행 |

### C 베이스라인 출처

- 가능한 경우 공개 레퍼런스 구현 사용 (Rosetta Code, Benchmark Game)
- 자체 작성 시 구현 의도를 주석으로 명시
- 모든 베이스라인은 `problems/XX_name/baseline.c`에 포함, 사전 등록에 동결

---

## 7. 핵심 차별화 시나리오

| 시나리오 | Rust/C 문제점 | BMB 이점 |
|---------|-------------|---------|
| 배열 접근 | 런타임 패닉 (Rust) / UB (C) | `pre idx < len` → 컴파일타임 거부 |
| 0 나눗셈 | 런타임 패닉/UB | `pre divisor != 0` → 컴파일타임 거부 |
| 정렬 증명 | 테스트로만 검증 | `post is_sorted(arr)` → 계약 위반 시 거부 |
| 타입 변환 | 암묵적 (C) / 장황한 `as` (Rust) | 명시적 변환 → AI 실수 감소 |
| 소유권 | borrow checker 에러 (Rust) | 단순한 소유권 모델 → AI 수정 용이 |

**H1 검증의 핵심**: 위 시나리오에서 BMB-계약 조건의 Type D (테스트 실패)가 BMB+계약 조건에서 Type A (계약 위반)로 "이동"하는지 직접 관찰.

---

## 8. 결과물 구조

```
ecosystem/ai-proof/
├── protocol/                  # 사전 등록 (실험 전 동결)
│   ├── PROTOCOL.md            # 실험 설계서 (이 문서의 실행본)
│   ├── bmb_reference.md       # BMB 문법 레퍼런스 (동결, 해시 기록)
│   └── prompt_template.md     # 프롬프트 템플릿
├── problems/                  # 문제 정의 (사전 등록에 포함)
│   ├── 01_binary_search/
│   │   ├── problem.md         # 문제 설명 (자연어)
│   │   ├── tests.json         # 테스트 케이스 (입력/기대출력)
│   │   ├── baseline.c         # C 베이스라인 (성능 기준)
│   │   ├── solution.bmb       # BMB 정답 (사전 검증용, 실험에 미사용)
│   │   └── solution.rs        # Rust 정답 (사전 검증용, 실험에 미사용)
│   └── ...
├── results/                   # 실험 결과
│   ├── raw/                   # 각 시도의 코드 + 에러 로그 + 루프 유형 태그
│   │   ├── 01_binary_search/
│   │   │   ├── run1/
│   │   │   │   ├── bmb_contract_attempt_1.bmb
│   │   │   │   ├── bmb_contract_attempt_1_error.json  # {type, raw, normalized}
│   │   │   │   ├── bmb_nocontract_attempt_1.bmb
│   │   │   │   ├── rust_attempt_1.rs
│   │   │   │   └── python_attempt_1.py
│   │   │   ├── run2/
│   │   │   └── run3/
│   │   └── ...
│   └── summary.json           # 전체 결과 요약
├── scripts/
│   ├── run_experiment.py      # E2E 자동화
│   ├── measure_perf.sh        # 성능 측정
│   ├── normalize_errors.py    # 에러 메시지 정규화
│   ├── classify_loops.py      # 루프 유형 분류 (A/B/C/D)
│   └── analyze.py             # 결과 분석 + 시각화
├── report/
│   └── AI_NATIVE_PROOF.md     # 최종 분석 리포트
└── README.md                  # 재현 방법
```

### summary.json 스키마

```json
{
  "experiment": "ai-native-proof-phase1",
  "protocol_hash": "abc123...",
  "date": "2026-03-XX",
  "llm": { "model": "claude-opus-4-6", "temperature": 0 },
  "problems": [
    {
      "name": "binary_search",
      "category": "algorithm",
      "results": {
        "bmb_contract": {
          "runs": [
            {
              "loop_count": 2,
              "loop_types": { "A": 1, "B": 0, "C": 0, "D": 0 },
              "final_correct": true,
              "perf_ratio": 1.01
            },
            { "loop_count": 2, "...": "..." },
            { "loop_count": 3, "...": "..." }
          ],
          "median_loops": 2,
          "correctness_rate": 1.0,
          "median_perf": 1.01
        },
        "bmb_nocontract": { "...": "..." },
        "rust": { "...": "..." },
        "python": { "...": "..." }
      }
    }
  ],
  "aggregate": {
    "h1_contract_effect": {
      "bmb_contract_median_loops": 1.8,
      "bmb_nocontract_median_loops": 2.9,
      "reduction_pct": 38,
      "wilcoxon_p": 0.003,
      "loop_migration": { "D_to_A": 12, "no_change": 18 }
    },
    "h2_cross_language": {
      "bmb": { "median_loops": 1.8, "correctness": 0.93, "median_perf": 1.02 },
      "rust": { "median_loops": 2.5, "correctness": 0.87, "median_perf": 1.03 },
      "python": { "median_loops": 1.2, "correctness": 0.80, "median_perf": null }
    }
  }
}
```

---

## 9. 분석 리포트 구조

`AI_NATIVE_PROOF.md`:

### 9.1 Executive Summary
- 한 문단: H1/H2 결과 요약, 효과 크기, 통계적 유의성

### 9.2 실험 설계
- 사전 등록 해시, 문제 세트, 프롬프트, 공정성 조건

### 9.3 H1 결과: 계약 효과 (PRIMARY)
- BMB+계약 vs BMB-계약: 루프 카운트 비교
- **루프 유형 이동 분석**: Type D → Type A 이동 건수
- "계약이 런타임 에러를 컴파일타임으로 끌어올렸는가?" 직접 답변

### 9.4 H2 결과: 크로스 언어 비교 (SECONDARY)
- 문제별 4조건 Loop Count / Correctness / Performance
- **카테고리별 분리 보고** (알고리즘 / 시스템 / 경계조건)
- 경계조건 카테고리(C)의 BMB 유리 편향 명시

### 9.5 심층 분석
- **왜 BMB+계약 루프가 적은가**: 계약이 잡은 에러 유형별 분류
- **왜 BMB-계약 루프가 많은가**: 런타임에서 발견된 버그 유형
- **Rust borrow checker 에러 분석**: 유형별 분류

### 9.6 성능 비교
- BMB vs C 베이스라인 (opt -O2)
- 계약이 성능에 미치는 영향 (llvm.assume → check elimination)

### 9.7 코드 품질 평가
- 블라인드 관용성 평가 (1-5 rubric)
- 평가 기준: 가독성, 관용적 표현, 유지보수성

### 9.8 제한사항
- BMB 학습 데이터 부재, 단일 LLM, 에러 메시지 품질 차이
- 카테고리 C의 의도적 편향
- 향후: 다중 LLM, 더 큰 문제 세트

### 9.9 재현 방법
- `python scripts/run_experiment.py --phase 1 --runs 3`

---

## 10. 단계별 실행 계획

| 단계 | 내용 | 산출물 |
|------|------|--------|
| **Step 0** | 자동화 프레임워크 구축 + 3문제 파일럿 | `scripts/`, 파일럿 결과 |
| **Step 1** | 30문제 정의 + 테스트 + C 베이스라인 + BMB/Rust 정답 검증 | `problems/` |
| **Step 2** | 사전 등록 커밋 (프로토콜 + 문제 + 프롬프트 동결) | `protocol/` |
| **Step 3** | H1 실험 (BMB+계약 vs BMB-계약 × 30 × 3회) | `results/raw/` |
| **Step 4** | H2 실험 (BMB vs Rust vs Python × 30 × 3회) | `results/raw/` |
| **Step 5** | 결과 분석 + 코드 품질 블라인드 평가 | `summary.json` |
| **Step 6** | 리포트 작성 | `AI_NATIVE_PROOF.md` |
| **Step 7** | Phase 2 설계 (Phase 1 결과 기반) | 확장 프로토콜 |

---

## 11. 리스크와 대응

| 리스크 | 영향 | 대응 |
|--------|------|------|
| BMB 학습 데이터 부재 | H2에서 BMB 루프 증가 | H1이 primary (동일 언어 비교), H2는 보조. 레퍼런스 유/무 양쪽 기록 |
| Rust가 BMB보다 루프 적은 경우 | H2 가설 반증 | 정직하게 보고 + H1 결과로 계약 효과는 별도 증명 |
| 성능이 C 대비 5% 초과 | 성능 주장 약화 | IR 분석으로 원인 규명, 컴파일러 이슈 등록 |
| Python이 정확도에서 이기는 경우 | "단순한 언어 = AI 친화적" 반론 | Python은 성능 경쟁 불가 → "정확하지만 느린 코드" vs "정확하고 빠른 코드" 프레이밍 |
| BMB 기능 한계로 일부 문제 풀 수 없음 | 실험 규모 축소 | Step 1에서 사전 검증, 불가 문제 교체 |
| 에러 메시지 품질이 결과 좌우 | H2 결과의 신뢰도 저하 | 에러 정규화 + H1(동일 언어)이 primary |
| 3회 반복으로도 분산 큼 | 통계적 유의성 미달 | 효과 크기 보고 (Cohen's d), 유의성 미달 시 정직하게 명시 |

---

## 12. 성공/실패 시나리오

### 최선: H1+H2 모두 성공
> "BMB 계약은 AI 루프를 38% 줄이고 (p<0.01), Rust 대비 25% 적은 반복으로 동일 정확도 달성, 성능은 C의 102%"
→ HN/블로그 포스트, 강력한 차별화

### 양호: H1 성공, H2 혼합
> "계약이 루프를 줄이는 효과는 명확 (p<0.01), 크로스 언어 비교는 학습 데이터 변수로 혼합 결과"
→ 계약 시스템의 가치는 증명됨, 언어 전체의 AI 친화성은 추가 실험 필요

### 최악: H1도 유의미하지 않음
> 정직하게 보고. "계약이 루프를 줄이는 효과는 통계적으로 유의미하지 않았다"
→ 실험 설계/문제 세트 재검토, 또는 계약의 가치가 "AI 루프 감소"가 아닌 다른 축에 있음을 인정

**어떤 결과든 정직하게 보고한다.** 이것이 HN 오디언스에서 신뢰를 얻는 유일한 방법.
