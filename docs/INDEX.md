# 통합 검색 인덱스 맵 (INDEX)

이 문서는 `history/` 및 `docs/` 디렉터리에 분산되어 있는 설정 변경 이력, 백업 자산, 그리고 세션 문서를 기록하는 **통합 인덱스 맵**입니다.

## 📌 변경 이력 및 세션 문서 목록

| 날짜 | 문서 | 요약 |
| --- | --- | --- |
| 2026-11-11 | [`sessions/20261111_163826_review_followup.md`](sessions/20261111_163826_review_followup.md) | hosts 원자적 쓰기, 레지스트리 값 형식 보존, 시작프로그램/매크로 상태, 창 투명도·숨김 상태, 정리 확인 절차 보완 |
| 2026-11-12 | [`sessions/20261112_091000_review_followup_2.md`](sessions/20261112_091000_review_followup_2.md) | `App Paths` 정리 실패·전원계획 상태 하드코딩·특수폴더 키 검증 부재 등 실제 버그 수정, 관리자 권한 실행 경로 추가, CSP 활성화, Clippy 0건, 단위 테스트 41건, 배포본 정합화 |
| 2026-11-12 | [`sessions/20261112_143000_residual_items_done.md`](sessions/20261112_143000_residual_items_done.md) | 특수 폴더 실제 파일 이관(롤백 포함), CPU 다중 표본 평균, `panic="abort"` 해제와 매크로 스레드 가드, `.cargo` 설정 정리, vitest 도입과 프론트 테스트 14건 |
| 2026-11-12 | [`sessions/20261112_160000_review_followup_3.md`](sessions/20261112_160000_review_followup_3.md) | 프론트↔Rust IPC 계약 불일치 5건 수정(청소/파쇄/예약종료 동작 복구), IPC 계약 테스트 추가, 복원 지점 롤백의 파일 이동 제거, 시작프로그램 격리 저장소 구현, 삭제 오류 전파, `cargo fmt` 전체 적용 |

## 📂 프로젝트 주요 위치

| 영역 | 경로 |
| --- | --- |
| Rust 백엔드 모듈 | `src-tauri/src/modules/` |
| 명령어 등록 | `src-tauri/src/lib.rs` |
| React 프론트엔드 | `src/` (`tabs/`, `components/`) |
| IPC 타입 정의 | `src/types.ts` |
| 프론트 공용 로직 | `src/lib/` (`format.ts`, `transparency.ts`) |
| 프론트 테스트 | `src/__tests__/`, `vitest.config.ts` |
| 배포 패키지 | `release/ClickToTweak-v1.0.0-Portable/` |
| 빌드 설정 | `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` |
| 사용자 데이터 저장 위치 | `%LOCALAPPDATA%\ClickToTweakModern` |
