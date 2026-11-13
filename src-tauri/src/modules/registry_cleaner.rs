use crate::modules::common::expand_env_vars;
use serde::{Deserialize, Serialize};
use std::path::Path;
use winreg::enums::*;
use winreg::{HKEY, RegKey};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryJunkItem {
    pub id: String,
    pub category: String,
    pub key_path: String,
    pub value_name: String,
    pub target_path: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryCleanResult {
    pub cleaned_count: usize,
    pub failed_count: usize,
    pub message: String,
}

/// 따옴표·인자·환경변수를 정리해 실제 파일 시스템 경로 후보를 추출한다.
/// 상대 경로(예: `notepad.exe`)나 리소스 참조(`@%SystemRoot%\...,-1`)는 None을 반환해 오탐을 막는다.
fn resolve_registry_file_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('@') {
        return None;
    }

    let extracted = if let Some(after_open) = trimmed.strip_prefix('"') {
        after_open.split('"').next().unwrap_or("").trim()
    } else {
        trimmed
    };

    if extracted.is_empty() {
        return None;
    }

    let expanded = expand_env_vars(extracted);
    let candidate = expanded.trim_matches('"').trim();
    if candidate.is_empty() {
        return None;
    }

    // 정의되지 않은 환경변수가 남아 있으면 존재 여부를 단정할 수 없으므로 건너뛴다.
    if candidate.contains('%') {
        return None;
    }

    let path = Path::new(candidate);
    if !path.is_absolute() {
        return None;
    }

    Some(candidate.to_string())
}

// 1. App Paths 무효 항목 검사
fn scan_app_paths(hk: HKEY, root_name: &str, items: &mut Vec<RegistryJunkItem>) {
    let root = RegKey::predef(hk);
    let app_paths_key =
        match root.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths") {
            Ok(k) => k,
            Err(_) => return,
        };

    for subkey_name in app_paths_key.enum_keys().filter_map(|k| k.ok()) {
        if let Ok(sub) = app_paths_key.open_subkey(&subkey_name) {
            let path_val: String = match sub.get_value("") {
                Ok(v) => v,
                Err(_) => continue,
            };

            let Some(clean_path) = resolve_registry_file_path(&path_val) else {
                continue;
            };

            if !Path::new(&clean_path).exists() {
                items.push(RegistryJunkItem {
                    id: format!("{}\\App Paths\\{}", root_name, subkey_name),
                    category: "무효한 프로그램 경로 (App Paths)".to_string(),
                    key_path: format!(
                        r"{}\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{}",
                        root_name, subkey_name
                    ),
                    value_name: subkey_name.clone(),
                    target_path: clean_path.clone(),
                    description: format!(
                        "파일이 존재하지 않는 무효한 실행 파일 경로: {}",
                        clean_path
                    ),
                });
            }
        }
    }
}

// 2. Shared DLLs 무효 항목 검사
fn scan_shared_dlls(items: &mut Vec<RegistryJunkItem>) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match hklm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs") {
        Ok(k) => k,
        Err(_) => return,
    };

    for (name, _) in key.enum_values().filter_map(|v| v.ok()) {
        let Some(clean_path) = resolve_registry_file_path(&name) else {
            continue;
        };

        if !Path::new(&clean_path).exists() {
            items.push(RegistryJunkItem {
                id: format!("SharedDLLs_{}", name),
                category: "누락된 공유 DLL (Shared DLLs)".to_string(),
                key_path: r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs".to_string(),
                value_name: name.clone(),
                target_path: clean_path.clone(),
                description: format!("실제 디스크에 없는 누락된 공유 DLL 참조: {}", clean_path),
            });
        }
    }
}

// 3. MuiCache 잔여 항목 검사
fn scan_mui_cache(items: &mut Vec<RegistryJunkItem>) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu
        .open_subkey(r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache")
    {
        Ok(k) => k,
        Err(_) => return,
    };

    for (name, _) in key.enum_values().filter_map(|v| v.ok()) {
        let path_candidate = name
            .strip_suffix(".FriendlyAppName")
            .or_else(|| name.strip_suffix(".ApplicationCompany"))
            .unwrap_or(&name);

        let Some(clean_path) = resolve_registry_file_path(path_candidate) else {
            continue;
        };

        if clean_path.len() > 3 && !Path::new(&clean_path).exists() {
            items.push(RegistryJunkItem {
                id: format!("MuiCache_{}", name),
                category: "MUI 캐시 잔여 항목".to_string(),
                key_path: r"HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache".to_string(),
                value_name: name.clone(),
                target_path: clean_path.clone(),
                description: format!("삭제된 프로그램의 MUI 캐시: {}", clean_path),
            });
        }
    }
}

#[tauri::command(async)]
pub fn scan_registry_junk() -> Result<Vec<RegistryJunkItem>, String> {
    let mut items = Vec::new();

    scan_app_paths(HKEY_CURRENT_USER, "HKCU", &mut items);
    scan_app_paths(HKEY_LOCAL_MACHINE, "HKLM", &mut items);
    scan_shared_dlls(&mut items);
    scan_mui_cache(&mut items);

    Ok(items)
}

fn delete_subkey_in(hive: &RegKey, parent_path: &str, subkey: &str) -> bool {
    hive.open_subkey_with_flags(parent_path, KEY_WRITE)
        .and_then(|parent| parent.delete_subkey_all(subkey))
        .is_ok()
}

fn delete_value_in(hive: &RegKey, key_path: &str, value_name: &str) -> bool {
    hive.open_subkey_with_flags(key_path, KEY_WRITE)
        .and_then(|key| key.delete_value(value_name))
        .is_ok()
}

#[tauri::command(async)]
pub fn clean_registry_junk(item_ids: Vec<String>) -> Result<RegistryCleanResult, String> {
    let mut cleaned = 0;
    let mut failed = 0;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    const APP_PATHS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths";
    const SHARED_DLLS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs";
    const MUI_CACHE: &str =
        r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache";

    for id in item_ids {
        let ok = if let Some(subkey) = id.strip_prefix("HKCU\\App Paths\\") {
            delete_subkey_in(&hkcu, APP_PATHS, subkey)
        } else if let Some(subkey) = id.strip_prefix("HKLM\\App Paths\\") {
            delete_subkey_in(&hklm, APP_PATHS, subkey)
        } else if let Some(val_name) = id.strip_prefix("SharedDLLs_") {
            delete_value_in(&hklm, SHARED_DLLS, val_name)
        } else if let Some(val_name) = id.strip_prefix("MuiCache_") {
            delete_value_in(&hkcu, MUI_CACHE, val_name)
        } else {
            false
        };

        if ok {
            cleaned += 1;
        } else {
            failed += 1;
        }
    }

    let message = if failed == 0 {
        format!("무효한 레지스트리 항목 {}개를 정리했습니다.", cleaned)
    } else {
        format!(
            "레지스트리 항목 {}개를 정리했고, {}개는 정리하지 못했습니다. (관리자 권한이 필요할 수 있습니다)",
            cleaned, failed
        )
    };

    Ok(RegistryCleanResult {
        cleaned_count: cleaned,
        failed_count: failed,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_registry_path_expands_env_vars_and_quotes() {
        unsafe { std::env::set_var("CTT_REG_ROOT", r"C:\Program Files\SampleApp") };
        assert_eq!(
            resolve_registry_file_path(r#""%CTT_REG_ROOT%\app.exe" --silent"#).as_deref(),
            Some(r"C:\Program Files\SampleApp\app.exe")
        );
    }

    #[test]
    fn resolve_registry_path_ignores_relative_resource_and_unknown_env() {
        assert_eq!(resolve_registry_file_path("notepad.exe"), None);
        assert_eq!(
            resolve_registry_file_path("@%SystemRoot%\\system32\\shell32.dll,-123"),
            None
        );
        assert_eq!(
            resolve_registry_file_path(r"%CTT_UNKNOWN_ENV_999%\app.exe"),
            None
        );
    }

    #[test]
    fn cleaning_unknown_item_is_reported_as_failure_not_success() {
        let result = clean_registry_junk(vec!["__ctt_unknown_prefix_item__".to_string()]).unwrap();

        assert_eq!(result.cleaned_count, 0);
        assert_eq!(result.failed_count, 1);
        assert!(result.message.contains("정리하지 못했습니다"));
    }

    #[test]
    fn cleaning_empty_selection_reports_nothing() {
        let result = clean_registry_junk(Vec::new()).unwrap();

        assert_eq!(result.cleaned_count, 0);
        assert_eq!(result.failed_count, 0);
    }

    #[test]
    fn app_paths_entry_is_removed_with_its_default_value() {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let app_paths = match hkcu.open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
            KEY_WRITE,
        ) {
            Ok(key) => key,
            Err(_) => return,
        };

        let name = "__ctt_delete_probe__";
        if app_paths.create_subkey(name).is_err() {
            return;
        }
        if let Ok(sub) = app_paths.open_subkey_with_flags(name, KEY_WRITE) {
            let _ = sub.set_value("", &r"C:\ctt\missing\probe.exe");
        }

        let id = format!("HKCU\\App Paths\\{}", name);
        let result = clean_registry_junk(vec![id]).unwrap();

        assert_eq!(result.cleaned_count, 1);
        assert_eq!(result.failed_count, 0);
        assert!(app_paths.open_subkey(name).is_err());
    }
}
