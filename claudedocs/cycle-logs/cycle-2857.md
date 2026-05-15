# Cycle 2857: str_count / str_pad_left / str_pad_right
Date: 2026-05-15

## Re-plan
Carry-Forward (2856): None. 문자열 조작 유틸리티 추가 — AI 생성 코드에서 출력 포맷팅, 패턴 분석에 사용.

## Scope & Implementation

**str_count(s, sub)** (Cycle 2857, interpreter-only):
- `str_count(s: String, sub: String) -> i64`: 문자열 `s`에서 `sub` 출현 횟수
- 빈 sub → 0 반환 (무한 루프 방지)
- Rust `str::matches()` 활용

**str_pad_left(s, width, pad_char)** (Cycle 2857, interpreter-only):
- `str_pad_left(s: String, width: i64, pad: String) -> String`: 좌측 패딩
- `pad`의 첫 글자 사용 (빈 string이면 공백)
- 이미 width 이상이면 원본 반환

**str_pad_right(s, width, pad_char)** (Cycle 2857, interpreter-only):
- `str_pad_right(s: String, width: i64, pad: String) -> String`: 우측 패딩

변경 파일:
- `bmb/src/interp/eval.rs`: 3종 함수 구현 + 등록
- `bmb/src/types/mod.rs`: 3종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_str_count_pad` (4케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 3종 문서화

## Verification & Defect Resolution
- test_interp_str_count_pad: 4/4 통과 ✅
- cargo test --release 전체: **2382 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ str API 대폭 강화: str_count + str_pad_left/right + str_to_upper/lower + str_char_at
- ✅ AI가 생성한 표 형식 출력 코드에서 패딩 패턴 필수 — 이제 네이티브 지원

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * 필드 복합 할당 native 지원 (codegen)
  * InterpMini `consume()` dead_code 경고 정리 (minor)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2858/2859/2860 — HANDOFF/ROADMAP 업데이트 + 커밋
