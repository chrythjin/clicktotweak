use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardTarget {
    pub id: String,
    pub path: String,
    pub name: String,
    pub enabled: bool,
    pub last_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,
    pub target_id: String,
    pub original_path: String,
    pub snapshot_path: String,
    pub timestamp: String,
    pub file_size: u64,
}

fn get_storage_dir() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local).join("ClickToTweakModern").join("GuardSnapshots");
        let _ = fs::create_dir_all(&p);
        p
    } else {
        PathBuf::from("./snapshots")
    }
}

fn get_config_file() -> PathBuf {
    let p = get_storage_dir().join("targets.json");
    p
}

fn load_targets() -> Vec<GuardTarget> {
    let file = get_config_file();
    if let Ok(data) = fs::read_to_string(file) {
        if let Ok(targets) = serde_json::from_str(&data) {
            return targets;
        }
    }
    Vec::new()
}

fn save_targets(targets: &[GuardTarget]) -> Result<(), String> {
    let file = get_config_file();
    let data = serde_json::to_string_pretty(targets).map_err(|e| e.to_string())?;
    fs::write(file, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_guard_targets() -> Vec<GuardTarget> {
    load_targets()
}

#[tauri::command]
pub fn add_guard_target(path: String, name: String) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("존재하지 않는 파일 또는 폴더 경로입니다.".to_string());
    }
    if !p.is_file() {
        return Err("현재는 파일만 보호 대상으로 등록할 수 있습니다.".to_string());
    }

    let mut targets = load_targets();
    let id = format!("guard_{}", chrono::Local::now().timestamp_millis());
    let display_name = if name.trim().is_empty() {
        p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Target".to_string())
    } else {
        name
    };

    targets.push(GuardTarget {
        id,
        path,
        name: display_name,
        enabled: true,
        last_snapshot: None,
    });

    save_targets(&targets)?;
    Ok("보호 대상 항목이 등록되었습니다.".to_string())
}

#[tauri::command]
pub fn remove_guard_target(id: String) -> Result<String, String> {
    let mut targets = load_targets();
    let orig_len = targets.len();
    targets.retain(|t| t.id != id);
    if targets.len() < orig_len {
        save_targets(&targets)?;
        Ok("보호 대상 항목이 제거되었습니다.".to_string())
    } else {
        Err("대상을 찾을 수 없습니다.".to_string())
    }
}

#[tauri::command]
pub fn create_target_snapshot(id: String) -> Result<SnapshotInfo, String> {
    let mut targets = load_targets();
    let target_idx = targets
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| "보호 대상을 찾을 수 없습니다.".to_string())?;

    let src_path = targets[target_idx].path.clone();
    let target_id = targets[target_idx].id.clone();
    let src = Path::new(&src_path);
    if !src.exists() {
        return Err("원본 대상 파일/폴더가 존재하지 않습니다.".to_string());
    }

    let storage = get_storage_dir();
    let now = Local::now();
    let ts_str = now.format("%Y%m%d_%H%M%S").to_string();
    let snap_id = format!("{}_{}", target_id, ts_str);

    let dest_name = format!("{}_{}", snap_id, src.file_name().unwrap_or_default().to_string_lossy());
    let dest_path = storage.join(dest_name);

    let total_size = if src.is_file() {
        fs::copy(src, &dest_path).map_err(|e| format!("파일 백업 복사 실패: {}", e))?;
        fs::metadata(src).map(|m| m.len()).unwrap_or(0)
    } else {
        return Err("현재는 단일 파일 스냅샷을 우선 지원합니다.".to_string());
    };

    let timestamp_display = now.format("%Y-%m-%d %H:%M:%S").to_string();
    targets[target_idx].last_snapshot = Some(timestamp_display.clone());
    save_targets(&targets)?;

    Ok(SnapshotInfo {
        id: snap_id,
        target_id,
        original_path: src_path,
        snapshot_path: dest_path.to_string_lossy().to_string(),
        timestamp: timestamp_display,
        file_size: total_size,
    })
}

#[tauri::command]
pub fn list_snapshots(target_id: String) -> Vec<SnapshotInfo> {
    let storage = get_storage_dir();
    let mut list = Vec::new();
    let original_path = load_targets().into_iter().find(|t| t.id == target_id)
        .map(|t| t.path).unwrap_or_default();

    if let Ok(entries) = fs::read_dir(storage) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            if fname.starts_with(&format!("{}_", target_id)) && path.is_file() {
                let metadata = match fs::metadata(&path) { Ok(m) => m, Err(_) => continue };
                let timestamp = fname.strip_prefix(&format!("{}_", target_id))
                    .and_then(|s| s.get(..15))
                    .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%d_%H%M%S").ok())
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default();
                list.push(SnapshotInfo {
                    id: fname.to_string(),
                    target_id: target_id.clone(),
                    original_path: original_path.clone(),
                    snapshot_path: path.to_string_lossy().to_string(),
                    timestamp,
                    file_size: metadata.len(),
                });
            }
        }
    }

    list
}

#[tauri::command]
pub fn restore_snapshot(snapshot_path: String, target_path: String) -> Result<String, String> {
    let snap = Path::new(&snapshot_path);
    let target = Path::new(&target_path);

    if !snap.exists() {
        return Err("스냅샷 백업 파일을 찾을 수 없습니다.".to_string());
    }

    fs::copy(snap, target).map_err(|e| format!("복원 파일 덮어쓰기 실패: {}", e))?;
    Ok(format!("'{}' 파일이 스냅샷 버전으로 복원되었습니다.", target.display()))
}
