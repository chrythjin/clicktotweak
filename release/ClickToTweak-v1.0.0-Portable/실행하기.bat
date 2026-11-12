@echo off
chcp 65001 >nul
title Click To Tweak Modern 2026 Launcher
cd /d "%~dp0"

echo ==============================================================================
echo   Click To Tweak Modern 2026 (포터블 시스템 최적화 유틸리티)
echo ==============================================================================
echo.
echo   * 드라이버 설치 없음, 시스템 트윅, 정크 파일 제거, 광고/트래픽 차단,
echo     기억력 최적화 지원, 확장자 표시, 안전 파일 스냅샷 기능.
echo.

:: 관리자 권한 확인
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [안내] 현재 관리자 권한으로 실행되지 않았습니다.
    echo        일부 기능(시스템 임시 폴더 정리, HKLM 레지스트리, hosts 광고 차단)은
    echo        관리자 권한이 필요합니다. 앱 상단의 "관리자 권한 필요" 버튼을 누르면
    echo        관리자 권한으로 다시 실행할 수 있습니다.
    echo.
)

start "" "%~dp0ClickToTweak.exe"
exit /b 0
