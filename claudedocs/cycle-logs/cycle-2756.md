# Cycle 2756: multiple-pre-clauses ISSUE — documentation close

Date: 2026-05-12

## Re-plan

Cycle 2755 Next Rec 그대로. Trigger ⚪ NONE.

## Scope & Implementation

### 분석: Rule 6 정렬

`bmb/src/grammar.lalrpop:857` 현 spec:
```
<pre:("pre" <SpannedExpr>)?> <post:("post" <SpannedExpr>)?>
```

다중 pre 지원하려면 `*` 로 변경 후 Vec 폴드. 하지만:
- **CLAUDE.md Rule 6**: "Rust 새 기능 추가 ❌ 금지"
- Multiple-pre는 부트스트래핑 차단 요소 아님 → Rust 변경 금지
- 본 ISSUE acceptance criteria: "Either parser support **or** clear documentation"
- 본 ISSUE priority: **LOW**
- ROADMAP Drift C 3 갭 (let-tuple/static-method/Option-expr)은 이미 모두 해소 — 본 ISSUE는 추가 부수적 갭

→ documentation 옵션 채택. parser 변경 회피.

### Documentation 변경

`docs/LANGUAGE_REFERENCE.md` § 10.4 (Legacy Pre/Post) 추가:

```
**Single-clause constraint**: each function accepts at most one `pre` clause
and one `post` clause. Combine multiple preconditions with `and`:

[example fails, example works, where{} preferred example]

The `where { }` block style is preferred for new code and supports
named contracts plus multiple conditions natively.
```

### ISSUE close

`ISSUE-20260326-multiple-pre-clauses.md`:
- Resolution: "documentation 채택, acceptance criterion (2) 충족"
- 향후 재고 조건 명시 (Rust 졸업 / 외부 사용자 use case 누적)
- closed/ 이관

### ROADMAP § 6 갱신

Cycle 2755-2756 sub-section 16 active 반영.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| LANGUAGE_REFERENCE.md § 10.4 단락 추가 | ✅ (grep "Single-clause constraint" 매치) |
| ISSUE closed/ 이관 | ✅ |
| Active ISSUE 17 → 16 | ✅ |
| ROADMAP § 6 갱신 | ✅ |

결함: 없음.

## Reflection

### Rule 6의 좁은 적용

Multiple-pre 파서 변경은 2-line grammar 변경 + 1 액션 코드 추가의 작은 작업. 하지만 Rule 6는 **모든** Rust 새 기능을 금지함. ROADMAP Drift C 3 갭은 명시적 예외가 아닌 "이미 해소" 상태이므로 새 예외 만들 명분 없음.

작은 변경이라도 Rule 6 vs M4 ② "언어 갭" 균형에서 Rule 6 우선 — 이는 BMB가 Rust 컴파일러 졸업으로 가는 점진 전략과 정합.

### Documentation의 명시적 가치

"and combinator 깨끗한 workaround"는 항상 알려진 것이 아님 — LLM/사용자가 `pre x; pre y;` 패턴 자연 시도 시 parser error 만나고 fix 검색해야 함. `LANGUAGE_REFERENCE.md`에 명시 = AI-native 가설 정렬 (LLM training corpus에 BMB reference 포함 시 1-shot 정답률 향상).

### 백로그 감소 속도

세션 시작 19 → 종료 (현재) 16 active = -3 net (-3 close + 0 신규에 가깝게 -1 quicksort-ffi 신규 + 3 close, 단일 세션 16% 감소). M4-1 baseline 후 9 B-track ISSUE 일괄 close 가능 시 16 → ~10 까지 회복 (별도 세션).

### Pacing 점검

소비 cycle: 7/10 (2750-2756). 잔여 3 cycles. 후보:
- D: FP arity guard 36 sites mechanical (1 cycle, low ROI) — bounded
- ad-hoc 작은 작업 (HANDOFF/ROADMAP 정합 점검 등)
- M3-5 narrative double-check or quicksort 회귀 분리 분석

3 cycles 잔여 — multi-cycle phase 시작 회피 (advisor 정합).

## Carry-Forward

### Actionable

- **Cycle 2757**: 후보 (D) FP arity guard 36 sites mechanical 또는 ad-hoc M3-5 narrative double-check
- **Cycle 2758-2759**: 잔여 ad-hoc 또는 HANDOFF 갱신 + 세션 wrap-up

### Structural Improvement Proposals

1. **LANGUAGE_REFERENCE 갱신 후 LLM training corpus 갱신 절차**: 만약 외부 LLM이 BMB 작성을 한다면 reference docs 갱신은 LLM 행동에 즉시 반영되어야. 현재 publish 절차 없음 — M3 publish 이후 자동화 필요. ad-hoc cycle candidate.
2. **`_template.md` "Rule 6 정합 명시" 추가**: 신규 ISSUE 작성 시 Rust 변경 필요 여부 점검 칸 추가 → close 시 documentation 옵션 우선 검토. 1 cycle ad-hoc.

### Pending Human Decisions

- 신규 없음

### Roadmap Revisions

ROADMAP § 6 "Cycle 2755-2756 갱신" sub-section 갱신됨 (`active 17 → 16`, close 2 → 3건).

### Next Recommendation

**Cycle 2757**: 옵션 (D) FP arity guard 36 sites mechanical, 또는 ad-hoc HANDOFF 정합 점검. mechanical 작업은 self-contained — bounded, low cognitive load, advisor pacing 권고 정합.

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| `docs/LANGUAGE_REFERENCE.md` § 10.4 (single-clause constraint 단락) | tracked |
| `claudedocs/issues/closed/ISSUE-20260326-multiple-pre-clauses.md` (mv) | gitignored |
| `claudedocs/ROADMAP.md` § 6 (백로그 카운트 갱신) | claudedocs | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2756.md` | gitignored |
