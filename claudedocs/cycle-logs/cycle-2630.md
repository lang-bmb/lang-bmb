# Cycle 2630: C# 바인딩 scaffold + PyPI 상태 모니터링
Date: 2026-05-10

## Re-plan
Plan valid. PyPI Windows 빌드 실패 발견. Linux/macOS 계속 진행 중.
C# 바인딩 M4-6 scaffold 시작 (패턴: Python/Node.js 패턴 참조, P/Invoke 방식).

## Scope & Implementation

**M4-6: C# 바인딩 scaffold 완료**

파일 생성:
- `ecosystem/bmb-algo/bindings/csharp/BmbAlgo.cs` — 55개 P/Invoke 바인딩 + safe wrapper
- `ecosystem/bmb-algo/bindings/csharp/BmbAlgo.csproj` — net10.0 라이브러리 프로젝트
- `ecosystem/bmb-algo/bindings/csharp/BmbAlgoTests.cs` — 33개 smoke test
- `ecosystem/bmb-algo/bindings/csharp/BmbAlgoTests.csproj` — 테스트 실행파일
- `ecosystem/bmb-algo/bindings/csharp/README.md` — 사용법 문서

테스트 결과: **33/33 PASS** (DLL 복사 후 로컬 실행)

**PyPI Windows 빌드 실패**: "Install Python build tooling" 단계 실패 (exit 1).
원인: 로그 미공개 (run 진행 중). 가능한 원인:
1. `pip install --upgrade pip setuptools wheel build twine` 네트워크 오류
2. `windows-latest` → `windows-2025-vs2026` 리다이렉션 이슈

## Verification & Defect Resolution

C# 빌드: `dotnet build BmbAlgo.csproj` → Build succeeded ✅
테스트: 33/33 PASS ✅
PyPI Windows: ❌ 실패 (로그 확인 필요)

## Reflection

**C# 설계 품질**: P/Invoke + GCHandle pinning 패턴이 Python ctypes/Node.js koffi와 동일한 수준의 안전성 제공. `Safe<T>()` + `WithPinned<T>()` 헬퍼가 반복 코드를 제거.

**문제**: 테스트 프로젝트와 라이브러리가 같은 디렉토리에 있어 CS0436 경고 발생. `Compile Remove` 로 해결.

**DLL 배포 문제**: C#도 Python/Node.js와 동일하게 DLL을 별도로 제공해야 함. NuGet 패키지에 플랫폼별 native DLL 포함 방식(runtimes/<platform>/native/) 검토 필요 (M4 이후 과제).

**PyPI Windows 실패**: 재시도 또는 단일 플랫폼 publish 검토 필요.

## Carry-Forward
- Actionable: PyPI run 완료 후 Windows 실패 원인 분석 + 재시도 (Cycle 2631)
- Structural Improvement Proposals: 
  1. C# NuGet 패키지에 prebuilt native runtimes 포함 (`runtimes/win-x64/native/bmb_algo.dll` 등) — 사용자 경험 개선, CI 변경 필요
  2. 테스트 파일 별도 디렉토리 이동 (CS0436 원천 제거)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2631 — PyPI 실패 분석 + 재시도 + C# 바인딩 나머지 패키지 확장 계획
