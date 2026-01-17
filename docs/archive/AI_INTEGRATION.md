# BMB AI Integration Guide

**Version**: v0.20.0 (2026-01-04)
**Status**: Draft

## 1. Overview

BMB는 AI-Native 언어로 설계되었습니다. 이 문서는 AI 코드 생성 도구(Claude, Cursor, Copilot 등)가 BMB 프로젝트와 효과적으로 통합하는 방법을 설명합니다.

### 1.1 AI-Native 철학

| Human-Centric (기존) | AI-Centric (BMB) |
|----------------------|------------------|
| 가독성 우선 | 정확성/검증 가능성 우선 |
| 직관적 문법 | 명시적/형식적 문법 |
| 주석/문서 의존 | 계약이 곧 문서 |

### 1.2 주요 통합 포인트

1. **계약 정보**: pre/post 조건으로 함수 의도 명확화
2. **타입 정보**: 정제 타입으로 값 범위 명시
3. **증명 상태**: SMT 검증 결과로 정확성 확인

---

## 2. 현재 사용 가능한 도구

### 2.1 AST 출력 (JSON/S-expression)

```bash
# JSON 형식 AST
bmb parse file.bmb --format json

# S-expression 형식 AST
bmb parse file.bmb --format sexpr
```

**출력 예시:**
```json
{
  "items": [
    {
      "FnDef": {
        "name": "divide",
        "params": [...],
        "ret_ty": "i64",
        "pre": "b != 0",
        "post": "ret * b == a"
      }
    }
  ]
}
```

### 2.2 계약 검증

```bash
# SMT 검증 실행
bmb verify file.bmb

# 상세 출력
bmb verify file.bmb --verbose
```

### 2.3 MIR 출력

```bash
# MIR 텍스트 출력
bmb build file.bmb --emit-ir
```

---

## 3. 단기 개선 권장 사항

### 3.1 `--format json` 확장

**현재 상태**: `bmb parse`에서만 지원

**권장 확장**:

| 명령 | 개선 내용 |
|------|----------|
| `bmb check --format json` | 타입 검사 결과 JSON 출력 |
| `bmb verify --format json` | 검증 결과 JSON 출력 |
| `bmb build --format json` | 빌드 결과 JSON 출력 |

**예상 출력 형식:**
```json
{
  "command": "verify",
  "file": "src/math.bmb",
  "result": "success",
  "functions": [
    {
      "name": "divide",
      "contracts": [
        {"kind": "pre", "expr": "b != 0", "status": "verified", "time_ms": 12}
      ]
    }
  ]
}
```

### 3.2 계약 정보 접근성

**현재**: AST 파싱 필요

**권장**: 간편 쿼리 명령
```bash
# 함수 계약 조회 (단기 대안)
bmb info fn:divide --contracts
```

---

## 4. AI 도구별 통합 가이드

### 4.1 Claude Code / Cursor

**프로젝트 컨텍스트 설정:**
```markdown
# CLAUDE.md에 추가

## AI Integration

BMB 프로젝트입니다. 다음을 활용하세요:
- `bmb parse file.bmb --format json` - AST 분석
- `bmb verify file.bmb` - 계약 검증
- pre/post 조건은 함수 명세입니다
```

**효과적인 프롬프트:**
```
다음 BMB 함수의 계약을 분석하세요:
[bmb parse output]

pre 조건이 충분히 강한지 확인하세요.
```

### 4.2 MCP 서버 통합 (향후)

BMB Query System (v0.26 예정)이 구현되면:

```json
{
  "mcpServers": {
    "bmb": {
      "command": "bmb",
      "args": ["q", "serve", "--port", "9999"]
    }
  }
}
```

---

## 5. 계약 기반 코드 생성

### 5.1 계약에서 구현 유도

BMB 계약은 AI 코드 생성의 정확도를 높입니다:

```bmb
-- 이 계약을 만족하는 구현을 생성하세요
fn binary_search(arr: &[i64], target: i64) -> ?usize
  pre forall(i in 0..len(arr)-1): arr[i] <= arr[i+1]
  post match ret {
    Some(i) -> arr[i] == target,
    None -> forall(x in arr): x != target
  }
= ???;
```

### 5.2 반례 활용

검증 실패 시 반례를 코드 수정에 활용:

```
[Verification Failed]
Function: buggy_divide
Contract: pre (b != 0)
Counterexample: b = 0 at line 42
Suggestion: Add check before call
```

---

## 6. 향후 계획 (v0.26 Query System)

[RFC-0001](RFC/RFC-0001-AI-Query-System.md) 참조

### 6.1 쿼리 명령

```bash
bmb q fn binary_search      # 함수 상세 정보
bmb q deps fn:main          # 의존성 그래프
bmb q proof --summary       # 프로젝트 검증 요약
bmb q ctx fn:process        # AI 컨텍스트 (의존성 포함)
```

### 6.2 출력 형식

```bash
--format json     # 구조화된 JSON (기본)
--format compact  # 압축 텍스트
--format llm      # AI 프롬프트 최적화
```

---

## 7. 모범 사례

### 7.1 AI 도구 활용 시

1. **계약 우선**: 구현 전 pre/post 조건 명세
2. **검증 반복**: `bmb verify`로 생성 코드 검증
3. **반례 학습**: 실패 시 반례로 수정 방향 파악

### 7.2 피해야 할 패턴

- ❌ 계약 없이 구현 생성
- ❌ `@trust` 남용
- ❌ 검증 오류 무시

---

## Appendix: 관련 문서

- [CLAUDE.md](../CLAUDE.md) - 프로젝트 가이드
- [SPECIFICATION.md](SPECIFICATION.md) - 언어 명세
- [RFC-0001](RFC/RFC-0001-AI-Query-System.md) - AI Query System RFC
