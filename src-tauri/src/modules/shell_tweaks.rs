use serde::{Deserialize, Serialize};
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellTweaksState {
    pub win11_classic_menu: bool,
    pub show_file_extensions: bool,
    pub show_hidden_files: bool,
    pub show_drive_letters_first: bool,
    pub hide_shortcut_arrow: bool,
    pub taskbar_search_mode: u32, // 0: hidden, 1: icon, 2: search box
    pub special_folders: Vec<SpecialFolderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialFolderInfo {
    pub key_name: String,
    pub display_name: String,
    pub current_path: String,
}

const REG_EXPLORER_ADVANCED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const REG_EXPLORER: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
const REG_USER_SHELL_FOLDERS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";
const REG_SEARCH: &str = r"Software\Microsoft\Windows\CurrentVersion\Search";
const REG_WIN11_CLASSIC_MENU: &str = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";
const REG_SHELL_ICONS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";

#[tauri::command]
pub fn get_shell_tweaks_state() -> ShellTweaksState {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // 1. Win11 classic menu
    let win11_classic_menu = hkcu.open_subkey(REG_WIN11_CLASSIC_MENU).is_ok();

    // 2. HideFileExt
    let show_file_extensions = if let Ok(adv) = hkcu.open_subkey(REG_EXPLORER_ADVANCED) {
        let val: Result<u32, _> = adv.get_value("HideFileExt");
        val.map(|v| v == 0).unwrap_or(false)
    } else {
        false
    };

    // 3. Hidden
    let show_hidden_files = if let Ok(adv) = hkcu.open_subkey(REG_EXPLORER_ADVANCED) {
        let val: Result<u32, _> = adv.get_value("Hidden");
        val.map(|v| v == 1).unwrap_or(false)
    } else {
        false
    };

    // 4. ShowDriveLettersFirst
    let show_drive_letters_first = if let Ok(exp) = hkcu.open_subkey(REG_EXPLORER) {
        let val: Result<u32, _> = exp.get_value("ShowDriveLettersFirst");
        val.map(|v| v == 4).unwrap_or(false)
    } else {
        false
    };

    // 5. Hide shortcut arrow
    let hide_shortcut_arrow = if let Ok(icons) = hklm.open_subkey(REG_SHELL_ICONS) {
        icons.get_value::<String, _>("29").is_ok()
    } else {
        false
    };

    // 6. SearchboxTaskbarMode
    let taskbar_search_mode = if let Ok(srch) = hkcu.open_subkey(REG_SEARCH) {
        srch.get_value::<u32, _>("SearchboxTaskbarMode").unwrap_or(1)
    } else {
        1
    };

    // 7. Special folders
    let mut special_folders = Vec::new();
    if let Ok(folders) = hkcu.open_subkey(REG_USER_SHELL_FOLDERS) {
        let targets = [
            ("Desktop", "바탕 화면"),
            ("{374DE290-123F-4565-9164-39C4925E467B}", "다운로드"),
            ("Personal", "내 문서"),
            ("My Pictures", "내 사진"),
            ("My Music", "내 음악"),
            ("My Video", "내 동영상"),
        ];

        for (k, d) in targets {
            if let Ok(p) = folders.get_value::<String, _>(k) {
                special_folders.push(SpecialFolderInfo {
                    key_name: k.to_string(),
                    display_name: d.to_string(),
                    current_path: p,
                });
            }
        }
    }

    ShellTweaksState {
        win11_classic_menu,
        show_file_extensions,
        show_hidden_files,
        show_drive_letters_first,
        hide_shortcut_arrow,
        taskbar_search_mode,
        special_folders,
    }
}

#[tauri::command]
pub fn set_win11_classic_menu(enable: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let clsid_root = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";

    if enable {
        let (key, _) = hkcu
            .create_subkey(REG_WIN11_CLASSIC_MENU)
            .map_err(|e| format!("레지스트리 생성 실패: {}", e))?;
        key.set_value("", &"")
            .map_err(|e| format!("값 설정 실패: {}", e))?;
        notify_shell();
        Ok("Windows 11 클래식 우클릭 메뉴가 활성화되었습니다. (탐색기 재시작 권장)".to_string())
    } else {
        let _ = hkcu.delete_subkey_all(clsid_root);
        notify_shell();
        Ok("Windows 11 기본 우클릭 메뉴로 복원되었습니다. (탐색기 재시작 권장)".to_string())
    }
}

#[tauri::command]
pub fn set_show_file_extensions(show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_EXPLORER_ADVANCED)
        .map_err(|e| e.to_string())?;
    let val: u32 = if show { 0 } else { 1 };
    key.set_value("HideFileExt", &val)
        .map_err(|e| e.to_string())?;
    notify_shell();
    Ok(if show {
        "알려진 파일 확장자가 항상 표시되도록 설정되었습니다."
    } else {
        "알려진 파일 확장자가 숨겨지도록 설정되었습니다."
    }
    .to_string())
}

#[tauri::command]
pub fn set_show_hidden_files(show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_EXPLORER_ADVANCED)
        .map_err(|e| e.to_string())?;
    let val: u32 = if show { 1 } else { 2 };
    key.set_value("Hidden", &val).map_err(|e| e.to_string())?;
    notify_shell();
    Ok(if show {
        "숨김 파일 및 폴더가 표시되도록 설정되었습니다."
    } else {
        "숨김 파일 및 폴더가 숨겨지도록 설정되었습니다."
    }
    .to_string())
}

#[tauri::command]
pub fn set_show_drive_letters_first(show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_EXPLORER)
        .map_err(|e| e.to_string())?;
    let val: u32 = if show { 4 } else { 0 };
    key.set_value("ShowDriveLettersFirst", &val)
        .map_err(|e| e.to_string())?;
    notify_shell();
    Ok(if show {
        "내 PC에서 드라이브 문자가 이름보다 먼저 표시됩니다."
    } else {
        "드라이브 문자가 기본 형태로 표시됩니다."
    }
    .to_string())
}

#[tauri::command]
pub fn set_taskbar_search_mode(mode: u32) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(REG_SEARCH).map_err(|e| e.to_string())?;
    key.set_value("SearchboxTaskbarMode", &mode)
        .map_err(|e| e.to_string())?;
    notify_shell();
    Ok("작업표시줄 검색 모드가 변경되었습니다.".to_string())
}

#[tauri::command]
pub fn set_hide_shortcut_arrow(hide: bool) -> Result<String, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if hide {
        let (key, _) = hklm
            .create_subkey(REG_SHELL_ICONS)
            .map_err(|e| format!("관리자 권한이 필요합니다: {}", e))?;
        key.set_value("29", &r"%windir%\System32\shell32.dll,-50")
            .map_err(|e| format!("설정 실패: {}", e))?;
        notify_shell();
        Ok("바로가기 화살표 아이콘이 제거되었습니다. (탐색기 재시작 권장)".to_string())
    } else {
        if let Ok(key) = hklm.open_subkey_with_flags(REG_SHELL_ICONS, KEY_WRITE) {
            let _ = key.delete_value("29");
        }
        notify_shell();
        Ok("바로가기 화살표 아이콘이 기본값으로 복원되었습니다.".to_string())
    }
}

#[tauri::command]
pub fn set_special_folder_path(key_name: String, new_path: String) -> Result<String, String> {
    if new_path.trim().is_empty() {
        return Err("유효한 폴더 경로를 입력해주세요.".to_string());
    }

    let p = std::path::Path::new(&new_path);
    if !p.exists() {
        std::fs::create_dir_all(p).map_err(|e| format!("폴더 생성 실패: {}", e))?;
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_USER_SHELL_FOLDERS)
        .map_err(|e| e.to_string())?;
    key.set_value(&key_name, &new_path)
        .map_err(|e| e.to_string())?;

    notify_shell();
    Ok(format!("폴더 경로가 '{}'로 변경되었습니다.", new_path))
}

#[tauri::command]
pub fn restart_explorer() -> Result<String, String> {
    // Kill explorer and restart
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "explorer.exe"])
        .status();

    // Spawn explorer.exe
    let _ = Command::new("cmd")
        .args(["/C", "start", "explorer.exe"])
        .spawn();

    notify_shell();
    Ok("Windows 탐색기(explorer.exe)가 재시작되었습니다.".to_string())
}

#[tauri::command]
pub fn set_animation_effects(disable: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Control Panel\Desktop\WindowMetrics")
        .map_err(|e| e.to_string())?;

    let val = if disable { "0" } else { "1" };
    key.set_value("MinAnimate", &val).map_err(|e| e.to_string())?;

    notify_shell();
    Ok(if disable {
        "창 최소화/최대화 시각 애니메이션이 꺼져 반응 속도가 최적화되었습니다."
    } else {
        "창 최소화/최대화 시각 애니메이션이 복원되었습니다."
    }
    .to_string())
}

fn notify_shell() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}
