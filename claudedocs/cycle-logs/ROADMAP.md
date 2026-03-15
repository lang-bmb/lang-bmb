# BMB Development Roadmap
Updated: 2026-03-16

---

## Completed (Cycles 1900-1928)

- **Codegen Fixes**: IfElseToSwitch non-Eq drop, void phi, norecurse indirect recursion
- **LSP Server**: 9 features in BMB (diagnostics/hover/completion/definition/documentSymbol/references/rename/formatting), 1160 LOC, jparse 직접 사용
- **Bootstrap IR**: `i8*`/`i64*` → `ptr` 완전 마이그레이션, noundef 2→1452, nonnull 0→40, 3-Stage Fixed Point 검증
- **Benchmark**: 3 WARN → LLVM-OK (IR 동등성 증명), 공정성 감사 완료
- **Dead Code**: LSP 미사용 함수 7개 제거, TRL workaround 6개 제거
- **Benchmark Statistics** (Cycle 1915): bench.sh `--stats` (95% CI, Mann-Whitney U), raw times in JSON, compare.py schema fix
- **FASTER Classification** (Cycle 1916): classify_faster.sh (METADATA/PIPELINE/MIXED 3-way)
- **Real-World Benchmarks** (Cycle 1917): bench.sh `--dir` flag, 7 real-world benchmarks verified
- **VS Code LSP Integration** (Cycle 1918-1919): bmb.lspServerPath, build-lsp.sh, TypeScript fix
- **stdlib math + collections** (Cycle 1920-1922): 2 new modules (170 + 180 LOC)
- **tree-sitter-bmb v0.3.0** (Cycle 1923-1924): 16 new features synced with BMB v0.96
- **gotgan Resolver** (Cycle 1925-1927): Registry dep resolution, topological build order

### 현재 게이지
```
Bootstrap   ████████████████████ 98%   Fixed Point ✅, i8*=0, inttoptr=7947
Self-Host   ████████████████████ 99%   CLI 41개, LSP 9개, Test Runner ✅
Benchmark   ████████████████████ 100%  0 FAIL, 0 WARN, 3 LLVM-OK, --stats ✅
Ecosystem   ████████████░░░░░░░░ 60%   VSCode LSP ✅, stdlib 12/12, tree-sitter ✅, gotgan resolver ✅
```

---

## Phase C: Bootstrap 코드젠 품질 (v0.98)

### C-1. Native Ptr 타입 시스템
- 현재: 모든 파라미터 i64 (타입 소거), inttoptr 7,947개
- 목표: 포인터 파라미터에 `ptr` 타입 + nonnull/noalias 적용
- 범위: lowering.bmb + llvm_ir.bmb + compiler.bmb (~6-8주)
- 위험: EXTREME HIGH — 3-Stage Fixed Point 깨질 가능성

### C-2. inttoptr 점진적 감소
- malloc/realloc 결과 → native ptr (이미 부분 구현: ptr_provenance)
- 함수 파라미터 → ptr 타입 추론
- 목표: inttoptr 7,947 → <1,000

---

## Phase D: Playground WASM

- WASM 빌드 셋업
- wasm-bindgen 인터페이스
- 프론트엔드 통합 + 배포

---

## Phase E: 에코시스템 성숙 (진행 중)

### 완료
- ✅ gotgan 의존성 해석 (로컬 레지스트리 + 토폴로지 정렬)
- ✅ tree-sitter-bmb v0.3.0 (16개 신규 기능)
- ✅ stdlib 확장: math, collections
- ✅ VS Code LSP 연결 (bmb.lspServerPath)

### 남은 작업
- stdlib 추가: net, fs, time
- gotgan: 실제 패키지 간 의존성 테스트
- 디버거 지원 (DWARF 정보 생성)
- Playground WASM 빌드 + 배포
- lang-bmb-site 문서화 완성

---

## 구조적 한계 (변경 불가)

| 항목 | 이유 |
|------|------|
| Z3 verify 자체호스팅 | 외부 SMT 솔버 의존 — IPC로만 연동 가능 |
| Rust 완전 퇴역 | CLI/에러의 clap/ariadne 대체는 이미 bootstrap에 구현, Rust는 회귀 방지 목적으로만 유지 |
| LLVM 한계 벤치마크 | insertion_sort/running_median/max_consecutive_ones — IR 동등, ISel 휴리스틱 차이 |
