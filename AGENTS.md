# PROJECT KNOWLEDGE BASE — CLICKTOTWEAK MODERN 2026

**Generated:** 2026-11-11  
**Version:** v1.0.0  
**Branch:** main  

## OVERVIEW

Click To Tweak Modern = 10년 전 개발 중단된 국민 PC 최적화 유틸리티 '클릭투트윅'을 Rust 2024 + Tauri v2 + React 19 기반으로 100% 재창조한 x64 무설치 포터블 유틸리티.
핵심 설계 의도: **BSOD 위험 0% (커널 드라이버 100% 배제)** 및 **12개 분산 바이너리를 단일 독립 바이너리로 통합**.

## STRUCTURE

```
CLICKTOTWEAK/
├── src/                         # React 19 + Tailwind CSS v4 프론트엔드 UI
│   ├── components/              # Sidebar, Header 등 공통 레이아웃
│   ├── tabs/                    # 대시보드, 제어판, 클리너, 트윅, 프로세스, 가드 등 탭별 구현체
│   ├── types.ts                 # Tauri IPC 공유 TypeScript 인터페이스 정의
│   └── App.tsx                  # 메인 레이아웃 및 탭 라우팅
│
├── src-tauri/                   # Rust 2024 백엔드 코어 (Tauri v2)
│   ├── src/modules/             # 도메인별 기능 모듈
│   │   ├── control_panel.rs     # 바탕화면 아이콘, 마우스/키보드 속도, 12종 애플릿 런처
│   │   ├── registry_cleaner.rs  # App Paths, Shared DLLs, MUI Cache 고아 레지스트리 청소
│   │   ├── file_shredder.rs     # DoD 5220.22-M 1~7단계 영구 삭제 파쇄 엔진
│   │   ├── auto_shutdown.rs     # 컴퓨터 자동 예약 종료/재부팅 타이머
│   │   ├── system_info.rs       # sysinfo 하드웨어 센서 & 메모리 트림
│   │   ├── startup.rs           # 32/64비트 시작프로그램 자체 격리 안전 관리
│   │   ├── shell_tweaks.rs      # Win11 클래식 우클릭 복원, 확장자 표시, 특수폴더 이동
│   │   ├── cleaner.rs           # 임시파일, 브라우저 캐시, 크래시 덤프 청소
│   │   ├── window_tool.rs       # 창 투명도(10~100%), Always On Top, 프로세스 일시중지
│   │   ├── backup_guard.rs      # 파일 스냅샷 타임라인 및 롤백 엔진
│   │   └── network.rs           # 실시간 트래픽 대역폭, hosts 광고 차단, DNS 플러시
│   ├── Cargo.toml               # Rust 의존성 및 프로덕션 릴리스 프로파일
│   └── tauri.conf.json          # Tauri 앱 윈도우 및 빌드 설정
│
├── release/                     # 배포 패키지 (포터블 에디션)
│   ├── ClickToTweak-v1.0.0-Portable/
│   │   ├── ClickToTweak.exe     # 약 4.9MB 단일 독립 x64 바이너리
│   │   ├── 실행하기.bat         # UAC 권한 자동 감지 런처
│   │   ├── 사용설명서.txt       # 상세 한글 매뉴얼
│   │   └── data/                # 격리 로컬 데이터 보관소
│   └── ClickToTweak-v1.0.0-Portable.zip # GitHub Release 배포용 압축 파일
│
├── docs/                        # 개발 문서 및 세션 이력
└── README.md                    # 사용자 안내 문서
```

## WHERE TO LOOK

| 작업 | 위치 | 비고 |
|------|------|------|
| 제어판/시스템 아이콘/Win11 트윅 | `src-tauri/src/modules/control_panel.rs` ↔ `src/tabs/ControlPanelTab.tsx` | 바탕화면 아이콘, 마우스 가속도 해제, 12종 제어판 런처 |
| 레지스트리 청소 & DoD 파일 파쇄기 | `src-tauri/src/modules/registry_cleaner.rs`, `file_shredder.rs` ↔ `src/tabs/CleanerTab.tsx` | App Paths, Shared DLLs, MUI Cache, DoD 다중 덮어쓰기 |
| 예약 종료 타이머 | `src-tauri/src/modules/auto_shutdown.rs` ↔ `src/tabs/ControlPanelTab.tsx` | shutdown.exe 호출 래퍼 및 취소 |
| 시작프로그램 관리 | `src-tauri/src/modules/startup.rs` ↔ `src/tabs/StartupTab.tsx` | 레지스트리 안전 격리 키 보관 |
| 창 투명도 & 프로세스 제어 | `src-tauri/src/modules/window_tool.rs` ↔ `src/tabs/WindowToolTab.tsx` | SetLayeredWindowAttributes, NtSuspendProcess |
| 배포 패키지 산출물 | `release/ClickToTweak-v1.0.0-Portable/` | 포터블 게임 클라이언트 레이아웃 |
