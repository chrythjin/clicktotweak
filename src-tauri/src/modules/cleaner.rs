use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerCategoryScan {
    pub category_id: String,
    pub name: String,
    pub description: String,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub sample_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanResult {
    pub deleted_files_count: usize,
    pub freed_bytes: u64,
    pub skipped_files_count: usize,
    pub message: String,
}

fn get_category_paths(category_id: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    match category_id {
        "user_temp" => {
            if let Ok(temp) = std::env::var("TEMP") {
                paths.push(PathBuf::from(temp));
            }
        }
        "system_temp" => {
            let p = PathBuf::from(r"C:\Windows\Temp");
            if p.exists() {
                paths.push(p);
            }
        }
        "crash_dumps" => {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                let p = PathBuf::from(local).join("CrashDumps");
                if p.exists() {
                    paths.push(p);
                }
            }
            let mini = PathBuf::from(r"C:\Windows\Minidump");
            if mini.exists() {
                paths.push(mini);
            }
        }
        "browser_cache" => {
            if let Ok(local) = std::env::var("LOCALAPPDATA") {
                let base = PathBuf::from(&local);
                // Chrome
                let chrome = base.join(r"Google\Chrome\User Data\Default\Cache\Cache_Data");
                if chrome.exists() {
                    paths.push(chrome);
                }
                // Edge
                let edge = base.join(r"Microsoft\Edge\User Data\Default\Cache\Cache_Data");
                if edge.exists() {
                    paths.push(edge);
                }
                // Whale
                let whale = base.join(r"Naver\Naver Whale\User Data\Default\Cache\Cache_Data");
                if whale.exists() {
                    paths.push(whale);
                }
            }
        }
        "zero_byte_files" => {
            // Downloads는 사용자가 의도적으로 만든 0바이트 파일을 포함할 수 있어 제외한다
            if let Ok(temp) = std::env::var("TEMP") {
                paths.push(PathBuf::from(temp));
            }
        }
        _ => {}
    }

    paths
}

#[tauri::command(async)]
pub fn scan_cleaner_targets() -> Vec<CleanerCategoryScan> {
    let categories = [
        (
            "user_temp",
            "사용자 임시 파일 (%TEMP%)",
            "응용 프로그램 실행 및 설치 시 생성된 잔여 캐시 및 임시 파일",
        ),
        (
            "system_temp",
            "Windows 시스템 임시 파일",
            "C:\\Windows\\Temp 내의 시스템 업데이트 및 서비스 임시 파일",
        ),
        (
            "crash_dumps",
            "충돌 덤프 및 오류 보고서",
            "프로그램 비정상 종료 시 기록된 CrashDump 및 미니덤프 파일",
        ),
        (
            "browser_cache",
            "웹 브라우저 캐시 (Edge, Chrome, Whale)",
            "웹 서핑 시 축적된 이미지, 스크립트, 오프라인 미디어 캐시",
        ),
        (
            "zero_byte_files",
            "0바이트 빈 파일",
            "사용자 임시 폴더에 남아 있는 용량 0의 불필요 파일",
        ),
    ];

    let mut results = Vec::new();

    for (cat_id, cat_name, cat_desc) in categories {
        let target_paths = get_category_paths(cat_id);
        let mut file_count = 0usize;
        let mut total_size = 0u64;
        let mut samples = Vec::new();

        for root in target_paths {
            if cat_id == "zero_byte_files" {
                // look only for files with 0 length
                for entry in WalkDir::new(&root)
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() && meta.len() == 0 {
                            file_count += 1;
                            if samples.len() < 5 {
                                samples.push(entry.path().to_string_lossy().to_string());
                            }
                        }
                    }
                }
            } else {
                for entry in WalkDir::new(&root)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            file_count += 1;
                            total_size += meta.len();
                            if samples.len() < 5 {
                                samples.push(entry.path().to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }

        results.push(CleanerCategoryScan {
            category_id: cat_id.to_string(),
            name: cat_name.to_string(),
            description: cat_desc.to_string(),
            file_count,
            total_size_bytes: total_size,
            sample_files: samples,
        });
    }

    results
}

#[tauri::command(async)]
pub fn clean_selected_targets(category_ids: Vec<String>) -> CleanResult {
    let mut deleted_count = 0usize;
    let mut freed_bytes = 0u64;
    let mut skipped_count = 0usize;

    for cat_id in &category_ids {
        let target_paths = get_category_paths(cat_id);

        for root in target_paths {
            if cat_id == "zero_byte_files" {
                for entry in WalkDir::new(&root)
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() && meta.len() == 0 {
                            if fs::remove_file(entry.path()).is_ok() {
                                deleted_count += 1;
                            } else {
                                skipped_count += 1;
                            }
                        }
                    }
                }
            } else {
                // Delete files inside root (do not delete root folder itself)
                for entry in WalkDir::new(&root)
                    .min_depth(1)
                    .max_depth(3)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let p = entry.path();
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            let size = meta.len();
                            if fs::remove_file(p).is_ok() {
                                deleted_count += 1;
                                freed_bytes += size;
                            } else {
                                skipped_count += 1; // Locked by running app
                            }
                        }
                    }
                }
            }
        }
    }

    let mb = freed_bytes as f64 / (1024.0 * 1024.0);
    CleanResult {
        deleted_files_count: deleted_count,
        freed_bytes,
        skipped_files_count: skipped_count,
        message: format!(
            "총 {}개의 불필요한 파일이 정리되어 {:.2} MB의 디스크 공간을 확보했습니다. (사용 중인 파일 {}개 제외)",
            deleted_count, mb, skipped_count
        ),
    }
}
