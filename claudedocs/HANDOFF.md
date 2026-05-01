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

## 6. 다음 세션 — Cycle 2507 메타 정렬 (코드 변경 0)

이 비전이 ROADMAP·CLAUDE.md에 반영되지 않으면 다음 세션이 옛 비전 답습. **메타 정렬은 후속 모든 사이클의 정합성 보장 작업.**

### 산출물 (Cycle 2507 종료 조건)

- [x] vision spec commit + push (이 세션 완료, push는 사용자 확인)
- [ ] **`docs/ROADMAP.md` 재작성** — M1~M4 마일스톤, 트랙 재분류표 (옛 A-L → 새 M/N/O/Q/R/S/T), 버전 정책 명시
- [ ] **`CLAUDE.md` 업데이트** — Workflow Rule 8 신규 ("출력 디폴트 = AI 친화 구조화"), Decision Framework에 마일스톤·버전 분리 추가
- [ ] **트랙 재분류표 ROADMAP 반영**:
  - 옛 G.1-G.4 → 새 M1 P1
  - 옛 Phase C (105 inttoptr) → 새 M2 후보 (M1 blocker 아님)
  - 옛 8/15 FAIL 비-도메인(mandelbrot, n-body 등) → 강등 (별도 추적)
  - D' Golden binary 결정 → M1 정책 결정으로 명시
- [ ] **신규 issue 생성 (7건)** — `claudedocs/issues/ISSUE-2026MMDD-{m,n,o,q,r,s,t}.md`
- [ ] **G.1 P1-P3 명시 연기** — Cycle 2508+로 재배치

### 메타 정렬 후 = Cycle 2508+ (M1 P1 G.1 fix)

Cycle 2506에서 도출된 G.1 P1-P3 시퀀스:
- **P1**: L.2 fix (`bmb build --shared --no-prelude` @bmb_user_main undefined → SharedLib mode에서 main injection skip)
- **P2**: prelude duplicate 제거 (clamp/in_range 등 prelude → use stdlib redirect)
- **P3**: `--trust-contracts` 플래그를 `ecosystem/build_all.py`에서 제거, Bindings CI 3-OS green 검증

### Cycle 2507 즉시 다음 액션 (writing-plans skill로 전환)

브레인스토밍 종료 후 자연스러운 흐름은 `superpowers:writing-plans` skill 호출 — Cycle 2507 메타 정렬 작업의 단계별 구현 계획 수립. 본 세션에서 도구 시간 제약 시 다음 세션 시작 시 호출.

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
