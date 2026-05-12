# ISSUE: Track M — Machine-First Output 잔여 작업

> **트랙**: M (Machine-First Output)
> **마일스톤**: M2 (AI-Ready Infrastructure)
> **현 상태**: ~75% 구현
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 4.2

## 현 상태

- ✅ CLI 디폴트 = machine, `--human` 명시 옵션 (`bmb/src/main.rs:7-12, 18-20`)
- ✅ 에러/경고 JSON (`bmb::error::report_error_machine`, `report_warnings_machine`)
- ✅ LSP query `--format json|compact|llm` (line 304-405)
- ✅ JSON output 표준 형식 (`{"type":"error","message":"..."}`)

## 잔여 작업

1. **`bmb dump-ast` 머신 출력 강화**
   - 현재: `serde_json::to_string_pretty(&ast)` (line 1603) — 토큰 비효율
   - 개선: `--format compact|pretty` 옵션 추가, 디폴트 = compact

2. **JSON 스키마 명세서 작성**
   - 신규 파일 `docs/AI_OUTPUT_SCHEMA.md`
   - 모든 명령 출력 (build/run/check/fmt/lint/verify/bench)의 JSON 스키마 명시
   - 스키마 버전 헤더 (`{"_schema":"bmb.v1",...}`) 도입 검토

3. **Output 일관성 회귀 테스트**
   - 모든 명령에 대해 `--human=false`/디폴트 시 valid JSON 검증
   - CI gate 추가 (선택)

4. **fmt diff 출력**
   - 현재 형식 점검 → JSON Patch 형식 또는 기존 unified diff 유지 결정

## 완료 조건 (M2 정합)

- [ ] dump-ast `--format` 추가
- [ ] AI_OUTPUT_SCHEMA.md 작성 (모든 명령 커버)
- [ ] 머신 출력 회귀 테스트 (최소 build/check/lint)
- [ ] CLAUDE.md Rule 8 적용 점검

## 추정 사이클

2-3 cycles, M1 완료 후 시작 권장.
