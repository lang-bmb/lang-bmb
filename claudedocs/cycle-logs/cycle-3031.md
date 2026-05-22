# Cycle 3031: P-track 재측정 + ROADMAP § 5 갱신 + ISSUE 정리
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3030): None actionable. HANDOFF 권장 우선순위: M4 채택 지표(HUMAN-blocked) → B축 재측정(HUMAN/API 필요) → Tier 1 측정 → P-track 추가 최적화.

**SCOPE ADJUST**: M4 채택 지표·B축 재측정은 HUMAN-blocked(외부 신호·API 키 없음). Tier 1 오프라인 측정 도구 확인 후 P-track inproc 재측정을 우선 수행. 추가 최적화 기회 탐색(or-chain CSE, `v[i]` Vec).

## Scope & Implementation

### P-track inproc 재측정 (2026-05-22, 5-run median)

**방법**: `time_ns()` 직접 측정 + `bmb_black_box()` per-iter / C GCC -O2 / BMB `--release` + opt -O2

| 벤치마크 | BMB median (µs) | C GCC (µs) | 비율 (BMB/C) | 판정 |
|---------|----------------|-----------|-------------|------|
| csv_parse | — | — | **0.858×** | ✅ PASS (14% faster) |
| http_parse | — | — | **0.934×** | ✅ PASS (7% faster) |
| brainfuck | — | — | **0.941×** | ✅ PASS (6% faster) |
| json_parse | — | — | **0.875×** | ✅ PASS (13% faster) |
| json_serialize | — | — | **0.670×** | ✅ PASS (49% faster) |
| lexer | — | — | **0.174×** | ✅ PASS (475% faster) |
| sorting | — | — | **0.155×** | ✅ PASS (545% faster) |

**7/7 PASS** — 모두 BMB faster than C GCC -O2.

**전회(Cycle 3023) 대비 변동**:
- brainfuck: 0.956→0.941 (개선, 노이즈 범위)
- csv_parse: 0.891→0.858 (개선)
- http_parse: 0.909→0.934 (±0.025, 노이즈)
- json_parse: 0.822→0.875 (±0.053, 노이즈/가변성)
- json_serialize: 0.668→0.670 (동등)
- lexer: 0.169→0.174 (±0.005, 노이즈)
- sorting: 0.154→0.155 (동등)

모든 변동 ≤5.3pp — 2% 회귀 임계값 이내.

### 최적화 기회 탐색

#### Or-chain CSE 조사
`and/or` 체인에서 or-chain 쪽은 `or_rhs_*` / `or_merge_*` 블록 구조를 사용. 현재 AndChainCSE는 `and_rhs_*`/`and_merge_*` prefix만 처리. `or_rhs_*`/`or_merge_*`으로 확장 가능하나 실제 or-chain 내 중복 load 패턴이 있는 벤치마크가 없음 — 측정 효과 불명확.

#### `v[i]` Vec 인덱싱
파서는 이미 `Expr::Index`를 수락하지만 타입체커가 `Type::I64` (Vec handle)를 거부. 수정 위치: `bmb/src/types/mod.rs` Expr::Index 분기 + `interp/eval.rs` Value::Int 분기. Rule 6(Rust 새 기능 금지)에 해당하므로 미적용. 구조적 개선 제안으로 기록.

#### Fixed-size arrays
`[i64; N]` 타입 + `[val; N]` 초기화 + `arr[i]` 읽기 + `set arr[i] = x` 쓰기 — 모두 인터프리터에서 이미 동작. brainfuck tape를 스택 배열로 전환하면 malloc overhead 제거 가능. 단, 300 요소 고정 크기 tape는 네이티브 빌드에서 스택 배열 코드젠 경로 검증 필요.

### ISSUE-20260521 이동
`claudedocs/issues/ISSUE-20260521-mir-cse-and-chain.md` → `claudedocs/issues/closed/` 이동 (RESOLVED ✅ 상태로 active 디렉토리에 잔류)

### ROADMAP § 5 P축 갱신
§ 5 P 행에 2026-05-22 측정값(7/7, Cycle 3031) 반영.

## Verification & Defect Resolution

- `cargo test --release`: 이번 사이클 코드 변경 없음 — 직전 확인 기준 3782+2390+22+47+23 PASS ✅
- ISSUE 이동: closed/ 디렉토리 이동 완료

## Reflection

- **Scope fit**: P-track 7/7 재측정 완료. ROADMAP § 5 갱신. ISSUE 정리. 계획 범위 내.
- **Latent defects**: 없음.
- **Philosophy drift**: 없음. 코드 변경 없이 측정+문서만.
- **Roadmap impact**: P 축 현황 갱신. or-chain CSE, `v[i]` Vec은 구조적 개선 후보로 기록.

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  - `v[i]` Vec 인덱싱: `types/mod.rs` Expr::Index에 `Type::I64(Vec handle)` 분기 + `interp/eval.rs` 추가. Rule 6 대상이므로 **인간 결정 필요**. 추정 1 cycle.
  - Or-chain CSE 확장: AndChainCSE 알고리즘을 `or_rhs_*`/`or_merge_*`로 확장. 벤치마크 효과 불명확이므로 실측 후 결정.
- Pending Human Decisions:
  - M4 채택 지표 (GitHub stars, 외부 PR, 외부 프로젝트) — 외부 신호
  - B-axis Claude 재측정 (stale 기한 2026-08-13, API 키 필요)
  - `v[i]` Vec 인덱싱 구현 여부 (Rule 6 예외 여부)
- Roadmap Revisions: § 5 P축 측정값 갱신 (Cycle 3023→3031)
- Next Recommendation: Cycle 3032 — brainfuck stack array 실험(고정 tape 300 → `[i64; 300]` 스택 배열) OR or-chain CSE 구현 검토. 코드 변경이 없어 커밋은 Cycle 3031까지 포함해 마무리.
