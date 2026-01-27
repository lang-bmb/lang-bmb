# Benchmark Analysis Findings (v0.51)

**Date:** 2026-01-21
**Version:** v0.51
**Type:** Performance Analysis

---

## Summary

48개 벤치마크 전체 실행 결과:
- **77% (37/48)** 목표 달성 (≤1.10x C)
- **54% (26/48)** C 추월
- **11개** 목표 미달

---

## Critical Issues

### ISSUE-001: syscall_overhead 성능 (P0)

**현상**: BMB 220ms vs C 73ms = **3.0x 느림**

**Root Cause**: BmbString 래퍼 오버헤드
- C: `stat(".", &st)` - 직접 syscall
- BMB: `file_exists(BmbString*)` → `bmb_file_exists()` → `stat(path->data, &st)`

**권장**: 타입 안전성 비용으로 문서화 + 향후 문자열 리터럴 FFI 최적화 검토

---

### ISSUE-002: fannkuch 재귀 오버헤드 (P0)

**현상**: BMB 169ms vs C 79ms = **2.13x 느림**

**Root Cause**: 깊은 재귀 호출 스택 설정 비용

**권장**: v0.51 while 문법으로 벤치마크 재작성

---

### ISSUE-003: http_parse/json_serialize 문자열 연결 (P1)

**현상**: http_parse 1.67x, json_serialize 1.37x

**Root Cause**: `+` 연산자 문자열 연결 = O(n) 매번 할당

**권장**: StringBuilder 사용 가이드 문서화

---

### ISSUE-004: fibonacci Non-tail 재귀 (P2)

**현상**: BMB 24ms vs C 17ms = **1.44x**

**Root Cause**: 알고리즘적 한계 (fib(n-1) + fib(n-2) = NOT tail recursive)

**권장**: 문서화로 종료 (알고리즘 한계)

---

## Positive Findings

| Benchmark | Ratio | 분석 |
|-----------|-------|------|
| n_body | 0.20x | LLVM 자동 벡터화 탁월 |
| typecheck_bootstrap | 0.23x | BMB 패턴 매칭 최적 |
| sorting | 0.27x | BMB quicksort 구현 우수 |
| hash_table | 0.53x | BMB hashmap 구현 효율적 |

---

## Action Items

1. [ ] fannkuch while 루프로 재작성
2. [ ] syscall_overhead 타입 안전성 비용 문서화
3. [ ] StringBuilder 사용 가이드 작성
