# BMB Binding Strategy — 언어를 알리는 성능 라이브러리

> **전략**: BMB의 증명된 성능 우위를 활용해 Python/JS/C# 개발자가 **일상적으로 사용하는 라이브러리**를 제공한다.
> 사용자는 "빠른 라이브러리"로 시작해서, "이게 BMB로 만든 거야?"로 언어를 발견한다.

---

## 1. 전략 개요

### 트로이 목마 전략

```
개발자가 겪는 고통         BMB가 제공하는 해결책         결과
─────────────────────    ─────────────────────────    ──────────────
Python DP가 느림     →   pip install bmb-algo      →  "BMB가 뭐지?"
Node.js JSON 파싱    →   npm install bmb-json      →  README에서 BMB 발견
C# 행렬 연산이 느림  →   NuGet bmb-math           →   벤치마크 보고 관심
```

### 왜 이 전략인가

1. **성능이 증명됨**: knapsack 6.8x, lcs 1.8x, floyd 1.4x (vs C, vs Rust 모두 추월)
2. **실용적 가치 선행**: "BMB를 배워라"가 아니라 "이 라이브러리가 빠르다"
3. **자연스러운 유입**: 라이브러리 README → 벤치마크 → "BMB로 작성됨" → 언어 탐색
4. **기존 생태계 존중**: 각 언어의 패키지 매니저, 관례, 문화를 따름

---

## 2. 선행 인프라 (Phase 0)

현재 BMB는 실행 파일(.exe)만 생성. 바인딩을 위해 필요한 인프라:

### 2-1. 공유 라이브러리 출력 (필수)

```
현재: BMB → .exe (실행 파일만)
목표: BMB → .dll/.so/.dylib (공유 라이브러리)
      BMB → .wasm (WebAssembly 모듈)
```

| 작업 | 상세 | 예상 규모 |
|------|------|----------|
| `OutputType::SharedLib` 추가 | build/mod.rs에 SharedLib 옵션 | 1-2일 |
| `@export` 어트리뷰트 | 함수를 외부에 노출 (private linkage 해제) | 1일 |
| C 헤더 자동 생성 | `bmb build --emit-header` → `.h` 파일 | 2-3일 |
| WASM 모듈 출력 | `bmb build --emit-wasm` → `.wasm` (이미 부분 구현) | 1-2일 |

### 2-2. ABI 안정화

```bmb
// 제안 문법
@export("C")
pub fn bmb_knapsack(weights: *i64, values: *i64, n: i64, capacity: i64) -> i64
  pre n > 0 and capacity >= 0
= knapsack_solve(weights, values, n, capacity);
```

- `@export("C")`: C ABI로 함수 노출, private linkage 해제
- 함수명 그대로 심볼 테이블에 등록 (mangling 없음)
- 계약은 컴파일 시 검증, 런타임에는 제로 오버헤드

---

## 3. 타겟 라이브러리 설계

### 성능 우위 매핑

```
BMB 증명된 강점              타겟 라이브러리           경쟁 대상
──────────────────────      ──────────────────      ──────────────
DP 알고리즘 (6.8x faster)  → bmb-algo              → scipy, ortools
그래프 알고리즘 (1.4x)      → bmb-graph             → networkx, petgraph
문자열 처리 (10-30%)        → bmb-text              → regex, ripgrep
JSON 파싱 (56% faster)     → bmb-json              → simdjson, serde_json
행렬/수학 (spectral 0.85x) → bmb-compute           → numpy (일부), nalgebra
계약 기반 안전성            → bmb-safe-collections  → (독자적 카테고리)
```

---

## 3-1. `bmb-algo` — 알고리즘 라이브러리 (최우선)

**핵심 가치**: "Python에서 C보다 빠른 DP/그래프 알고리즘을 pip install 한 줄로"

### 왜 이것이 첫 번째인가

| 근거 | 설명 |
|------|------|
| **극적인 성능 차이** | knapsack 6.8x, lcs 1.8x — "좀 더 빠른" 수준이 아님 |
| **Python 생태계 공백** | scipy는 범용, 순수 DP/그래프 고속 라이브러리 부재 |
| **교육적 가치** | CS 수업, 코딩 테스트에서 알고리즘 = 관심 높음 |
| **작은 API 표면** | 함수 10-20개로 임팩트 있는 패키지 가능 |

### API 설계

```python
# pip install bmb-algo

import bmb_algo

# DP
result = bmb_algo.knapsack(weights=[2,3,4,5], values=[3,4,5,6], capacity=8)
distance = bmb_algo.edit_distance("kitten", "sitting")
length = bmb_algo.lcs("ABCBDAB", "BDCAB")

# Graph
path = bmb_algo.dijkstra(edges=[(0,1,4),(0,2,1),(2,1,2)], source=0, n=3)
dist = bmb_algo.floyd_warshall(matrix=[[0,3,INF],[2,0,INF],[INF,7,0]])
components = bmb_algo.scc(edges=[(0,1),(1,2),(2,0),(2,3)], n=4)

# Sort (특수)
bmb_algo.radix_sort(arr)      # in-place, 10% faster
bmb_algo.counting_sort(arr)   # O(n+k), 10% faster
```

### 바인딩 구조

```
bmb-algo/
├── src/
│   ├── knapsack.bmb       ← BMB 핵심 알고리즘 (계약 포함)
│   ├── lcs.bmb
│   ├── floyd.bmb
│   ├── dijkstra.bmb
│   └── exports.bmb        ← @export("C") 래퍼
├── bindings/
│   ├── python/
│   │   ├── setup.py       ← cffi 또는 ctypes 바인딩
│   │   ├── bmb_algo.py    ← Python API
│   │   └── benchmark.py   ← 경쟁 라이브러리 비교
│   ├── node/
│   │   ├── package.json
│   │   ├── binding.js     ← node-ffi-napi
│   │   └── index.d.ts     ← TypeScript 타입
│   └── dotnet/
│       ├── BmbAlgo.csproj
│       └── BmbAlgo.cs     ← P/Invoke
├── bench/
│   └── compare.py         ← "bmb-algo vs scipy vs networkx"
└── README.md              ← 벤치마크 결과 + "Powered by BMB"
```

### README 전략 (발견 유도)

```markdown
# bmb-algo — Blazing Fast Algorithms

> 6.8x faster than C on knapsack. Yes, really.

## Benchmarks

| Algorithm | bmb-algo | scipy | networkx | speedup |
|-----------|----------|-------|----------|---------|
| knapsack  | 0.16s    | 1.12s | —        | **6.8x** |
| LCS       | 0.13s    | —     | —        | **1.8x** |
| Floyd     | 0.43s    | —     | 2.1s     | **4.9x** |

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb), a language where
compile-time contracts eliminate runtime overhead. The compiler *proves*
your algorithm is correct, then generates code faster than hand-tuned C.

→ [Learn more about BMB](https://bmb-lang.dev)
```

---

## 3-2. `bmb-json` — 고성능 JSON 파서 (두 번째)

**핵심 가치**: "Node.js/Python에서 simdjson급 JSON 파싱을 순수 라이브러리로"

### 왜 두 번째인가

- JSON은 **모든 개발자**가 매일 사용
- stdlib에 이미 제로카피 JSON 파서 구현됨 (stdlib/json/mod.bmb)
- 파서/직렬화는 BMB의 문자열 처리 강점이 직접 적용되는 영역

### API 설계

```python
# pip install bmb-json
import bmb_json

data = bmb_json.loads('{"name": "BMB", "version": 0.97}')
text = bmb_json.dumps({"key": "value", "array": [1, 2, 3]})

# Streaming (대용량)
for item in bmb_json.stream_array(open("huge.json")):
    process(item)
```

```javascript
// npm install bmb-json
const bmb = require('bmb-json');

const data = bmb.parse('{"hello": "world"}');  // WASM 기반
const text = bmb.stringify(data);
```

```csharp
// NuGet: BmbJson
using BmbJson;

var data = Json.Parse("{\"hello\": \"world\"}");
var text = Json.Stringify(data);
```

---

## 3-3. `bmb-compute` — 수치 계산 라이브러리 (세 번째)

**핵심 가치**: "numpy의 특정 연산을 추월하는 계약 기반 수치 계산"

### 타겟 연산

| 연산 | BMB 우위 | 이유 |
|------|---------|------|
| spectral_norm | 0.85x vs C | 순수 함수 → SIMD 자동 벡터화 |
| n_body | 0.77x vs C | @pure → 루프 최적화 |
| matrix_chain | 0.91x | DP 최적화 |
| 행렬 곱셈 | ~1.0x | 동등 (LLVM 백엔드 공유) |

```python
# pip install bmb-compute
import bmb_compute

# 특화 연산 (BMB가 numpy보다 빠른 영역)
result = bmb_compute.spectral_norm(n=5500)
bodies = bmb_compute.nbody_simulate(bodies, steps=50000000)

# 행렬 (numpy 대체가 아닌 보완)
optimal_order = bmb_compute.matrix_chain_order(dimensions=[10,20,30,40,30])
```

---

## 4. 언어별 바인딩 전략

### 4-1. Python (최우선 — 가장 큰 영향력)

| 항목 | 선택 | 이유 |
|------|------|------|
| **바인딩 기술** | cffi + ctypes | 의존성 최소, 설치 간편 |
| **배포** | PyPI (pip install) | 표준 채널 |
| **빌드** | manylinux wheel | 사전 컴파일된 .so 포함 |
| **대안** | pyo3 (Rust glue) | BMB→.o → Rust pyo3 래퍼 → Python |

```
사용자 경험:
$ pip install bmb-algo
$ python -c "import bmb_algo; print(bmb_algo.knapsack([2,3,4], [3,4,5], 7))"
12
```

### 4-2. JavaScript/Node.js (두 번째 — WASM 활용)

| 항목 | 선택 | 이유 |
|------|------|------|
| **바인딩 기술** | WASM (브라우저+Node) | 크로스 플랫폼, 설치 무관 |
| **배포** | npm | 표준 채널 |
| **Node native** | node-ffi-napi | .dll/.so 직접 로드 |
| **브라우저** | wasm-pack 스타일 | .wasm + JS glue |

```
장점:
- WASM 백엔드 이미 90% 구현
- 브라우저에서도 동작 → Playground와 시너지
- "npm install → import → 즉시 사용" 경험
```

### 4-3. C# / .NET (세 번째 — 엔터프라이즈)

| 항목 | 선택 | 이유 |
|------|------|------|
| **바인딩 기술** | P/Invoke + NativeAOT | .NET 표준 방식 |
| **배포** | NuGet | 표준 채널 |
| **래퍼** | Source Generator | 자동 P/Invoke 코드 생성 |

```csharp
// NuGet: BmbAlgo
using BmbAlgo;

// P/Invoke로 BMB 공유 라이브러리 호출
var result = Algorithms.Knapsack(weights, values, capacity);
```

---

## 5. 우선순위 로드맵

```
Phase 0: 인프라 (4-6주)
├── @export("C") 어트리뷰트 + SharedLib 출력
├── C 헤더 자동 생성 (bmb build --emit-header)
├── WASM 모듈 출력 안정화
└── 크로스 컴파일 (Linux x64 + macOS ARM64 + Windows x64)

Phase 1: bmb-algo Python (4-6주)
├── 핵심 알고리즘 10개 BMB 구현 + @export
├── cffi 바인딩 + PyPI 패키지
├── 벤치마크 스크립트 (vs scipy, vs pure Python)
├── README + 문서 + "Powered by BMB"
└── PyPI 퍼블리시

Phase 2: bmb-algo Node.js + bmb-json (4-6주)
├── WASM 컴파일 + npm 패키지
├── bmb-json: stdlib JSON 파서를 라이브러리로 패키징
├── 벤치마크 (vs JSON.parse, vs simdjson-js)
└── npm 퍼블리시

Phase 3: bmb-compute + C# (4-6주)
├── 수치 계산 함수 WASM + native
├── NuGet P/Invoke 패키지
├── 벤치마크 (vs System.Text.Json, vs MathNet)
└── 블로그 포스트: "How BMB beats C at knapsack"
```

---

## 6. 마케팅 연계

### 각 패키지가 BMB를 알리는 방법

| 접점 | 내용 |
|------|------|
| **README.md** | 벤치마크 테이블 + "Written in BMB" 배지 + 링크 |
| **PyPI 설명** | "Compiled from BMB, a language faster than C" |
| **npm 설명** | "Zero-overhead safety via compile-time contracts" |
| **벤치마크 페이지** | 인터랙티브 차트 (bmb-lang.dev/benchmarks) |
| **블로그 시리즈** | "Why BMB is 6.8x faster than C at knapsack" |
| **HN/Reddit 포스트** | 벤치마크 결과 → 라이브러리 소개 → 언어 소개 |

### 발견 퍼널

```
1. 개발자가 성능 문제 검색
   "fast knapsack python" / "fastest json parser node"
        ↓
2. bmb-algo / bmb-json 발견 (PyPI/npm 검색)
        ↓
3. 벤치마크 결과에 감탄 (6.8x faster!)
        ↓
4. README에서 "Powered by BMB" 클릭
        ↓
5. BMB 언어 페이지 방문 → Playground에서 체험
        ↓
6. 일부 개발자가 BMB 직접 사용 시작
```

---

## 7. 성공 지표

| 단계 | 지표 | 목표 |
|------|------|------|
| **Phase 1 완료** | PyPI 다운로드 | 1,000+/월 |
| **Phase 2 완료** | npm 다운로드 | 5,000+/월 |
| **인지도** | GitHub Stars (lang-bmb) | 500+ |
| **커뮤니티** | BMB 직접 사용자 | 50+ |
| **블로그 반응** | HN frontpage | 1회+ |

---

## 8. 리스크와 대응

| 리스크 | 확률 | 대응 |
|--------|------|------|
| SharedLib 구현 난이도 | 중 | LLVM 텍스트 백엔드로 .ll → clang -shared |
| 크로스 플랫폼 빌드 | 높 | GitHub Actions + manylinux Docker |
| 벤치마크 공정성 논란 | 중 | 모든 벤치마크 소스 공개 + 재현 스크립트 |
| 기존 라이브러리의 반격 | 낮 | BMB의 우위는 언어 수준 — 라이브러리 수준에서 복제 불가 |
| 사용자 관심 저조 | 중 | HN/Reddit 포스트 + 블로그 시리즈로 초기 트래픽 확보 |

---

## 9. 첫 번째 실행 가능 단계

**지금 바로 시작할 수 있는 것:**

1. `@export` 어트리뷰트 설계 + 구현 (codegen에서 private linkage 해제)
2. `OutputType::SharedLib` 추가 (clang -shared 파이프라인)
3. knapsack.bmb + lcs.bmb + floyd.bmb를 `@export("C")` 래퍼로 감싸기
4. Python cffi 바인딩 프로토타입 (ctypes로 .so 로드)
5. `pip install` 가능한 최소 패키지 만들기

**PoC 목표**: `pip install bmb-algo && python -c "import bmb_algo; print(bmb_algo.knapsack(...))"`
