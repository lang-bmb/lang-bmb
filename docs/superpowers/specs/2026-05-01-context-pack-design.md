# Context Pack Design — `bmb context-pack` (Track O Phase 1)
Date: 2026-05-01 (Cycle 2517)
Anchor: `docs/superpowers/specs/2026-05-01-vision-v1.0-realignment.md` § 4.2 Track O
Issue: `claudedocs/issues/ISSUE-20260501-track-o-context-pack.md`

> **목적**: AI agent가 BMB 프로젝트에 작업 시 토큰 효율적인 프로젝트 요약 제공. 모듈 트리 + 공개 인터페이스 + 주요 contract를 압축된 JSON으로 노출.

---

## 1. 사용 시나리오

### 시나리오 A — LLM이 새 모듈 추가

```
사용자: "프로젝트에 IR 변환 단계 추가해줘"
LLM: bmb context-pack . > context.json
LLM (context.json 컨텍스트 + 사용자 요청 → BMB 코드 생성)
LLM: bmb check new_module.bmb (컴파일러 피드백)
```

### 시나리오 B — Code review

```
LLM: bmb context-pack . --include-contracts | jq '.modules[] | select(.path == "src/foo.bmb")'
```

### 시나리오 C — 의존 분석

```
LLM: bmb context-pack . --depth 5 (full dependency tree)
```

---

## 2. CLI 설계

```
bmb context-pack <project_dir> \
    [--depth N]              # 모듈 트리 깊이 (디폴트 3)
    [--include-contracts]    # contract 본문 포함 (디폴트 시그니처만)
    [--include-private]      # 비-public 항목 포함 (디폴트 false)
    [--max-tokens N]         # 토큰 예산 (디폴트 5000)
    [--format json|md]       # 출력 형식 (디폴트 json, Track M 정합)
    [--include-tests]        # 테스트 파일 포함 (디폴트 false)
    [--exclude PATTERN]      # 제외 glob 패턴 (반복 가능)
```

### 인자 vs 환경변수

```
BMB_CONTEXT_DEPTH=3
BMB_CONTEXT_MAX_TOKENS=5000
```

CLI 인자 우선.

---

## 3. 출력 JSON 스키마 (v1)

```json
{
  "_schema": "bmb.context-pack.v1",
  "project": {
    "name": "string",
    "version": "string (선택)",
    "root": "string (절대 경로)"
  },
  "modules": [
    {
      "path": "src/foo.bmb",
      "kind": "source|stdlib|external",
      "exports": [
        {
          "kind": "fn",
          "name": "bar",
          "signature": "fn bar(x: i64) -> i64",
          "contract": {
            "pre": "x >= 0",
            "post": "ret > x"
          },
          "doc": "string (선택)"
        },
        {
          "kind": "type",
          "name": "Baz",
          "definition": "type Baz = struct { ... }"
        },
        {
          "kind": "const",
          "name": "MAX",
          "type": "i64",
          "value": "1024"
        }
      ],
      "uses": ["core::num", "core::string"],
      "lines": 250
    }
  ],
  "deps": [
    {
      "name": "stdlib/core/num",
      "kind": "stdlib",
      "exports_referenced": ["clamp", "abs"]
    }
  ],
  "stats": {
    "total_modules": 12,
    "total_exports": 45,
    "estimated_tokens": 4500
  }
}
```

### 필드 상세

#### `modules[].kind`
- `source` — 프로젝트 자체 모듈
- `stdlib` — `stdlib/...` 의존
- `external` — `gotgan` 패키지 의존

#### `exports[].kind`
- `fn` — 함수
- `type` — 타입 (struct, type alias)
- `const` — 상수
- `extern` — 외부 함수 선언
- `trait` — 트레이트 (BMB 미래 기능)

#### `contract` (`fn` exports)
- `pre`, `post` — 평문 문자열 (BMB syntax 그대로)
- `--include-contracts` 없으면 생략

#### `--max-tokens` 적용 우선순위

토큰 예산 초과 시 다음 순서로 절단:
1. `--include-private=false` 강제 (private 제거)
2. `--include-contracts=false` 강제 (contract 제거)
3. `doc` 필드 제거
4. `kind="external"` 모듈 요약화
5. 최종 `_truncated: true` 플래그 추가

토큰 추정: `len(json) / 4` (대략 BPE 비율).

---

## 4. 구현 옵션

### 옵션 A — Rust 추가 (`bmb/src/main.rs` + `bmb/src/...`)

장점:
- 기존 lexer/parser/types 인프라 재사용
- 빠른 구현 (1-2 cycles)

단점:
- ❌ Rule 6 (Rust frozen) 위배
- Track S 진척과 역행 (BMB rewrite 정책)

### 옵션 B — BMB 신규 (`bootstrap/context_pack.bmb`)

장점:
- ✅ Rule 6 정합
- ✅ Track S 진척 (ecosystem BMB-rewrite)
- BMB 자체로 BMB 컴파일러 인프라 활용 (`bootstrap/parser.bmb`, `bootstrap/types.bmb` 재사용)

단점:
- 구현 복잡도 큼 (3-4 cycles)
- BMB로 JSON 시리얼라이즈 (수동)

### 옵션 C — Python 외부 도구 (`ecosystem/bmb-context-pack/`)

장점:
- 빠른 prototyping
- 기존 ecosystem 패키지 패턴 정합

단점:
- BMB AST 파싱을 외부에서 (parser 호환성 위험)
- BMB context-pack는 컴파일러 명령으로 통합되는 게 자연스러움

### 권장: **옵션 B (BMB 신규)**

근거:
- Rule 6 정합 (Rust frozen)
- Track S 진척 (M3 게이트의 90% 진척에 기여)
- 부트스트랩 인프라 활용 가능

### 구현 시퀀스 (옵션 B)

| Phase | 내용 | 추정 사이클 |
|-------|------|---------|
| Phase 1 (본 cycle) | 본 설계 문서 | 1 ✅ |
| Phase 2 | BMB로 모듈 트리 walker (`bootstrap/context_pack/walker.bmb`) | 1 |
| Phase 3 | export extractor (`bootstrap/context_pack/extractor.bmb`) — BMB AST 파싱 후 public 항목 추출 | 1-2 |
| Phase 4 | JSON serializer (`bootstrap/context_pack/json.bmb`) | 1 |
| Phase 5 | CLI integration (`bootstrap/bmb_cli.bmb`에 `context-pack` 추가) | 1 |
| Phase 6 | 토큰 예산 + 절단 로직 | 1 |
| Phase 7 | 검증 (LLM 정답률, R 트랙 연계) | 1 (optional) |

**총 5-7 cycles** (Phase 1 본 cycle 포함).

---

## 5. 토큰 효율 목표

| 프로젝트 크기 | 모듈 수 | 토큰 목표 |
|------------|--------|---------|
| Small (단일 파일) | 1 | < 500 tokens |
| Typical | 5-15 | < 5,000 tokens |
| Large (BMB 자체) | 50+ | < 15,000 tokens (압축 모드 시 < 8,000) |

비교 기준: 같은 프로젝트의 raw source 토큰의 ~10-20%.

---

## 6. 검증 계획 (Phase 7, R 트랙 연계)

LLM 정답률 측정:
- 50개 task suite (R 트랙) 중 "context 의존" 부분
- A: raw source 컨텍스트 — 베이스라인
- B: context-pack 컨텍스트 — 비교군
- C: context-pack + relevant subset — 최적화 군

목표: 정답률 |B - A| < 5% with B의 토큰 < 30% of A.

---

## 7. M2 게이트 정합

Track O 완료 조건:
- [ ] CLI 명령 (`bmb context-pack`) — Phase 5
- [ ] JSON 스키마 v1 (Track M 정합) — 본 문서 ✅
- [ ] 5+ 프로젝트 출력 시연 — Phase 5/6
- [ ] LLM 정답률 측정 — Phase 7 (R 트랙 연계, optional)

---

**다음 단계 (Cycle 2521+ 또는 별도 트랙)**: Phase 2 — `bootstrap/context_pack/walker.bmb` 구현 시작.
