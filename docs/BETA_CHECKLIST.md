# v1.0.0-beta Release Checklist

> BMB 언어의 첫 번째 베타 릴리스를 위한 필수 검증 체크리스트

---

## Exit Criteria Summary (2026-01-14 업데이트)

| Category | Requirement | Status | Notes |
|----------|-------------|--------|-------|
| **Language** | stdlib API 확정 | ✅ | 10 모듈, 41 postcondition 경고 (P1) |
| **Compiler** | 자체 컴파일 (3-Stage) | ⏳ | **Stage 1만 검증, Stage 2/3 WSL 필요** |
| **Performance** | Gate #3.1 통과 | ⚠️ | 단일 벤치마크만 검증, 전체 스위트 미실행 |
| **Ecosystem** | 14+ 패키지 | ✅ | 14개 완료 |
| **Documentation** | 샘플 앱 5개 | ✅ | 5개 완료, 언어 제한 워크어라운드 |
| **Security** | 보안 감사 | 🔄 | Phase 1-2 완료, Phase 3 예정 |
| **Cross-compile** | 멀티 플랫폼 | ❌ | 설계 문서만, 구현 0% |

### 🔴 블로커 (v1.0.0-beta 전 필수)

1. **3-Stage Bootstrap 검증**: WSL Ubuntu에서 Stage 2/3 실행 필요
2. **전체 벤치마크 Gate**: `benchmark-bmb gate all` 실행 필요

---

## 1. Language & Compiler

### 1.1 Standard Library API (v0.45)

- [x] `core/` - 기본 타입 (i64, bool, String)
- [x] `string/` - 문자열 조작
- [x] `array/` - 배열 연산
- [x] `io/` - 파일 I/O
- [x] `process/` - 프로세스 제어
- [x] `test/` - 테스트 프레임워크
- [x] API 안정성 문서 (`docs/API_STABILITY.md`)

**검증 명령:**
```bash
bmb test stdlib/**/*.bmb
```

### 1.2 Error Messages (v0.45)

- [x] ariadne 기반 에러 포맷팅
- [x] 소스 위치 표시
- [x] 컬러 출력 지원
- [ ] 제안 메시지 (선택적)

### 1.3 Developer Tools (v0.45)

- [x] LSP 서버 (`bmb lsp`)
  - [x] 자동완성
  - [x] 정의로 이동
  - [x] 진단 메시지
  - [x] 호버 정보
- [x] Linter (`bmb lint`)
  - [x] 기본 규칙
  - [x] `--strict` 모드
- [ ] Formatter (`bmb fmt`)
  - [x] 기본 포맷팅
  - [ ] 주석 보존 (진행 중)

**검증 명령:**
```bash
bmb lsp --test
bmb lint --strict stdlib/**/*.bmb
bmb fmt --check examples/**/*.bmb
```

---

## 2. Self-Compilation (v0.46)

### 2.1 LLVM Backend

- [x] LLVM 18+ 지원
- [x] 네이티브 바이너리 생성
- [x] 런타임 함수 33개 구현

**검증 명령:**
```bash
# WSL Ubuntu에서 실행
cargo build --release --features llvm
./target/release/bmb build bootstrap/compiler.bmb -o bmb-stage1
./bmb-stage1 --version
```

### 2.2 3-Stage Bootstrap

- [x] Stage 1: Rust BMB → 네이티브
- [ ] Stage 2: BMB Stage 1 → 네이티브
- [ ] Stage 3: BMB Stage 2 → 네이티브 (동일성 검증)

**검증 명령:**
```bash
# WSL Ubuntu에서 실행
./scripts/bootstrap_3stage.sh
# Stage 2 == Stage 3 바이너리 해시 일치 확인
```

### 2.3 CLI Argument Passing

- [x] `bmb run file.bmb arg1 arg2` 지원
- [x] `arg_count()`, `get_arg()` 빌트인

**검증 명령:**
```bash
bmb run examples/sample-apps/bmb-calc/main.bmb -- add 10 20
# Expected: 30
```

---

## 3. Performance (v0.47)

### 3.1 Benchmark Gates

| Gate | Requirement | Current | Status |
|------|-------------|---------|--------|
| #3.1 | Compute ≤1.10x C | 0.89x-0.99x | ✅ |
| #3.2 | All Benchmarks ≤1.05x C | ⏳ | 진행 중 |
| #3.3 | 3+ faster than C | 4개 | ✅ |
| #4.1 | Self-compile <60s | 0.56s | ✅ |

**검증 명령:**
```bash
# WSL Ubuntu에서 실행
cd ecosystem/benchmark-bmb
./runner/target/release/benchmark-bmb gate all -v
```

### 3.2 Performance Results

```
Benchmark         C        BMB      Ratio    Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
fibonacci(45)     1.65s    1.63s    0.99x    ✅
fibonacci(40)     177ms    150ms    0.85x    ✅
mandelbrot        42ms     39ms     0.93x    ✅
spectral_norm     44ms     39ms     0.89x    ✅
```

---

## 4. Ecosystem (v0.48)

### 4.1 Core Packages (14/14)

- [x] `bmb-args` - CLI 인자 파싱
- [x] `bmb-collections` - HashMap, VecDeque, Stack
- [x] `bmb-fmt` - 문자열 포매팅
- [x] `bmb-fs` - 파일시스템
- [x] `bmb-http` - HTTP 유틸리티
- [x] `bmb-json` - JSON 파싱
- [x] `bmb-log` - 로깅
- [x] `bmb-math` - 수학 함수
- [x] `bmb-rand` - 난수 생성
- [x] `bmb-regex` - 정규표현식
- [x] `bmb-semver` - 시맨틱 버저닝
- [x] `bmb-testing` - 테스팅 프레임워크
- [x] `bmb-time` - 시간 유틸리티
- [x] `bmb-toml` - TOML 파싱

**검증 명령:**
```bash
for pkg in ecosystem/gotgan-packages/packages/*/; do
  echo "Testing $pkg"
  bmb run "$pkg/src/lib.bmb"
done
```

### 4.2 Cross-Compilation

- [ ] `--target x86_64-linux`
- [ ] `--target x86_64-windows`
- [ ] `--target x86_64-macos`
- [ ] `--target aarch64-macos`
- [ ] `--target wasm32`

---

## 5. Documentation (v0.49)

### 5.1 Sample Applications (5/5)

- [x] `bmb-calc` - 계산기 CLI (340 LOC)
- [x] `bmb-grep` - 패턴 매칭 (350 LOC)
- [x] `bmb-json-tool` - JSON 처리 (480 LOC)
- [x] `bmb-httpd` - HTTP 프로세서 (367 LOC)
- [x] `bmb-compiler` - 미니 컴파일러 (465 LOC)

**검증 명령:**
```bash
bmb run examples/sample-apps/bmb-calc/main.bmb -- fib 10
bmb run examples/sample-apps/bmb-grep/main.bmb -- "fn" examples/hello.bmb
bmb run examples/sample-apps/bmb-json-tool/main.bmb -- type '{"a":1}'
bmb run examples/sample-apps/bmb-httpd/main.bmb -- GET /api/hello
bmb run examples/sample-apps/bmb-compiler/main.bmb -- "2 + 3 * 4"
```

### 5.2 Scenario Documentation (5/5)

- [x] `SYSTEMS.md` - 시스템 프로그래밍
- [x] `CONTRACTS.md` - 계약 기반 검증
- [x] `PERFORMANCE.md` - 성능 최적화
- [x] `FROM_RUST.md` - Rust 마이그레이션
- [x] `AI_NATIVE.md` - AI 코드 생성

### 5.3 Tutorials

- [x] `GETTING_STARTED.md` - 15분 시작 가이드
- [x] `BY_EXAMPLE.md` - 예제로 배우기
- [x] `CONTRACT_PROGRAMMING.md` - 계약 프로그래밍
- [x] `FROM_RUST.md` - Rust 개발자 가이드
- [ ] `ADVANCED_CONTRACTS.md` - 고급 계약 (예정)

---

## 6. Security (v0.50)

### 6.1 Compiler Security

- [ ] 입력 검증 (악의적 소스 코드)
- [ ] LLVM IR 생성 안전성
- [ ] 버퍼 오버플로우 방지

### 6.2 Contract Security

- [ ] `@trust` 남용 탐지
- [ ] 검증 우회 방지

### 6.3 Package Security

- [ ] 의존성 무결성 검증
- [ ] 서명 확인

---

## 7. Testing (v0.50)

### 7.1 Test Coverage

| Category | Tests | Status |
|----------|-------|--------|
| Rust 단위 테스트 | 386+ | ✅ |
| Rust 통합 테스트 | 9+ | ✅ |
| Bootstrap 테스트 | 1,580+ | ✅ |
| 전체 | 1,753+ | ✅ |

**검증 명령:**
```bash
cargo test
bmb run bootstrap/types.bmb  # 530+ tests
bmb run bootstrap/lexer.bmb  # 777...888 markers
```

### 7.2 AI Query System

- [x] `bmb index` - 인덱스 생성
- [x] `bmb q sym` - 심볼 검색
- [x] `bmb q fn` - 함수 조회
- [x] `bmb q type` - 타입 조회
- [x] `bmb q metrics` - 프로젝트 통계
- [ ] `bmb q deps` - 의존성 쿼리 (예정)
- [ ] `bmb q ctx` - AI 컨텍스트 (예정)

---

## 8. Release Preparation

### 8.1 Final Verification

```bash
# 1. 전체 테스트 실행
cargo test --all-features

# 2. 벤치마크 게이트 확인 (WSL)
./runner/target/release/benchmark-bmb gate all

# 3. 샘플 앱 빌드/실행
for app in examples/sample-apps/*/; do
  bmb run "$app/main.bmb" --help
done

# 4. 패키지 테스트
for pkg in ecosystem/gotgan-packages/packages/*/; do
  bmb run "$pkg/src/lib.bmb"
done
```

### 8.2 Release Artifacts

- [ ] GitHub Release 태그 (`v1.0.0-beta`)
- [ ] 릴리스 노트 (`CHANGELOG.md`)
- [ ] 바이너리 빌드 (Linux, Windows, macOS)
- [ ] 문서 사이트 업데이트

### 8.3 Post-Release

- [ ] 커뮤니티 공지
- [ ] 피드백 수집 채널 준비
- [ ] 버그 트래커 준비

---

## Quick Status Check

```bash
# 현재 상태 확인
echo "=== BMB v1.0.0-beta Readiness ==="
echo "Tests: $(cargo test 2>&1 | grep -E '^\d+ passed' || echo 'run cargo test')"
echo "Bootstrap: $(bmb run bootstrap/compiler.bmb 2>&1 | grep -c '999' || echo 'N/A') markers"
echo "Packages: $(ls -d ecosystem/gotgan-packages/packages/*/ 2>/dev/null | wc -l)/14"
echo "Sample Apps: $(ls -d examples/sample-apps/*/ 2>/dev/null | wc -l)/5"
echo "Scenarios: $(ls docs/scenarios/*.md 2>/dev/null | wc -l)/5"
```

---

## Version History

| Date | Change |
|------|--------|
| 2026-01-14 | Initial checklist created |
| 2026-01-14 | Critical review: status adjusted to reflect actual verification state |

