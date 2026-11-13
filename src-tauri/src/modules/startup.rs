use crate::modules::common::validate_single_filename;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use winreg::RegKey;
use winreg::enums::*;
use winreg::reg_value::RegValue;

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

const LOC_HKCU_RUN: &str = "HKCU Run";
const LOC_HKLM_RUN: &str = "HKLM Run";
const LOC_HKLM_RUN32: &str = "HKLM Run 32비트";

fn approval_state_from_bytes(bytes: &[u8]) -> Option<bool> {
    match *bytes.first()? {
        APPROVED_DISABLED | APPROVED_DISABLED_ALT => Some(false),
        APPROVED_ENABLED | APPROVED_ENABLED_ALT => Some(true),
        _ => None,
    }
}

fn approval_bytes(enabled: bool, existing: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut bytes = match existing {
        Some(current) => {
            if current.len() < 12 {
                return Err("StartupApproved 값 형식이 올바르지 않습니다.".to_string());
            }
            current.to_vec()
        }
        None => vec![0u8; 12],
    };
    bytes[0] = if enabled {
        APPROVED_ENABLED
    } else {
        APPROVED_DISABLED
    };
    Ok(bytes)
}

fn approval_state(hive: &RegKey, approved_path: &str, subkey: &str, name: &str) -> Option<bool> {
    let key = hive.open_subkey(approved_path).ok()?;
    let sub = key.open_subkey(subkey).ok()?;
    let value = sub.get_raw_value(name).ok()?;
    if value.vtype != REG_BINARY {
        return None;
    }
    approval_state_from_bytes(&value.bytes)
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

    let existing = match sub.get_raw_value(name) {
        Ok(current) => {
            if current.vtype != REG_BINARY {
                return Err("StartupApproved 값 형식이 올바르지 않습니다.".to_string());
            }
            Some(current.bytes)
        }
        Err(_) => None,
    };

    let bytes = approval_bytes(enabled, existing.as_deref())?;
    let value = RegValue {
        bytes,
        vtype: REG_BINARY,
    };
    sub.set_raw_value(name, &value)
        .map_err(|e| format!("StartupApproved 상태 저장 실패(관리자 권한 필요): {}", e))
}

/// Run 항목을 비활성 격리 저장소로 옮긴다.
/// 원본 명령을 `위치|||명령` 형식으로 저장하고 Run 키에서 값을 제거한다.
fn isolate_startup_entry(
    hive: &RegKey,
    run_path: &str,
    location: &str,
    name: &str,
) -> Result<(), String> {
    let run_key = hive
        .open_subkey_with_flags(run_path, KEY_READ | KEY_WRITE)
        .map_err(|e| format!("{} 열기 실패(관리자 권한 필요): {}", location, e))?;

    let command: String = run_key
        .get_value(name)
        .map_err(|e| format!("'{}' 실행 명령을 읽지 못했습니다: {}", name, e))?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (store, _) = hkcu
        .create_subkey(REG_DISABLED)
        .map_err(|e| format!("비활성 저장소 생성 실패: {}", e))?;

    match store.get_value::<String, _>(name) {
        Ok(existing) => {
            let existing_location = existing.split("|||").next().unwrap_or("");
            return Err(format!(
                "'{}' 항목은 이미 {} 위치에서 격리되어 있습니다. 기존 백업을 먼저 확인해 주세요.",
                name, existing_location
            ));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("비활성 저장소 조회 실패: {}", e)),
    }

    store
        .set_value(name, &format!("{}|||{}", location, command))
        .map_err(|e| format!("비활성 저장소 저장 실패: {}", e))?;

    if let Err(e) = run_key.delete_value(name) {
        let _ = store.delete_value(name);
        return Err(format!("Run 값 삭제 실패: {}", e));
    }

    Ok(())
}

fn get_user_startup_dir() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn get_common_startup_dir() -> Option<PathBuf> {
    if let Ok(programdata) = std::env::var("ProgramData") {
        let p = PathBuf::from(programdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
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
            let is_enabled = approval_state(&hkcu, REG_HKCU_APPROVED, "Run", &name).unwrap_or(true);
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
            let is_enabled = approval_state(&hklm, REG_HKLM_APPROVED, "Run", &name).unwrap_or(true);
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
    if let Some(user_dir) = get_user_startup_dir()
        && let Ok(entries) = fs::read_dir(&user_dir)
    {
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

    // 4. Common Startup Folder
    if let Some(common_dir) = get_common_startup_dir()
        && let Ok(entries) = fs::read_dir(&common_dir)
    {
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

    // 5. Disabled items in ClickToTweak Modern backup registry
    if let Ok(key) = hkcu.open_subkey(REG_DISABLED) {
        for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
            let raw = value.to_string();
            // format: "ORIG_LOC|||COMMAND"
            let parts: Vec<&str> = raw.splitn(2, "|||").collect();
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
        return Err("시작 폴더 바로가기는 비활성화 대신 삭제 또는 이동을 권장합니다.".to_string());
    }

    if enable {
        if let Some(name) = id.strip_prefix("DISABLED_") {
            let dis_key = hkcu
                .open_subkey_with_flags(REG_DISABLED, KEY_READ | KEY_WRITE)
                .map_err(|e| format!("비활성 저장소 열기 실패: {}", e))?;

            let raw_val: String = dis_key
                .get_value(name)
                .map_err(|e| format!("비활성 항목 조회 실패: {}", e))?;

            let parts: Vec<&str> = raw_val.splitn(2, "|||").collect();
            let orig_loc = if parts.len() >= 2 {
                parts[0]
            } else {
                "HKCU Run"
            };
            let command = if parts.len() >= 2 {
                parts[1]
            } else {
                raw_val.as_str()
            };

            let mut warning = String::new();
            match orig_loc {
                "HKCU Run" => {
                    let (run_key, _) = hkcu
                        .create_subkey(REG_HKCU_RUN)
                        .map_err(|e| format!("HKCU Run 열기 실패: {}", e))?;
                    run_key
                        .set_value(name, &command)
                        .map_err(|e| format!("시작프로그램 활성화 실패: {}", e))?;
                    if let Err(e) = set_approval_state(&hkcu, REG_HKCU_APPROVED, "Run", name, true)
                    {
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
                    if let Err(e) = set_approval_state(&hklm, REG_HKLM_APPROVED, "Run", name, true)
                    {
                        warning = format!(" (작업 관리자 상태 동기화 실패: {})", e);
                    }
                }
                "HKLM Run 32비트" | "HKLM Run (32비트)" => {
                    let (run_key, _) = hklm.create_subkey(REG_HKLM_RUN32).map_err(|e| {
                        format!("HKLM Run 32비트 열기 실패(관리자 권한 필요): {}", e)
                    })?;
                    run_key
                        .set_value(name, &command)
                        .map_err(|e| format!("시작프로그램 활성화 실패: {}", e))?;
                    if let Err(e) =
                        set_approval_state(&hklm, REG_HKLM_APPROVED, "Run32", name, true)
                    {
                        warning = format!(" (작업 관리자 상태 동기화 실패: {})", e);
                    }
                }
                other => {
                    return Err(format!("알 수 없는 시작프로그램 위치입니다: {}", other));
                }
            }

            if !warning.is_empty() {
                return Err(format!(
                    "Run 값은 복원했지만 활성화에 실패했습니다. 백업을 보존합니다.{}",
                    warning
                ));
            }
            dis_key
                .delete_value(name)
                .map_err(|e| format!("비활성 백업 삭제 실패: {}", e))?;
            return Ok(format!(
                "'{}' 항목이 다시 활성화되었습니다.{}",
                name, warning
            ));
        }

        let (name, hive, approved_path, approved_sub) =
            if let Some(n) = id.strip_prefix("HKCU_RUN_") {
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

    let (name, hive, run_path, location) = if let Some(n) = id.strip_prefix("HKCU_RUN_") {
        (n, &hkcu, REG_HKCU_RUN, LOC_HKCU_RUN)
    } else if let Some(n) = id.strip_prefix("HKLM_RUN32_") {
        (n, &hklm, REG_HKLM_RUN32, LOC_HKLM_RUN32)
    } else if let Some(n) = id.strip_prefix("HKLM_RUN_") {
        (n, &hklm, REG_HKLM_RUN, LOC_HKLM_RUN)
    } else {
        return Err("알 수 없는 시작프로그램 항목입니다.".to_string());
    };
    isolate_startup_entry(hive, run_path, location, name)?;
    Ok(format!(
        "'{}' 항목이 격리 저장소로 옮겨져 비활성화되었습니다.",
        name
    ))
}

fn delete_approval_state(hive: &RegKey, approved_path: &str, subkey: &str, name: &str) {
    if let Ok(key) = hive.open_subkey(approved_path)
        && let Ok(sub) = key.open_subkey_with_flags(subkey, KEY_WRITE)
    {
        let _ = sub.delete_value(name);
    }
}

#[tauri::command]
pub fn delete_startup_item(id: String) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Some(name) = id.strip_prefix("HKCU_RUN_") {
        let run_key = hkcu
            .open_subkey_with_flags(REG_HKCU_RUN, KEY_WRITE)
            .map_err(|e| format!("HKCU Run 열기 실패: {}", e))?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        delete_approval_state(&hkcu, REG_HKCU_APPROVED, "Run", name);
        Ok(format!("'{}' 항목이 삭제되었습니다.", name))
    } else if let Some(name) = id.strip_prefix("HKLM_RUN32_") {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let run_key = hklm
            .open_subkey_with_flags(REG_HKLM_RUN32, KEY_WRITE)
            .map_err(|e| format!("HKLM Run 32비트 열기 실패: {}", e))?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        delete_approval_state(&hklm, REG_HKLM_APPROVED, "Run32", name);
        Ok(format!("'{}' 항목이 삭제되었습니다.", name))
    } else if let Some(name) = id.strip_prefix("HKLM_RUN_") {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let run_key = hklm
            .open_subkey_with_flags(REG_HKLM_RUN, KEY_WRITE)
            .map_err(|e| format!("HKLM Run 열기 실패 (관리자 권한 필요): {}", e))?;
        run_key.delete_value(name).map_err(|e| e.to_string())?;
        delete_approval_state(&hklm, REG_HKLM_APPROVED, "Run", name);
        Ok(format!("'{}' 항목이 삭제되었습니다.", name))
    } else if let Some(name) = id.strip_prefix("DISABLED_") {
        let dis_key = hkcu
            .open_subkey_with_flags(REG_DISABLED, KEY_WRITE)
            .map_err(|e| format!("비활성 저장소 열기 실패: {}", e))?;
        dis_key.delete_value(name).map_err(|e| e.to_string())?;
        Ok(format!("비활성 항목 '{}'이 삭제되었습니다.", name))
    } else if let Some(name) = id.strip_prefix("FOLDER_USER_") {
        validate_single_filename(name)?;
        let file_path = get_user_startup_dir().map(|d| d.join(name));
        if let Some(p) = file_path
            && p.exists()
        {
            fs::remove_file(&p).map_err(|e| format!("파일 삭제 실패: {}", e))?;
            return Ok(format!("'{}' 바로가기가 삭제되었습니다.", p.display()));
        }
        Err("파일을 찾을 수 없습니다.".to_string())
    } else if let Some(name) = id.strip_prefix("FOLDER_COMMON_") {
        validate_single_filename(name)?;
        let file_path = get_common_startup_dir().map(|d| d.join(name));
        if let Some(p) = file_path
            && p.exists()
        {
            fs::remove_file(&p).map_err(|e| format!("파일 삭제 실패: {}", e))?;
            return Ok(format!("'{}' 바로가기가 삭제되었습니다.", p.display()));
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

    Ok(format!(
        "'{}' 항목이 시작프로그램으로 등록되었습니다.",
        name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_byte_maps_to_enabled_state() {
        assert_eq!(approval_state_from_bytes(&[APPROVED_ENABLED]), Some(true));
        assert_eq!(
            approval_state_from_bytes(&[APPROVED_ENABLED_ALT]),
            Some(true)
        );
    }

    #[test]
    fn approval_byte_maps_to_disabled_state() {
        assert_eq!(approval_state_from_bytes(&[APPROVED_DISABLED]), Some(false));
        assert_eq!(
            approval_state_from_bytes(&[APPROVED_DISABLED_ALT]),
            Some(false)
        );
    }

    #[test]
    fn unknown_or_empty_approval_bytes_are_not_interpreted() {
        assert_eq!(approval_state_from_bytes(&[0x05]), None);
        assert_eq!(approval_state_from_bytes(&[]), None);
    }

    #[test]
    fn approval_bytes_creates_twelve_byte_value_when_absent() {
        let bytes = approval_bytes(true, None).unwrap();
        assert_eq!(bytes.len(), 12);
        assert_eq!(bytes[0], APPROVED_ENABLED);
    }

    #[test]
    fn approval_bytes_preserves_trailing_bytes_of_existing_value() {
        let mut existing = vec![0u8; 12];
        existing[0] = APPROVED_DISABLED;
        existing[5] = 0xAB;

        let bytes = approval_bytes(true, Some(&existing)).unwrap();

        assert_eq!(bytes[0], APPROVED_ENABLED);
        assert_eq!(bytes[5], 0xAB);
        assert_eq!(bytes.len(), 12);
    }

    #[test]
    fn approval_bytes_rejects_short_existing_value() {
        assert!(approval_bytes(true, Some(&[0u8; 6])).is_err());
    }

    #[test]
    fn approval_bytes_writes_disabled_marker() {
        let bytes = approval_bytes(false, None).unwrap();
        assert_eq!(bytes[0], APPROVED_DISABLED);
        assert_eq!(approval_state_from_bytes(&bytes), Some(false));
    }

    #[test]
    fn isolation_moves_run_value_into_disabled_store() {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let probe = "__ctt_isolation_probe__";
        let command = "cmd /c echo probe";

        let (run_key, _) = match hkcu.create_subkey(REG_HKCU_RUN) {
            Ok(pair) => pair,
            Err(_) => return,
        };
        if run_key.set_value(probe, &command).is_err() {
            return;
        }

        let result = isolate_startup_entry(&hkcu, REG_HKCU_RUN, LOC_HKCU_RUN, probe);

        let stored = hkcu
            .open_subkey(REG_DISABLED)
            .ok()
            .and_then(|store| store.get_value::<String, _>(probe).ok());
        let left_in_run = run_key.get_value::<String, _>(probe).ok();

        let _ = run_key.delete_value(probe);
        if let Ok(store) = hkcu.open_subkey_with_flags(REG_DISABLED, KEY_WRITE) {
            let _ = store.delete_value(probe);
        }

        assert!(result.is_ok(), "격리 실패: {:?}", result);
        assert!(left_in_run.is_none(), "Run 키에 값이 남아 있습니다.");
        assert_eq!(
            stored.unwrap_or_default(),
            format!("{}|||{}", LOC_HKCU_RUN, command)
        );
    }

    #[test]
    fn delete_folder_startup_item_rejects_path_traversal() {
        assert!(delete_startup_item("FOLDER_USER_..\\evil.lnk".to_string()).is_err());
        assert!(delete_startup_item("FOLDER_COMMON_sub/evil.lnk".to_string()).is_err());
    }
}
