# 클릭투트윅 (Click To Tweak) Modern 2026 - 종합 마스터 계획서
**문서 버전:** 2.0.0-MASTER  
**작성일자:** 2026-11-11  
**기반 기술:** REA (Reverse Engineer Anything) 역공학 분석 + Rust 2024 (x64) + Tauri v2 + React 19 + Tailwind CSS v4  

---

## 1. REA 역공학 정밀 조사 결과 보고서

### 1.1 레거시 설치 디렉터리 구성 및 바이너리 동일성 규명
`C:\Program Files (x86)\Click To Tweak [Basic]` 내의 실행 파일들을 분석한 결과, 클릭투트윅은 12개에 달하는 유틸리티 실행 파일이 외형상 분리되어 있으나 실질적으로는 **동일한 SHA-256 해시(`7a68dcbf38289797d025110e54d3d1e1f7053e1a8bbdd1e6e9b4ce3eb311adbb`, 212,992 바이트)를 갖는 Multi-call Dispatcher (BusyBox 구조)**로 제작되어 있었습니다.

```
[동일 바이너리 Multi-call 클러스터 (SHA-256 일치)]
 ├── CTTUtil.exe       : 통합 유틸리티 및 창 숨김/프로세스 제어
 ├── clsdesk.exe       : 바탕화면 정리
 ├── DesktopFixer.exe  : 바탕화면 아이콘 위치 저장 및 복구
 ├── FileBackup.exe    : 레지스트리 및 설정 파일 백업
 ├── FixedWall.exe     : 바탕화면 배경 고정
 ├── MemRes.exe        : 메모리 최적화 (Working Set Trimming)
 ├── pollock.exe       : 윈도우 속성 제어 (항상 위, 색상, 타이틀)
 ├── RsvAgent.exe      : 예약 실행 에이전트
 ├── shrestart.exe     : 쉘(explorer.exe) 재시작 도구
 ├── TrafficLogger.exe : 네트워크 트래픽 실시간 모니터
 ├── trans.exe         : 윈도우 투명도 조절기
 └── unregdll.exe      : COM DLL 등록 해제 유틸리티
```

이 바이너리들은 `GetModuleFileNameA`를 호출하여 실행된 자신의 파일명을 파싱한 뒤, 파일명에 따라 switch-case 분기하여 해당 기능을 실행하는 구조였습니다. **현대화 아키텍처에서는 이 12개 실행 파일을 분산 배포할 필요 없이 단일 x64 네이티브 바이너리(`clicktotweak.exe`)와 모듈형 함수 체계로 통합**합니다.

---

### 1.2 블루스크린(BSOD) 유발 메커니즘 역공학 분석
클릭투트윅이 현대 64비트 Windows 10/11에서 실행 시 즉시 시스템 크래시(BSOD)를 일으키는 원인은 다음 두 커널 드라이버 때문입니다:

1. **`SMFDDrv.Sys` (SHA-256: `955740fc...`, 16,896 바이트)**:
   - 32비트 무서명 커널 드라이버.
   - `ntoskrnl.exe`의 `KeServiceDescriptorTable`(SSDT) 주소를 찾아내어 `NtDeleteFile`, `NtSetInformationFile` 등의 시스템 콜 테이블 포인터를 직접 덮어쓰는 **SSDT 후킹(SSDT Hooking)** 기법을 사용함.
   - 64비트 Windows의 커널 패치 보호(Kernel Patch Protection, PatchGuard) 및 하이퍼바이저 보호 코드 무결성(HVCI) 환경에서 커널 코드 변조가 즉시 감지되어 `CRITICAL_STRUCTURE_CORRUPTION` 블루스크린을 발생시킴.
2. **`FProtMod.sys` (SHA-256: `4293fbf2...`, 16,896 바이트)**:
   - 파일 접근 모니터링을 위해 파일시스템 필터 드라이버 방식으로 동작하나, 64비트 드라이버 서명 강제(Driver Signature Enforcement, DSE)를 통과하지 못해 로드 실패 및 시스템 불안정 초래.

**[현대화 대책]**:
- 커널 드라이버를 **100% 영구 배제**합니다.
- 현대 Windows에서는 유저 모드에서 안전하게 `ReadDirectoryChangesW` 비동기 I/O와 NTFS DACL(Deny Delete ACE), 그리고 타임스탬프 기반 자동 스냅샷 엔진(`backup_guard.rs`)을 통해 커널 조작 없이도 블루스크린 위험 0%의 완전한 파일 보호 및 복원을 달성합니다.

---

### 1.3 IPC 및 권한 상승 파이프라인 분석
- **`CTTService.exe`**: 로컬 시스템 서비스로 등록되어 `\\.\pipe\CTTSvcPipe` 네임드 파이프 서버를 오픈함.
- 일반 권한 프로세스(`CTTBasic.exe`)가 관리자 권한 작업(SYSTEM 레벨 파일 삭제, 시스템 레지스트리 수정)을 요청할 때 명령 패킷(OpCode `0x3ea` 등)을 전송하여 대행하도록 설계됨.
- 현대 Windows 10/11에서는 **Tauri v2의 안전한 IPC 브릿지 및 UAC 네이티브 권한 획득(`is_elevated`, Manifest `requireAdministrator`)**을 통해 단일 바이너리 내에서 안전하게 격리 처리합니다.

---

### 1.4 레거시 DLL 인젝션 및 BHO 결함 분석
1. **`WebInterception.dll`**: Internet Explorer 6~8 전용 Browser Helper Object(BHO)로서 `IWebBrowser2` COM 인터페이스를 후킹하여 광고 URL을 가로챔. Edge 및 Chrome에서는 완전히 동작 불가능하며 브라우저 비정상 종료를 유발함.
   - **대체**: 시스템 전역 `hosts` 루프백(0.0.0.0) 필터링 블록 삽입으로 모든 브라우저 및 앱 전역 적용.
2. **`cdproc.dll`, `PrgFlt.dll`**: 32비트 전역 윈도우 훅 DLL로 다른 프로세스 주소 공간에 인젝션 시도.
   - **대체**: DLL 인젝션 없이 Windows 표준 `SetWindowsHookExW(WH_KEYBOARD_LL / WH_MOUSE_LL)` 및 `SendInput`으로 프로세스 침범 없이 완전 구현.

---

## 2. 구 클릭투트윅 기능 1:1 현대화 맵핑표

| 구 클릭투트윅 기능/모듈 | 구 구현 방식 (Legacy) | Modern 2026 구현 방식 (Rust + Tauri) | 개선 효과 |
| :--- | :--- | :--- | :--- |
| **시스템 정보 및 상태** | Win32 ANSI 호출, 고정 해상도 다이얼로그 | `sysinfo` + React 19 Fluent 다크 카드형 대시보드 | 실시간 하드웨어 게이지, UAC 권한 배지, 반응형 UI |
| **메모리 최적화 (MemRes)** | `SetProcessWorkingSetSize` (Multi-call) | `EmptyWorkingSet` (Tauri 커맨드 원클릭) | 원클릭 시스템 전체 및 특정 프로세스 메모리 즉시 트리밍 |
| **시작프로그램 관리** | ANSI `RegOpenKeyExA`, 단순 삭제 | 32/64비트 Run, StartupApproved, 시작폴더, 자체 격리 키 | 안전한 비활성화(원클릭 복원 가능), 영구 삭제 및 추가 |
| **시스템 & 탐색기 트윅** | Win98/XP 레지스트리 트윅 스크립트 | Win11 클래식 우클릭 메뉴, 화살표 제거, 특수폴더 이동 | Windows 11 Fluent OS에 완벽 적응 및 탐색기 즉각 재시작 |
| **파일 정리 (FileCleaner)** | `FileCleaner.els` 스크립트 기반 | `walkdir` 기반 정밀 타겟 스캔 & 잠금 파일 안전 건너뛰기 | 임시파일, 브라우저 캐시(Edge/Chrome/Whale), 0바이트 정리 |
| **창 투명도 (trans)** | `WS_EX_LAYERED`, 32비트 슬라이더 | `SetLayeredWindowAttributes(LWA_ALPHA)` 슬라이더 | 10%~100% 무단계 실시간 조절 |
| **창 속성/항상위 (pollock)** | 다이얼로그 후킹 | `SetWindowPos(HWND_TOPMOST)`, `SetWindowTextW` | 크로스헤어 피커로 마우스 아래 창 즉시 획득 및 제어 |
| **바탕화면 아이콘 고정 (DesktopFixer)**| `DesktopFixer.exe` (레지스트리 저장) | `GetDesktopWindow` + `ListView` API로 좌표 백업/복원 | 해상도 변경/모니터 연결 시 아이콘 배치 1클릭 복구 |
| **파일 보호 (FProtect)** | `FProtMod.sys`, `SMFDDrv.Sys` (BSOD 유발) | 유저모드 스냅샷 타임라인 및 원상 복원 엔진 | 블루스크린 위험 0%, 중요 설정/문서 롤백 보장 |
| **네트워크 모니터 (TrafficLogger)**| WinPcap 구형 패킷 후킹 | `sysinfo::Networks` 실시간 전송률 모니터링 | 드라이버 불필요, 실시간 다운/업 대역폭 게이지 |
| **웹 광고 필터 (adfilter)** | IE 전용 BHO DLL 인젝션 | 시스템 `hosts` 루프백 블록 안전 주입/제거 | 크롬/엣지/웨일 전역 광고 및 트래커 무력화 |
| **매크로 (CTT Macro)** | 전역 마우스 훅 레코더 | `SendInput` 기반 고속 자동 클릭 & 키보드 시퀀서 | 지연시간 정밀 제어, 반복 횟수 설정, 긴급 중지 |
| **복원 센터 (FileBackup)** | `.els` 스냅샷 | JSON 기반 트윅 상태 스냅샷 및 1클릭 시스템 롤백 | 트윅 전 자동 백업, 언제든 이전 상태로 원상 복구 |

---

## 3. 세부 아키텍처 및 구현 로드맵

### 3.1 Rust 백엔드 모듈 계층 구조
- `src-tauri/src/modules/system_info.rs`: CPU/RAM/스토리지 센서, UAC 검사, 시스템 메모리 트림
- `src-tauri/src/modules/startup.rs`: HKCU/HKLM Run, 시작 폴더, CTT Modern 안전 비활성화/삭제/추가
- `src-tauri/src/modules/shell_tweaks.rs`: Win11 클래식 메뉴, 화살표 제거, 확장자/숨김파일, 특수폴더 리로케이션, 탐색기 재시작
- `src-tauri/src/modules/cleaner.rs`: %TEMP%, Windows Temp, 브라우저 캐시, 0바이트 파일 스캔 및 삭제
- `src-tauri/src/modules/window_tool.rs`: WindowFromPoint, 투명도, 최상위, NtSuspendProcess/Resume, 프로세스 트림
- `src-tauri/src/modules/desktop_icons.rs`: 바탕화면 아이콘 좌표 저장 및 1클릭 복원 (DesktopFixer 부활)
- `src-tauri/src/modules/backup_guard.rs`: 중요 파일 로컬 스냅샷 타임라인 보관 및 원상 복구
- `src-tauri/src/modules/network.rs`: 대역폭 모니터링, Hosts 파일 광고 차단, DNS 캐시 플러시(`ipconfig /flushdns`)
- `src-tauri/src/modules/macro_engine.rs`: SendInput 기반 마우스/키보드 매크로 반복기
- `src-tauri/src/modules/backup_manager.rs`: 복원 지점 JSON 스냅샷 저장 및 롤백 엔진

### 3.2 프론트엔드 모던 카드형 대시보드 UI
- `Sidebar.tsx`: Windows 11 Fluent 스타일 사이드바 (9개 카테고리 탭, UAC 권한 상태 표시)
- `Header.tsx`: 실시간 CPU/RAM 게이지 칩, 원클릭 메모리 부스트 버튼, 알림 토스트
- `DashboardTab.tsx`: 종합 시스템 요약, 드라이브 용량 프로그레스 카드, 빠른 트윅 토글
- `StartupTab.tsx`: 시작프로그램 테이블 카드, 토글 스위치, 추가 모달
- `TweaksTab.tsx`: 쉘 트윅 스위치 그리드, 특수 폴더 경로 이동 패널, 탐색기 재시작
- `CleanerTab.tsx`: 카테고리별 정크 스캔 카드, 용량 집계, 안전 청소 실행
- `WindowToolTab.tsx`: 열린 창 목록, 커서 피커, 투명도 슬라이더, TopMost, 프로세스 제어
- `DesktopFixerCard`: 바탕화면 아이콘 위치 저장 및 복구 카드
- `BackupGuardTab.tsx`: 보호 대상 파일 등록, 스냅샷 히스토리, 복원
- `NetworkTab.tsx`: 실시간 수신/송신 차트 게이지, 시스템 광고 차단 스위치, DNS 캐시 정리
- `MacroTab.tsx`: 매크로 프리셋, 시퀀스 미리보기, 반복 실행 및 긴급 중지
- `RestoreTab.tsx`: 복원 지점 목록 및 1클릭 시스템 롤백

---

## 4. 보안 및 EDR/V3 방어 표준 준수

1. **무서명 바이너리 스폰 차단**: 인라인 파워쉘 루프를 지양하고 표준 Rust 네이티브 API 호출.
2. **로컬 루프백 바인딩**: 외부 노출 없이 `127.0.0.1` 내부 통신.
3. **Zero-Install 포터블 레이아웃**: 레지스트리 시스템 전역 영역을 무단 오염시키지 않고 `%LOCALAPPDATA%\ClickToTweakModern\` 내에 안전하게 격리 저장.
4. **ANSI -> 100% Unicode `*W` API 전환**: 다국어, 한글, 특수문자 경로에서 발생하던 레거시 크래시 완전 차단.

