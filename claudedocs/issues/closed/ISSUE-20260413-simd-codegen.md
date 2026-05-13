# ISSUE-20260413 — SIMD 1급 타입 코드젠

**우선순위**: P1
**영역**: codegen, mir, runtime
**상태**: Closed (Cycle 2801 검증 — 완료 기준 3/3 충족, 2026-05-13)
**전제 작업 완료**: Cycles 2215-2219 (Type system + lexer + grammar 스캐폴딩, 회귀 0)

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | n/a (구현 미완성 — feature spec) |
| `stale_after` | n/a |
| `measurement_source` | n/a |
| `observed_rate` | n/a (Phase 6 검증 시 측정 — 스칼라 대비 1.5x+ 기대) |
| `scope` | global codegen path (모든 vector 코드) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 |

## 배경

Cycles 2215-2219에서 `f64x4`, `i32x8` 등 SIMD 1급 타입을 lexer/parser/타입체커까지 인식하도록 추가 완료. 그러나 MIR/codegen 레이어에서 placeholder lowering (Vector → I64) 사용 중이라 실제 LLVM `<N x T>` IR은 미발생.

## 현재 한계 (Cycle 2219 기준)

| 동작 | 상태 |
|------|------|
| `let v: f64x4 = ...` 파라미터/반환 타입 | ✅ 통과 |
| `f64x4 + f64x4` 연산자 | ❌ 컴파일 안됨 (BinOp가 vector 인지 못함) |
| LLVM `<4 x double>` IR emit | ❌ I64로 lower됨 |
| `splat`, `load`, `store` 헬퍼 | ❌ 미구현 |
| Stage 2/3 부트스트랩 | 미검증 (현재 vector 사용 코드 없음) |

## 작업 항목

### Phase 1: MIR Vector 타입 (Cycle 2220)
- `bmb/src/mir/mod.rs` `MirType::Vector { elem: Box<MirType>, lanes: u32 }` variant 추가
- `is_vector()` helper
- `mir/lower.rs` placeholder 제거, 진짜 lowering 연결

### Phase 2: BinOp Vector 분기 (Cycle 2220-2221)
- `mir/lower.rs::ast_binop_to_mir`에서 `ty.is_vector()` 체크 → 동일 BinOp 사용 (FAdd/Add 등) — LLVM이 operand 타입으로 자동 결정
- 타입체커 `check_binary_op`에서 Vector 매칭

### Phase 3: LLVM 텍스트 codegen (Cycle 2221)
- `bmb/src/codegen/llvm_text.rs::mir_type_to_llvm`: `MirType::Vector { elem: F64, lanes: 4 }` → `"<4 x double>"` 매핑
- BinOp 방출 시 vector 타입 감지 → `fadd fast <4 x double>` 등
- Vector load/store: `load <4 x double>, ptr %p, align 32` (alignment 정확히)

### Phase 4: inkwell codegen (Cycle 2221, Rule 7)
- `bmb/src/codegen/llvm.rs`도 동일하게 — 두 백엔드 IR diff 검증

### Phase 5: stdlib `simd` 모듈 (Cycle 2222)
최소 헬퍼 5개:
- `splat(x: f64) -> f64x4`
- `load(p: *f64) -> f64x4`
- `store(p: *f64, v: f64x4)`
- `horizontal_sum(v: f64x4) -> f64`
- `dot(a: f64x4, b: f64x4) -> f64`

### Phase 6: 검증 (Cycles 2223-2225)
- `tests/bench/simd_dot_product.bmb` 작성
- `@bench`로 스칼라 vs SIMD 비교 — 2-4x 기대
- 부트스트랩 3-Stage Fixed Point 유지 확인

## 의사결정 필요 (인간 판단)

**f32 1급 타입 추가 여부**:
- SIMD에서 f32x4/8/16은 핵심 width (AVX2/AVX-512에서 자연 매핑)
- 현재 BMB는 f32 자체를 지원 안함
- 옵션 A: f32 primitive 추가 (또 모든 match arm 영향)
- 옵션 B: f64 only SIMD로 시작, f32는 후속 사이클
- **권장 (이번 옵션 권고)**: 옵션 B — 점진 추가, 첫 SIMD wave는 f64x{2,4,8} + i32/i64 정수만

## 완료 기준

1. `let a: f64x4 = simd::splat(1.0); let b = a + a;`이 LLVM IR `fadd fast <4 x double>` 방출
2. `bmb bench` 로 SIMD dot product가 스칼라 대비 1.5x+ 빠름 (CPU에 SIMD 있는 가정)
3. 부트스트랩 Fixed Point 유지

## 참고

- Cycle 2207 design doc — 영향 범위 매핑
- Cycles 2215-2219 — 스캐폴딩 구현
- `tests/bench/simd_smoke.bmb` — 기준 스모크 테스트
