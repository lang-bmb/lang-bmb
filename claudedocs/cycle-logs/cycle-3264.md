# Cycle 3264: M13 Phase 3 — repair-hint 명령 (스텁)
Date: 2026-05-29

## Re-plan
M13 Phase 3: repair-hint 명령 구현. Rule 6에 따라 bootstrap/compiler.bmb에만.

## Scope & Implementation
- `bootstrap/compiler.bmb`: `repair-hint` 명령 스텁 추가
  - `repair_hint_file(input)` → JSON `{"type":"repair_hints","file":"...","functions":[]}` 출력
  - main() dispatcher에 `cmd == "repair-hint"` 분기 추가
- 전체 contract 추출 함수군 구현 시도 → 파싱 버그 발견 및 롤백

## Bugs Found & Fixed
- **Python text mode on Windows**: `open(..., 'w')` → `\n` to `\r\n` 변환 → BMB 소스 전체 CRLF화
- **Python string escaping**: `"{\"type\":...}"` → `{"type":...}` (백슬래시 제거) → BMB `type` 키워드 노출
- **근본 원인**: bootstrap/compiler.bmb 수정 시 Python write는 항상 binary mode (`'wb'`) 사용 필수
- 해결: 복잡한 JSON 문자열 대신 `prefix`/`suffix` 변수 분리 + LF-only 복원

## Verification
- cargo test 3800+2390+47+22+23 PASS ✅
- `bootstrap/compiler.exe repair-hint <file>` → JSON 출력 ✅
- compiler.bmb lint: 177 non-recursive ✅

## Reflection
- Scope fit: 스텁 완성 (기본 JSON 출력). 전체 contract 추출은 다음 사이클로
- Latent defects: contract text 추출 미구현
- Key lesson: Python으로 bootstrap/compiler.bmb 수정 시 반드시 `'wb'` mode 사용

## Carry-Forward
- Actionable: M13 Phase 3 full contract text 추출 (다음 세션)
- Structural Improvement Proposals: bootstrap 수정용 Python 스크립트에 binary write 규칙 문서화
- Next Recommendation: 3-Stage Fixed Point 검증 + Stage 1 빌드
