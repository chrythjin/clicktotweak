use serde::{Deserialize, Serialize};
use std::path::Path;
use winreg::enums::*;
use winreg::{RegKey, HKEY};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegistryJunkItem {
    pub id: String,
    pub category: String, // "무효한 응용프로그램 경로(App Paths)", "누락된 공유 라이브러리(Shared DLLs)", "MUI 캐시 찌꺼기"
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

// 1. App Paths 무효 항목 검사
fn scan_app_paths(hk: HKEY, root_name: &str, items: &mut Vec<RegistryJunkItem>) {
    let root = RegKey::predef(hk);
    let app_paths_key = match root.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths") {
        Ok(k) => k,
        Err(_) => return,
    };

    for subkey_name in app_paths_key.enum_keys().filter_map(|k| k.ok()) {
        if let Ok(sub) = app_paths_key.open_subkey(&subkey_name) {
            // 기본값 (경로)
            let path_val: String = match sub.get_value("") {
                Ok(v) => v,
                Err(_) => continue,
            };

            // 따옴표 제거
            let clean_path = path_val.trim_matches('"').trim();
            if clean_path.is_empty() {
                continue;
            }

            // 파일이 존재하는지 검사
            let p = Path::new(clean_path);
            if !p.exists() {
                items.push(RegistryJunkItem {
                    id: format!("{}\\App Paths\\{}", root_name, subkey_name),
                    category: "무효한 프로그램 경로 (App Paths)".to_string(),
                    key_path: format!(r"{}\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\{}", root_name, subkey_name),
                    value_name: subkey_name.clone(),
                    target_path: clean_path.to_string(),
                    description: format!("파일이 존재하지 않는 무효한 실행 파일 경로: {}", clean_path),
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
        let clean_path = name.trim_matches('"').trim();
        if clean_path.is_empty() {
            continue;
        }

        let p = Path::new(clean_path);
        if !p.exists() {
            items.push(RegistryJunkItem {
                id: format!("SharedDLLs_{}", name),
                category: "누락된 공유 DLL (Shared DLLs)".to_string(),
                key_path: r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs".to_string(),
                value_name: name.clone(),
                target_path: clean_path.to_string(),
                description: format!("실제 디스크에 없는 누락된 공유 DLL 참조: {}", clean_path),
            });
        }
    }
}

// 3. MuiCache 잔여 항목 검사
fn scan_mui_cache(items: &mut Vec<RegistryJunkItem>) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = match hkcu.open_subkey(r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache") {
        Ok(k) => k,
        Err(_) => return,
    };

    for (name, _) in key.enum_values().filter_map(|v| v.ok()) {
        // MuiCache의 값 이름은 "C:\Path\To\file.exe.FriendlyAppName" 형태이거나 경로 자체
        let path_candidate = if let Some(idx) = name.find(".FriendlyAppName") {
            &name[..idx]
        } else if let Some(idx) = name.find(".ApplicationCompany") {
            &name[..idx]
        } else {
            &name
        };

        let clean_path = path_candidate.trim_matches('"').trim();
        if clean_path.len() > 3 && clean_path.contains(":\\") {
            let p = Path::new(clean_path);
            if !p.exists() {
                items.push(RegistryJunkItem {
                    id: format!("MuiCache_{}", name),
                    category: "MUI 캐시 잔여 항목".to_string(),
                    key_path: r"HKCU\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache".to_string(),
                    value_name: name.clone(),
                    target_path: clean_path.to_string(),
                    description: format!("삭제된 프로그램의 MUI 캐시: {}", clean_path),
                });
            }
        }
    }
}

#[tauri::command]
pub fn scan_registry_junk() -> Result<Vec<RegistryJunkItem>, String> {
    let mut items = Vec::new();

    scan_app_paths(HKEY_CURRENT_USER, "HKCU", &mut items);
    scan_app_paths(HKEY_LOCAL_MACHINE, "HKLM", &mut items);
    scan_shared_dlls(&mut items);
    scan_mui_cache(&mut items);

    Ok(items)
}

#[tauri::command]
pub fn clean_registry_junk(item_ids: Vec<String>) -> Result<RegistryCleanResult, String> {
    let mut cleaned = 0;
    let mut failed = 0;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    for id in item_ids {
        if id.starts_with("HKCU\\App Paths\\") {
            let subkey = id.trim_start_matches("HKCU\\App Paths\\");
            if let Ok(app_paths) = hkcu.open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
                KEY_WRITE,
            ) {
                if app_paths.delete_subkey(subkey).is_ok() {
                    cleaned += 1;
                } else {
                    failed += 1;
                }
            }
        } else if id.starts_with("HKLM\\App Paths\\") {
            let subkey = id.trim_start_matches("HKLM\\App Paths\\");
            if let Ok(app_paths) = hklm.open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
                KEY_WRITE,
            ) {
                if app_paths.delete_subkey(subkey).is_ok() {
                    cleaned += 1;
                } else {
                    failed += 1;
                }
            }
        } else if let Some(val_name) = id.strip_prefix("SharedDLLs_") {
            if let Ok(key) = hklm.open_subkey_with_flags(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs",
                KEY_WRITE,
            ) {
                if key.delete_value(val_name).is_ok() {
                    cleaned += 1;
                } else {
                    failed += 1;
                }
            }
        } else if let Some(val_name) = id.strip_prefix("MuiCache_") {
            if let Ok(key) = hkcu.open_subkey_with_flags(
                r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache",
                KEY_WRITE,
            ) {
                if key.delete_value(val_name).is_ok() {
                    cleaned += 1;
                } else {
                    failed += 1;
                }
            }
        }
    }

    Ok(RegistryCleanResult {
        cleaned_count: cleaned,
        failed_count: failed,
        message: format!("무효한 레지스트리 항목 {}개를 성공적으로 정리했습니다.", cleaned),
    })
}
