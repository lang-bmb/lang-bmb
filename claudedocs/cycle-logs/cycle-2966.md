# Cycle 2966: &&/|| 문서화 + 86_heap_sort CRITICAL 경고 수정
Date: 2026-05-19

## Re-plan
Cycle 2965 Carry-Forward: Bootstrap `and`/`or` codegen 검증 + 추가 언어 갭.
검증 결과:
- Bootstrap: `and`/`or` short-circuit 이미 구현 완료 (lines 5679-5681)
- Interpreter: `and`/`or` short-circuit 이미 구현 완료 (lines 688-703)
- Bootstrap lexer: `&&`/`||` 토큰 미지원 (keywords `and`/`or`만) — 설계상 의도적
- Rust parser: `&&`/`||` v0.32부터 지원 (grammar.lalrpop lines 2156, 2175)
- `bool_operators` 진단: 이미 올바름 — `&&`/`||`는 OK, 단일 `|`/`&`만 경고

→ SCOPE ADJUST: Bootstrap 변경 불필요. 대신 (1) 잘못된 경고 제거, (2) 문서 정확화.

## Scope & Implementation

### 86_heap_sort problem.md CRITICAL 경고 제거
Cycle 2964에서 `&&`/`||` 미지원으로 추가한 CRITICAL 경고가 이제 잘못됨:
- Cycle 2965에서 MIR short-circuit 구현 완료 → `&&`/`||` 완전 지원
- 경고 제거: "CRITICAL: BMB does NOT support `&&` or `||` — use nested `if`"
- 대체: "`&&` and `||` work in BMB with short-circuit semantics"

### LANGUAGE_REFERENCE.md 논리 연산자 문서화
- Logical Operators 표: `and`/`&&`, `or`/`||` 별칭 + short-circuit 명시
- Operator Precedence 표 (§3.1): `and`/`&&`, `or`/`||` 업데이트
- Appendix B 우선순위 표: 동일 업데이트

## Verification & Defect Resolution

`cargo test --release`: 실행 중 (이전 사이클 통과 기록 있음, 코드 변경은 .md 파일만)

## Reflection

- `&&`/`||` 지원 상태: Rust parser(v0.32) + MIR(Cycle 2965) + interpreter(기존) = 완전 지원
- Bootstrap는 `and`/`or` keywords만 사용 — 언어 설계상 의도적 (BMB keyword syntax)
- B-axis 86_heap_sort: CRITICAL 경고 제거 → 모델이 `&&` 사용 가능 (short-circuit 보장)
- 86_heap_sort bubble sort 예시는 유지 (명확하고 간단함)

## Carry-Forward
- Actionable: 전체 테스트 통과 확인 후 커밋
- Structural Improvement Proposals: None
- Pending Human Decisions: GPUStack 재측정 (3개 문제 수정 + &&/|| short-circuit 반영)
- Roadmap Revisions: None
- Next Recommendation: 추가 언어 갭 또는 P축 성능 개선
