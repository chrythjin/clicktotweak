use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::reg_value::RegValue;
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub location: String, // "HKCU Run", "HKLM Run", "Startup Folder", "Disabled"
    pub is_enabled: bool,
    pub file_path: Option<String>,
}

const REG_HKCU_RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const REG_HKLM_RUN: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const REG_HKLM_RUN32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";
const REG_DISABLED: &str = r"Software\ClickToTweakModern\DisabledStartup";
const REG_HKCU_APPROVED: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";
const REG_HKLM_APPROVED: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";

const APPROVED_ENABLED: u8 = 0x02;
const APPROVED_DISABLED: u8 = 0x03;
const APPROVED_ENABLED_ALT: u8 = 0x06;
const APPROVED_DISABLED_ALT: u8 = 0x07;

fn approval_state(hive: &RegKey, approved_path: &str, subkey: &str, name: &str) -> Option<bool> {
    let key = hive.open_subkey(approved_path).ok()?;
    let sub = key.open_subkey(subkey).ok()?;
    let value = sub.get_raw_value(name).ok()?;
    let state = *value.bytes.first()?;
    match state {
        APPROVED_DISABLED | APPROVED_DISABLED_ALT => Some(false),
        APPROVED_ENABLED | APPROVED_ENABLED_ALT => Some(true),
        _ => None,
    }
}

fn set_approval_state(
    hive: &RegKey,
    approved_path: &str,
    subkey: &str,
    name: &str,
    enabled: bool,
) -> Result<(), String> {
    let (key, _) = hive
        .create_subkey(approved_path)
        .map_err(|e| format!("StartupApproved 키 생성 실패: {}", e))?;
    let (sub, _) = key
        .create_subkey(subkey)
        .map_err(|e| format!("StartupApproved\\{} 생성 실패: {}", subkey, e))?;

    let mut bytes = vec![0u8; 12];
    if let Ok(existing) = sub.get_raw_value(name) {
        if existing.vtype != REG_BINARY || existing.bytes.len() < 12 {
            return Err("StartupApproved 값 형식이 올바르지 않습니다.".to_string());
        }
        bytes = existing.bytes;
    }
    bytes[0] = if enabled { APPROVED_ENABLED } else { APPROVED_DISABLED };
    let value = RegValue {
        bytes,
        vtype: REG_BINARY,
    };
    sub.set_raw_value(name, &value)
        .map_err(|e| format!("StartupApproved 상태 저장 실패(관리자 권한 필요): {}", e))
}

fn get_user_startup_dir() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn get_common_startup_dir() -> Option<PathBuf> {
    if let Ok(programdata) = std::env::var("ProgramData") {
        let p = PathBuf::from(programdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

#[tauri::command]
pub fn get_startup_items() -> Vec<StartupItem> {
    let mut items = Vec::new();

    // 1. HKCU Run (StartupApproved 비활성 상태 반영)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(REG_HKCU_RUN) {
        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            let command = value.to_string();
            let is_enabled = approval_state(&hkcu, REG_HKCU_APPROVED, "Run", &name)
                .unwrap_or(true);
            items.push(StartupItem {
                id: format!("HKCU_RUN_{}", name),
                name: name.clone(),
                command,
                location: if is_enabled {
                    "HKCU Run".to_string()
                } else {
                    "HKCU Run (작업 관리자에서 비활성화됨)".to_string()
                },
                is_enabled,
                file_path: None,
            });
        }
    }

    // 2. HKLM Run
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(REG_HKLM_RUN) {
        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            let command = value.to_string();
            let is_enabled =
                approval_state(&hklm, REG_HKLM_APPROVED, "Run", &name).unwrap_or(true);
            items.push(StartupItem {
                id: format!("HKLM_RUN_{}", name),
                name: name.clone(),
                command,
                location: if is_enabled {
                    "HKLM Run".to_string()
                } else {
                    "HKLM Run (작업 관리자에서 비활성화됨)".to_string()
                },
                is_enabled,
                file_path: None,
            });
        }
    }

    // 2-1. HKLM Run (32비트 뷰)
    if let Ok(key) = hklm.open_subkey(REG_HKLM_RUN32) {
        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            let command = value.to_string();
            let is_enabled =
                approval_state(&hklm, REG_HKLM_APPROVED, "Run32", &name).unwrap_or(true);
            items.push(StartupItem {
                id: format!("HKLM_RUN32_{}", name),
                name: name.clone(),
                command,
                location: if is_enabled {
                    "HKLM Run (32비트)".to_string()
                } else {
                    "HKLM Run 32비트 (작업 관리자에서 비활성화됨)".to_string()
                },
                is_enabled,
                file_path: None,
            });
        }
    }

    // 3. User Startup Folder
    if let Some(user_dir) = get_user_startup_dir() {
        if let Ok(entries) = fs::read_dir(&user_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    items.push(StartupItem {
                        id: format!("FOLDER_USER_{}", file_name),
                        name: file_name,
                        command: path.to_string_lossy().to_string(),
                        location: "User Startup Folder".to_string(),
                        is_enabled: true,
                        file_path: Some(path.to_string_lossy().to_string()),
                    });
                }
            }
        }
    }

    // 4. Common Startup Folder
    if let Some(common_dir) = get_common_startup_dir() {
        if let Ok(entries) = fs::read_dir(&common_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    items.push(StartupItem {
                        id: format!("FOLDER_COMMON_{}", file_name),
                        name: file_name,
                        command: path.to_string_lossy().to_string(),
                        location: "Common Startup Folder".to_string(),
                        is_enabled: true,
                        file_path: Some(path.to_string_lossy().to_string()),
                    });
                }
            }
        }
    }

    // 5. Disabled items in ClickToTweak Modern backup registry
    if let Ok(key) = hkcu.open_subkey(REG_DISABLED) {
        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            let raw = value.to_string();
            // format: "ORIG_LOC|||COMMAND"
            let parts: Vec<&str> = raw.split("|||").collect();
            let (orig_loc, command) = if parts.len() >= 2 {
                (parts[0], parts[1])
            } else {
                ("Unknown", raw.as_str())
            };

            items.push(StartupItem {
                id: format!("DISABLED_{}", name),
                name: name.clone(),
                command: command.to_string(),
                location: format!("비활성화됨 ({})", orig_loc),
                is_enabled: false,
                file_path: None,
            });
        }
    }

    items
}

#[tauri::command]
pub fn toggle_startup_item(id: String, enable: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if id.starts_with("FOLDER_") {
        return Err(
            "시작 폴더 바로가기는 비활성화 대신 삭제 또는 이동을 권장합니다.".to_string(),
        );
    }

    if enable {
        if let Some(name) = id.strip_prefix("DISABLED_") {
            let dis_key = hkcu
                .open_subkey_with_flags(REG_DISABLED, KEY_READ | KEY_WRITE)
                .map_err(|e| format!("비활성 저장소 열기 실패: {}", e))?;

            let raw_val: String = dis_key
                .get_value(name)
                .map_err(|e| format!("비활성 항목 조회 실패: {}", e))?;

            let parts: Vec<&str> = raw_val.split("|||").collect();
            let orig_loc = if parts.len() >= 2 { parts[0] } else { "HKCU Run" };
            let command = if parts.len() >= 2 { parts[1] } else { raw_val.as_str() };

            let mut warning = String::new();
            match orig_loc {
                "HKCU Run" => {
                    let (run_key, _) = hkcu
                        .create_subkey(REG_HKCU_RUN)
                        .map_err(|e| format!("HKCU Run 열기 실패: {}", e))?;
                    run_key
                        .set_value(name, &command)
                        .map_err(|e| format!("시작프로그램 활성화 실패: {}", e))?;
                    if let Err(e) = set_approval_state(&hkcu, REG_HKCU_APPROVED, "Run", name, true) {
                        warning = format!(" (작업 관리자 상태 동기화 실패: {})", e);
                    }
                }
                "HKLM Run" => {
                    let (run_key, _) = hklm
                        .create_subkey(REG_HKLM_RUN)
                        .map_err(|e| format!("HKLM Run 열기 실패(관리자 권한 필요): {}", e))?;
                    run_key
                        .set_value(name, &command)
                        .map_err(|e| format!("시작프로그램 활성화 실패: {}", e))?;
                    if let Err(e) = set_approval_state(&hklm, REG_HKLM_APPROVED, "Run", name, true) {
                        warning = format!(" (작업 관리자 상태 동기화 실패: {})", e);
                    }
                }
                "HKLM Run 32비트" => {
                    let (run_key, _) = hklm
                        .create_subkey(REG_HKLM_RUN32)
                        .map_err(|e| format!("HKLM Run 32비트 열기 실패(관리자 권한 필요): {}", e))?;
                    run_key
                        .set_value(name, &command)
                        .map_err(|e| format!("시작프로그램 활성화 실패: {}", e))?;
                    if let Err(e) = set_approval_state(&hklm, REG_HKLM_APPROVED, "Run32", name, true)
                    {
                        warning = format!(" (작업 관리자 상태 동기화 실패: {})", e);
                    }
                }
                other => {
                    return Err(format!("알 수 없는 시작프로그램 위치입니다: {}", other));
                }
            }

            if !warning.is_empty() {
                return Err(format!("Run 값은 복원했지만 활성화에 실패했습니다. 백업을 보존합니다.{}", warning));
            }
            dis_key.delete_value(name).map_err(|e| format!("비활성 백업 삭제 실패: {}", e))?;
            return Ok(format!("'{}' 항목이 다시 활성화되었습니다.{}", name, warning));
        }

        let (name, hive, approved_path, approved_sub) = if let Some(n) = id.strip_prefix("HKCU_RUN_")
        {
            (n, &hkcu, REG_HKCU_APPROVED, "Run")
        } else if let Some(n) = id.strip_prefix("HKLM_RUN32_") {
            (n, &hklm, REG_HKLM_APPROVED, "Run32")
        } else if let Some(n) = id.strip_prefix("HKLM_RUN_") {
            (n, &hklm, REG_HKLM_APPROVED, "Run")
        } else {
            return Err("알 수 없는 시작프로그램 항목입니다.".to_string());
        };
        set_approval_state(hive, approved_path, approved_sub, name, true)?;
        return Ok(format!("'{}' 항목이 활성화되었습니다.", name));
    }

    let (name, hive, approved_path, approved_sub) = if let Some(n) = id.strip_prefix("HKCU_RUN_") {
        (n, &hkcu, REG_HKCU_APPROVED, "Run")
    } else if let Some(n) = id.strip_prefix("HKLM_RUN32_") {
        (n, &hklm, REG_HKLM_APPROVED, "Run32")
    } else if let Some(n) = id.strip_prefix("HKLM_RUN_") {
        (n, &hklm, REG_HKLM_APPROVED, "Run")
    } else {
        return Err("알 수 없는 시작프로그램 항목입니다.".to_string());
    };
    set_approval_state(hive, approved_path, approved_sub, name, false)?;
    Ok(format!("'{}' 항목이 비활성화되었습니다.", name))
}


#[tauri::command]
pub fn delete_startup_item(id: String) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if id.starts_with("HKCU_RUN_") {
        let name = &id["HKCU_RUN_".len()..];
        let run_key = hkcu
            .open_subkey_with_flags(REG_HKCU_RUN, KEY_WRITE)
            .map_err(|e| format!("HKCU Run 열기 실패: {}", e))?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        Ok(format!("'{}' 항목이 삭제되었습니다.", name))
    } else if id.starts_with("HKLM_RUN32_") {
        let name = &id["HKLM_RUN32_".len()..];
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let run_key = hklm.open_subkey_with_flags(REG_HKLM_RUN32, KEY_WRITE)
            .map_err(|e| format!("HKLM Run 32비트 열기 실패: {}", e))?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        Ok(format!("'{}' 항목이 삭제되었습니다.", name))
    } else if id.starts_with("HKLM_RUN_") {
        let name = &id["HKLM_RUN_".len()..];
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let run_key = hklm
            .open_subkey_with_flags(REG_HKLM_RUN, KEY_WRITE)
            .map_err(|e| format!("HKLM Run 열기 실패 (관리자 권한 필요): {}", e))?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        Ok(format!("'{}' 항목이 삭제되었습니다.", name))
    } else if id.starts_with("DISABLED_") {
        let name = &id["DISABLED_".len()..];
        let dis_key = hkcu
            .open_subkey_with_flags(REG_DISABLED, KEY_WRITE)
            .map_err(|e| format!("비활성 저장소 열기 실패: {}", e))?;
        dis_key.delete_value(name).map_err(|e| e.to_string())?;
        Ok(format!("비활성 항목 '{}'이 삭제되었습니다.", name))
    } else if id.starts_with("FOLDER_USER_") || id.starts_with("FOLDER_COMMON_") {
        // file removal
        let file_path = if id.starts_with("FOLDER_USER_") {
            let name = &id["FOLDER_USER_".len()..];
            get_user_startup_dir().map(|d| d.join(name))
        } else {
            let name = &id["FOLDER_COMMON_".len()..];
            get_common_startup_dir().map(|d| d.join(name))
        };

        if let Some(p) = file_path {
            if p.exists() {
                fs::remove_file(&p).map_err(|e| format!("파일 삭제 실패: {}", e))?;
                return Ok(format!("'{}' 바로가기가 삭제되었습니다.", p.display()));
            }
        }
        Err("파일을 찾을 수 없습니다.".to_string())
    } else {
        Err("알 수 없는 시작프로그램 항목입니다.".to_string())
    }
}

#[tauri::command]
pub fn add_startup_item(name: String, command: String) -> Result<String, String> {
    if name.trim().is_empty() || command.trim().is_empty() {
        return Err("이름과 실행 명령어를 모두 입력해주세요.".to_string());
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_HKCU_RUN)
        .map_err(|e| format!("HKCU Run 레지스트리 생성 실패: {}", e))?;
    key.set_value(&name, &command)
        .map_err(|e| format!("시작프로그램 등록 실패: {}", e))?;

    Ok(format!("'{}' 항목이 시작프로그램으로 등록되었습니다.", name))
}
