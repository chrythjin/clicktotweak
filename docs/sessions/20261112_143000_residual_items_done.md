# 잔여 항목 전량 반영 (2026-11-12)

`docs/sessions/20261112_091000_review_followup_2.md` 이후 남겨 둔 항목을 모두 반영했다.
사용자 지시에 따라 "실동 미검증" 항목은 제외하고 나머지를 구현·검증했다.

## 1. 특수 폴더 실제 파일 이관 (기능 공백 해소)

`src-tauri/src/modules/shell_tweaks.rs`

이전에는 `HKCU\...\Explorer\User Shell Folders`의 경로 값만 바꾸고 기존 파일은 그대로 두었다.
이제 경로 적용 시 실제 파일을 이동한다.

- `User Shell Folders` 와 구형 `Shell Folders` 양쪽 값을 함께 갱신한다.
- 대상 폴더의 **내용물**(하위 폴더 포함)을 새 경로로 이동한다. 폴더 자체를 옮기지 않는다.
- 서로 다른 볼륨이라 `rename`이 실패하면 `복사 → 원본 삭제` 경로로 처리한다.
- 이름이 겹치면 `이름 (1).txt` 형태로 번호를 붙여 기존 파일을 덮어쓰지 않고 함께 보존한다.
- 레지스트리 갱신이 실패하면 이관한 내용을 원래 폴더로 되돌린다(롤백).
- 이동 전 안전 검사: 빈 경로, 상대 경로, 드라이브 루트, 현재 경로와 동일,
  대상이 현재 폴더 내부이거나 현재 폴더가 대상 내부인 경우를 거부한다.
- `%USERPROFILE%` 형태의 환경변수는 `expand_env_vars`로 확장해 비교·이동에 사용한다.
- 프론트(`src/tabs/TweaksTab.tsx`)에는 "파일이 실제 이동된다"는 안내와 확인 대화상자를 추가했다.

테스트 8개 추가: 환경변수 확장, 경로 포함 관계, 충돌 시 이름 분리, 폴더/파일 이동,
크기 집계(하위 폴더 재귀), 없는 원본은 무동작.

## 2. CPU 사용률 표본 개선

`src-tauri/src/modules/system_info.rs`

100ms 단일 표본 → 3회 × 80ms 표본 평균(`CPU_SAMPLE_COUNT`, `CPU_SAMPLE_INTERVAL`).
대시보드 게이지가 폴링마다 크게 요동치던 현상이 줄었다.

## 3. 패닉 정책 변경

`src-tauri/Cargo.toml` 의 `panic = "abort"` 제거 → 기본 `unwind` 로 복귀.

`panic = "abort"` 상태에서는 매크로 스레드에서 패닉이 나면 앱 전체가 즉시 종료된다.
`src-tauri/src/modules/macro_engine.rs` 에 `PlayingGuard` 를 추가해 스레드가 어떤 경로로
종료되든 `IS_PLAYING` 이 해제되도록 했다.

부수 효과: unwinding 테이블이 늘어 릴리스 바이너리가 5.25MB → 6.42MB 로 증가했다.

## 4. `.cargo/config.toml` 정리

루트와 `src-tauri/`에 중복돼 있던 `.cargo/config.toml`(빌드하지 않는
`x86_64-pc-windows-gnullvm` 타깃의 `-static` 플래그)을 양쪽 모두 삭제했다.
두 디렉터리도 함께 제거했다.

## 5. 프론트 테스트 도입

- `vitest` 5.1.0 을 devDependency 로 추가, `npm test` 스크립트 추가.
- `vitest.config.ts` 추가 (node 환경, `src/**/*.test.ts`).
- 중복되어 있던 포맷 로직을 `src/lib/format.ts` 로 분리하고
  `src/lib/transparency.ts` 에 알파 변환을 모아 `DashboardTab`·`CleanerTab`·`WindowToolTab` 이 공유한다.
- 테스트 14개: 바이트 단위 자동 변환, GB 변환, 가동 시간 표기, 알파↔퍼센트 변환과 경계값.

## 검증

| 항목 | 결과 |
| --- | --- |
| `cargo test -p clicktotweak --lib` | 48 passed / 0 failed |
| `cargo clippy --all-targets -p clicktotweak` | 경고 0건 |
| `npm test` | 2 files / 14 tests passed |
| `npm run build` (tsc + vite) | 성공 |
| `cargo build --release --bin clicktotweak` | 성공 (6,419,968 bytes) |
| GUI 스모크 | 배포 exe 실행, 창 제목 확인, 화면 캡처로 전체 UI 렌더링 확인 |
| 배포 정합 | `release/ClickToTweak-v1.0.0-Portable/ClickToTweak.exe` 내장 애셋 해시가 `dist/assets` 와 일치, zip 재생성 |

## 남은 제약 (의도적으로 유지)

- UAC 승격, HKLM 쓰기, hosts 실제 변경, 창 투명도 조작, 매크로 입력 주입은 시스템 상태를
  바꾸므로 자동 검증하지 않았다. 코드 경로와 단위 테스트로만 검증했다.
- `Cargo.lock` 에 `windows` 0.57/0.58/0.62 가 공존한다. 전이 의존성이라 직접 정리 대상이 아니다.
- 파일 파쇄는 저장장치/스냅샷/백업에 따라 원본이 남을 수 있다.
