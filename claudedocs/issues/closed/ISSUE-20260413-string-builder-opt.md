# ISSUE-20260413 — StringBuilder 성능 개선

**우선순위**: P1
**영역**: runtime, stdlib
**상태**: ✅ **CLOSE — Cycle 2724 진단 (false positive)**
**관련 벤치마크**: fasta (108%) — 측정일 2026-04-13 (stale 1년)

## Cycle 2724 진단 결과 (2026-05-11)

fasta 벤치마크는 **StringBuilder를 사용하지 않음**:
- BMB: `malloc(61)` + `store_u8` + `puts_cstr` (raw byte buffer)
- C: 동일 — `char line[LINE_WIDTH+1]` + `puts(line)`

108% 측정값의 원인은 StringBuilder와 무관. 가능한 후보:
- `select_iub_code` if-else 체인 14 비교 (`<` 비교, dense switch 불가)
- LCG 모듈로 연산 `(seed * IA + IC) % IM`
- `byte_at` overhead (print_repeat만)
- puts_cstr libc 호출 오버헤드

→ **본 ISSUE close**. fasta 측정 재실행 필요 (Cycle 2725 Tier 1 bulk re-measurement).

## (이하 원본 보존)


## 문제

문자열 빌딩 핫 경로에서 C 대비 8% 느림. 그러나 http_parse/json_serialize에서는 C를 1.65-1.77x 추월 — 즉 부분적 성공.

## 원인 분석 필요

- v0.51.22에서 global BmbString 최적화로 http_parse 개선됨
- fasta는 동적 문자열 생성이 많아 **heap 할당 오버헤드**가 병목 가능성
- StringBuilder의 재할당 정책 (2x growth vs golden ratio) 검토

## 해결 방안

1. Arena allocator 도입 — StringBuilder 수명이 짧으면 arena에서 일괄 해제
2. Small String Optimization (SSO) — 짧은 문자열은 스택 저장
3. 초기 capacity hint 기반 할당

## 완료 기준

- fasta ≤ 100% (vs Clang)
- 다른 문자열 벤치마크 회귀 없음 (http_parse 유지)
