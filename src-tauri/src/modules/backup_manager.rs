use crate::modules::shell_tweaks::{
    get_shell_tweaks_state, set_hide_shortcut_arrow, set_show_drive_letters_first,
    set_show_file_extensions, set_show_hidden_files, set_special_folder_path,
    set_taskbar_search_mode, set_win11_classic_menu, ShellTweaksState,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    pub id: String,
    pub created_at: String,
    pub description: String,
    pub shell_state: ShellTweaksState,
}

fn get_restore_dir() -> PathBuf {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local).join("ClickToTweakModern").join("RestorePoints");
        let _ = fs::create_dir_all(&p);
        p
    } else {
        PathBuf::from("./restore_points")
    }
}

#[tauri::command]
pub fn create_restore_point(description: String) -> Result<RestorePoint, String> {
    let now = Local::now();
    let id = format!("rp_{}", now.format("%Y%m%d_%H%M%S"));
    let created_at = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let shell_state = get_shell_tweaks_state();

    let point = RestorePoint {
        id: id.clone(),
        created_at,
        description: if description.trim().is_empty() {
            "수동 생성된 복원 지점".to_string()
        } else {
            description
        },
        shell_state,
    };

    let dir = get_restore_dir();
    let file_path = dir.join(format!("{}.json", id));
    let data = serde_json::to_string_pretty(&point).map_err(|e| e.to_string())?;
    fs::write(file_path, data).map_err(|e| e.to_string())?;

    Ok(point)
}

#[tauri::command]
pub fn get_restore_points() -> Vec<RestorePoint> {
    let dir = get_restore_dir();
    let mut list = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(rp) = serde_json::from_str::<RestorePoint>(&content) {
                        list.push(rp);
                    }
                }
            }
        }
    }

    list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    list
}

#[tauri::command]
pub fn restore_from_point(id: String) -> Result<String, String> {
    let dir = get_restore_dir();
    let file_path = dir.join(format!("{}.json", id));

    if !file_path.exists() {
        return Err("선택한 복원 지점을 찾을 수 없습니다.".to_string());
    }

    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let rp: RestorePoint = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let s = rp.shell_state;
    let mut applied: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();

    let mut apply = |label: &str, result: Result<String, String>| match result {
        Ok(_) => applied.push(label.to_string()),
        Err(e) => failed.push(format!("{}: {}", label, e)),
    };

    apply(
        "클래식 우클릭 메뉴",
        set_win11_classic_menu(s.win11_classic_menu),
    );
    apply(
        "파일 확장자 표시",
        set_show_file_extensions(s.show_file_extensions),
    );
    apply("숨김 파일 표시", set_show_hidden_files(s.show_hidden_files));
    apply(
        "드라이브 문자 우선 표시",
        set_show_drive_letters_first(s.show_drive_letters_first),
    );
    apply(
        "바로가기 화살표 제거",
        set_hide_shortcut_arrow(s.hide_shortcut_arrow),
    );
    apply(
        "작업표시줄 검색 모드",
        set_taskbar_search_mode(s.taskbar_search_mode),
    );

    for folder in &s.special_folders {
        apply(
            &format!("특수 폴더 '{}'", folder.display_name),
            set_special_folder_path(folder.key_name.clone(), folder.current_path.clone()),
        );
    }

    if applied.is_empty() && !failed.is_empty() {
        return Err(format!(
            "복원 지점 '{}' ({}) 적용에 실패했습니다: {}",
            rp.description,
            rp.created_at,
            failed.join(" / ")
        ));
    }

    if failed.is_empty() {
        Ok(format!(
            "복원 지점 '{}' ({}) 상태로 시스템 설정이 {}개 항목 롤백되었습니다.",
            rp.description,
            rp.created_at,
            applied.len()
        ))
    } else {
        Ok(format!(
            "복원 지점 '{}' ({}) 부분 롤백: {}개 항목 적용됨, {}개 실패: {}",
            rp.description,
            rp.created_at,
            applied.len(),
            failed.len(),
            failed.join(" / ")
        ))
    }
}

