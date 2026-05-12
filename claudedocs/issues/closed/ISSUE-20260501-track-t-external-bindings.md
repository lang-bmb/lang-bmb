# ISSUE: Track T — External Bindings (Python ✅ → Node)

> **트랙**: T (External Bindings, orthogonal)
> **마일스톤**: M3 (현재 Python ✅, Node ❌) → M4 (C#, Java, C 추가)
> **현 상태**: Python 5종 ✅ (algo/compute/crypto/text/json), Node 0%
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 5

## 현 상태

### C ABI 인프라 — ✅

- `ecosystem/build_all.py` 5종 라이브러리 빌드 (`.dll`/`.so`/`.dylib`)
- `ecosystem/gen_headers.py` C header 자동생성
- 빌드 인프라 완성

### Python bindings — ✅ (M3 절반)

| 라이브러리 | 함수 수 | 디렉토리 |
|----------|--------|--------|
| bmb-algo | 55 | `ecosystem/bmb-algo/bindings/python/` |
| bmb-compute | 33 | `ecosystem/bmb-compute/bindings/python/` |
| bmb-crypto | 14 | `ecosystem/bmb-crypto/bindings/python/` |
| bmb-text | 23 | `ecosystem/bmb-text/bindings/python/` |
| bmb-json | 12 | `ecosystem/bmb-json/bindings/python/` |

- 테스트 + 벤치마크 + setup.py 완비
- TestPyPI 배포 인프라 (`Bindings CI`) — TestPyPI org secret 등록 (HUMAN-gated) 잔여

### Node bindings — ❌ 0%

- `ecosystem/`에 node binding 디렉토리 없음
- 후보 도구: N-API (node-addon-api, napi-rs), Bun FFI, Deno FFI

## 잔여 작업

1. **Phase 1 — Node binding 도구 선정**
   - 옵션 A: N-API (Node-API, ABI-stable across versions)
   - 옵션 B: Deno/Bun FFI (단순, 단일 런타임)
   - 옵션 C: WebAssembly (브라우저 호환 — M4 후보)
   - 권장: A (N-API), 사용자 베이스 가장 큼

2. **Phase 2 — 1개 라이브러리 PoC**
   - bmb-algo 또는 bmb-text (단순 함수 다수)
   - C ABI → N-API wrapper
   - npm 패키지 구조

3. **Phase 3 — 5개 라이브러리 모두**
   - bmb-{algo,compute,crypto,text,json} Node bindings
   - 테스트 (Node `node:test` 또는 vitest)
   - npm 배포 (org-scoped: `@bmb/algo` 등)

4. **Phase 4 — 통합 테스트 + 문서**
   - `ecosystem/test_all_bindings.py`의 Node 등가물
   - README + 사용 예시

## 완료 조건 (M3 정합)

- [ ] Node bindings 5종 (N-API 또는 등가)
- [ ] 테스트 통과 (Node-side)
- [ ] npm 패키지 게시 (TestNPM 또는 GitHub Packages)
- [ ] 사용 가이드 (Python/Node 동등 API 보장)

## 추정 사이클

5-7 cycles. Phase 1 = 결정 (1 cycle), Phase 2 PoC = 2 cycles, Phase 3 5종 = 2-3 cycles, Phase 4 = 1 cycle.

## M4 (장기)

- C# bindings (P/Invoke)
- Java bindings (JNI 또는 Project Panama)
- C 바인딩 (이미 C ABI ✅, header 정리)
- 최종: 5 라이브러리 × 4 언어 = 20개 binding suite
