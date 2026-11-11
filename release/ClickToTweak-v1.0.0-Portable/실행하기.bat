@echo off
chcp 65001 >nul
title Click To Tweak Modern 2026 Launcher
cd /d "%~dp0"

echo ==============================================================================
echo   Click To Tweak Modern 2026 (클릭투트윅 모던 무설치 포터블)
echo ==============================================================================
echo.
echo   * 시작프로그램 관리, 시스템 트윅, 레지스트리 청소, 창/프로세스 제어,
echo     바탕화면 아이콘 복원, 실시간 파일 보호, 광고 차단 통합 유틸리티
echo.

:: 관리자 권한(UAC) 확인
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [안내] 현재 일반 사용자 권한으로 실행되었습니다.
    echo        일부 시스템 트윅(클래식 우클릭 메뉴, HKLM 레지스트리, hosts 광고 차단)을
    echo        적용하려면 '마우스 우클릭 -> 관리자 권한으로 실행'을 권장합니다.
    echo.
)

:: 메인 바이너리 실행
start "" "%~dp0ClickToTweak.exe"
exit /b 0

