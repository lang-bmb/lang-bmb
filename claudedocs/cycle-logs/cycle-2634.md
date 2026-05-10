# Cycle 2634: M5-2 준비 — wildcard 확인 + Rule 문서화 + Arena OOM 분석
Date: 2026-05-10

## Re-plan
Cycle 2633 Carry-Forward:
- ✅ 회귀 테스트 완료 (8/8 PASS) — 이전 세션 완료
- ✅ HANDOFF + ROADMAP 갱신 — 이전 세션 완료
- ✅ 커밋 — 이전 세션 완료
이번 범위: CLAUDE.md Rule 2 wildcard 오류 수정 + Rule 3 이중 lowering 문서화 + M5-2 범위 분석 + Arena OOM 근본 원인 조사.

## Scope & Implementation

**발견: `_` wildcard는 이미 지원됨**
- CLAUDE.md Rule 2에 "미지원"으로 잘못 표기됨
- `bootstrap/compiler.bmb` line 2146: `if pat_text == "_"` → `parse_match_wildcard` 분기
- 테스트 확인: 정수 match + payload enum match 모두 `_` wildcard 정상 작동
- CLAUDE.md 수정: wildcard = ✅ 지원, payload enum = ✅ M5-1 완료로 업데이트

**CLAUDE.md Rule 3 보완**:
- 이중 Lowering 시스템 요구사항 명시
- recursive(`lower_expr_sb`) + iterative(`step_expr`) 양쪽 처리 필수
- `%_t-1` 생성 실패 패턴 문서화
- 선례: struct_init, lambda, enum_val

**Arena OOM 근본 원인 분석**:
- `types.bmb` (434KB) → 4G로 컴파일 가능 (IR 1.4MB 생성)
- `compiler.bmb` (1MB) → 32G 초과 실패
- 파일 크기 2.3배 증가 → 메모리 8배 이상 → O(n²)+ 성장 패턴 확인
- 원인 추정: 문자열 기반 AST 전처리 중 반복 복사 (prefix/suffix 패턴이 O(n) 호출마다 O(n) 복사 → O(n²))
- 근본 해결: 문자열 → 구조체 기반 AST 표현 전환 (매우 큰 리팩터, 장기 과제)
- 단기 우회 불가능: 32G 이상 RAM 필요 또는 증분 컴파일

**신규 골든 테스트**: `test_golden_enum_wildcard.bmb`
- `enum Shape { Circle(i64), Rect(i64), Other }` + `_` wildcard
- Circle(7) + Rect(5) + Other → 49 + 25 + 0 = 74
- `tests/bootstrap/golden_tests.txt` 엔트리 2835 추가

## Verification & Defect Resolution

**cargo test --release**: ✅ 6210 passed (3773 + 47 + 13 + 2354 + 23)

**Stage 1 golden tests**: ✅ 3/3 PASS
- test_golden_enum_wildcard.bmb (=74) ✅
- test_golden_enum_payload.bmb (=42) ✅
- test_golden_enum_match.bmb (=610) ✅

**발견된 결함**: 없음 (Rule 2 표기 오류는 문서 결함, 코드 결함 아님)

## Reflection

**Scope fit**: 예상 범위 대비 M5-2 범위가 재정의됨. `_` wildcard가 이미 지원되어 M5-2 범위를 Result enum으로 재집중.

**Latent defects**:
- Arena OOM: 구조적 문제. 32G에서도 실패 → 기하급수적 메모리 성장. 단기 패치 불가능.
- Fixed Point 검증 경로 없음: Stage 2가 실행 불가능이므로 Fixed Point 확인 방법이 없음.

**Structural improvement opportunities**:
- 문자열 AST → 구조체 AST 전환: bootstrap 컴파일러의 핵심 아키텍처 변경. `prefix+suffix` 패턴이 O(n²) 메모리의 원인.
- 전체 단계에서 중간 AST 문자열을 재활용 없이 누적 → arena 고갈.

**Philosophy drift**: 없음. M5-2 준비 + 문서 정확성 유지.

**Roadmap impact**: M5-2 = `_` wildcard (이미 완료됨) → 재정의 필요. 실제 M5-2 목표: Result<T, E> 또는 arena 구조 개선.

## Carry-Forward
- Actionable: M5-2 실제 범위 결정 — Result<Ok(i64), Err(i64)> enum 구현 vs arena 구조 개선
- Actionable: PyPI windows-2022 수정 커밋 push (이번 세션 커밋에 포함됨, 재실행 트리거 필요)
- Structural Improvement Proposals:
  - **Bootstrap 문자열 AST → 구조체 전환**: prefix/suffix 누적 패턴 제거. Arena OOM 근본 해결. Rationale: Stage 2 Fixed Point 복원에 필수적인 변경.
- Pending Human Decisions: 없음
- Roadmap Revisions: M5-2 = `_` wildcard 항목 제거 → Result<T, E> enum + 다중 payload 로 재정의
- Next Recommendation: Cycle 2635 — M5-2 Result enum 설계 OR arena OOM 구조 분석 심화
