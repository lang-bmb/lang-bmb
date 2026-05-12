# ISSUE-20260413 — Linter 규칙 고도화

**우선순위**: P2
**영역**: tools
**상태**: Open (Cycle 2703 부분 진척 — Lint 11 builtin_name_collision 추가)

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-10 (Cycle 2703 — Lint 11 추가) |
| `stale_after` | 2026-08-10 |
| `measurement_source` | `bootstrap/lint.bmb` lint count 10→11 |
| `observed_rate` | 11 rules / N planned (전체 목표 미정 — sub-issues로 분기 권고) |
| `scope` | `bmb lint` 단독 (CI 통합은 별도) |
| `env_hash` | n/a (lint 정적 분석) |

## 배경

`bmb lint`는 현재 라인 길이, 문자 분류만 체크. 실용적 정적 분석이 부족.

## 추가할 규칙

- 미사용 변수/함수/import
- 네이밍 컨벤션 (snake_case fn, PascalCase Type)
- 순환 복잡도 경고 (cyclomatic complexity > N)
- 불가능한 패턴 경고 (exhaustiveness 이미 컴파일러 수행)
- 성능 힌트: O(n²) 루프 감지, 불필요한 clone()
- 계약 누락 경고: public fn에 pre/post 없음

## 완료 기준

- 20+ 린트 규칙
- JSON 출력으로 VSCode 통합
- `docs/LINT_RULES.md` 업데이트
