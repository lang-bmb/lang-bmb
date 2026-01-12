# v0.46 Independence Phase - Session State (2026-01-12)

## 현재 진행 상황

### 완료된 작업 (46.1 ~ 46.3)

1. **WSL Ubuntu LLVM 백엔드 검증** ✅
   - LLVM 21 설정: `LLVM_SYS_211_PREFIX=/usr/lib/llvm-21`
   - 빌드 명령: `cargo build --release --features llvm`

2. **Golden Binary 생성** ✅
   - Bootstrap 컴파일러 (`compiler.bmb`) 네이티브 바이너리 컴파일 성공
   - 실행 결과: 777 → 888 → 999 (모든 테스트 통과)

3. **수정된 버그들** (커밋 `55b5953`):

   **a) PHI 노드 타입 등록** (`bmb/src/mir/lower.rs`)
   - If 표현식 (line 326-329): PHI 결과 타입을 `ctx.locals`에 등록
   - Match 표현식 (line 746-750): 동일한 수정

   **b) 메서드 호출 반환 타입** (`bmb/src/mir/lower.rs:852-860`)
   ```rust
   let ret_type = match method.as_str() {
       "len" | "byte_at" => MirType::I64,
       "slice" => MirType::String,
       _ => ctx.func_return_types.get(method).cloned().unwrap_or(MirType::I64),
   };
   ```

   **c) 런타임 함수 반환 타입** (`bmb/src/mir/lower.rs:462-472`)
   ```rust
   match func.as_str() {
       "int_to_string" | "read_file" | "slice" | "digit_char" => MirType::String,
       "byte_at" | "len" | "strlen" | "cstr_byte_at" => MirType::I64,
       "file_exists" | "cstr_eq" => MirType::Bool,
       _ => MirType::I64,
   }
   ```

   **d) constant_type 헬퍼** (`bmb/src/codegen/llvm.rs:808-820`)
   - PHI 할당 시 부작용 없는 타입 결정 함수 추가

## 다음 단계

### 46.4 Self-compile 검증 (3-Stage) - 대기
- Stage 1: Rust 컴파일러로 Bootstrap 빌드
- Stage 2: Stage 1로 Bootstrap 재빌드
- Stage 3: Stage 2로 Bootstrap 빌드 → Stage 2와 동일 출력 확인

### 46.5 BMB-only 빌드 문서화 - 대기
- Rust 의존성 없이 BMB만으로 빌드하는 방법

## 알려진 제한사항

- `bmb_unified_cli.bmb`: `arg_count` 런타임 함수 미구현
- `compiler.bmb`는 테스트 하네스 (실제 CLI 아님)

## 테스트 명령어

```bash
# WSL에서 빌드
wsl bash -c "source ~/.cargo/env; export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21; cd /mnt/d/data/lang-bmb && cargo build --release --features llvm"

# Bootstrap 컴파일러 빌드 및 실행
wsl bash -c "source ~/.cargo/env; export LLVM_SYS_211_PREFIX=/usr/lib/llvm-21; cd /mnt/d/data/lang-bmb && ./target/release/bmb build bootstrap/compiler.bmb -o bootstrap_compiler && ./bootstrap_compiler"
```

## Git 상태

- 브랜치: main (origin/main보다 19 커밋 앞섬)
- 최신 커밋: `55b5953` - v0.46: Fix PHI type inference for bootstrap compiler
