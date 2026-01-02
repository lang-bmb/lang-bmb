# BMB Ecosystem Setup Guide

각 서브모듈 레포지토리의 초기 구조 가이드.

---

## 서브모듈 상태

| Repository | Status | Description | Version |
|------------|--------|-------------|---------|
| [bmb-samples](https://github.com/lang-bmb/bmb-samples) | 🟢 OK | 예제 프로그램 및 튜토리얼 | v0.6+ |
| [gotgan](https://github.com/lang-bmb/gotgan) | 🟢 OK | BMB 패키지 매니저, Rust fallback 지원 | v0.8+ |
| [benchmark-bmb](https://github.com/lang-bmb/benchmark-bmb) | 🟢 OK | C/Rust/BMB 표준 벤치마크 | v0.9+ |
| [action-bmb](https://github.com/lang-bmb/action-bmb) | 🟢 OK | GitHub Actions 지원 | v0.7+ |
| [tree-sitter-bmb](https://github.com/lang-bmb/tree-sitter-bmb) | 🟢 OK | 에디터 구문 분석 | v0.9+ |
| [vscode-bmb](https://github.com/lang-bmb/vscode-bmb) | 🟢 OK | VS Code 확장 | v0.9+ |
| [playground](https://github.com/lang-bmb/playground) | 🟢 OK | 온라인 플레이그라운드 | v0.9+ |
| [lang-bmb-site](https://github.com/lang-bmb/lang-bmb-site) | 🟢 OK | 공식 웹사이트 (docs, download, blog) | v0.9+ |

---

## 1. bmb-samples

예제 프로그램 및 튜토리얼.

### 디렉토리 구조

```
bmb-samples/
├── README.md
├── basics/                 # 기본 문법 예제
│   ├── 01_hello.bmb
│   ├── 02_variables.bmb
│   ├── 03_functions.bmb
│   ├── 04_if_else.bmb
│   └── 05_loops.bmb
├── contracts/              # 계약 검증 예제
│   ├── 01_preconditions.bmb
│   ├── 02_postconditions.bmb
│   ├── 03_invariants.bmb
│   └── 04_refinement_types.bmb
├── data_structures/        # 자료구조 예제
│   ├── struct.bmb
│   ├── enum.bmb
│   └── arrays.bmb
├── algorithms/             # 알고리즘 구현
│   ├── sort/
│   ├── search/
│   └── math/
├── projects/               # 완전한 프로젝트 예제
│   ├── calculator/
│   ├── todo-cli/
│   └── json-parser/
└── tutorials/              # 단계별 튜토리얼
    ├── 01_getting_started.md
    ├── 02_contracts_intro.md
    └── 03_building_cli.md
```

### README.md 템플릿

```markdown
# BMB Samples

BMB 프로그래밍 언어 예제 모음.

## 시작하기

\`\`\`bash
# 예제 실행
bmb run basics/01_hello.bmb

# 계약 검증
bmb verify contracts/01_preconditions.bmb
\`\`\`

## 카테고리

- **basics/** - 기본 문법
- **contracts/** - 계약 검증
- **data_structures/** - 자료구조
- **algorithms/** - 알고리즘
- **projects/** - 완전한 프로젝트
- **tutorials/** - 단계별 가이드
```

---

## 2. gotgan (곳간)

BMB 패키지 매니저. Rust fallback 생태계 지원 및 Rust→BMB 마이그레이션 도구 제공.

### 주요 기능
- **패키지 관리**: new, build, run, test, publish
- **Rust Fallback**: Cargo/crates.io 의존성 사용 가능
- **마이그레이션 도구**: Rust crate를 BMB로 점진적 변환

### 디렉토리 구조

```
gotgan/
├── README.md
├── LICENSE
├── Cargo.toml              # Rust 구현 (v0.8)
├── src/
│   ├── main.rs             # CLI 진입점
│   ├── lib.rs
│   ├── cli/                # 명령어 처리
│   │   ├── mod.rs
│   │   ├── new.rs          # gotgan new
│   │   ├── build.rs        # gotgan build
│   │   ├── run.rs          # gotgan run
│   │   ├── test.rs         # gotgan test
│   │   └── publish.rs      # gotgan publish
│   ├── manifest/           # gotgan.toml 파싱
│   │   ├── mod.rs
│   │   └── package.rs
│   ├── resolver/           # 의존성 해결
│   │   ├── mod.rs
│   │   └── version.rs
│   ├── registry/           # 패키지 레지스트리
│   │   ├── mod.rs
│   │   └── client.rs
│   └── build/              # 빌드 시스템
│       ├── mod.rs
│       └── cache.rs
├── bmb-src/                # BMB 재작성 (v0.11+)
│   └── main.bmb
└── docs/
    ├── manifest.md         # gotgan.toml 스펙
    └── commands.md         # CLI 명령어 문서
```

### Cargo.toml 템플릿

```toml
[package]
name = "gotgan"
version = "0.1.0"
edition = "2024"
description = "BMB package manager (곳간)"
repository = "https://github.com/lang-bmb/gotgan"
license = "MIT"

[[bin]]
name = "gotgan"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
semver = "1"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

### README.md 템플릿

```markdown
# 곳간 (Gotgan)

BMB 패키지 매니저.

## 설치

\`\`\`bash
cargo install gotgan
\`\`\`

## 사용법

\`\`\`bash
gotgan new hello            # 새 프로젝트 생성
gotgan build                # 빌드
gotgan run                  # 실행
gotgan test                 # 테스트
gotgan verify               # 계약 검증
gotgan add json             # 의존성 추가
gotgan publish              # 패키지 배포
\`\`\`

## gotgan.toml

\`\`\`toml
[package]
name = "hello"
version = "0.1.0"
edition = "2025"

[dependencies]
json = "0.1"
\`\`\`
```

---

## 3. benchmark-bmb

표준 벤치마크 스위트. C/Rust/BMB 간 성능 비교.

### 목표
- **BMB >= C -O3** (모든 케이스)
- **BMB > C -O3** (계약 활용 케이스)

### 디렉토리 구조

```
benchmark-bmb/
├── README.md
├── benches/
│   ├── compute/            # n-body, mandelbrot, spectral-norm
│   ├── memory/             # binary-trees, k-nucleotide
│   ├── realworld/          # json-parse, http-throughput
│   └── contract/           # BMB 계약 최적화 벤치마크
├── runner/                 # 벤치마크 러너 (Rust)
├── results/                # 결과 저장
└── dashboard/              # 웹 대시보드
```

### 주요 벤치마크

| Category | Benchmarks |
|----------|------------|
| Compute | n-body, mandelbrot, fannkuch, spectral-norm |
| Memory | binary-trees, reverse-complement |
| Real-world | json-parse, regex-redux, http-throughput |
| Contract | bounds-check-elim, null-check-elim, purity-opt |

---

## 4. action-bmb

GitHub Actions 지원.

### 디렉토리 구조

```
action-bmb/
├── README.md
├── action.yml              # GitHub Action 정의
├── src/
│   └── main.sh             # Action 로직
└── examples/
    └── ci.yml              # 사용 예제
```

### action.yml 템플릿

```yaml
name: 'BMB Build & Verify'
description: 'Build, test, and verify BMB projects'
author: 'lang-bmb'

inputs:
  command:
    description: 'Command to run (build, test, verify, check)'
    required: true
    default: 'build'
  bmb-version:
    description: 'BMB compiler version'
    required: false
    default: 'latest'

runs:
  using: 'composite'
  steps:
    - name: Install BMB
      shell: bash
      run: |
        curl -sSf https://bmb-lang.org/install.sh | sh
        echo "$HOME/.bmb/bin" >> $GITHUB_PATH

    - name: Run BMB command
      shell: bash
      run: |
        bmb ${{ inputs.command }}
```

### 사용 예제

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: lang-bmb/action-bmb@v1
        with:
          command: build

  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: lang-bmb/action-bmb@v1
        with:
          command: verify
```

---

## 5. tree-sitter-bmb

에디터 구문 분석.

### 디렉토리 구조

```
tree-sitter-bmb/
├── README.md
├── package.json
├── grammar.js              # Tree-sitter 문법 정의
├── src/                    # 생성된 파서 (자동 생성)
│   ├── parser.c
│   └── ...
├── queries/
│   ├── highlights.scm      # 구문 하이라이팅
│   ├── folds.scm           # 코드 접기
│   ├── indents.scm         # 들여쓰기
│   └── locals.scm          # 로컬 변수
└── bindings/
    ├── node/               # Node.js 바인딩
    └── rust/               # Rust 바인딩
```

### grammar.js 골격

```javascript
module.exports = grammar({
  name: 'bmb',

  rules: {
    source_file: $ => repeat($._definition),

    _definition: $ => choice(
      $.function_definition,
      $.type_definition,
    ),

    function_definition: $ => seq(
      'fn',
      field('name', $.identifier),
      field('parameters', $.parameter_list),
      optional(seq('->', field('return_type', $.type))),
      optional(field('precondition', $.precondition)),
      optional(field('postcondition', $.postcondition)),
      '=',
      field('body', $.expression),
      ';'
    ),

    // ... 더 많은 규칙
  }
});
```

---

## 6. vscode-bmb

VS Code 확장.

### 디렉토리 구조

```
vscode-bmb/
├── README.md
├── package.json            # 확장 매니페스트
├── tsconfig.json
├── src/
│   ├── extension.ts        # 진입점
│   └── lsp-client.ts       # LSP 클라이언트
├── syntaxes/
│   └── bmb.tmLanguage.json # TextMate 문법
├── snippets/
│   └── bmb.json            # 코드 스니펫
└── language-configuration.json
```

### package.json 템플릿

```json
{
  "name": "vscode-bmb",
  "displayName": "BMB Language",
  "description": "BMB language support for VS Code",
  "version": "0.1.0",
  "publisher": "lang-bmb",
  "repository": "https://github.com/lang-bmb/vscode-bmb",
  "engines": {
    "vscode": "^1.85.0"
  },
  "categories": ["Programming Languages"],
  "activationEvents": ["onLanguage:bmb"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [{
      "id": "bmb",
      "aliases": ["BMB", "bmb"],
      "extensions": [".bmb"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "bmb",
      "scopeName": "source.bmb",
      "path": "./syntaxes/bmb.tmLanguage.json"
    }],
    "snippets": [{
      "language": "bmb",
      "path": "./snippets/bmb.json"
    }]
  }
}
```

### TextMate 문법 (bmb.tmLanguage.json) 핵심

```json
{
  "scopeName": "source.bmb",
  "patterns": [
    { "include": "#comments" },
    { "include": "#keywords" },
    { "include": "#strings" },
    { "include": "#numbers" },
    { "include": "#functions" }
  ],
  "repository": {
    "comments": {
      "match": "--.*$",
      "name": "comment.line.double-dash.bmb"
    },
    "keywords": {
      "match": "\\b(fn|let|var|if|then|else|pre|post|struct|enum|match|for|while)\\b",
      "name": "keyword.control.bmb"
    }
  }
}
```

---

## 7. playground

온라인 플레이그라운드.

### 디렉토리 구조

```
playground/
├── README.md
├── package.json
├── src/
│   ├── App.tsx             # React 앱
│   ├── Editor.tsx          # Monaco 에디터
│   ├── Output.tsx          # 실행 결과 표시
│   └── wasm/               # WASM 바인딩
│       └── bmb.wasm
├── public/
│   └── index.html
└── examples/               # 미리 로드된 예제
    ├── hello.bmb
    ├── fibonacci.bmb
    └── contract.bmb
```

### 주요 기능

1. **에디터**: Monaco Editor + BMB 문법 하이라이팅
2. **실행**: WASM 컴파일된 BMB 인터프리터
3. **검증**: 실시간 타입 체크 + 계약 검증 결과
4. **공유**: URL 공유 링크 생성
5. **예제**: 미리 정의된 예제 불러오기

---

## 8. lang-bmb-site

공식 웹사이트. 문서, 다운로드, 블로그.

### 디렉토리 구조

```
lang-bmb-site/
├── README.md
├── package.json
├── astro.config.mjs        # Astro 설정
├── src/
│   ├── pages/
│   │   ├── index.astro     # Landing page
│   │   ├── docs/           # Documentation
│   │   ├── download.astro  # Download page
│   │   ├── changes.astro   # Changelog
│   │   └── blog/           # Blog posts
│   ├── components/
│   │   ├── Header.astro
│   │   ├── Footer.astro
│   │   ├── CodeBlock.astro # BMB syntax highlighting
│   │   └── Playground.astro
│   ├── layouts/
│   └── styles/
├── public/
│   ├── favicon.ico
│   ├── logo.svg
│   └── downloads/          # Binary releases
└── content/
    ├── docs/               # Markdown documentation
    └── blog/               # Blog posts
```

### 기술 스택

- **Framework**: Astro (Static site generator)
- **Styling**: Tailwind CSS
- **Code Highlighting**: Shiki with BMB grammar
- **Search**: Pagefind
- **Hosting**: GitHub Pages / Cloudflare Pages

### 주요 페이지

| Page | Description |
|------|-------------|
| `/` | Landing page - 언어 소개, 핵심 기능 |
| `/docs` | Documentation - 레퍼런스, 튜토리얼 |
| `/download` | Download - 설치 가이드, 바이너리 |
| `/changes` | Changelog - 버전별 변경사항 |
| `/blog` | Blog - 개발 소식, 기술 블로그 |

---

## 초기화 명령

### bmb-samples 초기화

```bash
cd ecosystem/bmb-samples
git init
mkdir -p basics contracts data_structures algorithms projects tutorials
# README.md 생성
git add .
git commit -m "Initial structure"
git remote add origin https://github.com/lang-bmb/bmb-samples.git
git push -u origin main
```

### gotgan 초기화

```bash
cd ecosystem/gotgan
cargo init --name gotgan
# Cargo.toml, src/ 구조 설정
git add .
git commit -m "Initial Rust implementation"
git remote add origin https://github.com/lang-bmb/gotgan.git
git push -u origin main
```

---

## 버전 로드맵

| Version | bmb-samples | gotgan | action-bmb | tree-sitter | vscode | playground | site |
|---------|-------------|--------|------------|-------------|--------|------------|------|
| v0.6 | basics/ | - | - | - | - | - | - |
| v0.7 | contracts/ | - | v0.1 | - | - | - | - |
| v0.8 | data_structures/ | v0.1 | v0.2 | - | - | - | - |
| v0.9 | algorithms/ | v0.2 | v0.3 | v0.1 | v0.1 | v0.1 | v0.1 |
| v0.10 | projects/ | v0.3 | v0.4 | v0.2 | v0.2 | v0.2 | v0.2 |
| v1.0 | complete | v1.0 | v1.0 | v1.0 | v1.0 | v1.0 | v1.0 |
