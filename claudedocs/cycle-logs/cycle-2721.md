# Cycle 2721 (RE-PLAN): P축 backlog 평가
Date: 2026-05-11

## Re-plan (🟠 RE-PLAN trigger)

**기존 plan**: FP 1-arg arity guard 확장.

**Trigger**: HANDOFF L115 "FP guard 낮은 우선순위" + Cycle 2716 triage HashMap/StringBuilder **High** priority. ROI 차이가 명확 — FP guard는 가상 충돌 (no incidents), P-track은 실측 성능 데이터.

**Adjustment**: Cycle 2721 P-track 평가로 교체. Cycle 2722-2725 P-track 구현. Cycle 2726 FP arity guard 통합 (mechanical, 1 cycle). Cycle 2727 closeout.

Advisor 자문 일치 (re-validation 아닌 RE-PLAN 정정).

## Scope & Implementation

### 5개 P-track ISSUE 평가표

| ISSUE | 우선순위 표시 | 영역 | 영향 벤치마크 | 사이클 비용 | 단일 cycle 가능? | ROI |
|-------|------------|------|-------------|----------|----------------|------|
| match-jump-table | P0 | codegen, mir | brainfuck 111%, lexer 109% | **재진단 1 cycle** | ✅ | ⭐⭐⭐ |
| hashmap-perf | P0 | stdlib, runtime | hash_table 111% | **다중 (4단계)** | ❌ | ⭐⭐ |
| string-builder-opt | P1 | runtime, stdlib | fasta 108% | 1-2 cycles | ⚠️ SSO만 | ⭐⭐ |
| compare-inline | P1 | mir, monomorphization | sorting 110% | 1-2 cycles | ⚠️ threshold만 | ⭐⭐ |
| alloc-optimization | P1 | runtime | binary_trees 106% | **매우 큰** (런타임 Arena 신규) | ❌ | ⭐ |

### 주요 발견

1. **match-jump-table은 Rediagnosis 상태**: Cycle 362 (2026-04-13)에서 "infrastructure 이미 존재" 확정 — MIR `Switch` (mir/mod.rs:797), `IfElseToSwitch` pass (optimize.rs:3541), LLVM `switch` codegen (llvm.rs:5231, llvm_text.rs:7401), lowering (lower.rs:1820). brainfuck/lexer slowness의 실제 원인은 다른 곳. **단일 cycle로 IR 검증 가능**.

2. **HashMap/Alloc은 multi-cycle**: 새 자료구조 (오픈 어드레싱) 또는 런타임 구조 (Arena allocator) 추가 — 1 cycle 부적합. **다음 세션 또는 carry-forward**.

3. **StringBuilder SSO + Compare inline은 부분 가능**: 핵심 변경만 (1 cycle).

4. **벤치마크 일관성 주의**: 위 % 수치는 v0.51.22 기준 (2026-04-13). 현재 ROADMAP § 5의 inproc 측정 (nqueen/fibonacci/knapsack/mandelbrot)과 중첩 없음 — 이슈가 다루는 벤치마크는 Tier 1/3에 있음. 사이클 진행 시 **현재 측정 재실행 필수**.

### 우선순위 결정 (Cycles 2722-2725)

| Cycle | 작업 | 근거 |
|-------|------|------|
| 2722 | **Match jump table 재진단** | 단일 cycle 가능, P0, IR 검증 |
| 2723 | **Compare inline (MIR threshold)** | mechanical, sorting 110% |
| 2724 | **StringBuilder SSO** | fasta 108%, runtime 변경 작음 |
| 2725 | **재진단 결과 기반 후속 작업** | 2722 결과로 결정 |

HashMap/Alloc은 **carry-forward** (next session, multi-cycle scope).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| 5개 ISSUE 모두 읽음 | ✅ |
| 우선순위 평가 근거 명확 | ✅ |
| 단일 cycle 가능성 분류 | ✅ |
| ROADMAP revision 기록 | ✅ |

결함: 없음 (평가 사이클).

## Reflection

### 외부 관찰자 관점

1. **Rediagnosis status 발견의 가치**: ISSUE 표지에 "⚠️ Rediagnosis needed (Cycle 362)"라 명시. 평가 없이 진입했으면 이미 있는 infra를 재구현했을 가능성. **이슈 표지 읽기 의무** 교훈.

2. **벤치마크 측정 불일치**: ROADMAP § 5 inproc 데이터 (4 도메인)와 ISSUE 데이터 (Tier 1/3 6 도메인)가 분리됨. 작업 완료 검증 시 같은 벤치마크 재측정 필수.

3. **multi-cycle scope의 인식**: HashMap/Alloc은 작업 범위가 1 cycle 초과. 무리하게 진입하면 partial completion (insights doc의 반복 패턴). **다음 세션 또는 별도 다중 cycle phase**.

4. **advisor RE-PLAN 트리거의 정당성**: 인수받은 plan을 그대로 진행하지 않고, HANDOFF의 명시적 우선순위를 따른 것이 정상. STEP 0의 "drift check"가 작동.

### Roadmap impact (large)

- ROADMAP.md Phase 2/3 순서 교체 (이미 적용)
- HashMap/Alloc는 carry-forward (next session)
- Match jump table 재진단이 가장 큰 미지수 — 결과 따라 후속 사이클 reshape

## Carry-Forward

- Actionable (Cycle 2722): Match jump table **재진단** — 실제 brainfuck/lexer LLVM IR이 switch 사용 여부 검증
- Structural Improvement Proposals:
  - **HashMap 재설계 (multi-cycle phase)**: 다음 세션 또는 별도 phase
  - **Allocator Arena 인프라**: 런타임 구조 변경, 다중 cycle
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: Phase 2/3 순서 교체 + Cycle 2726으로 FP 통합 deprioritize (ROADMAP.md 적용 완료)
- Next Recommendation: Cycle 2722 = brainfuck/lexer LLVM IR 검증 + 실제 원인 진단
