# Cycle 3289: M15 Phase 5 — module-suggest + platform swallow 버그 수정
Date: 2026-05-29

## Re-plan
M15 Phase 5: platform↔module capability 자동 연계. [P4] platform swallow 버그가 Phase 5 구현의 선결 조건임을 발견.

## Scope & Implementation

### [P4] callers_collect_source platform 블록 swallow 버그 수정
**원인**: `callers_collect_source`가 platform 블록 내부의 `fn` 선언을 일반 함수로 파싱 시도 → 바디 없는 platform fn의 body scanner가 이후 소스를 swallow.

**수정**: `callers_collect_source`에 platform 블록 감지 + 스킵 추가:
- `skip_nested_brace(src, pos, depth)`: 중첩 brace를 재귀적으로 스킵 (기존 `skip_brace_block` 2-arg 버전과 이름 충돌 방지)
- `skip_platform_block(src, pos)`: platform 키워드 이후 블록 전체 스킵
- `callers_collect_source` TK_IDENT 케이스 추가: "platform" 식별 시 `skip_platform_block` 호출

### M15 Phase 5: module-suggest 명령
**새 기능**: `compiler module-suggest <file>` → JSON 출력
```json
{"type":"module_suggest","module":"X","declared":["IO"],"suggested":["File"],"status":"mismatch"}
```
- `status`: "ok" (declared==suggested) | "mismatch" | "needs_module" (no module decl)
- `ms_gather_used_caps`: entries + transitive_map에서 실제 사용 capability 수집
- `scan_module_name`: 파일에서 module 이름 추출
- `ms_caps_to_json`: capability 리스트 → JSON array

**검증**:
- mismatch: requires [IO] but actual [File] → 탐지 ✅
- ok: requires [File] matches actual [File] → ok ✅
- needs_module: no declaration but uses [File] → needs_module ✅
- cargo test 3800+2390+23 PASS ✅

## Verification & Defect Resolution
모든 테스트 통과.

## Reflection
- **P4 수정**: platform swallow 버그 수정으로 module-suggest, contracts-check 정확도 향상.
- **Scope**: Phase 5 목표 달성 — platform capability ↔ module requires 자동 연계 제안.
- **Note**: `index` 명령은 별도 코드 경로 사용, platform 버그 여전히 존재 (낮은 우선순위).
- **Roadmap impact**: ROADMAP에 M15 Phase 5 완료 마킹 필요.

## Carry-Forward
- Actionable: ROADMAP Phase 완료 마킹 + 커밋
- Structural Improvement Proposals: index 명령의 platform 스킵 버그 수정 (별도 P3)
- Pending Human Decisions: 없음
- Roadmap Revisions: M15 Phase 5 ✅ 추가
- Next Recommendation: contracts-check 개선 (P4) 또는 커밋 + ROADMAP 업데이트
