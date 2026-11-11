use serde::{Deserialize, Serialize};
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DesktopSystemIcons {
    pub this_pc: bool,
    pub user_files: bool,
    pub control_panel: bool,
    pub network: bool,
    pub recycle_bin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputResponseSettings {
    pub keyboard_delay: u32,
    pub keyboard_speed: u32,
    pub menu_show_delay: u32,
    pub mouse_speed_acceleration: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemPowerSettings {
    pub hibernate_enabled: bool,
    pub ultimate_performance_unlocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskbarSettings {
    pub searchbox_mode: u32, // 0: 숨김, 1: 검색 아이콘, 2: 검색 상자, 3: 검색 아이콘 및 레이블
    pub show_task_view: bool,
    pub show_widgets: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvancedWin11Tweaks {
    pub taskbar_alignment_center: bool, // true: 가운데 정렬, false: 왼쪽 정렬
    pub disable_bing_search: bool,      // 시작 메뉴 웹 검색 끄기
    pub prevent_auto_reboot: bool,      // 윈도우 업데이트 자동 재부팅 방지
    pub take_ownership_menu: bool,      // 파일 우클릭 소유권 가져오기 메뉴
}

const DESKTOP_ICON_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel";
const CLSID_THIS_PC: &str = "{20D04FE0-3AEA-1069-A2D8-08002B30309D}";
const CLSID_USER_FILES: &str = "{59031a47-072d-4514-9c55-08035c777b45}";
const CLSID_CONTROL_PANEL: &str = "{5399E694-6CE5-4D6C-8FCE-1D8870FDCBA0}";
const CLSID_NETWORK: &str = "{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}";
const CLSID_RECYCLE_BIN: &str = "{645FF040-5081-101B-9F08-00AA002F954E}";

fn notify_shell_change() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}

// ==========================================
// 1. 바탕화면 시스템 아이콘 제어
// ==========================================
#[tauri::command]
pub fn get_desktop_system_icons() -> Result<DesktopSystemIcons, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(DESKTOP_ICON_KEY).ok();

    let is_shown = |clsid: &str, default_shown: bool| -> bool {
        if let Some(ref k) = key {
            if let Ok(val) = k.get_value::<u32, _>(clsid) {
                // 0 = 표시, 1 = 숨김
                return val == 0;
            }
        }
        default_shown
    };

    Ok(DesktopSystemIcons {
        this_pc: is_shown(CLSID_THIS_PC, false),
        user_files: is_shown(CLSID_USER_FILES, false),
        control_panel: is_shown(CLSID_CONTROL_PANEL, false),
        network: is_shown(CLSID_NETWORK, false),
        recycle_bin: is_shown(CLSID_RECYCLE_BIN, true),
    })
}

#[tauri::command]
pub fn set_desktop_system_icon(icon_type: String, show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(DESKTOP_ICON_KEY)
        .map_err(|e| format!("레지스트리 키 열기 실패: {}", e))?;

    let clsid = match icon_type.as_str() {
        "this_pc" => CLSID_THIS_PC,
        "user_files" => CLSID_USER_FILES,
        "control_panel" => CLSID_CONTROL_PANEL,
        "network" => CLSID_NETWORK,
        "recycle_bin" => CLSID_RECYCLE_BIN,
        _ => return Err("알 수 없는 아이콘 유형입니다.".to_string()),
    };

    // 0 = 표시, 1 = 숨김
    let val: u32 = if show { 0 } else { 1 };
    key.set_value(clsid, &val)
        .map_err(|e| format!("아이콘 설정 저장 실패: {}", e))?;

    notify_shell_change();
    let state_str = if show { "표시" } else { "숨김" };
    Ok(format!("바탕화면 시스템 아이콘이 '{}' 상태로 변경되었습니다.", state_str))
}

// ==========================================
// 2. 키보드 & 마우스 응답속도 제어판
// ==========================================
#[tauri::command]
pub fn get_input_response_settings() -> Result<InputResponseSettings, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Keyboard
    let kb_key = hkcu.open_subkey(r"Control Panel\Keyboard").ok();
    let kb_delay: u32 = kb_key
        .as_ref()
        .and_then(|k| k.get_value::<String, _>("KeyboardDelay").ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let kb_speed: u32 = kb_key
        .as_ref()
        .and_then(|k| k.get_value::<String, _>("KeyboardSpeed").ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(31);

    // Desktop MenuShowDelay
    let desk_key = hkcu.open_subkey(r"Control Panel\Desktop").ok();
    let menu_delay: u32 = desk_key
        .as_ref()
        .and_then(|k| k.get_value::<String, _>("MenuShowDelay").ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);

    // Mouse Acceleration
    let mouse_key = hkcu.open_subkey(r"Control Panel\Mouse").ok();
    let mouse_accel: bool = mouse_key
        .as_ref()
        .and_then(|k| k.get_value::<String, _>("MouseSpeed").ok())
        .map(|s| s != "0")
        .unwrap_or(true);

    Ok(InputResponseSettings {
        keyboard_delay: kb_delay,
        keyboard_speed: kb_speed,
        menu_show_delay: menu_delay,
        mouse_speed_acceleration: mouse_accel,
    })
}

#[tauri::command]
pub fn set_input_response_settings(settings: InputResponseSettings) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Keyboard Delay & Speed
    if let Ok((kb_key, _)) = hkcu.create_subkey(r"Control Panel\Keyboard") {
        let _ = kb_key.set_value("KeyboardDelay", &settings.keyboard_delay.to_string());
        let _ = kb_key.set_value("KeyboardSpeed", &settings.keyboard_speed.to_string());
    }

    // MenuShowDelay
    if let Ok((desk_key, _)) = hkcu.create_subkey(r"Control Panel\Desktop") {
        let _ = desk_key.set_value("MenuShowDelay", &settings.menu_show_delay.to_string());
    }

    // Mouse Acceleration
    if let Ok((mouse_key, _)) = hkcu.create_subkey(r"Control Panel\Mouse") {
        let mouse_speed = if settings.mouse_speed_acceleration { "1" } else { "0" };
        let mouse_thresh = if settings.mouse_speed_acceleration { "6" } else { "0" };
        let _ = mouse_key.set_value("MouseSpeed", &mouse_speed);
        let _ = mouse_key.set_value("MouseThreshold1", &mouse_thresh);
        let _ = mouse_key.set_value("MouseThreshold2", &if settings.mouse_speed_acceleration { "10" } else { "0" });
    }

    // SystemParametersInfoW notify
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETKEYBOARDDELAY,
            SPI_SETKEYBOARDSPEED,
        };
        let _ = SystemParametersInfoW(
            SPI_SETKEYBOARDDELAY,
            settings.keyboard_delay,
            None,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
        let _ = SystemParametersInfoW(
            SPI_SETKEYBOARDSPEED,
            settings.keyboard_speed,
            None,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
    }

    Ok("키보드 및 마우스 반응 속도 설정이 즉시 저장되었습니다.".to_string())
}

// ==========================================
// 3. 전원 및 시스템 드라이브 용량 제어판
// ==========================================
#[tauri::command]
pub fn get_system_power_settings() -> Result<SystemPowerSettings, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hibernate_enabled = if let Ok(power_key) =
        hklm.open_subkey(r"SYSTEM\CurrentControlSet\Control\Power")
    {
        power_key.get_value::<u32, _>("HibernateEnabled").unwrap_or(0) == 1
    } else {
        false
    };

    Ok(SystemPowerSettings {
        hibernate_enabled,
        ultimate_performance_unlocked: true,
    })
}

#[tauri::command]
pub fn set_hibernate_enabled(enable: bool) -> Result<String, String> {
    let arg = if enable { "/h on" } else { "/h off" };
    let output = Command::new("powercfg")
        .args(arg.split_whitespace())
        .output()
        .map_err(|e| format!("powercfg 실행 실패: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("최대 절전 모드 변경 실패 (관리자 권한 필요): {}", err));
    }

    let msg = if enable {
        "최대 절전 모드(Hibernate)가 활성화되었습니다."
    } else {
        "최대 절전 모드가 해제되어 C: 드라이브(hiberfil.sys) 공간이 즉시 확보되었습니다."
    };
    Ok(msg.to_string())
}

#[tauri::command]
pub fn unlock_ultimate_performance() -> Result<String, String> {
    // Ultimate Performance GUID
    let output = Command::new("powercfg")
        .args(&[
            "-duplicatescheme",
            "e9a42b02-d5df-448d-aa00-03f14749eb61",
        ])
        .output()
        .map_err(|e| format!("powercfg 실행 실패: {}", e))?;

    if output.status.success() {
        Ok("Windows '최고의 성능(Ultimate Performance)' 전원 계획이 잠금 해제되었습니다.".to_string())
    } else {
        Err("전원 계획 생성 실패 (관리자 권한이 필요합니다).".to_string())
    }
}

// ==========================================
// 4. Windows 11 작업표시줄 제어판 설정
// ==========================================
#[tauri::command]
pub fn get_taskbar_settings() -> Result<TaskbarSettings, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Searchbox mode
    let search_mode: u32 = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Search")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("SearchboxTaskbarMode").ok())
        .unwrap_or(1);

    // Task View & Widgets
    let adv_key = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
        .ok();

    let show_task_view = adv_key
        .as_ref()
        .and_then(|k| k.get_value::<u32, _>("ShowTaskViewButton").ok())
        .unwrap_or(1)
        == 1;

    let show_widgets = adv_key
        .as_ref()
        .and_then(|k| k.get_value::<u32, _>("TaskbarDa").ok())
        .unwrap_or(1)
        == 1;

    Ok(TaskbarSettings {
        searchbox_mode: search_mode,
        show_task_view,
        show_widgets,
    })
}

#[tauri::command]
pub fn set_taskbar_setting(key: String, value: u32) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    match key.as_str() {
        "searchbox_mode" => {
            let (k, _) = hkcu
                .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Search")
                .map_err(|e| e.to_string())?;
            k.set_value("SearchboxTaskbarMode", &value)
                .map_err(|e| e.to_string())?;
        }
        "show_task_view" => {
            let (k, _) = hkcu
                .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
                .map_err(|e| e.to_string())?;
            k.set_value("ShowTaskViewButton", &value)
                .map_err(|e| e.to_string())?;
        }
        "show_widgets" => {
            let (k, _) = hkcu
                .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
                .map_err(|e| e.to_string())?;
            k.set_value("TaskbarDa", &value)
                .map_err(|e| e.to_string())?;
        }
        _ => return Err("알 수 없는 설정 키입니다.".to_string()),
    }

    notify_shell_change();
    Ok("작업 표시줄 설정이 적용되었습니다.".to_string())
}

// ==========================================
// 5. 클래식 제어판 및 윈도우 관리 도구 런처
// ==========================================
#[tauri::command]
pub fn launch_control_panel_applet(applet: String) -> Result<String, String> {
    let (cmd, args): (&str, Vec<&str>) = match applet.as_str() {
        "control" => ("control.exe", vec![]),
        "godmode" => (
            "explorer.exe",
            vec![r"shell:::{ED7BA470-8E54-465E-825C-99712043E01C}"],
        ),
        "sysdm" => ("control.exe", vec!["sysdm.cpl"]),
        "appwiz" => ("control.exe", vec!["appwiz.cpl"]),
        "ncpa" => ("control.exe", vec!["ncpa.cpl"]),
        "services" => ("services.msc", vec![]),
        "diskmgmt" => ("diskmgmt.msc", vec![]),
        "devmgmt" => ("devmgmt.msc", vec![]),
        "compmgmt" => ("compmgmt.msc", vec![]),
        "gpedit" => ("gpedit.msc", vec![]),
        "regedit" => ("regedit.exe", vec![]),
        "taskmgr" => ("taskmgr.exe", vec![]),
        "mmsys" => ("control.exe", vec!["mmsys.cpl"]),
        "powercfg" => ("control.exe", vec!["powercfg.cpl"]),
        "firewall" => ("control.exe", vec!["firewall.cpl"]),
        _ => return Err("지원되지 않는 제어판 애플릿입니다.".to_string()),
    };

    Command::new(cmd)
        .args(&args)
        .spawn()
        .map_err(|e| format!("도구 실행 실패 ({}): {}", cmd, e))?;

    Ok(format!("'{}' 도구를 실행했습니다.", applet))
}

// ==========================================
// 6. Windows 11 고급 시스템 & 작업표시줄 트윅
// ==========================================
#[tauri::command]
pub fn get_advanced_win11_tweaks() -> Result<AdvancedWin11Tweaks, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    // 1. Taskbar alignment: 0=왼쪽, 1=가운데
    let is_center = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("TaskbarAl").ok())
        .unwrap_or(1)
        == 1;

    // 2. Disable Bing search
    let bing_disabled = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Search")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("BingSearchEnabled").ok())
        .unwrap_or(1)
        == 0;

    // 3. Prevent auto reboot
    let prevent_reboot = hklm
        .open_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU")
        .ok()
        .and_then(|k| k.get_value::<u32, _>("NoAutoRebootWithLoggedOnUsers").ok())
        .unwrap_or(0)
        == 1;

    // 4. Take ownership menu
    let has_take_ownership = hkcr.open_subkey(r"*\shell\runas").is_ok();

    Ok(AdvancedWin11Tweaks {
        taskbar_alignment_center: is_center,
        disable_bing_search: bing_disabled,
        prevent_auto_reboot: prevent_reboot,
        take_ownership_menu: has_take_ownership,
    })
}

#[tauri::command]
pub fn set_advanced_win11_tweak(key: String, enable: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    match key.as_str() {
        "taskbar_alignment_center" => {
            let (k, _) = hkcu
                .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
                .map_err(|e| e.to_string())?;
            let val: u32 = if enable { 1 } else { 0 };
            k.set_value("TaskbarAl", &val).map_err(|e| e.to_string())?;
            notify_shell_change();
            let state = if enable { "가운데 정렬" } else { "왼쪽 정렬" };
            Ok(format!("작업 표시줄이 '{}' 상태로 변경되었습니다.", state))
        }
        "disable_bing_search" => {
            let (search_k, _) = hkcu
                .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Search")
                .map_err(|e| e.to_string())?;
            let val_bing: u32 = if enable { 0 } else { 1 };
            search_k.set_value("BingSearchEnabled", &val_bing).map_err(|e| e.to_string())?;

            let (policy_k, _) = hkcu
                .create_subkey(r"Software\Policies\Microsoft\Windows\Explorer")
                .map_err(|e| e.to_string())?;
            let val_sugg: u32 = if enable { 1 } else { 0 };
            policy_k.set_value("DisableSearchBoxSuggestions", &val_sugg).map_err(|e| e.to_string())?;

            notify_shell_change();
            let state = if enable { "차단됨 (로컬 검색만 수행)" } else { "기본값 복원" };
            Ok(format!("시작 메뉴 Bing 웹 검색 결과가 '{}' 상태로 변경되었습니다.", state))
        }
        "prevent_auto_reboot" => {
            let (k, _) = hklm
                .create_subkey(r"SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU")
                .map_err(|e| format!("정책 키 생성 실패 (관리자 권한 필요): {}", e))?;
            let val: u32 = if enable { 1 } else { 0 };
            k.set_value("NoAutoRebootWithLoggedOnUsers", &val)
                .map_err(|e| format!("설정 저장 실패 (관리자 권한 필요): {}", e))?;
            let state = if enable { "활성화 (자동 재부팅 방지)" } else { "기본값 복원" };
            Ok(format!("Windows 업데이트 후 자동 재부팅 방지 정책이 '{}'되었습니다.", state))
        }
        "take_ownership_menu" => {
            if enable {
                let (runas, _) = hkcr
                    .create_subkey(r"*\shell\runas")
                    .map_err(|e| format!("키 생성 실패 (관리자 권한 필요): {}", e))?;
                runas.set_value("", &"파일 소유권 가져오기 (Take Ownership)").map_err(|e| e.to_string())?;
                runas.set_value("NoWorkingDirectory", &"").map_err(|e| e.to_string())?;

                let (cmd, _) = runas.create_subkey("command").map_err(|e| e.to_string())?;
                let cmd_str = r#"cmd.exe /c takeown /f "%1" && icacls "%1" /grant administrators:F"#;
                cmd.set_value("", &cmd_str).map_err(|e| e.to_string())?;
                cmd.set_value("IsolatedCommand", &cmd_str).map_err(|e| e.to_string())?;

                notify_shell_change();
                Ok("우클릭 메뉴에 '파일 소유권 가져오기'가 등록되었습니다.".to_string())
            } else {
                if let Ok(shell) = hkcr.open_subkey_with_flags(r"*\shell", KEY_WRITE) {
                    let _ = shell.delete_subkey_all("runas");
                }
                notify_shell_change();
                Ok("우클릭 메뉴에서 '파일 소유권 가져오기'가 제거되었습니다.".to_string())
            }
        }
        _ => Err("알 수 없는 트윅 키입니다.".to_string()),
    }
}


