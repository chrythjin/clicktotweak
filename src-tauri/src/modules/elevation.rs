use crate::modules::system_info::is_process_elevated;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationStatus {
    pub is_elevated: bool,
    pub user_name: String,
    pub can_elevate: bool,
}

fn current_user_name() -> String {
    std::env::var("USERNAME").unwrap_or_default()
}

#[cfg(windows)]
fn shell_execute_runas(file: &str, directory: &str) -> Result<(), String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::HSTRING;

    let instance = unsafe {
        ShellExecuteW(
            HWND::default(),
            &HSTRING::from("runas"),
            &HSTRING::from(file),
            &HSTRING::from(""),
            &HSTRING::from(directory),
            SW_SHOWNORMAL,
        )
    };

    // ShellExecuteW 는 오류 codes 1..=32 를 HINSTANCE 로 돌려준다.
    if (instance.0 as isize) <= 32 {
        return Err(
            "관리자 권한 승인을 요청하지 못했습니다. 실행 파일 경로를 확인해 주세요.".to_string(),
        );
    }
    Ok(())
}

#[cfg(not(windows))]
fn shell_execute_runas(_file: &str, _directory: &str) -> Result<(), String> {
    Err("Windows 외 플랫폼에서는 관리자 권한 승인을 지원하지 않습니다.".to_string())
}

#[tauri::command]
pub fn get_elevation_status() -> ElevationStatus {
    let elevated = is_process_elevated();
    ElevationStatus {
        is_elevated: elevated,
        user_name: current_user_name(),
        can_elevate: !elevated,
    }
}

#[tauri::command]
pub fn relaunch_as_admin() -> Result<String, String> {
    if is_process_elevated() {
        return Ok("이미 관리자 권한으로 실행 중입니다.".to_string());
    }

    let exe = std::env::current_exe()
        .map_err(|e| format!("실행 파일 경로를 확인할 수 없습니다: {}", e))?;
    let directory = exe
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    shell_execute_runas(&exe.to_string_lossy(), &directory)?;

    Ok(
        "관리자 권한으로 다시 실행하는 창을 띄웠습니다. UAC 창에서 '예'를 선택해 주세요."
            .to_string(),
    )
}
