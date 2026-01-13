# v0.46 Independence Phase - Session State

**Last Updated**: 2026-01-13
**Phase Status**: 진행중 (60% 완료)

---

## 현재 진행 상황

### 완료된 태스크

| ID | 태스크 | 완료일 | 상세 |
|----|--------|--------|------|
| 46.1 | LLVM 백엔드 검증 | 2026-01-12 | WSL Ubuntu, LLVM 21 |
| 46.2 | Golden Binary 생성 | 2026-01-12 | `bootstrap/compiler.bmb` 네이티브 컴파일 성공 |
| 46.7 | 빌드 문서화 | 2026-01-13 | `docs/BUILD_FROM_SOURCE.md` 작성 |

### 대기 중인 태스크

| ID | 태스크 | 블로커 | 다음 단계 |
|----|--------|--------|----------|
| 46.3 | 3-Stage 검증 | CLI 미구현 | `bmb_unified_cli.bmb` 완성 |
| 46.4 | Cargo.toml 제거 | 46.3 완료 필요 | 3-Stage 성공 후 진행 |
| 46.5 | DWARF 지원 | P1 우선순위 | 선택적 |
| 46.6 | 소스맵 | P1 우선순위 | 선택적 |

---

## v0.46 핵심 커밋

### 2026-01-12: PHI 타입 추론 수정 (`55b5953`)

**문제**: Bootstrap 컴파일러를 네이티브로 컴파일하면 SIGSEGV 발생

**원인** (4개 버그):
1. PHI 결과 타입이 `ctx.locals`에 등록되지 않음
2. 메서드 호출 (`slice()` 등) 반환 타입 미추적
3. 런타임 함수 반환 타입 테이블 불완전
4. `constant_type()` 헬퍼의 부작용 문제

**수정** (`bmb/src/mir/lower.rs`):
```rust
// If 표현식 PHI 타입 등록 (line 326-329)
let phi_var = ctx.fresh_var();
ctx.locals.insert(phi_var.clone(), result_type.clone());

// 메서드 호출 반환 타입 (line 852-860)
let ret_type = match method.as_str() {
    "len" | "byte_at" => MirType::I64,
    "slice" => MirType::String,
    _ => ctx.func_return_types.get(method).cloned().unwrap_or(MirType::I64),
};
```

### 2026-01-12: 문자열 연산 개선 (`d6dae1c`)

**추가된 기능**:
- `bmb_string_from_cstr`: C 문자열 → BmbString 래핑
- StringBuilder API: `sb_new`, `sb_push`, `sb_build`, `sb_clear`
- 포인터 산술 연산 (`Add`, `Sub`)

---

## 환경 설정

### WSL Ubuntu 빌드

```bash
# WSL 진입
wsl

# 환경 변수
export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21
export PATH="/usr/lib/llvm-21/bin:$PATH"

# 빌드
cd /mnt/d/data/lang-bmb
cargo build --release --features llvm

# Bootstrap 테스트
./target/release/bmb build bootstrap/compiler.bmb -o bootstrap_compiler
./bootstrap_compiler
# Expected: 777 → 385 → 888 → 8 → 393 → 999
```

### 검증 명령어

```bash
# 3-Stage Bootstrap (스크립트)
./scripts/bootstrap_3stage.sh

# 수동 검증
./target/release/bmb build bootstrap/compiler.bmb -o bmb-stage1
./bmb-stage1  # 테스트 실행 (777...999)
```

---

## 알려진 제한사항

1. **`compiler.bmb`는 테스트 하네스**
   - `build` CLI 명령 없음
   - 3-Stage 자체 컴파일에 사용 불가

2. **런타임 함수 미구현**
   - `arg_count()`: CLI 인자 개수
   - `get_arg(n)`: n번째 인자 가져오기

3. **Windows 네이티브 빌드 불가**
   - LLVM 미지원
   - WSL Ubuntu 사용 필수

---

## 다음 단계

### 단기 (v0.46 완료)

1. **`bmb_unified_cli.bmb` 완성**
   - `arg_count`, `get_arg` 런타임 함수 구현
   - `build` 서브커맨드 추가

2. **3-Stage Bootstrap 완료**
   - `scripts/bootstrap_3stage.sh` 실행
   - Stage 2 == Stage 3 바이너리 동일성 검증

3. **Cargo.toml 제거**
   - BMB-only 빌드 체인 확립

### 중기 (v0.47 준비)

1. **성능 Gate 검증**
   - WSL에서 벤치마크 실행
   - Gate #3.1 통과 확인

---

## Git 상태

- **브랜치**: main
- **최신 커밋**: `25109bb` - Update submodule references
- **v0.46 관련 커밋**:
  - `55b5953` - Fix PHI type inference
  - `d6dae1c` - LLVM codegen string improvements
  - `4e65560` - LLVM codegen string improvements (initial)

---

## 문서 현황

| 문서 | 상태 | 위치 |
|------|------|------|
| BUILD_FROM_SOURCE.md | ✅ 완료 | `docs/BUILD_FROM_SOURCE.md` |
| ROADMAP.md | ✅ 최신화 | `docs/ROADMAP.md` |
| bootstrap_3stage.sh | ✅ 완료 | `scripts/bootstrap_3stage.sh` |

---

## 참고 자료

- [Bootstrapping (compilers) - Wikipedia](https://en.wikipedia.org/wiki/Bootstrapping_(compilers))
- [Ken Thompson - Reflections on Trusting Trust](https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf)
- [Reproducible Builds](https://reproducible-builds.org/)
