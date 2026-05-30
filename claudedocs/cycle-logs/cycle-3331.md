# Cycle 3331: contracts_check_run에 module_capability 포함
Date: 2026-05-30

## Re-plan
SCOPE ADJUST 적용 — 상속된 Actionable(L1 stack-allocated tuple ABI, 3-5 사이클)은 Pending Human Decision
(csv 1.039× 측정 노이즈 허용 여부)이 미결이므로 단일 사이클 시작 부적절. P3 태스크
(contracts_check_run + module_capability, 1 사이클 완결 가능)로 조정.

## Scope & Implementation
**목표**: `contracts-check` 명령이 module_capability 섹션을 포함하도록.

**접근 방식**:
1. `cc_json_prefix_sb(s, pos, limit, sb)` — sb_push_char 기반 prefix 복사 헬퍼 (line ~46637)
2. `cc_combine_mc(cc_json, mc_json)` — 두 JSON 결합: cc_json 닫는 `}` 제거 후 `,\"module_capability\":mc_json}` 삽입 (line ~46654)
3. `contracts_check_run` — `mc_build_json` 호출 추가 + `cc_combine_mc` 적용 (line ~46741)

**cc_build_json 시그니처 불변**: diagnose_file 콜사이트 영향 없음. 7-param lint 경고 회피.

**diagnose 분리 유지**: `diagnose_file`은 `cc_build_json(..."")` 아닌 원본 6-param 호출 → `contracts_check` 섹션에 module_capability 미포함, 최상위 `module_capability` 별도 섹션으로 유지.

**변경 파일**: `bootstrap/compiler.bmb`

## Verification & Defect Resolution
- Stage 1 bootstrap 빌드: ✅ (25.6s compile, 12.2s link)
- cargo test --release: ✅ 6282 PASS, 0 FAILED
- contracts-check 출력: `{"type":"contracts_check","file":"...","status":"safe","violations_count":0,"module_capability":{"type":"module_capability","status":"skipped","total_violations":0}}` ✅
- diagnose top-level keys: `['effect_verify', 'contracts_check', 'module_capability', ...]` — contracts_check 내부에 module_capability 미중복 ✅
- Within-gen Fixed Point: fp3331a.ll == fp3331b.ll ✅
- lint: 180 warnings (pre-existing baseline) — 0 missing_postcondition / chained_comparison / unused_binding ✅

**결함 발견 및 해결**: 첫 구현이 cc_build_json 7-param → `[params] cc_build_json: 7 parameters (max 6)` lint 경고 발생 → sb_push_char 기반 prefix copy 헬퍼로 우회 → 경고 해소.

## Reflection
- **Scope fit**: P3 태스크 완결. contracts-check가 이제 module_capability를 단일 JSON에 포함.
- **Latent defects**: 없음. diagnose 중복 없음 확인.
- **Structural improvements**: 없음 (cc_json_prefix_sb는 최소 필요 헬퍼).
- **Philosophy drift**: Rule 8(AI-friendly default output) 준수 — contracts-check가 더 완전한 진단 정보 제공.
- **Roadmap impact**: HANDOFF P3 항목 완료. L1 tuple ABI (Human Decision 대기) 변동 없음.
- **User-facing quality**: contracts-check JSON 스키마 확장 (기존 4 필드 → 5 필드). AI 파싱 관점에서 additive — backwards compatible.

## Carry-Forward
- Actionable: L1 stack-allocated tuple ABI (csv 1.039× 근본 해결) — Human Decision 대기
- Structural Improvement Proposals: 없음
- Pending Human Decisions: csv 1.039× 측정 노이즈 허용 여부 (결정 시 L1 ABI 착수)
- Roadmap Revisions: HANDOFF P3 항목 완료로 mibi/결함 테이블 업데이트
- Next Recommendation: L1 스택 할당 tuple ABI Phase 1 (파서 + IR lowering 설계), 또는 다른 P2 항목
