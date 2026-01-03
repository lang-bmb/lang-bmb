# BMB Standard Library Packages

> v0.14 Foundation - Core 패키지 표준화

## 패키지 구조 표준

```
packages/
├── bmb-core/           # 핵심 타입 및 프리미티브
│   ├── Gotgan.toml
│   └── src/
│       └── lib.bmb
├── bmb-option/         # Option<T> 제네릭 타입
├── bmb-result/         # Result<T, E> 제네릭 타입
├── bmb-traits/         # 핵심 트레이트 정의
└── bmb-iter/           # Iterator 트레이트
```

## 패키지 명명 규칙

| 접두사 | 용도 | 예시 |
|--------|------|------|
| `bmb-` | 공식 표준 라이브러리 | `bmb-core`, `bmb-option` |
| `bmb-x-` | 실험적 패키지 | `bmb-x-async` |

## Gotgan.toml 표준

```toml
[package]
name = "bmb-core"
version = "0.14.0"
description = "BMB core types and primitives"
license = "MIT OR Apache-2.0"
authors = ["BMB Team"]

[dependencies]
# 의존성 목록

[contracts]
# 계약 검증 설정
verify = true
```

## 버전 체계

- `0.14.x`: Foundation phase (핵심 타입)
- `0.15.x`: Stream phase (컬렉션/IO)
- `0.16.x`: Connect phase (네트워크/직렬화)

## AI-Native 설계 원칙

1. **계약 우선**: 모든 함수에 `pre`/`post` 조건
2. **제네릭**: 단형 대신 다형 타입 사용
3. **@derive 지원**: Debug, Clone, PartialEq 자동 구현
4. **토큰 효율**: 간결하고 명확한 API
