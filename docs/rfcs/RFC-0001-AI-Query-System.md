# BMB AI Query System RFC

**RFC**: 0001
**Title**: AI-Native Code Query System
**Status**: Implemented (Phase 3)
**Created**: 2026-01-04
**Updated**: 2026-01-12

---

## 1. 개요

### 1.1 동기

BMB는 강력한 계약 시스템(pre/post, forall/exists, refinement types)을 내장하고 있어 컴파일 타임에 풍부한 의미 정보를 추출할 수 있다. 이 정보를 AI 코드 생성 도구가 효과적으로 활용할 수 있도록 전용 쿼리 시스템을 제공한다.

### 1.2 목표

```
1. 컴파일 부산물로 인덱스 자동 생성
2. AI 도구가 프로젝트를 이해하기 위한 쿼리 인터페이스
3. 계약/증명 상태 기반 코드 탐색
4. 인간 친화성 배제, AI 효율성 극대화
```

### 1.3 비목표

```
- IDE 통합 (별도 LSP로 처리)
- 인간 가독성 출력 형식
- 대화형 인터페이스
```

---

## 2. 현재 BMB 문법 기반 추출 가능 정보

### 2.1 함수 정의에서 추출

```bmb
fn binary_search(arr: &[i64], target: i64) -> ?usize
  pre forall(i in 0..len(arr)-1): arr[i] <= arr[i+1]
  post match ret {
    Some(i) -> arr[i] == target,
    None -> forall(x in arr): x != target
  }
= ...;
```

**추출 가능:**
| 항목 | 값 |
|------|-----|
| name | binary_search |
| params | [(arr, &[i64]), (target, i64)] |
| return_type | ?usize |
| preconditions | [forall(i in 0..len(arr)-1): arr[i] <= arr[i+1]] |
| postconditions | [match ret { ... }] |
| uses_old | false |
| quantifiers | [forall] |
| calls | [len] |
| proof_status | verified/failed/timeout |

### 2.2 타입 정의에서 추출

```bmb
type NonZero = i64
  where self != 0;

type Sorted<T: Ord> = [T]
  where forall(i in 0..len(self)-1): self[i] <= self[i+1];
```

**추출 가능:**
| 항목 | 값 |
|------|-----|
| name | NonZero, Sorted |
| base_type | i64, [T] |
| constraints | [self != 0], [forall...] |
| type_params | [], [T: Ord] |
| is_refinement | true |

### 2.3 계약 정의에서 추출

```bmb
contract Sortable<T: Ord> for fn(&mut [T]) -> () {
  post sorted(self)
  post perm(old(self), self)
}
```

**추출 가능:**
| 항목 | 값 |
|------|-----|
| name | Sortable |
| for_signature | fn(&mut [T]) -> () |
| postconditions | [sorted(self), perm(old(self), self)] |
| uses_old | true |

### 2.4 증명 상태에서 추출

```
[Verification Result]
fn: binary_search
  pre[0]: verified (12ms, 45 clauses)
  post[0]: verified (89ms, 234 clauses)
  termination: not required (non-recursive)
```

---

## 3. 인덱스 구조

### 3.1 디렉토리 구조

```
project/
├── bmb.toml
├── src/
│   └── *.bmb
└── .bmb/
    └── index/
        ├── manifest.json      # 인덱스 메타데이터
        ├── symbols.idx        # 심볼 테이블
        ├── contracts.idx      # 계약 인덱스
        ├── types.idx          # 타입 그래프
        ├── deps.idx           # 의존성 그래프
        ├── proofs.idx         # 증명 결과
        └── source.idx         # 소스 위치 매핑
```

### 3.2 manifest.json

```json
{
  "version": "1",
  "bmb_version": "0.2.0",
  "project": "my-project",
  "indexed_at": "2026-01-04T12:00:00Z",
  "files": 42,
  "functions": 312,
  "types": 67,
  "contracts": 489,
  "proof_coverage": 0.87
}
```

### 3.3 인덱스 생성 시점

```bash
bmb build          # 빌드 시 자동 생성
bmb check          # 타입 체크 시 자동 생성
bmb verify         # 검증 시 증명 결과 추가
bmb index          # 명시적 인덱스 갱신
bmb index --watch  # 파일 감시 모드
```

---

## 4. 쿼리 명령어

### 4.1 기본 구조

```bash
bmb q <query-type> [target] [options]
```

**공통 옵션:**
| 옵션 | 설명 |
|------|------|
| --format json | JSON 출력 (기본) |
| --format compact | 한 줄 압축 |
| --format llm | AI 프롬프트용 |
| --depth N | 재귀 깊이 |
| --limit N | 결과 수 제한 |

### 4.2 심볼 쿼리

```bash
bmb q sym <pattern>
bmb q sym sort              # 이름에 "sort" 포함
bmb q sym --kind fn         # 함수만
bmb q sym --kind type       # 타입만
bmb q sym --kind contract   # 계약만
bmb q sym --file src/math.bmb
bmb q sym --pub             # pub 심볼만
```

**응답:**
```json
{
  "query": "sort",
  "matches": [
    {
      "kind": "fn",
      "name": "quicksort",
      "file": "src/sort.bmb",
      "line": 15,
      "pub": true,
      "signature": "fn<T: Ord>(&mut [T]) -> ()"
    },
    {
      "kind": "type",
      "name": "Sorted",
      "file": "src/types.bmb",
      "line": 8,
      "pub": true
    },
    {
      "kind": "contract",
      "name": "Sortable",
      "file": "src/contracts.bmb",
      "line": 20,
      "pub": true
    }
  ]
}
```

### 4.3 함수 쿼리

```bash
bmb q fn <name>
bmb q fn binary_search
bmb q fn --has-pre           # pre 조건 있는 함수
bmb q fn --has-post          # post 조건 있는 함수
bmb q fn --uses-old          # old() 사용 함수
bmb q fn --uses-forall       # forall 사용 함수
bmb q fn --recursive         # 재귀 함수
bmb q fn --unverified        # 미검증 함수
bmb q fn --pure              # pure 함수
bmb q fn --trust             # @trust 함수
```

**응답:**
```json
{
  "name": "binary_search",
  "file": "src/search.bmb",
  "line": 15,
  "pub": true,
  "signature": {
    "params": [
      { "name": "arr", "type": "&[i64]" },
      { "name": "target", "type": "i64" }
    ],
    "return": "?usize"
  },
  "contracts": {
    "pre": [
      {
        "expr": "forall(i in 0..len(arr)-1): arr[i] <= arr[i+1]",
        "quantifiers": ["forall"],
        "calls": ["len"]
      }
    ],
    "post": [
      {
        "expr": "match ret { Some(i) -> arr[i] == target, None -> forall(x in arr): x != target }",
        "uses_ret": true,
        "quantifiers": ["forall"]
      }
    ]
  },
  "body": {
    "calls": ["len"],
    "recursive": false,
    "has_loop": true
  },
  "proof": {
    "status": "verified",
    "pre": [{ "index": 0, "status": "verified", "time_ms": 12 }],
    "post": [{ "index": 0, "status": "verified", "time_ms": 89 }]
  }
}
```

### 4.4 타입 쿼리

```bash
bmb q type <name>
bmb q type NonZero
bmb q type --refined          # 정제 타입만
bmb q type --has-constraint   # 제약 있는 타입
bmb q type --base i64         # 기반 타입이 i64
bmb q type --uses-forall      # forall 사용 타입
```

**응답:**
```json
{
  "name": "Sorted",
  "file": "src/types.bmb",
  "line": 15,
  "pub": true,
  "params": [
    { "name": "T", "constraint": "Ord" }
  ],
  "base": "[T]",
  "refinement": {
    "expr": "forall(i in 0..len(self)-1): self[i] <= self[i+1]",
    "quantifiers": ["forall"],
    "calls": ["len"]
  },
  "used_by": {
    "functions": ["binary_search", "merge"],
    "types": ["SortedVec"]
  }
}
```

### 4.5 계약 쿼리

```bash
bmb q contract <name>
bmb q contract Sortable
bmb q contract --uses-old     # old() 사용 계약
bmb q contract --for-sig "fn(&mut [T]) -> ()"
```

**응답:**
```json
{
  "name": "Sortable",
  "file": "src/contracts.bmb",
  "line": 20,
  "pub": true,
  "params": [
    { "name": "T", "constraint": "Ord" }
  ],
  "for_signature": "fn(&mut [T]) -> ()",
  "conditions": {
    "post": [
      { "expr": "sorted(self)", "calls": ["sorted"] },
      { "expr": "perm(old(self), self)", "uses_old": true, "calls": ["perm"] }
    ]
  },
  "satisfied_by": ["quicksort", "mergesort", "heapsort"]
}
```

### 4.6 의존성 쿼리

```bash
bmb q deps <target>
bmb q deps fn:quicksort
bmb q deps type:Sorted
bmb q deps --reverse fn:partition  # 역방향 (누가 호출하는지)
bmb q deps --transitive            # 전이적 의존성
bmb q deps --contract              # 계약 의존성만
```

**응답:**
```json
{
  "target": "fn:quicksort",
  "calls": [
    { "fn": "partition", "count": 1 },
    { "fn": "quicksort", "count": 2, "recursive": true }
  ],
  "called_by": [
    { "fn": "sort_wrapper", "file": "src/api.bmb", "line": 45 },
    { "fn": "benchmark", "file": "tests/bench.bmb", "line": 12 }
  ],
  "type_deps": ["Ordering"],
  "contract_deps": ["Sortable", "Ord"],
  "satisfies": ["Sortable<T>"]
}
```

### 4.7 증명 쿼리

```bash
bmb q proof <target>
bmb q proof fn:divide
bmb q proof --summary            # 프로젝트 전체 요약
bmb q proof --unverified         # 미검증만
bmb q proof --timeout            # 타임아웃만
bmb q proof --failed             # 실패만
bmb q proof --slow               # 느린 검증 (>1초)
```

**함수별 응답:**
```json
{
  "target": "fn:divide",
  "status": "verified",
  "contracts": [
    {
      "kind": "pre",
      "index": 0,
      "expr": "b != 0",
      "status": "verified",
      "time_ms": 5,
      "smt": { "clauses": 12, "decisions": 45 }
    },
    {
      "kind": "post",
      "index": 0,
      "expr": "ret * b == a",
      "status": "verified",
      "time_ms": 23,
      "smt": { "clauses": 34, "decisions": 156 }
    }
  ]
}
```

**요약 응답:**
```json
{
  "total_functions": 312,
  "with_contracts": 245,
  "proof_status": {
    "verified": 230,
    "failed": 5,
    "timeout": 3,
    "skipped": 7,
    "trust": 12
  },
  "coverage": 0.938,
  "total_time_ms": 45230,
  "avg_time_ms": 184
}
```

### 4.8 반례 쿼리

```bash
bmb q counterexample <target>
bmb q counterexample fn:buggy_func
bmb q counterexample --all       # 모든 실패 함수
```

**응답:**
```json
{
  "target": "fn:buggy_divide",
  "contract": {
    "kind": "pre",
    "index": 0,
    "expr": "b != 0"
  },
  "status": "violated",
  "counterexample": {
    "inputs": {
      "a": 10,
      "b": 0
    },
    "at": {
      "file": "src/main.bmb",
      "line": 42,
      "expr": "buggy_divide(x, y)"
    }
  },
  "suggestion": "Add check: if b == 0 then return Err(...)"
}
```

### 4.9 컨텍스트 쿼리 (AI 프롬프트용)

```bash
bmb q ctx <target>
bmb q ctx fn:process_order
bmb q ctx fn:process_order --depth 2   # 의존성 깊이
bmb q ctx fn:process_order --include-tests
bmb q ctx file:src/trading.bmb
```

**응답:**
```json
{
  "target": {
    "kind": "fn",
    "name": "process_order",
    "source": "fn process_order(order: Order) -> Result ! OrderError\n  pre valid_order(order)\n  post ...\n= { ... };",
    "file": "src/trading.bmb",
    "lines": [45, 78]
  },
  "dependencies": {
    "functions": [
      {
        "name": "validate_order",
        "source": "fn validate_order(...) -> ... = ...;",
        "contracts_summary": "pre: order.id > 0, post: ret => valid_order(order)"
      },
      {
        "name": "match_order",
        "source": "fn match_order(...) -> ... = ...;",
        "contracts_summary": "pre: valid_order(order), post: ..."
      }
    ],
    "types": [
      {
        "name": "Order",
        "source": "struct Order { id: u64, price: Decimal, qty: u32 }"
      },
      {
        "name": "OrderError",
        "source": "enum OrderError { InvalidId, InvalidPrice, ... }"
      }
    ],
    "contracts": [
      {
        "name": "valid_order",
        "source": "contract valid_order for Order { ... }"
      }
    ]
  },
  "dependents": [
    {
      "name": "handle_request",
      "file": "src/api.bmb",
      "line": 23,
      "call_site": "process_order(req.order)"
    }
  ],
  "proof_status": "verified",
  "related_tests": [
    {
      "name": "test_process_order",
      "file": "tests/trading.bmb",
      "line": 15
    }
  ]
}
```

### 4.10 시그니처 쿼리

```bash
bmb q sig <pattern>
bmb q sig "(&[i64]) -> i64"          # 정확한 시그니처
bmb q sig --accepts "&[i64]"         # 이 타입을 받는 함수
bmb q sig --returns "?usize"         # 이 타입을 반환하는 함수
bmb q sig --accepts-refined          # 정제 타입 파라미터 함수
```

**응답:**
```json
{
  "query": "--accepts &[i64]",
  "matches": [
    {
      "fn": "sum",
      "signature": "fn(arr: &[i64]) -> i64",
      "param_match": "arr"
    },
    {
      "fn": "binary_search",
      "signature": "fn(arr: &[i64], target: i64) -> ?usize",
      "param_match": "arr"
    },
    {
      "fn": "first",
      "signature": "fn(arr: &NonEmpty<[i64]>) -> i64",
      "param_match": "arr",
      "note": "accepts refined subtype"
    }
  ]
}
```

### 4.11 변경 영향 쿼리

```bash
bmb q impact <target> --change "<description>"
bmb q impact fn:calculate_fee --change "add param discount: f64"
bmb q impact type:Order --change "add field timestamp: u64"
bmb q impact contract:Sortable --change "add post stable(self)"
```

**응답:**
```json
{
  "target": "fn:calculate_fee",
  "change": "add param discount: f64",
  "impact": {
    "breaking": true,
    "direct_callers": [
      { "fn": "total_cost", "file": "src/billing.bmb", "line": 34 },
      { "fn": "invoice", "file": "src/invoice.bmb", "line": 56 }
    ],
    "transitive_callers": 12,
    "contract_effects": [
      {
        "fn": "total_cost",
        "contract": "post",
        "expr": "ret == base + calculate_fee(order)",
        "effect": "signature mismatch after change"
      }
    ],
    "files_affected": [
      "src/billing.bmb",
      "src/invoice.bmb",
      "tests/billing_test.bmb"
    ]
  }
}
```

### 4.12 메트릭스 쿼리

```bash
bmb q metrics
bmb q metrics --file src/trading.bmb
bmb q metrics --contracts
bmb q metrics --verification
```

**응답:**
```json
{
  "project": {
    "files": 42,
    "lines": 8540,
    "functions": 312,
    "types": 67,
    "contracts": 45
  },
  "contract_usage": {
    "functions_with_pre": 234,
    "functions_with_post": 198,
    "functions_with_both": 187,
    "uses_forall": 56,
    "uses_exists": 12,
    "uses_old": 34,
    "pure_functions": 145,
    "trust_functions": 8
  },
  "type_usage": {
    "refinement_types": 23,
    "contract_types": 12
  },
  "verification": {
    "coverage": 0.87,
    "verified": 298,
    "failed": 5,
    "timeout": 8,
    "avg_time_ms": 45,
    "total_time_s": 234
  }
}
```

---

## 5. 배치 쿼리

### 5.1 파이프라인

```bash
# 미검증 함수 중 호출 빈도 높은 것
bmb q fn --unverified --format json | bmb q deps --stdin --sort callers

# 특정 타입을 사용하는 함수들의 계약
bmb q type --users Price --format json | bmb q fn --stdin
```

### 5.2 배치 파일

```bash
bmb q batch queries.json
```

```json
{
  "queries": [
    { "type": "fn", "name": "main" },
    { "type": "deps", "target": "fn:main", "transitive": true },
    { "type": "proof", "target": "fn:main" }
  ]
}
```

**응답:**
```json
{
  "results": [
    { "query": 0, "result": { ... } },
    { "query": 1, "result": { ... } },
    { "query": 2, "result": { ... } }
  ]
}
```

---

## 6. 출력 형식

### 6.1 --format json (기본)

구조화된 JSON, 프로그래매틱 파싱용

### 6.2 --format compact

```bash
bmb q fn divide --format compact
```

```
fn:divide (src/math.bmb:15) [verified]
  sig: (a: i64, b: i64) -> i64
  pre: b != 0
  post: ret * b == a
```

### 6.3 --format llm

```bash
bmb q ctx fn:divide --format llm
```

```
=== FUNCTION: divide ===
FILE: src/math.bmb:15-18
SIGNATURE: (a: i64, b: i64) -> i64

PRECONDITIONS:
  - b != 0

POSTCONDITIONS:
  - ret * b == a

PROOF: verified (28ms)

SOURCE:
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  post ret * b == a
= a / b;

CALLERS: safe_divide, calculate_ratio
CALLS: (none, primitive operation)
```

---

## 7. 에러 응답

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Function 'nonexistent' not found",
    "suggestions": ["divide", "dividend"]
  }
}
```

| 코드 | 설명 |
|------|------|
| NOT_FOUND | 대상 없음 |
| INDEX_STALE | 인덱스 갱신 필요 |
| INVALID_QUERY | 쿼리 문법 오류 |
| NO_INDEX | 인덱스 없음 |

---

## 8. 구현 우선순위

### Phase 1 (v0.25) ✅ 완료

| 명령 | 설명 | 상태 |
|------|------|------|
| `bmb index` | 인덱스 생성 | ✅ 구현됨 |
| `bmb q sym` | 심볼 검색 | ✅ 구현됨 |
| `bmb q fn` | 함수 조회 | ✅ 구현됨 |
| `bmb q type` | 타입 조회 | ✅ 구현됨 |
| `bmb q metrics` | 메트릭스 | ✅ 구현됨 |
| `bmb q proof` | 증명 상태 | ⏳ v0.46 예정 |

### Phase 2 (v0.47) ✅ 완료

| 명령 | 설명 | 상태 |
|------|------|------|
| `bmb q contract` | 계약 조회 | ✅ 구현됨 |
| `bmb q deps` | 의존성 (`--reverse`, `--transitive`) | ✅ 구현됨 |
| `bmb q ctx` | 컨텍스트 (`--depth`, `--include-tests`) | ✅ 구현됨 |
| `bmb q counterexample` | 반례 | ⏳ v0.48 예정 |

### Phase 3 (v0.48-v0.49) ✅ 완료

| 명령 | 설명 | 상태 |
|------|------|------|
| `bmb q sig` | 시그니처 검색 (`--accepts`, `--returns`) | ✅ 구현됨 |
| `bmb q impact` | 영향 분석 (`--change`) | ✅ 구현됨 |
| `bmb q batch` | 배치 쿼리 (queries.json) | ✅ 구현됨 |
| `--format llm` | LLM 최적화 출력 형식 | ✅ 구현됨 |
| `bmb q serve` | HTTP 쿼리 서버 | ⏳ v0.50 계획 |

---

## 9. 향후 확장

### 9.1 쿼리 서버 모드

```bash
bmb q serve --port 9999
```

```bash
curl http://localhost:9999/q/fn/divide
```

### 9.2 실시간 인덱스

```bash
bmb index --watch --serve
```

파일 변경 시 자동 갱신 + HTTP API 제공

### 9.3 AI 피드백 루프

```bash
bmb q suggest --for fn:buggy_func
```

검증 실패 시 수정 제안 생성

---

## 10. 참고

### 10.1 관련 도구

- .serena: 프로젝트 컨텍스트 (정적)
- CLAUDE.md: 프로젝트 규칙 (수동)
- BMB Query: 계약 기반 동적 쿼리 (자동)

### 10.2 차별점

| 기존 도구 | BMB Query |
|----------|-----------|
| 수동 작성 | 자동 생성 |
| 텍스트 기반 | 구조화된 데이터 |
| 정적 | 증명 상태 반영 |
| 일반적 | 계약/타입 특화 |

---

## 11. 결론

BMB의 계약 시스템은 코드의 의미와 의도를 명시적으로 표현한다. 이 정보를 AI 도구가 효과적으로 활용할 수 있도록 전용 쿼리 시스템을 제공하여, AI 코드 생성의 정확성과 효율성을 극대화한다.

---

**Author**: BMB Team
**Reviewers**: TBD
**Implementation**: v0.2 target