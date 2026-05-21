# Cycle 2632: PyPI 워크플로우 수정 + C# 테스트 파일 완성
Date: 2026-05-10

## Re-plan
Plan valid. PyPI Windows 빌드 실패 계속. 워크플로우 수정 + 나머지 테스트 파일 작성.

## Scope & Implementation

**PyPI Windows 수정**:
- 원인 추정: `windows-latest` → `windows-2025-vs2026` 리다이렉션으로 pip install 실패
- 수정: `.github/workflows/pypi-publish.yml` `windows-latest` → `windows-2022` 고정
  (Cycle 2632 주석 추가)
- macOS-13 build 아직 진행 중 (run #25628128772)

**C# 테스트 파일 4개 추가**:
- `bmb-compute/bindings/csharp/BmbComputeTests.cs` + `.csproj` — 17개 tests
- `bmb-json/bindings/csharp/BmbJsonTests.cs` + `.csproj` — 13개 tests
- `bmb-crypto/bindings/csharp/BmbCryptoTests.cs` + `.csproj` — 9개 tests
- `bmb-text/bindings/csharp/BmbTextTests.cs` + `.csproj` — 19개 tests (StrToUpper/StrTrim/StrReverse 등 실제 메서드명으로 수정)

빌드 결과: 4/4 Build succeeded ✅

## Verification & Defect Resolution

**발견한 결함**: BmbTextTests.cs가 잘못된 메서드명 사용 (ToUpper → StrToUpper, Trim → StrTrim 등)
→ BmbText.cs 실제 메서드명으로 수정 완료

전체 C# 바인딩 빌드 상태:
```
bmb-algo:    BmbAlgo.csproj ✅, BmbAlgoTests.csproj ✅ (33/33 테스트 실행 확인)
bmb-compute: BmbCompute.csproj ✅, BmbComputeTests.csproj ✅
bmb-json:    BmbJson.csproj ✅, BmbJsonTests.csproj ✅
bmb-crypto:  BmbCrypto.csproj ✅, BmbCryptoTests.csproj ✅
bmb-text:    BmbText.csproj ✅, BmbTextTests.csproj ✅
```

## Reflection

**PyPI 부분 성공**: Ubuntu ✅, macOS-latest ✅, macOS-13 진행 중, Windows ❌
→ 수정된 워크플로우 push + 재실행 필요. Windows wheels 없이는 publish job이 실행되지 않음 (needs: build-wheels).

**C# 바인딩 완성도**: scaffold 100% + 테스트 파일 100% 완성. 실제 DLL 실행 테스트는 bmb-algo만 완료(33/33). 나머지 4개는 DLL 빌드 후 실행 가능.

**Roadmap impact**: M4-6 ~90% 완성. 남은 것: PyPI publish + NuGet publish.

## Carry-Forward
- Actionable: PyPI 수정 push + 재실행 (현재 run 완료 후 실패 로그 확인 → windows-2022가 올바른 수정인지 확인 후 push)
- Actionable: M5-1 시작 (다음 5 cycles)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: PyPI Windows 재실행 트리거 (push 후)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2633 — PyPI run 완료 확인 + M5-1 분석 시작
