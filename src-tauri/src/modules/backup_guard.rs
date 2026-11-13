use crate::modules::common::app_data_subdir;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardTarget {
    pub id: String,
    pub path: String,
    pub name: String,
    pub enabled: bool,
    pub last_snapshot: Option<String>,
    #[serde(default)]
    pub is_folder: bool,
    #[serde(default)]
    pub snapshot_capable: bool,
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

thread_local! {
    static STORAGE_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

fn default_storage_dir() -> PathBuf {
    app_data_subdir("GuardSnapshots")
}

fn get_storage_dir() -> PathBuf {
    STORAGE_OVERRIDE.with(|slot| match slot.borrow().clone() {
        Some(dir) => dir,
        None => default_storage_dir(),
    })
}

fn get_config_file() -> PathBuf {
    get_storage_dir().join("targets.json")
}

fn refresh_target_capabilities(targets: &mut [GuardTarget]) {
    for target in targets.iter_mut() {
        let path = Path::new(&target.path);
        target.is_folder = path.is_dir();
        target.snapshot_capable = path.is_file();
    }
}

fn load_targets_checked() -> Result<Vec<GuardTarget>, String> {
    let config_file = get_config_file();
    if !config_file.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&config_file)
        .map_err(|e| format!("보호 대상 설정 파일 읽기 실패: {}", e))?;
    let mut targets: Vec<GuardTarget> = serde_json::from_str(&data)
        .map_err(|e| format!("보호 대상 설정 파일(targets.json)이 손상되었습니다: {}", e))?;
    refresh_target_capabilities(&mut targets);
    Ok(targets)
}

fn load_targets() -> Vec<GuardTarget> {
    load_targets_checked().unwrap_or_default()
}

fn save_targets(targets: &[GuardTarget]) -> Result<(), String> {
    let data = serde_json::to_string_pretty(targets).map_err(|e| e.to_string())?;
    fs::write(get_config_file(), data).map_err(|e| e.to_string())?;
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

    let mut targets = load_targets_checked()?;
    static TARGET_SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = TARGET_SEQ.fetch_add(1, Ordering::Relaxed);
    let id = format!("guard_{}_{}", Local::now().timestamp_millis(), seq);
    let display_name = if name.trim().is_empty() {
        p.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Target".to_string())
    } else {
        name
    };

    targets.push(GuardTarget {
        id,
        path,
        name: display_name,
        enabled: true,
        last_snapshot: None,
        is_folder: false,
        snapshot_capable: true,
    });

    save_targets(&targets)?;
    Ok("보호 대상 항목이 등록되었습니다.".to_string())
}

#[tauri::command]
pub fn remove_guard_target(id: String) -> Result<String, String> {
    let mut targets = load_targets_checked()?;
    let orig_len = targets.len();
    targets.retain(|t| t.id != id);
    if targets.len() < orig_len {
        save_targets(&targets)?;
        let prefix = format!("{}_", id);
        if let Ok(entries) = fs::read_dir(get_storage_dir()) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file()
                    && path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(&prefix))
                {
                    let _ = fs::remove_file(path);
                }
            }
        }
        Ok("보호 대상 항목이 제거되었습니다.".to_string())
    } else {
        Err("대상을 찾을 수 없습니다.".to_string())
    }
}

#[tauri::command]
pub fn create_target_snapshot(id: String) -> Result<SnapshotInfo, String> {
    let mut targets = load_targets_checked()?;
    let target_idx = targets
        .iter()
        .position(|t| t.id == id)
        .ok_or_else(|| "보호 대상을 찾을 수 없습니다.".to_string())?;

    let src_path = targets[target_idx].path.clone();
    let target_id = targets[target_idx].id.clone();
    let src = Path::new(&src_path);

    if src.is_dir() {
        return Err(
            "폴더는 스냅샷을 지원하지 않습니다. 보호 대상은 개별 파일만 등록할 수 있습니다."
                .to_string(),
        );
    }
    if !src.is_file() {
        return Err("원본 대상 파일이 존재하지 않습니다.".to_string());
    }

    let storage = get_storage_dir();
    let now = Local::now();
    let ts_str = format!(
        "{}_{:03}",
        now.format("%Y%m%d_%H%M%S"),
        now.timestamp_subsec_millis()
    );
    let snap_id = format!("{}_{}", target_id, ts_str);

    let dest_name = format!(
        "{}_{}",
        snap_id,
        src.file_name().unwrap_or_default().to_string_lossy()
    );
    let dest_path = storage.join(dest_name);

    fs::copy(src, &dest_path).map_err(|e| format!("파일 백업 복사 실패: {}", e))?;
    let total_size = fs::metadata(src).map(|m| m.len()).unwrap_or(0);

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
    let original_path = load_targets()
        .into_iter()
        .find(|t| t.id == target_id)
        .map(|t| t.path)
        .unwrap_or_default();

    if let Ok(entries) = fs::read_dir(storage) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            if fname.starts_with(&format!("{}_", target_id)) && path.is_file() {
                let metadata = match fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let timestamp = fname
                    .strip_prefix(&format!("{}_", target_id))
                    .and_then(|s| s.get(..15))
                    .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y%m%d_%H%M%S").ok())
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .or_else(|| {
                        metadata.modified().ok().map(|m| {
                            let dt: chrono::DateTime<Local> = m.into();
                            dt.format("%Y-%m-%d %H:%M:%S").to_string()
                        })
                    })
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

    let registered = load_targets().into_iter().any(|t| t.path == target_path);
    if !registered {
        return Err("등록되지 않은 보호 대상 경로로는 복원할 수 없습니다.".to_string());
    }

    let canonical_storage = get_storage_dir()
        .canonicalize()
        .map_err(|e| format!("스냅샷 저장소 경로를 확인할 수 없습니다: {}", e))?;
    let canonical_snap = snap
        .canonicalize()
        .map_err(|e| format!("스냅샷 백업 파일을 찾을 수 없습니다: {}", e))?;
    if !canonical_snap.starts_with(&canonical_storage) {
        return Err("스냅샷 저장소 밖의 파일은 복원할 수 없습니다.".to_string());
    }

    fs::copy(&canonical_snap, target).map_err(|e| format!("복원 파일 덮어쓰기 실패: {}", e))?;
    Ok(format!(
        "'{}' 파일이 스냅샷 버전으로 복원되었습니다.",
        target.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        dir: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let dir =
                std::env::temp_dir().join(format!("ctt_guard_{}_{}", std::process::id(), seq));
            fs::create_dir_all(&dir).unwrap();
            STORAGE_OVERRIDE.with(|slot| *slot.borrow_mut() = Some(dir.clone()));
            Self { dir }
        }

        fn write_targets(&self, targets: &[GuardTarget]) {
            let data = serde_json::to_string_pretty(targets).unwrap();
            fs::write(self.dir.join("targets.json"), data).unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            STORAGE_OVERRIDE.with(|slot| *slot.borrow_mut() = None);
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn legacy_target(path: &Path) -> GuardTarget {
        GuardTarget {
            id: "guard_test".to_string(),
            path: path.to_string_lossy().to_string(),
            name: "Legacy target".to_string(),
            enabled: true,
            last_snapshot: None,
            is_folder: false,
            snapshot_capable: false,
        }
    }

    #[test]
    fn folder_target_stays_visible_and_is_marked_unsupported() {
        let fixture = Fixture::new();
        let folder = fixture.dir.join("legacy_folder");
        fs::create_dir_all(&folder).unwrap();
        fixture.write_targets(&[legacy_target(&folder)]);

        let targets = get_guard_targets();

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id, "guard_test");
        assert!(targets[0].is_folder);
        assert!(!targets[0].snapshot_capable);
    }

    #[test]
    fn snapshot_on_folder_target_returns_specific_error() {
        let fixture = Fixture::new();
        let folder = fixture.dir.join("legacy_folder");
        fs::create_dir_all(&folder).unwrap();
        fixture.write_targets(&[legacy_target(&folder)]);

        let error = create_target_snapshot("guard_test".to_string()).unwrap_err();

        assert_eq!(
            error,
            "폴더는 스냅샷을 지원하지 않습니다. 보호 대상은 개별 파일만 등록할 수 있습니다."
        );
    }

    #[test]
    fn missing_target_is_visible_but_not_snapshot_capable() {
        let fixture = Fixture::new();
        let missing = fixture.dir.join("deleted_file.txt");
        fixture.write_targets(&[legacy_target(&missing)]);

        let targets = get_guard_targets();

        assert_eq!(targets.len(), 1);
        assert!(!targets[0].is_folder);
        assert!(!targets[0].snapshot_capable);
    }

    #[test]
    fn file_target_snapshot_and_list_round_trip() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fixture.write_targets(&[legacy_target(&source)]);

        let snapshot = create_target_snapshot("guard_test".to_string()).unwrap();
        assert_eq!(snapshot.file_size, 7);
        assert!(Path::new(&snapshot.snapshot_path).exists());

        let listed = list_snapshots("guard_test".to_string());
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].original_path, source.to_string_lossy());
        assert!(!listed[0].timestamp.is_empty());
    }

    #[test]
    fn snapshot_listing_falls_back_to_file_modification_time() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fixture.write_targets(&[legacy_target(&source)]);

        let malformed = fixture.dir.join("guard_test_not_a_timestamp_original.txt");
        let file = fs::File::create(&malformed).unwrap();
        let modified = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        file.set_times(fs::FileTimes::new().set_modified(modified))
            .unwrap();
        drop(file);

        let snapshots = list_snapshots("guard_test".to_string());

        assert_eq!(snapshots.len(), 1);
        assert_eq!(
            snapshots[0].timestamp,
            chrono::DateTime::<Local>::from(modified)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        );
    }

    #[test]
    fn restore_rejects_unregistered_target_path() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fixture.write_targets(&[legacy_target(&source)]);
        let snapshot = create_target_snapshot("guard_test".to_string()).unwrap();

        let error = restore_snapshot(
            snapshot.snapshot_path.clone(),
            fixture.dir.join("other.txt").to_string_lossy().to_string(),
        )
        .unwrap_err();

        assert_eq!(
            error,
            "등록되지 않은 보호 대상 경로로는 복원할 수 없습니다."
        );
    }

    #[test]
    fn restore_rejects_snapshot_outside_storage_directory() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fixture.write_targets(&[legacy_target(&source)]);
        let snapshot = create_target_snapshot("guard_test".to_string()).unwrap();

        let outside = std::env::temp_dir().join("ctt_outside_snapshot_probe.txt");
        fs::write(&outside, b"external").unwrap();

        let error = restore_snapshot(
            outside.to_string_lossy().to_string(),
            source.to_string_lossy().to_string(),
        )
        .unwrap_err();

        let _ = fs::remove_file(&outside);
        assert_eq!(error, "스냅샷 저장소 밖의 파일은 복원할 수 없습니다.");
        assert!(snapshot.snapshot_path.ends_with("original.txt"));
    }

    #[test]
    fn restore_writes_snapshot_back_to_registered_target() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fixture.write_targets(&[legacy_target(&source)]);
        let snapshot = create_target_snapshot("guard_test".to_string()).unwrap();

        fs::write(&source, b"changed").unwrap();
        restore_snapshot(
            snapshot.snapshot_path.clone(),
            source.to_string_lossy().to_string(),
        )
        .unwrap();

        assert_eq!(fs::read(&source).unwrap(), b"payload");
    }

    #[test]
    fn corrupted_targets_json_blocks_mutations_instead_of_overwriting() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fs::write(fixture.dir.join("targets.json"), b"{corrupted json").unwrap();

        let err =
            add_guard_target(source.to_string_lossy().to_string(), "test".to_string()).unwrap_err();
        assert!(err.contains("손상되었습니다"));
        assert_eq!(
            fs::read_to_string(fixture.dir.join("targets.json")).unwrap(),
            "{corrupted json"
        );
    }

    #[test]
    fn remove_guard_target_cleans_up_associated_snapshots() {
        let fixture = Fixture::new();
        let source = fixture.dir.join("original.txt");
        fs::write(&source, b"payload").unwrap();
        fixture.write_targets(&[legacy_target(&source)]);
        let snapshot = create_target_snapshot("guard_test".to_string()).unwrap();
        assert!(Path::new(&snapshot.snapshot_path).exists());

        remove_guard_target("guard_test".to_string()).unwrap();

        assert!(!Path::new(&snapshot.snapshot_path).exists());
        assert!(get_guard_targets().is_empty());
    }
}
