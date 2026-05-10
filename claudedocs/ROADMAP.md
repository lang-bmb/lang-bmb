# BMB 로드맵 — 철학 정렬 앵커
> 최종 업데이트: 2026-05-10 (Cycle 2618 — 전체 작업 검토 후 갱신)
> 이 문서는 매 세션의 **유일한 실무 앵커**다.
> 상세 사이클 로그: `docs/ROADMAP.md` | 개발 규칙: `CLAUDE.md` | 세션 상태: `claudedocs/HANDOFF.md`

---

## § 1 가설 (The Hypothesis)

### 1.1 왜 BMB가 존재하는가

**① 기존 언어의 한계**

모든 기존 언어는 *인간이 직접 작성해야 한다*는 전제 위에 설계됐다.

```
인간 인식 한계 수용
→ 쉬운 문법 · 모호성 허용
→ 컴파일러가 런타임에 판단을 미룸
→ 런타임 오버헤드 구조적으로 잔존
```

**② AI가 바꾼 것**

AI가 코드를 작성하면 언어가 어려워도 무관하다.

```
모호성 · 암묵 변환 · 런타임 체크 완전 제거 가능
→ "Performance > Everything" 처음으로 실현 가능한 조건이 생겼다
```

**③ AI의 한계가 만드는 BMB의 위치**

AI도 기계어를 직접 작성할 수 없다.  
(컨텍스트 한계, 대규모 프로젝트 구조 파악, 속도, 비용)

```
더 낮추면: hallucination · 검증 불가 · 비용 폭발
         ↑
        BMB  ← AI가 다룰 수 있는 가장 낮은 추상화 수준
         ↓
더 높이면: 런타임 오버헤드 잔존
```

> **결론**: BMB는 AI 이전에는 존재할 수 없었던 언어다.  
> 인간 인식 한계를 전제하지 않고 설계된 첫 시스템 언어.

---

### 1.2 목표 #1: 성능 — 존재가치 증명

이것은 단순 목표가 아니라 프로젝트의 존재 근거다.

```
BMB가 C/Rust와 동등하거나 초월하지 못한다
→ "AI 시대에 새 언어가 필요하다"는 가설이 거짓
→ BMB의 존재 이유 없음
```

| 상황 | 판정 |
|------|------|
| LLVM 백엔드 버그·한계 | ✅ 외부 요인, 수용 |
| "언어 한계"로 성능 포기 | ❌ 언어 스펙을 바꾼다 |
| "컴파일러 한계"로 성능 포기 | ❌ 컴파일러를 바꾼다 |
| "부트스트랩 복잡도"로 기피 | ❌ 복잡도는 이유가 아니다 |

성능 달성 수단: **AI-friendly 언어 설계**  
(예측 가능한 패턴, 명시적 계약, 구조화 출력, 도구 통합)

---

### 1.3 존재가치 증명의 메커니즘: 도그푸딩

BMB가 가설을 증명하는 방법은 벤치마크만이 아니다.  
BMB로 만든 모든 것이 가설 검증의 데이터포인트다.

| 활동 | 증명하는 것 |
|------|------------|
| `bootstrap/compiler.bmb` (32K LOC) | BMB로 컴파일러 작성 가능 |
| `bootstrap/lsp.bmb` (~1450 LOC) | BMB로 언어 도구 작성 가능 |
| `bootstrap/lint.bmb` | BMB로 정적 분석 작성 가능 |
| `ecosystem/bmb-algo` 등 5개 | BMB로 배포 가능한 라이브러리 작성 가능 |
| `bmb-mcp` / `context-pack` | BMB가 AI 워크플로우에 통합 가능 |

**따라서**: 컴파일러·부트스트랩·에코시스템·외부 패키지 작업은  
별개 활동이 아니라 **도그푸딩 활동의 일환**이며, 가설 검증에 직접 기여한다.

**게이트 커밋먼트**:  
도그푸딩 과정에서 문제가 발견되면 — 언어 스펙·컴파일러·부트스트랩 어느 레벨이든 —  
**low level부터 개선될 수 있는 게이트가 항상 열려있어야 한다.**  
고수준 workaround로 문제를 덮는 것 = 도그푸딩의 실패.

---

## § 1.4 역할 및 범위 (Role & Scope)

### 1차 사용자

```
인간 → 자연어 의도 → LLM → BMB 코드 작성
                              ↓
                    BMB 컴파일러 (contract 검증 + IR 생성)
                              ↓
                         최적 기계어
```

- **인간**: 자연어로 의도 전달 (LLM에게)
- **LLM**: BMB 코드 작성 — BMB 컴파일러와 직접 통합 없음
- **BMB 컴파일러**: contract 검증, 결정론적 IR 생성, 구조화 출력

AI-readiness는 **언어 자체의 속성**이다 — 외부 LLM 채널·합성기·AI API 통합 없음.

### 1차 도메인

**컴파일러 · 언어 도구 · DSL · 검증기** (자기 자신 포함)

이 도메인인 이유:
- BMB가 이미 BMB로 자신을 컴파일한다 (`bootstrap/compiler.bmb` 32K LOC) — 도메인이 코드에 박혀있음
- Contract 검증의 가치가 가장 명확한 영역 (parser invariants, type checker soundness, IR rewriter correctness)
- AI가 가장 잘 생성하는 영역 (정형적 · 알고리즘적 · 트리 변환)

### 범위 외 (Out-of-scope)

| 항목 | 이유 |
|------|------|
| 수치 계산 (mandelbrot, n-body 등) | 도메인 외, 벤치 강등 대상 |
| 외부 LLM API 통합 | AI-readiness는 언어 설계로, 외부 채널로 X |
| 일반 웹/앱 개발 도구 | BMB 1차 도메인 밖 |

---

## § 2 진단 (Diagnosis) — 4가지 Drift

M1·M2를 달성하는 동안 활동들이 가설과의 연결 없이 독립적으로 진행된 결과:

### Drift A — 도그푸딩 프레임 누락
- **현상**: 생태계(MCP/LSP/바인딩) 작업이 "성능 목표와 별개"로 보임
- **근본 원인**: 생태계 작업이 도그푸딩임이 명시되지 않았음
- **결과**: 문제 발견 시 low-level 게이트로 돌아가야 함이 문서화되지 않음

### Drift B — B(Failure Rate) #1 우선순위 vs. 미선언
- **현상**: B가 최우선이라 선언, 비공식 실험(90.9%, 2026-03-26)은 있으나 공식 baseline 미선언
- **근본 원인**: 인프라는 완성됐으나 공식 실험 실행(API key + 모델 고정)이 미수행
- **결과**: M4 첫 액션으로 연기 중

### Drift C — AI-native 선언 vs. 언어 갭
- **현상**: "LLM이 쓰는 언어" 선언, 그러나 LLM 자연 패턴 다수 미지원
- **확인된 실제 파서 갭**:
  - `let (a, b) = expr` — tuple destructuring (파서 미지원)
  - `Type::method()` — static method call expression (파서 미지원)
  - `Option::Some(x)` — 표현식 위치에서 enum 생성 (패턴만 지원)
- **지원됨 (CLAUDE.md와 다름)**: `_` wildcard pattern, trait impl blocks
- **근본 원인**: 언어 갭 해소가 명시적 작업 항목으로 등록되지 않음

### Drift D — 문서 분산
- **현상**: `CLAUDE.md` / `docs/ROADMAP.md` / vision spec이 각각 독립 진화
- **근본 원인**: 세션마다 진짜 앵커가 어디인지 불명확
- **결과**: 새 세션마다 정렬 비용 반복 발생

---

## § 3 처방 (Prescription)

### A → 도그푸딩 분류 명시
- 모든 활동은 도그푸딩 분류로 관리
- 도그푸딩 중 발견된 성능·언어 문제: workaround 금지, low-level 게이트로 처리
- "왜 이 작업을 하는가?" = 항상 가설로 traceable해야 함

### B → B축 baseline 공식 선언 (M4 첫 액션)
- 인프라 완성 (`bmb-ai-bench run`), 비공식 결과 존재 (90.9%, 2026-03-26, ~100문제)
- 공식 실험: 고정 모델 + `BMB_BENCH_API_KEY` + 결과 커밋 → baseline 선언
- 공식 선언 전까지: "B 비공식 ~90.9%, 공식 미확정"으로 명시

### C → 언어 갭 백로그 등록
- 확인된 갭 3개 → `claudedocs/issues/` 이슈 등록
- 원칙: "AI-native 선언 = 언어 갭 해소 의무를 수반함"

### D → 문서 역할 분리 (이 문서로 확정)

| 문서 | 역할 |
|------|------|
| `claudedocs/ROADMAP.md` | **유일한 실무 앵커** — 매 세션 시작 시 참조 |
| `docs/ROADMAP.md` | 공개용 상세 문서 (사이클 로그, 트랙 상세) |
| `CLAUDE.md` | 개발 규칙 (이 ROADMAP을 전제) |
| `claudedocs/HANDOFF.md` | 세션 간 상태 전달 |

---

## § 4 로드맵

### 현재 위치

```
M1  Self-Validated      ████████████████████  ✅ COMPLETE
M2  AI-Ready Infra      ████████████████████  ✅ COMPLETE
M3  External Bindings   ███████████████████░  🔄 ~90%  ← 현재
M4  Adopted             ████░░░░░░░░░░░░░░░░  🔄 ~40%
M5  Language Complete   ██░░░░░░░░░░░░░░░░░░  🔄 M5-1 ✅ (payload enum Stage 1)
v1.0                    ░░░░░░░░░░░░░░░░░░░░  ⬜ 외부 신호 대기
```

현재 버전: `0.98.0` | 권장 다음: `v0.100` (M3 완료 후, 메인테이너 결정)

#### M5 언어 완성도 현황 (신규 트랙)

| 기능 | 상태 |
|------|------|
| M5-1: `enum Option { None, Some(i64) }` + match payload | ✅ Stage 1 구현 (Cycle 2633) |
| M5-1: Fixed Point 검증 | ❌ arena OOM (pre-existing, 32G+ 초과, O(n²) 문자열 AST 성장) |
| M5-2: Result<Ok(i64), Err(i64)> + 다중 payload enum | ✅ 완료 (Cycle 2635, M5-1 인프라 재사용) |
| M5-3: String payload + multi-field enum | ⬜ 다음 3-5 cycles |

---

### M3 완료 조건

| 조건 | 상태 | 잔여 |
|------|------|------|
| C ABI + 빌드 인프라 | ✅ | — |
| Python + Node 바인딩 5/5 | ✅ | — |
| Track S 90% | ✅ (~99%) | — |
| Showcase 라이브러리 선정 | ⏳ | **HUMAN** (bmb-algo 1순위, bmb-json 2순위) |
| 공식 벤치마크 측정 | ⏳ | 선정 후 1-2 cycles |
| npm / PyPI publish | ⏳ | workflow_dispatch (HUMAN) |

**ETA**: HUMAN 결정 즉시 → 1-2 cycles

#### M3 잔여 태스크 (우선순위순)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M3-1 | [HUMAN] showcase 선정: bmb-algo vs bmb-json | 결정 | 즉시 |
| M3-2 | showcase 공식 벤치마크 측정 (v0.98 기준) | 자율 | 1-2 cycles |
| M3-3 | [HUMAN] npm publish (`dry_run: false`) | 실행 | 즉시 |
| M3-4 | [HUMAN] PyPI publish (`publish: true, repository: pypi`) | 실행 | 즉시 |
| M3-5 | bmb-mcp 미커밋 변경사항 커밋+push | 위생 | 즉시 |

---

### M4 경로

| 우선순위 | 축 | 태스크 |
|---------|-----|--------|
| ① | **B** | LLM 1-shot 성공률 공식 baseline 측정 (`BMB_BENCH_API_KEY` 필요) |
| ② | **언어 갭** | let-tuple destructuring + static method call + Option::Some expr → 파서 추가 |
| ③ | **P** | 도메인 핵심 ≤1.00x 유지, FAST 확장 |
| ④ | **바인딩** | C# / Java / C (M3 showcase 확장) |
| ⑤ | **Track S** | gotgan / tree-sitter BMB-rewrite (장기) |

#### M4 준비 태스크 (선행 가능)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M4-1 | B 공식 측정 실행 (API key + 고정 모델) | B축 | 1 cycle |
| M4-2 | 언어 갭 이슈 등록: let-tuple, static-method, Option-expr | Drift C | 즉시 |
| M4-3 | let (a, b) = expr — tuple destructuring 파서 추가 | 언어 | 2-3 cycles |
| M4-4 | Type::method() — static method call expression 파서 추가 | 언어 | 2-3 cycles |
| M4-5 | Option::Some(x) 표현식 위치 지원 | 언어 | 1-2 cycles |
| M4-6 | C# 바인딩 scaffold | 바인딩 | 3-5 cycles |

---

### v1.0 선언 조건 (비자율, 외부 신호)

```
GitHub stars      ≥ 1,000
외부 PR merged    ≥ 10 (각각 다른 contributor)
외부 이슈 (월)    ≥ 10
외부 BMB 프로젝트 ≥ 5
부정 평가 비율    < 30% (HN/Reddit 노출 후)
결정권: 메인테이너 + 외부 contributor 협의
```

마일스톤(M1~M4)은 자율 게이트. 버전(v0.x → v1.0+)은 **비자율, 외부 신호 게이트**.

---

## § 5 측정 지표 (B > P > A > D > C)

| 축 | 현재값 | 목표 | 측정 방법 |
|----|--------|------|----------|
| **B** Failure Rate | ⚠️ **비공식 ~90.9%** (2026-03-26, ~100문제) — 공식 미선언 | 공식 baseline 확보 → 개선 추적 | LLM 1-shot 컴파일+verifier 통과율 |
| **P** Performance | ✅ 16/16 ≤1.05x | 도메인 핵심 ≤1.00x, 일부 FAST | Tier 1/3 벤치마크 |
| **A** Token Efficiency | ❌ 미측정 | BMB ≤ Rust LOC (동일 알고리즘) | LOC·토큰 비교 |
| **D** Verification | ❌ 미측정 | contract 자동 증명률 추적 | `bmb verify` 통과율 |
| **C** Navigability | ❌ 미측정 | LLM N-파일 정답률 추적 | Track R suite |

> **진단**: P만 측정됨. B는 비공식 데이터 존재, 공식 선언 필요. M4 첫 액션 = **B 공식 baseline 선언**.

---

## HUMAN 결정 사항 (2026-05-10 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 라이브러리 선정 | ✅ **bmb-algo** |
| npm publish | ✅ **즉시 진행** — `workflow_dispatch` → `dry_run: false` |
| PyPI publish | ✅ **즉시 진행** — `workflow_dispatch` → `publish: true, repository: pypi` |
| v0.100 버전 선언 | ✅ **M3 publish 완료 직후** 메인테이너 결정 |
| B 공식 측정 | ✅ **즉시 실행** — `BMB_BENCH_API_KEY` 설정 후 `bmb-ai-bench run` |
| M5-1 하위 호환성 | ✅ unit enum + payload enum 모두 `{i64, i64}` 통일 |
| M5-1 LLVM 표현 | ✅ `%EnumValue = type { i64, i64 }` 고정 2-word (스택 할당) |
| M5-1 가변 페이로드 | ✅ M5-1 = i64 단일 페이로드만. Result<T,E>는 M5-2로 defer |
