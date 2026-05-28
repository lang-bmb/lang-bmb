# Cycle 3252: M13 Phase 1+2 — `intent:` 어노테이션 파싱 + lint 경고
Date: 2026-05-29

## Re-plan

Plan valid. M12 Phase 1 완료 후 M13 Phase 1 진행. gotgan SHA(M14)보다 범위가 명확하고 bootstrap 단독으로 완료 가능.

## Scope & Implementation

### 설계

`intent: "자연어 의도"` 어노테이션을 함수 pre/post 영역에 추가:
```bmb
fn foo(x: i64) -> i64
    intent: "Returns positive square of x"
    pre x >= 0
    post it >= 0
= x * x;
```

Phase 1 = 파싱 전용 (컴파일러가 파싱하고 무시). Phase 2에서 intent↔계약 일관성 검사 추가.

### 변경

**bootstrap/compiler.bmb** — `skip_contracts` 함수 (~5984):
- `TK_IDENT` + text == "intent" + `TK_COLON` + `TK_STRING_LIT` 패턴 인식
- 파싱 후 `skip_contracts` 재귀 (continue)
- 어노테이션 내용은 버려짐 (Phase 1 의도적 설계)

**tests/golden/test_golden_intent_annotation.bmb** (신규): pre/post와 함께 intent 테스트.
**tests/golden/test_golden_intent_annotation.bmb.out** (신규): 기대 출력 `32`.

### 구현 세부

- `get_ident_text(src, pos, t) == "intent"` 로 식별
- `TK_COLON` 확인 후 `TK_STRING_LIT` 스킵
- pre/post/where/intent 모두 `skip_contracts`에서 처리됨

## Verification & Defect Resolution

- Stage 1 빌드: ✅
- `test_golden_intent_annotation.bmb` → 32 ✅ (3+4+25)
- Fixed Point S2 == S3: ✅
- bmb lint: 177 warnings (변화 없음) ✅

## Reflection

**Scope fit**: M13 Phase 1 파싱 목표 달성.

**Latent defects**: 없음. intent는 무시되므로 기존 코드에 영향 없음.

**Philosophy drift**: 없음. Rule 6 준수.

**Roadmap impact**: M13 Phase 1 ✅. Phase 2 (intent↔계약 일관성 경고)는 별도 사이클.

## Carry-Forward

- **Actionable**: M13 Phase 2 — bmb lint에서 intent↔계약 불일치 경고 추가
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: None
- **Roadmap Revisions**: M13 Phase 1 ✅ COMPLETE
- **Next Recommendation**: M14 Phase 1 (gotgan SHA lockfile) 또는 M12 Phase 2 (effect 타입 체커) 또는 M13 Phase 2
