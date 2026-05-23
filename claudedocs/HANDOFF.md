# BMB Session Handoff — 2026-05-23 (Cycles 3075-3078 — M7-1 COMPLETE)

> **HEAD**: `474e4d4c` (feat(cycles-3075-3077): M7-1 COMPLETE — 17종 contract 부착, 25 llvm.assume 주입)
> **이전 HEAD**: `3827e001` (feat(cycles-3069-3074): M6 완료 선언 + bootstrap str_sb 추적 완전화)
> **3-Stage Fixed Point**: ✅ `dc57beff` (이전: `745082F5`)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: **M7-2 착수** — Rust SMT translator String theory 지원 추가 (`Type::String → SmtSort::Str`)

---

## 이번 세션 작업 요약 (Cycles 3075-3078)

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3075 | M7 전제 검증 + 스모크 테스트 | 5개 진단 + Z3/assume 파이프라인 정상 확인 |
| 3076 | M7-1 Track A 17종 계약 | 스캐너/패턴매처 25 llvm.assume 주입, Fixed Point dc57beff |
| 3077 | M7-1 Track B 결정 | Z3 String 조건 미지원 확인, invariant 주석 문서화 |
| 3078 | M7-2 범위 확정 | SMT translator String 근사화 발견, 조기 종료 |

### 핵심 성과: M7-1 COMPLETE (Track A)

**17개 함수, 25개 llvm.assume 주입**:
- 스캐너: `skip_ws`, `skip_ws_comments`, `scan_int/hex/bin/oct`, `scan_digits_end`, `scan_exponent`, `scan_ident_end`, `scan_string_end`, `scan_char_end`
- 패턴 매처: `find_char`, `find_comma`, `find_comma_or_end`, `find_pattern_noa`, `match_bytes`, `find_pattern_noa_range`
- 모두 `pre pos >= 0` 계열 — LLVM이 음수 포지션 케이스 제거 가능

**Track B 결정**: `method_to_runtime_fn`, `get_call_return_type`, `is_string_returning_fn`
- Z3 String 조건 미지원 (`total:0`) → `// invariant:` 주석으로 보존
- M7-2에서 실제 계약으로 승격 예정

### Z3 String 미지원 근본 원인 발견 (Cycle 3078)

`bmb/src/smt/translator.rs`:
```
Type::String => SmtSort::Int, // String as Int (simplified) v0.5
Expr::StringLit(_) => Ok("0".to_string()) // approximated as 0
```

**M7-2 핵심 작업**: `Type::String → SmtSort::Str` + String literal/`.len()` SMT 번역 추가.

### 문법 발견 (Cycle 3075)

다중 `pre` 절은 `pre A\n  pre B` 형식 미지원 → `pre A and B` 로 결합 필수.

---

## 테스트 상태

- `cargo test --release`: **6264 PASS** ✅
- 3-Stage Fixed Point: `dc57beff` ✅
- Z3: `bmb verify bootstrap/compiler.bmb` → 1513/1513 ✅

---

## 현재 로드맵 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 | ✅ COMPLETE |
| M2 | ✅ COMPLETE |
| M3 | ✅ COMPLETE (2026-05-21) |
| M4 | ✅ COMPLETE |
| M5 | ✅ COMPLETE (Native Complete 포함) |
| M6 | ✅ COMPLETE (2026-05-23) |
| M7 | 🔄 **Contract Pipeline** — M7-1 ✅ COMPLETE, M7-2 착수 예정 |

---

## Known Issues (Active, 모두 HUMAN-blocked)

- `ISSUE-20260326-external-problem-validation.md` — B축 외부 검증 방법론
- `ISSUE-20260326-integration-category-weakness.md` — 통합 카테고리 취약점
- `ISSUE-20260326-multi-model-validation.md` — 다중 모델 검증
- `ISSUE-20260326-problem-difficulty-bias.md` — 문제 난이도 편향
- `ISSUE-20260511-golden-flakiness-inttoptr.md` — 골든 테스트 비결정성

---

## 다음 세션 권장 사항 (M7-2 착수)

### 즉시 착수 가능 (P1)

1. **Rust SMT String theory 추가** (Rule 6 P0 예외 해당):
   - `bmb/src/smt/translator.rs`: `Type::String → SmtSort::Str` (현재 `SmtSort::Int`)
   - `Expr::StringLit(s)` 번역 (현재 `Ok("0")`)
   - `.len()` 메서드 호출 → `(str.len var)` SMT 인코딩
   - 완료 후 Track B 3개 함수 `pre fn_name.len() > 0` 검증 → `total:3, verified:3` 목표

2. **M7-2 검증**: `bmb verify bootstrap/compiler.bmb` → Track B 계약 포함 검증 통과

### 백로그

3. BMB 트랙 Z3 IPC (bootstrap/compiler.bmb에서 exec_output으로 z3 호출) — 대형 작업
4. untracked golden tests 처리
