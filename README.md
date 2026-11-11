# Click To Tweak Modern 2026 (클릭투트윅 모던)

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20x64-0078D6?style=for-the-badge&logo=windows&logoColor=white" alt="Windows 11" />
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-black?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/Tauri-v2.0-24C8D8?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri" />
  <img src="https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react&logoColor=black" alt="React 19" />
  <img src="https://img.shields.io/badge/TailwindCSS-v4-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white" alt="Tailwind" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="MIT License" />
  <img src="https://img.shields.io/badge/Release-v1.0.0-blue?style=for-the-badge" alt="v1.0.0" />
</p>

---

## ⚡ 빠르고 간편한 다운로드

설치 과정이나 복잡한 런타임 설치 없이, 압축만 풀면 즉시 실행되는 **단일 무설치 포터블 에디션**입니다.

<div align="center">

### 🚀 [**ClickToTweak-v1.0.0-Portable.zip 다운로드 (최신 릴리스)**](https://github.com/chrythjin/clicktotweak/releases/latest/download/ClickToTweak-v1.0.0-Portable.zip)

*(압축 해제 후 `ClickToTweak.exe` 또는 `실행하기.bat`을 더블 클릭하여 즉시 실행)*

</div>

---

## 📖 프로젝트 개요

2000년대 중후반 국민 PC 최적화 유틸리티로 사랑받았던 **'클릭투트윅(Click To Tweak)'**은 약 10년 전 원작자의 개발 중단 이후 현대 64비트 Windows 10/11 환경에서 정상적인 구동이 불가능했습니다.

본 프로젝트는 **REA(Reverse Engineer Anything) 역공학 정밀 분석**을 토대로 구 클릭투트윅의 12대 분산 바이너리와 유틸리티를 현대 기술 스택(**Rust 2024 + Tauri v2 + React 19 + Fluent 다크 카드형 UI**)으로 100% 새롭게 재작성한 오픈소스 프로젝트입니다.

### 💡 왜 새롭게 만들었는가?
1. **블루스크린(BSOD) 위험 100% 제거**:
   - 구버전의 32비트 무서명 커널 드라이버(`SMFDDrv.Sys`, `FProtMod.sys`)가 SSDT 커널 후킹을 시도하여 Windows 10/11 64비트의 PatchGuard(KPP) 및 HVCI 보안 기술과 충돌해 시스템 충돌(`CRITICAL_STRUCTURE_CORRUPTION`)을 일으키던 원인을 완벽히 규명하고, **커널 드라이버를 100% 영구 배제**하였습니다.
2. **단일 네이티브 바이너리 통합**:
   - 과거 12개로 쪼개져 있던 실행 파일(`CTTUtil`, `DesktopFixer`, `trans`, `pollock`, `MemRes` 등)을 **약 4.9MB 크기의 초경량 단일 Rust 바이너리(`ClickToTweak.exe`)로 통합**하였습니다.
3. **Windows 11 네이티브 최적화**:
   - 현대 OS에서 역효과를 내는 90년대식 TCP/IP 패치, IE 전용 BHO, CD/DVD 자동실행 트윅은 과감히 걷어내고, **Windows 11 클래식 우클릭 복원, 작업표시줄 정렬, 시작 메뉴 Bing 검색 끄기, 제어판 센터 복원, DoD 군사 규격 파일 파쇄기** 등 현대에 필수적인 도구들로 전면 현대화했습니다.

---

## 🛠️ 주요 기능 소개 (10대 핵심 모듈)

```
클릭투트윅 Modern 2026 통합 대시보드
 ├── 1. 대시보드 개요       : 실시간 CPU/RAM/스토리지 모니터링 & 원클릭 메모리 부스트
 ├── 2. 시작프로그램 관리자 : 32/64비트 레지스트리 및 시작폴더 탐색 & 자체 격리 안전 비활성화
 ├── 3. 시스템 & 탐색기 트윅: Win11 클래식 우클릭 복원, 바로가기 화살표 제거, 특수폴더 이동
 ├── 4. 제어판 & 도구 센터 : 바탕화면 시스템 아이콘 꺼내기, 마우스 가속도 끄기, 12종 제어판 런처
 ├── 5. 정크 & 레지스트리  : 임시파일/캐시 청소, 고아 레지스트리(App Paths, Shared DLLs) 정리
 ├── 6. 보안 파일 파쇄기   : DoD 5220.22-M 1~7단계 군사 규격 복구 불가 영구 삭제
 ├── 7. 컴퓨터 예약 종료   : N분 후 시스템 종료 / 재부팅 / 최대 절전 타이머 및 1클릭 취소
 ├── 8. 창 & 프로세스 제어 : 창 투명도 10~100%, Always On Top 고정, 프로세스 일시중지
 ├── 9. 파일 백업 가드     : 무중단 로컬 스냅샷 타임라인 보관 및 1클릭 파일 롤백
 ├── 10. 네트워크 & 광고차단: 실시간 트래픽 대역폭 측정, 시스템 전역 hosts 루프백 광고 필터링
 └── 11. 트윅 복원 센터    : 레지스트리 트윅 자동 스냅샷 생성 및 1클릭 원상 복구
```

### 1. 제어판 & 시스템 도구 센터 (구 클릭투트윅 제어판 계승)
- **바탕화면 시스템 아이콘 꺼내기**: Windows 11에서 기본 숨김 처리된 `내 PC`, `휴지통`, `제어판`, `사용자 파일`, `네트워크` 아이콘을 원클릭으로 바탕화면에 표시/숨김.
- **마우스 & 키보드 반응 속도 극대화**:
  - 키보드 반복 지연 시간 초고속(0단계) 단축
  - 메뉴 팝업 지연(`MenuShowDelay`) 시간 최소화
  - 게이밍 및 정밀 작업을 위한 **마우스 포인터 정확도 향상(마우스 가속도) 1:1 센서 매핑 해제**
- **전원 & 용량 제어**:
  - 최대 절전 모드(`hiberfil.sys`) 해제로 C: 드라이브 **수십 GB 용량 즉각 확보**
  - 마이크로 레이턴시를 제거하는 숨김 전원 플랜 **'최고의 성능 (Ultimate Performance)'** 잠금 해제
- **Windows 11 맞춤 트윅**:
  - 작업 표시줄 아이콘 위치 변경 (왼쪽 정렬 ↔ 가운데 정렬)
  - 시작 메뉴 **Bing 웹 검색 결과 차단** (순수 로컬 앱/파일만 즉각 검색)
  - Windows 업데이트 후 **강제 자동 재부팅 방지**
  - 파일 우클릭 **'관리자 소유권 가져오기 (Take Ownership)'** 쉘 메뉴 등록
- **12대 핵심 제어판 애플릿 원클릭 런처**:
  - 클래식 제어판, **갓모드(GodMode - 200개 이상 통합 제어판)**, 시스템 속성, 네트워크 어댑터, 장치 관리자, 디스크 관리 등 1클릭 실행.

### 2. 정크 파일 & 레지스트리 청소 및 DoD 파일 보안 파쇄기
- **정크 & 캐시 청소**:
  - %TEMP%, Windows Temp, 크래시 덤프, 웹 브라우저(Edge, Chrome, Whale) 캐시 안전 정리.
- **레지스트리 불필요 항목 청소 (Registry Cleaner)**:
  - 삭제된 프로그램의 잔여 실행 경로 (`App Paths`) 고아 키 탐지 및 정리
  - 존재하지 않는 공유 라이브러리 참조 (`Shared DLLs`) 정리
  - MUI 캐시(`MuiCache`)의 삭제된 프로그램 명칭 잔여 기록 청소
- **DoD 5220.22-M 군사 규격 보안 파일 파쇄기 (Shredder)**:
  - 포렌식 복구 툴(Recuva, R-Studio 등)로 절대 복구할 수 없는 파일 완전 삭제
  - 1단계(제로 필) / 3단계(DoD 표준: 0x00 -> 0xFF -> 난수) / 7단계(DoD ECE 최고 보안)

### 3. 시작프로그램 안전 관리자 (Startup Manager)
- 32/64비트 `Run`, `RunOnce`, `StartupApproved`, 시작 폴더 전수 탐색.
- 레지스트리를 단순 삭제하지 않고 `ClickToTweakModern\Disabled` 격리 키에 안전 보관하여 **언제든 원클릭으로 활성화/비활성화 전환 가능**.

### 4. 컴퓨터 자동 예약 종료 타이머 (Shutdown Timer)
- 15분 / 30분 / 1시간 / 2시간 / 분 단위 직접 입력.
- 시스템 종료, 다시 시작, 최대 절전 모드 예약 및 언제든 1클릭 즉시 취소.

### 5. 창 & 프로세스 제어 (trans / pollock 현대화)
- 크로스헤어 피커로 마우스 아래의 창 즉시 선택.
- 창 투명도(Alpha) 10% ~ 100% 무단계 실시간 조절.
- 창 항상 위(TopMost) 고정, 창 숨기기/보이기, 프로세스 일시중지(`NtSuspendProcess`) 및 메모리 트리밍.

---

## 🚫 Windows 11 환경에서 제외된 레거시 항목

| 레거시 기능 | 제외 사유 |
| :--- | :--- |
| **MTU / TCP 수신 윈도우(RWIN) 패치** | Windows 11의 최신 CTCP 및 네트워크 자동 튜닝 엔진과 충돌하여 기가비트 인터넷 속도 급감 유발 |
| **IE BHO 광고 가로채기 (`WebInterception.dll`)** | Internet Explorer 완전 퇴출로 동작 불가 (시스템 `hosts` 필터링으로 대체) |
| **CD/DVD 자동 실행(Autorun) 방지** | 최신 PC 환경에서 광학 드라이브(ODD) 부재로 실효성 상실 |
| **32비트 커널 드라이버 (`SMFDDrv`, `FProtMod`)** | 64비트 DSE(드라이버 서명 강제) 및 PatchGuard 위반으로 BSOD 크래시 유발 |

---

## 🛡️ 엔터프라이즈 보안 및 무설치(Zero-Install) 포터블

- **사내 보안 V3 백신 / EDR / DLP 오탐 방지**:
  - 무서명 커널 드라이버(`.sys`) 일체 미사용.
  - `%TEMP%`에 임의로 바이너리를 풀어서 실행하는 Dropper 방식이 아닌 **순수 100% 독립 포터블 바이너리**.
  - 인라인 파워쉘 무작위 프로세스 스폰 배제.
- **포터블 디렉터리 구성**:
  - 시스템 전역이나 레지스트리를 무단 오염시키지 않고, 모든 설정 및 스냅샷은 로컬 `data/` 폴더 내에 안전하게 보관됩니다.
  - USB나 외장 드라이브, 다른 PC로 폴더 전체를 복사해도 즉시 구동됩니다.

---

## 📂 프로젝트 구조

```
CLICKTOTWEAK/
├── src/                         [React 19 + Tailwind v4 프론트엔드]
│   ├── components/              Sidebar, Header 등 공통 컴포넌트
│   ├── tabs/                    대시보드, 제어판, 클리너, 트윅 등 각 탭 화면
│   ├── types.ts                 TypeScript 타입 정의
│   └── App.tsx                  메인 레이아웃 및 라우팅
│
├── src-tauri/                   [Rust 2024 백엔드 코어]
│   ├── src/modules/
│   │   ├── control_panel.rs     바탕화면 아이콘, 마우스/키보드 반응속도, 12종 런처
│   │   ├── registry_cleaner.rs  App Paths, Shared DLLs, MUI Cache 고아 항목 청소
│   │   ├── file_shredder.rs     DoD 5220.22-M 1~7단계 영구 삭제 파쇄 엔진
│   │   ├── auto_shutdown.rs     컴퓨터 예약 종료/재부팅 타이머 및 취소
│   │   ├── system_info.rs       sysinfo 기반 하드웨어 센서 및 메모리 최적화
│   │   ├── startup.rs           32/64비트 시작프로그램 관리 및 격리
│   │   ├── shell_tweaks.rs      Win11 우클릭 복원, 확장자 표시, 특수폴더 이동
│   │   ├── cleaner.rs           정크 파일, 브라우저 캐시, 덤프 청소
│   │   ├── window_tool.rs       창 투명도, 항상 위, 프로세스 일시중지
│   │   ├── backup_guard.rs      파일 스냅샷 및 롤백 엔진
│   │   └── network.rs           실시간 대역폭 측정, hosts 광고 차단, DNS 플러시
│   └── Cargo.toml               Rust 의존성 및 프로덕션 릴리스 프로파일
│
└── release/                     [배포 패키지]
    ├── ClickToTweak-v1.0.0-Portable/
    │   ├── ClickToTweak.exe     (약 4.9MB 단일 독립 바이너리)
    │   ├── 실행하기.bat         (원클릭 권한 런처)
    │   ├── 사용설명서.txt       (상세 가이드)
    │   └── data/                (격리 설정 보관소)
    └── ClickToTweak-v1.0.0-Portable.zip
```

---

## 🔨 소스코드 빌드 방법

직접 소스코드를 컴파일하여 빌드하고자 하는 개발자는 다음 환경이 필요합니다:
- **Rust**: 1.85 이상 (`rustup default stable`)
- **Node.js**: v18 이상 (`npm`)
- **C++ Build Tools**: Visual Studio C++ x64 Build Tools

```powershell
# 1. 저장소 복제
git clone https://github.com/chrythjin/clicktotweak.git
cd clicktotweak

# 2. 프론트엔드 의존성 설치
npm install

# 3. 프론트엔드 빌드
npm run build

# 4. Rust 릴리스 바이너리 빌드
cargo build --release --manifest-path src-tauri/Cargo.toml

# 5. 빌드 완료 확인
# src-tauri/target/release/clicktotweak.exe 바이너리가 생성됩니다.
```

---

## 📜 라이선스

본 프로젝트는 [MIT License](LICENSE)를 따릅니다. 누구나 자유롭게 사용, 수정, 배포할 수 있습니다.
