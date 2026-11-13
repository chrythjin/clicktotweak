use crate::modules::common::{app_data_subdir, notify_shell_change, validate_safe_id};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use winreg::RegKey;
use winreg::enums::*;
use winreg::reg_value::RegValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopLayoutSnapshot {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredRegValue {
    pub bytes: Vec<u8>,
    #[serde(default)]
    pub reg_type: Option<u32>,
}

fn get_storage_dir() -> PathBuf {
    app_data_subdir("DesktopIcons")
}

const REG_BAGS_DESKTOP: &str = r"Software\Microsoft\Windows\Shell\Bags\1\Desktop";

fn is_managed_value(name: &str) -> bool {
    name.starts_with("ItemPos") || name.starts_with("IconLayouts")
}

fn validate_snapshot_id(id: &str) -> Result<(), String> {
    validate_safe_id(id).map_err(|_| "유효하지 않은 스냅샷 식별자입니다.".to_string())
}

fn reg_type_from_u32(value: u32) -> Result<RegType, String> {
    match value {
        v if v == REG_NONE as u32 => Ok(REG_NONE),
        v if v == REG_SZ as u32 => Ok(REG_SZ),
        v if v == REG_EXPAND_SZ as u32 => Ok(REG_EXPAND_SZ),
        v if v == REG_BINARY as u32 => Ok(REG_BINARY),
        v if v == REG_DWORD as u32 => Ok(REG_DWORD),
        v if v == REG_DWORD_BIG_ENDIAN as u32 => Ok(REG_DWORD_BIG_ENDIAN),
        v if v == REG_LINK as u32 => Ok(REG_LINK),
        v if v == REG_MULTI_SZ as u32 => Ok(REG_MULTI_SZ),
        v if v == REG_RESOURCE_LIST as u32 => Ok(REG_RESOURCE_LIST),
        v if v == REG_FULL_RESOURCE_DESCRIPTOR as u32 => Ok(REG_FULL_RESOURCE_DESCRIPTOR),
        v if v == REG_RESOURCE_REQUIREMENTS_LIST as u32 => Ok(REG_RESOURCE_REQUIREMENTS_LIST),
        v if v == REG_QWORD as u32 => Ok(REG_QWORD),
        _ => Err(format!("지원하지 않는 레지스트리 값 형식: {}", value)),
    }
}

fn parse_snapshot_dump(
    content: &str,
) -> Result<std::collections::HashMap<String, StoredRegValue>, serde_json::Error> {
    match serde_json::from_str::<std::collections::HashMap<String, StoredRegValue>>(content) {
        Ok(map) => Ok(map),
        Err(_) => {
            let legacy: std::collections::HashMap<String, Vec<u8>> = serde_json::from_str(content)?;
            Ok(legacy
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        StoredRegValue {
                            bytes: v,
                            reg_type: None,
                        },
                    )
                })
                .collect())
        }
    }
}

fn validate_snapshot_types(
    dump: &std::collections::HashMap<String, StoredRegValue>,
) -> Result<(), String> {
    for (name, stored) in dump {
        let value_type = stored.reg_type.ok_or_else(|| {
            format!(
                "이전 백업의 '{}' 값에는 형식 정보가 없어 안전하게 복원할 수 없습니다.",
                name
            )
        })?;
        reg_type_from_u32(value_type)?;
    }
    Ok(())
}

#[tauri::command]
pub fn backup_desktop_icon_layout(name: String) -> Result<DesktopLayoutSnapshot, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(REG_BAGS_DESKTOP)
        .map_err(|e| format!("바탕화면 레이아웃 레지스트리 읽기 실패: {}", e))?;

    let mut values_dump: std::collections::HashMap<String, StoredRegValue> =
        std::collections::HashMap::new();

    for (val_name, _) in key.enum_values().filter_map(|r| r.ok()) {
        if is_managed_value(&val_name)
            && let Ok(raw) = key.get_raw_value(&val_name)
        {
            values_dump.insert(
                val_name,
                StoredRegValue {
                    bytes: raw.bytes,
                    reg_type: Some(raw.vtype as u32),
                },
            );
        }
    }

    if values_dump.is_empty() {
        for (val_name, _) in key.enum_values().filter_map(|r| r.ok()) {
            if let Ok(raw) = key.get_raw_value(&val_name) {
                values_dump.insert(
                    val_name,
                    StoredRegValue {
                        bytes: raw.bytes,
                        reg_type: Some(raw.vtype as u32),
                    },
                );
            }
        }
    }

    let now = Local::now();
    let id = format!(
        "layout_{}_{}",
        now.format("%Y%m%d_%H%M%S"),
        now.timestamp_subsec_millis()
    );
    let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let display_name = if name.trim().is_empty() {
        format!("바탕화면 배치 백업 ({})", now.format("%m/%d %H:%M"))
    } else {
        name
    };

    let dir = get_storage_dir();
    let file_path = dir.join(format!("{}.json", id));
    let serialized = serde_json::to_string_pretty(&values_dump).map_err(|e| e.to_string())?;
    fs::write(&file_path, serialized).map_err(|e| e.to_string())?;

    // Also record metadata
    let snapshot = DesktopLayoutSnapshot {
        id,
        name: display_name,
        created_at,
        item_count: values_dump.len(),
    };

    let meta_path = dir.join(format!("{}.meta.json", snapshot.id));
    let meta_str = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
    if let Err(e) = fs::write(&meta_path, meta_str) {
        let _ = fs::remove_file(&file_path);
        return Err(format!("메타데이터 저장 실패: {}", e));
    }

    Ok(snapshot)
}

#[tauri::command]
pub fn list_desktop_icon_layouts() -> Vec<DesktopLayoutSnapshot> {
    let dir = get_storage_dir();
    let mut list = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            if fname.ends_with(".meta.json")
                && let Ok(content) = fs::read_to_string(&path)
                && let Ok(snap) = serde_json::from_str::<DesktopLayoutSnapshot>(&content)
            {
                list.push(snap);
            }
        }
    }

    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    list
}

#[tauri::command]
pub fn restore_desktop_icon_layout(id: String) -> Result<String, String> {
    validate_snapshot_id(&id)?;
    let dir = get_storage_dir();
    let file_path = dir.join(format!("{}.json", id));

    if !file_path.exists() {
        return Err("선택한 바탕화면 레이아웃 백업을 찾을 수 없습니다.".to_string());
    }

    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let dump = parse_snapshot_dump(&content).map_err(|e| e.to_string())?;
    validate_snapshot_types(&dump)?;
    let typed_values: Vec<_> = dump
        .iter()
        .map(|(name, stored)| {
            let value_type = stored.reg_type.ok_or_else(|| {
                format!(
                    "이전 백업의 '{}' 값에는 형식 정보가 없어 안전하게 복원할 수 없습니다.",
                    name
                )
            })?;
            Ok((name, stored, reg_type_from_u32(value_type)?))
        })
        .collect::<Result<_, String>>()?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_BAGS_DESKTOP)
        .map_err(|e| format!("레지스트리 키 생성 실패: {}", e))?;

    let mut restored = 0usize;
    let mut failed: Vec<String> = Vec::new();

    for (val_name, stored, reg_type) in typed_values {
        let reg_val = RegValue {
            bytes: stored.bytes.clone(),
            vtype: reg_type,
        };
        match key.set_raw_value(val_name, &reg_val) {
            Ok(()) => restored += 1,
            Err(e) => failed.push(format!("{}: {}", val_name, e)),
        }
    }

    notify_shell_change();

    if !failed.is_empty() {
        return Err(format!(
            "바탕화면 아이콘 위치 복원 중 {}개 값 적용에 실패했습니다: {}",
            failed.len(),
            failed.join(", ")
        ));
    }

    Ok(format!(
        "바탕화면 아이콘 위치 복원 값을 {}개 적용했습니다. 화면 반영에는 탐색기 재시작이 필요할 수 있습니다.",
        restored
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_snapshot_with_unknown_type_is_rejected() {
        let dump = parse_snapshot_dump(r#"{"Mode":[1,0,0,0]}"#).unwrap();
        assert_eq!(dump["Mode"].bytes, [1, 0, 0, 0]);
        assert!(validate_snapshot_types(&dump).is_err());
    }

    #[test]
    fn typed_snapshot_preserves_registry_types() {
        let dump = parse_snapshot_dump(
            r#"{"Mode":{"bytes":[1,0,0,0],"reg_type":4},"ItemPos":{"bytes":[1,2],"reg_type":3}}"#,
        )
        .unwrap();
        assert!(validate_snapshot_types(&dump).is_ok());
        assert_eq!(
            reg_type_from_u32(dump["Mode"].reg_type.unwrap()).unwrap(),
            REG_DWORD
        );
        assert_eq!(
            reg_type_from_u32(dump["ItemPos"].reg_type.unwrap()).unwrap(),
            REG_BINARY
        );
    }

    #[test]
    fn unsupported_registry_type_is_rejected() {
        let dump = parse_snapshot_dump(r#"{"Mode":{"bytes":[0],"reg_type":999}}"#).unwrap();
        assert!(validate_snapshot_types(&dump).is_err());
    }

    #[test]
    fn snapshot_id_rejects_path_traversal_segments() {
        assert!(validate_snapshot_id("layout_20261111_120000").is_ok());
        assert!(validate_snapshot_id("..\\..\\evil").is_err());
        assert!(validate_snapshot_id("layout/../../evil").is_err());
        assert!(validate_snapshot_id("").is_err());
        assert!(validate_snapshot_id("layout 2026").is_err());
    }
}
