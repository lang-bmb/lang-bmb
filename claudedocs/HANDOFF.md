# BMB Session Handoff — 2026-05-22 (Cycles 3027-3030 — MIR AndChainCSE P2 구현)

> **HEAD**: `(커밋 예정)` (Cycles 3027-3030 — MIR AndChainCSE: and/or 체인 중복 load_u8 자동 CSE)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 3031

---

## 이번 세션 작업 요약 (Cycles 3027-3030)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3027 | MIR CSE 인프라 설계 | canonical form 값 동등성 설계, `and_rhs`→`and_merge`→`and_rhs_P` 패턴 분석 |
| 3028 | AndChainCSE 구현 | `optimize.rs` Phase 1-3 전체 구현: phi 삽입 + Copy 교체, 4 unit test ✅ |
| 3029 | P-track 검증 + ISSUE 조사 | 7/7 PASS, double-load+CSE = break-based 동등 성능, json_parse 가변성 확인 |
| 3030 | ISSUE close + HANDOFF + commit | RESOLVED 마킹, 문서 갱신, 커밋 |

### 핵심 결과

| 항목 | 이번 세션 전 | 이번 세션 후 |
|------|------------|------------|
| brainfuck | 0.956× ✅ | **0.983×** ✅ (측정 가변성) |
| csv_parse | 0.891× ✅ | **0.855×** ✅ |
| http_parse | 0.909× ✅ | **0.917×** ✅ |
| lexer | 0.169× ✅ | **0.201×** ✅ (측정 가변성) |
| json_parse | 0.822× ✅ | **0.913×** ✅ (측정 가변성, CSE 영향 없음) |
| json_serialize | 0.668× ✅ | **0.736×** ✅ |
| sorting | 0.154× ✅ | **0.151×** ✅ |

**P-track 7/7: 세션 전후 모두 PASS. AndChainCSE 최적화 pass 구현으로 double-load 패턴 자동 CSE.**

### AndChainCSE 최적화 pass

| 파일 | 변경 내용 |
|------|---------|
| `bmb/src/mir/optimize.rs` | `AndChainCSE` struct + impl (Phase 1-3) + 4 unit test 추가 |
| `bmb/src/mir/mod.rs` | `AndChainCSE` pub use export 추가 |
| Release/Aggressive 파이프라인 | `GlobalFieldAccessCSE` 직후 `AndChainCSE` 추가 |

**알고리즘**:
1. Phase 1: `and_rhs_*` 블록에서 pure load call(`load_u8` 등)과 canonical 주소 키 수집
2. Phase 2: `and_merge_*` 블록에서 동일 canonical key의 인접 `and_rhs` 쌍 감지
3. Phase 3: `and_merge_*`에 phi 삽입 + `and_rhs_P`에서 중복 Call → Copy 교체

**canonical form**: `binop:Add:param:ptr:var:pos_phi` 형식으로 주소 동등성 판단

**IR 검증 결과**:
```llvm
bb_and_rhs_3:
  %_t5.u8.0 = load i8, ptr %gep_elem.0   ; ← 첫 번째 load (유지)
bb_and_merge_5:
  %_and_cse_1 = phi i64 [%_t5, bb_and_rhs_3], [0, bb_and_false_4]  ← CSE phi
bb_and_rhs_6:
  %_t10 = icmp ne i64 %_and_cse_1, 13   ; ← load i8 없음! phi 재사용
```

### CSE 성능 측정 (직접 비교)

| 방식 | median | min |
|------|--------|-----|
| double-load + CSE (자동) | 6ms | 5ms |
| break-based 단일 load (수동) | 6ms | 5ms |

→ **완벽히 동등한 성능** ✅ — ISSUE-20260521 근본 해결. Principle 2 (Workaround 금지) 준수.

### ISSUE 처리

| ISSUE | 변경 | 비고 |
|-------|------|------|
| ISSUE-20260521-mir-cse-and-chain | OPEN → **RESOLVED ✅** | AndChainCSE 구현으로 근본 해결 |

---

## 다음 세션 (Cycle 3031+)

### 권장 우선순위

1. **M4 채택 지표** — GitHub stars, 외부 PR, 외부 프로젝트 추적 (HUMAN-blocked 잔여)
2. **B축 Claude 재측정** — 98.0% stale 기한 2026-08-13 (아직 여유)
3. **Tier 1 벤치마크** — binary_trees/fasta/mandelbrot 등 최신 측정 확인
4. **추가 P-track 최적화** — 다음 단일-load 기회 탐색 (or-chain CSE 확장 등)

### 알려진 HUMAN-blocked 항목

- GPT-4o 실험 (multi-model-validation)
- golden-flakiness-inttoptr Option A/B/C
- problem-difficulty-bias 신규 hard 문제 20개
- crosslang 측정 (stale)

### ISSUE 현황 (Active 5개)

| ISSUE | 상태 | 우선순위 |
|-------|------|---------|
| ~~mir-cse-and-chain~~ | ~~OPEN~~ → **RESOLVED ✅** | — |
| multi-model-validation | PARTIALLY RESOLVED | MEDIUM |
| external-problem-validation | PARTIALLY RESOLVED | MEDIUM |
| integration-category-weakness | PARTIALLY RESOLVED | LOW |
| problem-difficulty-bias | OPEN | LOW |
| golden-flakiness-inttoptr | OPEN | P3 |

### 알려진 BMB 언어 특성 (중요도 순)

- `else if` 체인 세미콜론: statement 위치에서 `};` 필수 (Cycle 2984 발견)
- `fn main() -> i64 = { ... };` 끝에 `;` 필수 (Cycle 2986 발견)
- `match`: integer literal, char literal, OR pattern (`a | b => ...`) 지원
- `match` arm body: block + comma 필요 (`{ expr }` 후 `,` 필수, 마지막 arm 제외)
- `band`/`bor`/`bxor`/`bnot`: bitwise 연산자 지원
- `break`/`continue`/`return`: ✅ 지원 (단, break는 while에서만)
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `memset_fill(ptr, val, count)`: ✅ native-only builtin (v0.100.1 신규)

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (stale 기한: 2026-08-13) |
| GPUStack qwen3.6-35b-a3b | **100.0% (2026-05-21)** | **최신 공식 측정** |
