# ISSUE: Track Q — Ambiguity Audit (신규)

> **트랙**: Q (Ambiguity Audit)
> **마일스톤**: M2 (AI-Ready Infrastructure)
> **현 상태**: 0% — 신규
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 4.2, spec § 8

## 목적

grammar 정적 분석 + `bmb lint --ai-friendly`. 모호 파스 가능 위치 0 보장 → LLM이 BMB 코드 작성 시 syntax 오해 최소화.

## 운영 정의 (spec § 8 지연 결정 영역)

"모호 파스 가능 위치 0"의 정확한 정의가 본 트랙 첫 작업.

후보:
- **A**: LALRPOP grammar에 LR(1) conflict 0건 (이미 부분 충족 — `grammar.lalrpop` 빌드 시 검증)
- **B**: 같은 토큰 시퀀스가 사람 눈에 두 가지 의미로 보이는 위치 0건 (예: `a < b > c` — generic args vs comparison)
- **C**: 인접 키워드/식별자 충돌 (예: `if cond { ... }` vs `if(cond)`)
- **D**: 위 모두

## 작업 단계

1. **Phase 1 — 운영 정의 합의**
   - 위 후보 중 선택 + 근거 문서화
   - 실제 BMB 코드에서 모호 패턴 grep (LLM 작성 코드 샘플)

2. **Phase 2 — 정적 분석 도구**
   - LALRPOP report 활용 (현재 빌드 시 출력)
   - BMB 추가 분석 (선언 형식 자명성, 키워드 중첩 등)
   - `tools/grammar-audit.rs` 또는 BMB 도구

3. **Phase 3 — `bmb lint --ai-friendly`**
   - 사용자 코드 분석 → 모호 패턴 경고
   - 자동 수정 제안

4. **Phase 4 — 회귀 잠금**
   - grammar 변경 시 audit 자동 실행 (CI)

## 완료 조건

- [ ] 운영 정의 ROADMAP에 영속화 (또는 별도 spec)
- [ ] grammar audit 도구 (현재 모호도 0 또는 명시 카운트)
- [ ] `bmb lint --ai-friendly` 옵션 + 1+ 모호 패턴 검출
- [ ] CI 회귀 테스트

## 추정 사이클

4-6 cycles (운영 정의가 중간 난이도, 정적 분석 구현 깊이에 따라 변동).
