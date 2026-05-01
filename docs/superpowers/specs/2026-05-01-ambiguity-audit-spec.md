# Ambiguity Audit Spec — Track Q (Phase 1)
Date: 2026-05-01 (Cycle 2518)
Anchor: `docs/superpowers/specs/2026-05-01-vision-v1.0-realignment.md` § 4.2 Track Q + § 8
Issue: `claudedocs/issues/ISSUE-20260501-track-q-ambiguity-audit.md`

> **목적**: BMB 문법의 모호성을 정량화·감사·잠금. AI agent가 BMB 코드 작성 시 syntax 오해 최소화. spec § 8 "지연 결정 영역" 중 첫 결정.

---

## 1. "모호성 0" 운영 정의

### 1.1 후보 평가

| 후보 | 정의 | 측정 가능성 | 본 cycle 평가 |
|------|------|----------|------------|
| **A. LR(1) parser conflict 0** | LALRPOP 빌드 시 shift-reduce / reduce-reduce conflict 0건 | ✅ 자동 (LALRPOP report) | 이미 충족 (현재 LALRPOP 빌드 통과) |
| **B. 인접 토큰 시퀀스 ambiguity 0** | 같은 토큰 시퀀스가 2가지 의미로 파스되는 위치 0건 | ⚠️ 수동 (검토 필요) | 추적 인프라 필요 |
| **C. AI 작성 시 syntax 혼동 0** | LLM이 BMB 작성 시 syntax error 발생률 = 0 | ⚠️ 외부 변수 (R 트랙 측정) | "0" 불가능 — 도구 보완 (MCP, context-pack) |
| **D. 식별자 vs 키워드 충돌 0** | reserved keyword가 자주 쓰이는 단어와 충돌 0건 | ✅ 수동 (인벤토리 가능) | 점검 가능 |
| **E. 중첩 표현 우선순위 자명** | 운영자 우선순위 중첩 시 사용자 예상과 일치 | ⚠️ 사용성 평가 | 설계 가이드 필요 |

### 1.2 본 cycle 결정

**A + D + E** 조합:
- A (LR(1) conflict 0) — 정량 baseline
- D (키워드 충돌 0) — 식별 가능 결함 추적
- E (우선순위 자명) — 설계 가이드라인 영속화

**B (시퀀스 ambiguity)** 와 **C (LLM 정답률)** 는 R 트랙 측정 결과 후 별도 평가.

→ Track Q 운영 정의 = "**LR(1) conflict 0 + 키워드 충돌 0 + 운영자 우선순위 가이드 영속화**".

---

## 2. 정적 분석 도구

### 2.1 LR(1) Conflict 추적 (자동)

LALRPOP는 빌드 시 conflict 보고. 현재:
- `bmb/src/grammar.lalrpop` 2,340 LOC
- 빌드 시 `lalrpop` 호출 → conflict 0 (현재 통과)

**잠금 정책**: `cargo build` 자체가 LR(1) conflict 회귀 방지 (CI gate 자동).

추가 작업: 없음. ✅ 이미 충족.

### 2.2 키워드 충돌 인벤토리 (수동)

#### 현재 키워드 (grammar.lalrpop:9-90 추정)

```
fn, let, var, if, then, else, pre, post, true, false, ret, and, or, not,
struct, enum, match, new, mut, set, while, for, in, loop, break, continue,
return, band, bor, bxor, bnot, invariant, implies, forall, exists, pub,
use, mod, where, it
```

41개 reserved 키워드.

#### 충돌 후보 (자주 쓰이는 영어 변수명)

| 키워드 | 충돌 가능성 | 영향 |
|--------|---------|------|
| `let` | 중 (Rust 영향) | OK (관례적) |
| `var` | 중 | OK |
| `set` | **높음** (`set` 변수명 자주 씀) | ⚠️ — 평가 필요 |
| `it` | **높음** (`it` 자주 씀) | ⚠️ — 평가 필요 |
| `new` | **높음** (객체지향 영향) | ⚠️ — 평가 필요 |
| `where` | 낮음 | OK (Rust/SQL 영향) |
| `loop` | 중 | OK |
| `mod` | 중 (`mod` 함수명) | ⚠️ — 평가 필요 |
| `mut` | 낮음 | OK |
| `ret` | 낮음 | OK (BMB 특유) |

**자율 결정 시점 deferred**: 이는 언어 스펙 변경 영역 — Track Q Phase 2+에서 BMB syntax review 시.

### 2.3 운영자 우선순위 가이드

#### 현재 BMB 우선순위 (gram검증 필요)

```
높음 → 낮음:
1. unary (- + not bnot)
2. *  /  %
3. +  -
4. <<  >>  band  bor  bxor
5. <  <=  >  >=
6. ==  !=
7. and
8. or
9. implies
10. =  +=  -=  ...  (assignment)
```

#### AI 친화 가이드라인 (영속화)

1. **명시적 괄호 권장**: `(a band b) shl 2` 등 비전형 조합
2. **bitwise vs comparison**: 같은 줄에 섞을 시 괄호 필수
3. **assignment vs equality**: BMB는 `=` 정의, `==` 비교 — Rust 영향. 헷갈릴 시 lint 경고

→ 별도 spec `docs/STYLE_GUIDE_AI.md` 또는 `LANGUAGE_REFERENCE.md` 보강.

---

## 3. `bmb lint --ai-friendly` 옵션

### 3.1 검출 패턴

| 패턴 | 경고 메시지 | 우선순위 |
|------|----------|------|
| 중첩 `band`/`bor` 괄호 없음 (`a band b bor c`) | "Add parentheses for clarity in mixed bitwise expressions" | High |
| 중첩 `and`/`or`/`implies` 괄호 없음 | "Add parentheses for clarity in mixed logical expressions" | High |
| 함수명 `set`, `it`, `new`, `mod` (예약어 충돌 후보) | "Identifier '{}' shadows BMB keyword — consider rename" | Med |
| 긴 if-else 체인 (5+ branch) | "Consider match expression for clarity" | Low |
| 반복적 `byte_at(pos)` 호출 (피크 패턴) | "Cache result if same position accessed multiple times" | Low |

### 3.2 구현 옵션

- **Rust 추가**: `bmb/src/lint/ai_friendly.rs` (Rule 6 위배)
- **BMB 추가**: `bootstrap/lint/ai_friendly.bmb` (Rule 6 + Track S 정합) — 권장

### 3.3 통합

`bmb lint --ai-friendly` 옵션 활성 시 위 패턴 추가 검출. 디폴트 lint 결과에 추가만, 기존 lint 무영향.

---

## 4. 회귀 잠금

### 4.1 LR(1) 회귀

`cargo build` 자체가 잠금. ✅ 이미 충족.

### 4.2 키워드 변경 회귀

`tests/golden/`에 모든 키워드 사용 예시 → 새 키워드 추가가 기존 식별자 깨뜨릴 시 트립.

추가 작업: 키워드 사용 회귀 테스트 (Phase 2).

### 4.3 우선순위 변경 회귀

`tests/golden/precedence_*.bmb` — 운영자 우선순위 회귀 잠금.

확인: 현재 존재 여부. (검증 후속 cycle)

---

## 5. M2 Track Q 게이트

### 본 cycle (Phase 1) 충족

- [x] 운영 정의 합의 (A + D + E 조합)
- [x] 정적 분석 도구 옵션 평가 (LALRPOP report 자동)
- [x] `bmb lint --ai-friendly` 옵션 설계
- [x] 회귀 잠금 정책

### Phase 2-4 잔여

- [ ] 키워드 충돌 후보 (`set`/`it`/`new`/`mod`) 자율 결정 (BMB syntax review)
- [ ] BMB 우선순위 가이드 영속화 (`docs/LANGUAGE_REFERENCE.md` 추가)
- [ ] `bootstrap/lint/ai_friendly.bmb` 구현 (Phase 2-3, 2-3 cycles)
- [ ] 회귀 테스트 추가 (Phase 4)

→ Track Q 완료 ETA: 4-6 cycles.

---

**다음 단계 (Cycle 2521+ 또는 별도 트랙)**: 
- Phase 2 — 키워드 충돌 후보 결정 + 우선순위 가이드 영속화 (1 cycle)
- Phase 3 — `bootstrap/lint/ai_friendly.bmb` 구현 시작 (2-3 cycles)
