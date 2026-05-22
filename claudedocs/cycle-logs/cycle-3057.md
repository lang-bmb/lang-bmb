# Cycle 3057: M6-P3 gotgan Porting 분석 — TOML 전략 확정
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3056): M6-P3 gotgan (Rust→BMB) 분석 시작.
계획 유효. 이번 사이클은 **분석 전용** — 코드 없음, 계획 문서만.

## Scope & Implementation

### gotgan 전체 규모
| 파일 | 라인 | 비고 |
|------|------|------|
| bmbx.rs | 1271 | Bundle/Explore/Compat (MVP 제외) |
| build.rs | 1199 | 핵심 — 의존성 해소 + bmb 호출 |
| registry.rs | 1039 | 원격 레지스트리 (MVP 제외) |
| resolver.rs | 421 | 의존성 resolution |
| cache.rs | 396 | 빌드 캐시 |
| config.rs | 228 | TOML 매니페스트 (serde) |
| project.rs | 214 | new/init |
| main.rs | 264 | CLI (clap) |
| lock.rs | 181 | Lock 파일 |
| error.rs | 28 | 에러 타입 |
| **합계** | **5,241** | |

### MVP 범위 결정 (6 commands)
- **포함**: new, init, build, check, clean, tree
- **제외(defer)**: bmbx/bundle/explore/compat(1271줄), publish, 원격 레지스트리(1039줄), git deps, workspace glob

MVP는 **로컬 path 의존성 + 빌드 위임** 기능에 집중.
실제 구현 대상 ≈ 2,000–2,500줄 Rust → BMB 약 800–1,000줄 추정 (표현성 차이).

### TOML 파싱 전략 결정: Option (a) — 최소 BMB TOML 파서

**검토한 옵션**:
- (a) 최소 BMB TOML 파서 (~200 LOC) ← **선택**
- (b) `exec_with_stdin`을 통한 toml2json 호출 — 외부 도구 의존성 부적절
- (c) gotgan.json으로 매니페스트 교체 — 기존 호환성 파괴

**선택 근거**:
```
gotgan.toml 실제 포맷 (극히 제한적):
  [package]
  name = "pkg-top"
  version = "0.1.0"
  description = "..."

  [dependencies]
  pkg-mid = { path = "../pkg-mid" }
  libfoo = "1.0.0"
```

파서가 처리해야 할 것:
1. 빈 줄 / `#` 주석 스킵
2. `[section]` → 현재 섹션 추적
3. `key = "value"` → quoted string 추출
4. `key = { path = "..." }` → inline table (MVP에선 path 값만 추출)

단 3가지 패턴. 파서 규모 ≈ 150–200 줄 BMB. 외부 의존성 0.

### 역량 갭 감사 (Capability Gap Audit)

| 갭 | 상태 | 해결 |
|----|------|------|
| TOML 파싱 | ❌ 없음 | Cycle 3058: 최소 BMB 파서 구현 |
| String→String HashMap | ⚠️ 부분 | 병렬 svec 방식 사용 (deps_names/deps_specs) |
| 재귀 디렉토리 탐색 | ❌ 없음 | BMB 재귀 함수 + `list_dir` + `is_dir` |
| 경로 결합(path_join) | ❌ 없음 | BMB 헬퍼 함수 (str concat + `/`) |
| file_exists | ✅ | 바로 사용 가능 |
| read_file / write_file | ✅ | 바로 사용 가능 |
| make_dir / is_dir | ✅ | 바로 사용 가능 |
| list_dir | ✅ (단일 레벨) | 개행 구분 파일명 목록 반환 |
| exec_output(cmd, args) | ✅ | 외부 명령 실행 (bmb 호출용) |
| current_dir / getcwd | ✅ | 작업 디렉토리 확인 |
| svec | ✅ | String 벡터 |

**str_hashmap 분석**: `str_hashmap_insert(handle, key: String, value: i64)` — 값이 i64로 고정.
의존성 데이터(path/version)는 **병렬 svec** 패턴으로 처리:
```bmb
// dep_names: svec (dependency 이름들)
// dep_specs: svec (version 문자열 or path 문자열)
// dep_is_path: vec (0=버전, 1=path dep)
```
최대 20개 의존성 → O(n) linear scan 충분.

### 발견된 잠재 결함

**CRITICAL: bootstrap compiler.bmb `exec_with_stdin` String-반환 누락**

`bmb/src/mir/lower.rs:1685`는 Cycle 3056에서 수정했으나,
`bootstrap/compiler.bmb`의 `get_fn_return_type` 함수(line 6872–6876)에서 동일 갭 존재:

```bmb
// 현재 (6876줄 근처):
else if fn_name == "@bmb_sb_build" or fn_name == "@bmb_read_file" or fn_name == "@bmb_getenv" or fn_name == "@get_arg" { "ptr" }

// 필요:
else if fn_name == "@exec_with_stdin" or fn_name == "@bmb_exec_with_stdin" { "ptr" }  // 추가 필요
```

고치지 않으면 `exec_with_stdin`을 호출해 String에 assign하는 BMB 코드를
bootstrap으로 컴파일 시 잘못된 IR 생성. gotgan.bmb MVP에서 exec_with_stdin은
사용하지 않지만(TOML option(a) 선택), bootstrap 정합성을 위해 Cycle 3058에 수정.

### gotgan.bmb MVP 아키텍처 스케치

```
gotgan.bmb
├── toml_parser.bmb    — 최소 TOML 파서 (~200 LOC)
├── manifest.bmb       — GotganPackage + deps 구조체
├── project.bmb        — new/init 명령
├── builder.bmb        — build/check 명령 (exec_output으로 bmb 호출)
├── cleaner.bmb        — clean 명령
├── tree.bmb           — tree 명령 (재귀 dep 출력)
└── main.bmb           — CLI arg 파싱 + dispatch
```

데이터 구조:
```bmb
struct GotganManifest {
  name: String,
  version: String,
  description: String,
  dep_names: i64,   // svec handle
  dep_specs: i64,   // svec handle (version or path)
  dep_is_path: i64, // vec handle (0/1)
}
```

### Cycle 계획 (3058–3063)

| Cycle | 내용 | 예상 규모 |
|-------|------|----------|
| 3058 | TOML 파서 + Manifest struct + new/init + bootstrap exec_with_stdin fix | ~350 LOC |
| 3059 | build + check 명령 (find_root, collect_files, exec_output) | ~300 LOC |
| 3060 | clean + tree 명령 + path_join 헬퍼 + 재귀 dir walk | ~250 LOC |
| 3061 | 통합 테스트 + fixture 기반 검증 | ~테스트 중심 |
| 3062 | CLI polish (help text, exit codes) + gotgan.bmb self-hosting 검증 | ~마무리 |
| 3063 | 버퍼/완료 사이클 | - |

## Verification & Defect Resolution

분석 사이클 — 코드 변경 없음. 결함 수정 없음.

## Reflection

- **Scope fit**: 100% — 분석 전용 사이클, 완전히 수행됨
- **핵심 결정**: TOML 파싱 Option(a) 선택 — gotgan.toml 포맷이 극히 제한적이어서 200줄 파서로 충분
- **Bootstrap 결함 발견**: Cycle 3056에서 lower.rs를 고쳤으나 compiler.bmb 동일 패턴 누락 확인 → 즉시 다음 사이클 carry-forward
- **역량 갭 실질적**: path_join, 재귀 walk 2종이 없으나 BMB 함수로 구현 가능한 수준
- **규모 추정 현실적**: Rust 5,241줄 → BMB MVP ≈ 1,100줄 (6 commands만)

## Carry-Forward
- Actionable: Cycle 3058 — TOML 파서 + Manifest + new/init + bootstrap exec_with_stdin fix
- Structural Improvement Proposals:
  - `path_join(dir, file) -> String` 내장 추가 제안 (현재 string concat 우회)
  - `walk_dir(path) -> svec` 내장 추가 제안 (현재 BMB 재귀 함수로 우회)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음 (ROADMAP.md의 P3 6-12사이클 예상 유지)
- Next Recommendation: Cycle 3058 — TOML 파서 구현 시작
