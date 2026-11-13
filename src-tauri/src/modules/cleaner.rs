use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

struct CleanerCategoryDef {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    max_depth: usize,
    zero_byte_only: bool,
}

const CLEANER_CATEGORIES: &[CleanerCategoryDef] = &[
    CleanerCategoryDef {
        id: "user_temp",
        name: "사용자 임시 파일 (%TEMP%)",
        description: "응용 프로그램 실행 및 설치 시 생성된 잔여 캐시 및 임시 파일",
        max_depth: 3,
        zero_byte_only: false,
    },
    CleanerCategoryDef {
        id: "system_temp",
        name: "Windows 시스템 임시 파일",
        description: "C:\\Windows\\Temp 내의 시스템 업데이트 및 서비스 임시 파일",
        max_depth: 3,
        zero_byte_only: false,
    },
    CleanerCategoryDef {
        id: "crash_dumps",
        name: "충돌 덤프 및 오류 보고서",
        description: "프로그램 비정상 종료 시 기록된 CrashDump 및 미니덤프 파일",
        max_depth: 3,
        zero_byte_only: false,
    },
    CleanerCategoryDef {
        id: "browser_cache",
        name: "웹 브라우저 캐시 (Edge, Chrome, Whale)",
        description: "웹 서핑 시 축적된 이미지, 스크립트, 오프라인 미디어 캐시",
        max_depth: 3,
        zero_byte_only: false,
    },
    CleanerCategoryDef {
        id: "zero_byte_files",
        name: "0바이트 빈 파일",
        description: "사용자 임시 폴더에 남아 있는 용량 0의 불필요 파일",
        max_depth: 2,
        zero_byte_only: true,
    },
];

fn find_category_def(category_id: &str) -> Option<&'static CleanerCategoryDef> {
    CLEANER_CATEGORIES.iter().find(|def| def.id == category_id)
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
                let chrome = base.join(r"Google\Chrome\User Data\Default\Cache\Cache_Data");
                if chrome.exists() {
                    paths.push(chrome);
                }
                let edge = base.join(r"Microsoft\Edge\User Data\Default\Cache\Cache_Data");
                if edge.exists() {
                    paths.push(edge);
                }
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

fn collect_files_under_roots(
    roots: &[PathBuf],
    max_depth: usize,
    zero_byte_only: bool,
) -> Vec<(PathBuf, u64)> {
    let mut files = Vec::new();
    for root in roots {
        for entry in WalkDir::new(root)
            .min_depth(1)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if let Ok(meta) = entry.metadata()
                && meta.is_file()
            {
                let size = meta.len();
                if zero_byte_only && size != 0 {
                    continue;
                }
                files.push((entry.into_path(), size));
            }
        }
    }
    files
}

fn clean_file_list(files: Vec<(PathBuf, u64)>) -> CleanResult {
    let mut deleted_count = 0usize;
    let mut freed_bytes = 0u64;
    let mut skipped_count = 0usize;
    let mut seen: HashSet<PathBuf> = HashSet::new();

    for (path, size) in files {
        if !seen.insert(path.clone()) {
            continue;
        }
        if fs::remove_file(&path).is_ok() {
            deleted_count += 1;
            freed_bytes += size;
        } else {
            skipped_count += 1;
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

#[tauri::command(async)]
pub fn scan_cleaner_targets() -> Vec<CleanerCategoryScan> {
    CLEANER_CATEGORIES
        .iter()
        .map(|def| {
            let roots = get_category_paths(def.id);
            let files = collect_files_under_roots(&roots, def.max_depth, def.zero_byte_only);
            let file_count = files.len();
            let total_size_bytes = files.iter().map(|(_, size)| *size).sum();
            let sample_files = files
                .iter()
                .take(5)
                .map(|(path, _)| path.to_string_lossy().to_string())
                .collect();

            CleanerCategoryScan {
                category_id: def.id.to_string(),
                name: def.name.to_string(),
                description: def.description.to_string(),
                file_count,
                total_size_bytes,
                sample_files,
            }
        })
        .collect()
}

#[tauri::command(async)]
pub fn clean_selected_targets(category_ids: Vec<String>) -> CleanResult {
    let mut all_files = Vec::new();
    for cat_id in &category_ids {
        let Some(def) = find_category_def(cat_id) else {
            continue;
        };
        let roots = get_category_paths(def.id);
        all_files.extend(collect_files_under_roots(
            &roots,
            def.max_depth,
            def.zero_byte_only,
        ));
    }

    clean_file_list(all_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_root(tag: &str) -> PathBuf {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "ctt_cleaner_{}_{}_{}",
            tag,
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn zero_byte_collection_filters_non_empty_files() {
        let root = temp_root("zero");
        fs::write(root.join("empty.tmp"), b"").unwrap();
        fs::write(root.join("data.tmp"), b"abc").unwrap();

        let zero_only = collect_files_under_roots(std::slice::from_ref(&root), 2, true);
        let all = collect_files_under_roots(std::slice::from_ref(&root), 3, false);

        assert_eq!(zero_only.len(), 1);
        assert!(zero_only[0].0.ends_with("empty.tmp"));
        assert_eq!(all.len(), 2);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn clean_file_list_deduplicates_overlapping_categories() {
        let root = temp_root("dedup");
        let empty = root.join("empty.tmp");
        fs::write(&empty, b"").unwrap();

        let result = clean_file_list(vec![(empty.clone(), 0), (empty.clone(), 0)]);

        assert_eq!(result.deleted_files_count, 1);
        assert_eq!(result.skipped_files_count, 0);
        assert!(!empty.exists());

        let _ = fs::remove_dir_all(&root);
    }
}
