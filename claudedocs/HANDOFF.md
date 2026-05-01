# BMB Session Handoff — 2026-05-01 (Vision v1.0 Realignment Session)

> **이전 HEAD**: `b275166f` (Cycles 2505-2506 docs commit)
> **새 HEAD**: `2839f003` (vision realignment spec)
> **원격 상태**: `origin/main` 1 commit 앞 — push 필요 시 사용자 확인
> **세션 성격**: **코드 변경 0, 비전 정렬 세션**. 9-라운드 브레인스토밍으로
> BMB의 정체성·도메인·우선순위·마일스톤·트랙·버전 정책을 합의·영속화.
> 다음 세션은 메타 정렬 사이클(2507) — 합의된 비전을 ROADMAP/CLAUDE.md에 반영.

---

## 1. 이번 세션 요약

| 단계 | 작업 | 산출물 |
|------|------|--------|
| Q1-Q9 | 9-라운드 브레인스토밍 (1차 사용자, 도메인, AI-readiness, 우선순위, 마일스톤, v1.5 정의, 다음 액션, 마일스톤 매핑, 버전 정책) | 9개 결정 합의 |
| Q10 | 명료화 종료 결정 — 추가 영역(Trust 정책 등)은 메타 정렬에서 자연 처리 | 종료 |
| Spec 작성 | `docs/superpowers/specs/2026-05-01-vision-v1.0-realignment.md` (219줄) | 영속화 |
| Spec self-review | 정확도 정정 1건 (8/15 FAIL "절반" → "다수, 정확 카운팅은 메타 정렬에서") | inline fix |
| 사용자 승인 | spec 리뷰 후 commit + 다음 단계 진행 승인 | 합의 |
| Commit | `2839f003` (spec only, 219+ lines) | git |

---

## 2. 9개 핵심 결정 (Quick Reference)

| Q | 결정 | 기존 상태와의 차이 |
|---|------|----------------|
| Q1 | 1차 사용자 = **인간+AI 협업** | 명시 X → 명시 |
| Q2 | 1차 도메인 = **컴파일러·언어 도구·DSL·검증기** | 명시 X → 명시 (8/15 FAIL의 비-도메인 강등 근거) |
| Q3 | AI-readiness = **언어 자체 속성** (외부 도구 X) | 모호 → 명확 (별도 AI 채널/합성기 금지) |
| Q4 | 우선순위 **B > P > A > D > C** | "Performance > Everything" 단일 → 5축 우선순위 |
| Q5 | 단계별 마일스톤 **M1 → M2 → M3 → M4** | v1.0 단독 정의 → 4단계 binary |
| Q6 | M2 = AI 친화 인프라 5축 (Track M/N/O/Q/R) | 트랙 명명 신규 |
| Q7 | 다음 사이클 = 메타 정렬 | 신규 |
| Q8 | 마일스톤 매핑 + 직교 트랙 **S(에코시스템)** + **T(바인딩)** | 트랙 신규 |
| Q9 | **마일스톤(자율) ↔ 버전(외부 신호 게이트) 분리** | 통합 → 분리 (메이저 버전 비자율 결정) |

---

## 3. 마일스톤 정의 (M1~M4)

### M1 Self-Validated (내부 자기검증 완성)

| 조건 | 현 상태 | Cycle |
|------|--------|-------|
| Bootstrap Fixed Point | ✅ S2 == S3 (Cycle 2237) | 완료 |
| G.1 verifier 결함 fix | 🔄 진단 완료 (Cycle 2506), P1-P3 시퀀스 대기 | 2508+ |
| 컴파일러 도메인 벤치마크 | ⚠️ brainfuck/lexer/hash_table FAIL — 분류·해소 필요 | 2509+ |
| 3-OS CI green | ⚠️ Linux `-lm` 후속, Windows MinGW 안정화 | 진행 |
| Trust 정책 (D' Golden) | ⏳ (B) 권장됨, 메인테이너 결정 대기 | M1 종료 시 |

### M2 AI-Ready Infrastructure (5 트랙)

| 트랙 | 내용 | 현 상태 |
|------|------|--------|
| **M (Machine-First Output)** | 모든 출력 기본 JSON, `--human` 옵션 | **부분 구현** — `--human` 플래그 이미 존재 (`docs/superpowers/specs/2026-03-25-ai-friendly-tooling-design.md`). 잔여: 모든 명령에 일관 적용 + 스키마 안정화 |
| **N (MCP Server)** | `bmb mcp` 명령 | 신규 |
| **O (Context Pack)** | `bmb context-pack <project>` | 신규 |
| **Q (Ambiguity Audit)** | grammar 정적 분석 + `bmb lint --ai-friendly` | 신규 |
| **R (LLM Bench Tracking)** | `bmb llm-bench` + 50개 task suite (합격선 X) | 신규 |

### M3 External Bindings PoC

- BMB 라이브러리 (BMB 특징을 드러내는) 1개
- C ABI 노출
- Python + Node 바인딩 (AI 코딩 사용 빈도 1, 2위)
- 트랙 S 90%

### M4 Adopted

- 추가 바인딩: C#, Java, C
- 트랙 S 100% (gotgan, tree-sitter 포함)
- 외부 채택 신호 충족 (§v1.0 외부 게이트)

---

## 4. 직교 트랙 진척표

| 트랙 | M1 진척 | M2 진척 | M3 진척 | M4 진척 |
|------|---------|---------|---------|---------|
| **S (Ecosystem BMB-rewrite)** | 부트스트랩만 ✅ | + LSP, fmt, lint | + verify, bench, mcp | 100% |
| **T (External Bindings)** | 0 | C ABI 설계 | Python + Node PoC | C#, Java, C 추가 |

---

## 5. 버전 정책 (마일스톤 ≠ 메이저 버전)

| 마일스톤 도달 | 권장 버전 |
|--------------|---------|
| M1 도달 | v0.99 |
| M2 도달 | v0.100 / v0.110 |
| M3 도달 | v0.150 / v0.200 — **v1.0 후보, 외부 신호 평가 시작** |
| M4 도달 | **v1.0 선언** (외부 신호 충족 시) |

### v1.0 외부 신호 (가-합의, M3 진입 시 정식 확정)

- GitHub stars ≥ 1,000
- 외부 PR merged ≥ 10 (각각 다른 contributor)
- 외부 이슈 (월) ≥ 10
- 부정 평가 비율 < 30% (HN/Reddit 등 노출 후)
- 외부 BMB 프로젝트 ≥ 5
- 결정자: 메인테이너 + 외부 contributor 협의

---

## 6. 다음 세션 — 분석 → 반영 → 실행 (3단계 분리)

> **재구조화 (사용자 요청)**: 메타 정렬을 단순 "ROADMAP 재작성"으로 처리하면
> spec과 실제 코드 사이의 간극이 가시화되지 않아 후속 작업이 표면적이 된다.
> **분석 단계(일관성 평가 + 갭 분석)를 먼저** 수행하여 진짜 격차를 드러낸 후
> ROADMAP을 재작성한다.

### 6.1 Cycle 2507 — Vision Alignment Assessment (분석 단계, 구현 0)

> **세션 성격**: 분석·진단만, 코드/ROADMAP 변경 없음. 산출물은 두 분석 문서.
> 결론은 Cycle 2508의 입력이 됨.

#### Phase A — 일관성 평가 (Consistency Audit)

spec의 9개 결정 각각에 대해 **현재 코드/문서/CI/벤치마크가 어디에 어떻게 반영되어 있고, 어디에 모순되는가**를 점검.

체크리스트 (각 결정마다):
- [ ] Q1 (인간+AI 협업): 현재 LSP/에러 메시지/도구가 이 가정과 일치? (예: 친절한 인간 에러 우선 vs 구조화 출력 디폴트)
- [ ] Q2 (컴파일러 도메인): 현재 벤치마크/예제/stdlib가 도메인 정합? (8/15 FAIL 도메인 분류표)
- [ ] Q3 (AI-readiness = 언어 속성): 현재 컴파일러에 외부 LLM 통합/AI 채널이 있는가? (없어야 정합)
- [ ] Q4 (B>P>A>D>C 우선순위): 현재 사이클 작업이 이 순서를 따랐는가? (Phase C 등 P 작업 위주 검증)
- [ ] Q5 (M1~M4 단계): 현재 ROADMAP에 마일스톤 정의가 있는가? 어디에?
- [ ] Q6 (M2 = 5 트랙): Track M(`--human` 이미 존재), N/O/Q/R 어디까지 진척?
- [ ] Q8 (트랙 S/T): 에코시스템 도구 BMB 재작성 진척? 외부 바인딩 PoC?
- [ ] Q9 (마일스톤 vs 버전 분리): 현재 v0.98 명명 정책이 이 분리 원칙과 일치?

산출물: `claudedocs/vision-consistency-audit-2026-05-XX.md`

#### Phase B — 갭 분석 (Gap Analysis)

합의된 비전과 현재 상태의 격차를 정량/정성 측정.

| 트랙 | 현재 진척 | 목표 (마일스톤별) | 격차 |
|------|---------|---------------|------|
| M (Output) | `--human` 존재, 기본 JSONL | M2: 모든 명령 일관 + 스키마 안정화 | ? |
| N (MCP Server) | 0 | M2: `bmb mcp` 가동 | 100% |
| O (Context Pack) | 0 | M2: `bmb context-pack` | 100% |
| Q (Ambiguity Audit) | 0 | M2: 0 모호 파스 | grammar 분석 필요 |
| R (LLM Bench) | 0 | M2: 50 task suite + 추적 | 100% |
| S (Ecosystem) | 부트스트랩 ✅ | M3: LSP/fmt/lint/verify/bench 90% | LSP·fmt·lint 등 BMB 재작성률 측정 |
| T (Bindings) | gotgan 등 별도 | M3: Python+Node PoC | 100% |

마일스톤별 잔여 작업:
- **M1**: G.1 fix(시퀀스 알려짐), 컴파일러 도메인 벤치 분류·해소(brainfuck/lexer/hash_table 우선), 3-OS CI green(부분 진행), Trust 정책(권장 (B), 사용자 결정 대기)
- **M2~M4**: 신규 트랙 작업, Phase A 결과 위에서 산출

산출물: `claudedocs/vision-gap-analysis-2026-05-XX.md`

#### Phase C — 우선순위 매핑

Phase A·B 결과를 트랙·마일스톤에 매핑하여 Cycle 2508+의 작업 순서를 도출.

산출물: 위 두 문서의 결론 섹션 + Cycle 2508 입력으로 사용

### 6.2 Cycle 2508 — Meta Alignment Implementation (반영 단계)

Phase A·B·C 결과 위에서 진행:

- [ ] **`docs/ROADMAP.md` 재작성** — M1~M4 마일스톤, 트랙 재분류표 (옛 A-L → 새 M/N/O/Q/R/S/T), 버전 정책 명시, **갭 분석 결과 반영**
- [ ] **`CLAUDE.md` 업데이트** — Workflow Rule 8 신규 ("출력 디폴트 = AI 친화 구조화"), Decision Framework에 마일스톤·버전 분리 추가
- [ ] **트랙 재분류 commit**:
  - 옛 G.1-G.4 → 새 M1 P1
  - 옛 Phase C (105 inttoptr) → 새 M2 후보 (M1 blocker 아님)
  - 옛 8/15 FAIL 비-도메인 → 강등 (별도 추적)
  - D' Golden binary 결정 → M1 정책 결정으로 명시
- [ ] **신규 issue 생성 (7건)** — `claudedocs/issues/ISSUE-2026MMDD-{m,n,o,q,r,s,t}.md` (갭 분석에서 도출된 구체 작업 항목 포함)
- [ ] **G.1 P1-P3 명시 연기** — Cycle 2509+로 재배치

### 6.3 Cycle 2509+ — M1 P1 (G.1 fix 실행)

Cycle 2506에서 도출된 G.1 P1-P3 시퀀스:
- **P1**: L.2 fix (`bmb build --shared --no-prelude` @bmb_user_main undefined → SharedLib mode에서 main injection skip)
- **P2**: prelude duplicate 제거 (clamp/in_range 등 prelude → use stdlib redirect)
- **P3**: `--trust-contracts` 플래그를 `ecosystem/build_all.py`에서 제거, Bindings CI 3-OS green 검증

### 6.4 Cycle 2507 즉시 다음 액션

분석 단계는 brainstorming의 자연스러운 후속이 아니므로 `writing-plans` skill 보다는:
1. spec § 1·2 + vision 메모리 재로드
2. Phase A·B 체크리스트를 TodoList로 등록
3. Phase A 시작 (각 Q마다 grep + 코드 점검)

또는 분석 자체를 plan으로 구조화하려면 `superpowers:writing-plans` skill로 Cycle 2507 (분석)의 단계별 plan 작성 — 권장.

---

## 7. 참고 자료

- **Spec**: `docs/superpowers/specs/2026-05-01-vision-v1.0-realignment.md` (219줄)
- **이전 세션 핸드오프**: 이 파일 이전 버전 (commit `b275166f`) — Cycles 2505-2506
- **이전 비전 메모리**: `~/.claude/projects/D--data-lang-bmb/memory/project_vision_v1_realignment.md` (자동 로드)

---

## 8. HUMAN-decision 미결 (이번 세션과 별개)

이전 세션(Cycles 2505-2506)에서 미결된 HUMAN-only 항목들. 새 비전과 무관하게 보류:

- **TestPyPI org secret 등록** — 사용자 admin 권한 필요
- **WSL2 admin 설치** — 사용자 admin 권한 필요
- **D' Golden binary 정책 최종 확정** — (B) 권장됨, M1 종료 시 결정으로 본 세션에서 일정화

---

**세션 종료**: 2026-05-01
**다음 세션 시작 시**: 이 HANDOFF 1-2장 + spec § 1·9 + 자동 로드된 vision 메모리 참조하여 Cycle 2507 메타 정렬 즉시 시작 가능.
