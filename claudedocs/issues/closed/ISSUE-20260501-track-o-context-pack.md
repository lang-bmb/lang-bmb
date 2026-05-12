# ISSUE: Track O — Context Pack (신규)

> **트랙**: O (Context Pack)
> **마일스톤**: M2 (AI-Ready Infrastructure)
> **현 상태**: 0% — 신규
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 4.2

## 목적

`bmb context-pack <project>` — AI 토큰 효율적 프로젝트 요약. LLM이 N-파일 BMB 프로젝트를 컨텍스트로 받을 때 압축된 형태로 제공.

## 설계 (1차안)

### CLI

```
bmb context-pack <project_dir> \
    [--depth N]              # 모듈 트리 깊이 (디폴트 3)
    [--include-contracts]    # contract 본문 포함 (디폴트 시그니처만)
    [--include-private]      # 비-public 항목 포함 (디폴트 false)
    [--max-tokens N]         # 토큰 예산 (디폴트 5000)
    [--format json|md]       # 출력 형식 (디폴트 json)
```

### 출력 형식 (1차안)

```json
{
  "_schema": "bmb.context-pack.v1",
  "project": {"name": "...", "version": "..."},
  "modules": [
    {
      "path": "src/foo.bmb",
      "exports": [
        {"kind": "fn", "name": "bar", "signature": "fn bar(x: i64) -> i64", "contract": {"pre": "x >= 0", "post": "ret > x"}},
        {"kind": "type", "name": "Baz", "definition": "type Baz = ..."}
      ]
    }
  ],
  "deps": [...],
  "stats": {"total_modules": N, "total_exports": M, "estimated_tokens": K}
}
```

## 작업 단계

1. **Phase 1 — 설계 합의**
   - Spec 정련 (출력 형식, 토큰 산정 방법)
   - 기존 도구 조사 (Rust `cargo doc`, Python `sphinx-apidoc`, Go `godoc` 등 reference)

2. **Phase 2 — 구현**
   - `bmb/src/main.rs`에 `ContextPack` 명령 추가
   - 또는 BMB로 작성 (Track S 정합 — `bmb context-pack` 자체가 BMB 도구)
   - public 항목 필터링 (resolver 사용)
   - contract 추출 (CIR 또는 AST 직접)

3. **Phase 3 — 검증**
   - 실제 LLM에 context-pack 결과 + 작업 prompt → 정답률 측정
   - 토큰 효율 (KB → tokens) 검증

## 완료 조건

- [ ] CLI 명령 구현 (`bmb context-pack`)
- [ ] JSON 스키마 v1 명시 (Track M 정합)
- [ ] 5+ 프로젝트 (BMB 자기 자신, stdlib, ecosystem libraries) 출력 시연
- [ ] LLM 정답률 측정 (R 트랙과 연계, optional)

## 추정 사이클

3-4 cycles. Phase 1 = 설계 (1 cycle), Phase 2 = 구현 (2 cycles), Phase 3 = 검증 (1 cycle, optional).
