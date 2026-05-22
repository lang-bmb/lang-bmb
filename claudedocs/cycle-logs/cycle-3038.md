# Cycle 3038: file_mtime 빌트인 + rebuild scripts BMB 포팅
Date: 2026-05-22

## Re-plan
Cycle 3037 carry-forward: scripts/ 핵심 스크립트 BMB 포팅. Plan valid.

## Scope & Implementation

### file_mtime() 빌트인 추가 (6개 파일)

| 파일 | 변경 내용 |
|------|----------|
| `bmb/runtime/bmb_runtime.c` | `bmb_file_mtime(const char*)` (stat) + `file_mtime(BmbString*)` wrapper + sys/stat.h include |
| `bmb/src/interp/eval.rs` | `builtin_file_mtime` 등록 + 구현 (fs::metadata + modified()) |
| `bmb/src/types/mod.rs` | `file_mtime: (String) -> i64` 타입 시그니처 등록 |
| `bmb/src/codegen/llvm_text.rs` | bmb_file_mtime/file_mtime 선언 + 반환 타입 매핑 + 이름 매핑 |
| `bmb/src/codegen/llvm.rs` | file_mtime inkwell 함수 등록 |

**API**: `file_mtime(path: String) -> i64` — Unix 초 단위 수정 시각, -1 = 오류

### scripts 포팅 (2개)

**`scripts/rebuild-runtime.bmb`** (BMB port of rebuild-runtime.sh):
- `is_stale(lib, src, evl, force)` — file_mtime으로 staleness 판단
- `pick_cc()` — exec_output("clang", "--version") 확인 → clang/gcc 선택
- `run_rebuild(...)` — exec_output(cc, "-c ... -o ...") + exec_output("ar", "rcs ...")
- `has_flag(args, flag)` + `check_flag(...)` — env var BMB_ARGS 파싱
- JSON 모드, --check-only, --force 지원
- **한계**: secondary copy sync (binary 파일 복사) 제외

**`scripts/rebuild-bootstrap-exe.bmb`** (BMB port of rebuild-bootstrap-exe.sh):
- `rust_bmb(root)` — .exe/.elf 위치 자동 탐지
- `do_rebuild(bmb, src, exe, json)` — exec_output(bmb, "build ... --fast-compile")
- staleness check, JSON 모드, --check-only, --force 지원
- stack size check 제외 (BMB에서 PE32+ binary parsing 불필요)

### 수정된 타입 오류
- `acc or 1` (i64 or i64) → `or`는 bool 연산자 → has_flag 방식으로 재설계
- `== false` redundant → 구조 재정렬

## Verification & Defect Resolution

**cargo test --release**: ✅ 3782 passed; 0 failed

**file_mtime 동작 테스트**:
```
file_mtime("bmb/runtime/bmb_runtime.c") → 1779421381 (Unix timestamp)
file_mtime("bmb/runtime/libbmb_runtime.a") → 1779350585
→ src newer than lib → "stale" 정확히 감지
```

**rebuild-runtime.bmb 실행**:
- 첫 실행: "libbmb_runtime.a is stale — rebuilding with clang ... OK: rebuilt in 1413 ms"
- 재실행: "libbmb_runtime.a is current (226166 bytes)" ✅

**rebuild-bootstrap-exe.bmb 실행**:
- "bootstrap/compiler.exe is current" ✅

**type check**: ✅ rebuild-runtime.bmb 12 warnings / rebuild-bootstrap-exe.bmb 8 warnings (모두 missing_postcondition)

## Reflection

- **Scope fit**: file_mtime 빌트인 + 2개 핵심 스크립트 포팅 완료.
- **Latent defects**: secondary copy sync 미지원 — binary 파일 복사는 추가 빌트인(copy_file) 필요. 허용 가능.
- **Structural**: BMB_ARGS env var을 통한 CLI 인수 전달 — bmb run이 argv를 노출하지 않는 현재의 workaround. 장기적으로 `argc/argv` 또는 `args()` 빌트인 추가 검토 가능.
- **Philosophy drift**: None.
- **Roadmap impact**: M6 scripts/ P1 진행중 (2/n 완료).

## Carry-Forward

- Actionable:
  - `scripts/check-version-sync.sh` → BMB 포팅 (간단한 파일 두 곳 버전 비교)
  - 추가 핵심 스크립트: `scripts/quick-check.sh` 일부 기능 (복잡한 bash 기능 포함)
  - `copy_file` 빌트인 추가 시 secondary copy sync 완성 가능
- Structural Improvement Proposals:
  - `args()` 빌트인 — `bmb run script.bmb arg1 arg2` → `args()` 반환 SvecHandle. BMB_ARGS env var workaround보다 자연스러운 argv 접근.
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3039 — check-version-sync.sh BMB 포팅 + args() 빌트인 구현
