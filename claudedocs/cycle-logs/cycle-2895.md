# Cycle 2895: 문서 완성도 정리 — interpreter-only 레이블 전면 갱신
Date: 2026-05-15

## Re-plan
Cycle 2894 Carry-Forward 인계사항 없음. 조사 결과 bmb_reference.md에 Cycle 2894로 해소된 "interpreter-only" 레이블 14개가 잔존. ROADMAP.md ① 우선순위 항목이 stale (for-in-svec 완료 미반영). HANDOFF.md가 Cycle 2893 기준으로 멈춤. Cycle 2895 범위: **문서 완성도 정리**.

## Scope & Implementation
**Files changed**: `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`, `claudedocs/ROADMAP.md`, `claudedocs/HANDOFF.md`

**bmb_reference.md — 갱신 항목 (14개)**:
1. Line 176: str_hashmap 섹션 헤더 `interpreter-only v0.98.5+` → `native-supported v0.98.9+/v0.98.10+`
2. Line 505: Pattern: Iterate vec `interpreter-only` → `native-supported v0.98.9+`
3. Line 556: Pattern: Iterate str_hashmap keys `interpreter-only svec` → `native-supported v0.98.9+`
4. Line 577: for-in-svec 주석 `interpreter-only` → `native-supported v0.98.9+`
5. Line 647: Positional format string `interpreter-only` → `native-supported v0.98.9+`
6. Line 707: str_replace 패턴 `interpreter-only` → `native-supported v0.98.9+`
7. Line 738: Pattern: Vec iteration `interpreter-only` → `native-supported v0.98.9+`
8. Line 759: Pattern: String expression interpolation `interpreter-only` → `native-supported v0.98.9+`
9. Line 783: Pattern: String interpolation `interpreter-only` → `native-supported v0.98.9+`
10. Line 835: Pattern: Palindrome check `interpreter-only` → `native-supported v0.98.9+`
11. Line 841: Pattern: Whitespace-tokenized input `interpreter-only` → `native-supported v0.98.9+`
12. Line 862: Pattern: Float parsing/line-by-line `interpreter-only` → `native-supported v0.98.9+`
13. Line 914: Caveats `interp-only` 2곳 → `native-supported v0.98.9+`
14. Line 918: `str_hashmap_values still interpreter-only` → `native-supported (v0.98.10+, Cycle 2894)`
15. Line 929: "interpreter-only builtins: str_hashmap_values" → "No interpreter-only builtins remain"

**ROADMAP.md**:
- 최상단 갱신 로그: Cycles 2891-2894 native 포팅 완전 달성 기록 추가
- ① 우선순위: `다음: for x in svec {}` → `interpreter-only 제로, 전체 완료`

**HANDOFF.md**:
- HEAD: `ea4b5002` → `ac2b4d80`
- 세션 범위: Cycles 2877-2893 → 2877-2895
- Cycle 2894/2895 작업 행 추가
- native 포팅 현황: `str_hashmap_values` 마지막 항목 추가, interpreter-only 섹션 제거
- 다음 세션 권장: Carry-Forward "없음", B축 재측정/C# 바인딩 권장으로 갱신

## Verification & Defect Resolution
- **mir 코드 복구 발견**: `git diff --stat HEAD` 실행 중 `bmb/src/mir/lower.rs` + `bmb/src/mir/mod.rs` 미커밋 상태 발견 (Cycles 2884-2890 작업 — for-in-vec/svec native handle tracking + format() MIR lowering, ~395줄). 해당 코드는 올바르게 작동하나 커밋 누락 상태였음.
- `cargo test --release -p bmb` → **2388 passed, 0 failed** ✅ (mir 복구 코드 포함 전체 테스트 통과)
- `grep -n "interpreter-only" bmb_reference.md` → 단 1건 (line 929: "No interpreter-only builtins remain" 긍정 문장) ✅
- ROADMAP ① 항목 확인: `interpreter-only 제로, 전체 완료` ✅
- HANDOFF HEAD 확인: `372e8bf8` ✅

## Reflection
- **Scope fit**: Cycle 2894의 Residual로 명시된 stale 레이블 정리 완료. 문서와 코드 현실 일치.
- **Latent defects**: 없음. ROADMAP의 이전 갱신 로그 내 "interpreter-only" 표현은 역사적 사실 기록으로 변경 불필요.
- **Structural improvement**: bmb_reference.md에 "interpreter-only" 레이블이 장기간 잔존한 이유 — 포팅 완료와 문서 갱신이 분리 사이클에서 이루어짐. 향후: 빌트인 native 포팅 완료 시 동일 사이클 내 문서 동기화 선호.
- **Roadmap impact**: M4 ① 완전 완료 공식 기록. 다음 자율 작업은 ROADMAP ②(B축), ④(C# 바인딩), ③(P-track) 순.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - 런타임 라이브러리 단일화 (bmb/runtime ↔ runtime 이중 관리 해소)
  - bmb_runtime.c 변경 시 CI 자동 rebuild
  - inkwell/text 백엔드 함수 등록 정합성 테스트 (Rule 7 위반 방지)
- Pending Human Decisions: B축 재측정 (API key 필요), tier3-spawn-overhead Option A/B/C
- Roadmap Revisions: ROADMAP.md ① 항목 갱신 완료
- Next Recommendation: Cycle 2896 — B축 재측정 준비(API key 없이 가능한 스크립트 점검) 또는 C# 바인딩 scaffold 시작
