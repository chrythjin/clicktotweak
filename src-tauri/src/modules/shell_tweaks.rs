use crate::modules::common::{expand_env_vars, hidden_command, notify_shell_change};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use winreg::RegKey;
use winreg::enums::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellTweaksState {
    pub win11_classic_menu: bool,
    pub show_file_extensions: bool,
    pub show_hidden_files: bool,
    pub show_drive_letters_first: bool,
    pub hide_shortcut_arrow: bool,
    #[serde(default)]
    pub animations_disabled: bool,
    pub taskbar_search_mode: u32, // 0: hidden, 1: icon, 2: search box, 3: icon + label
    pub special_folders: Vec<SpecialFolderInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialFolderInfo {
    pub key_name: String,
    pub display_name: String,
    pub current_path: String,
}

const REG_EXPLORER_ADVANCED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const REG_EXPLORER: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
const REG_USER_SHELL_FOLDERS: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";
const REG_SHELL_FOLDERS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders";
const REG_SEARCH: &str = r"Software\Microsoft\Windows\CurrentVersion\Search";
const REG_WIN11_CLASSIC_MENU: &str =
    r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";
const REG_SHELL_ICONS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";

const SPECIAL_FOLDER_KEYS: &[(&str, &str)] = &[
    ("Desktop", "바탕 화면"),
    ("{374DE290-123F-4565-9164-39C4925E467B}", "다운로드"),
    ("Personal", "내 문서"),
    ("My Pictures", "내 사진"),
    ("My Music", "내 음악"),
    ("My Video", "내 동영상"),
];

fn special_folder_display_name(key_name: &str) -> Option<&'static str> {
    SPECIAL_FOLDER_KEYS
        .iter()
        .find(|(key, _)| *key == key_name)
        .map(|(_, display)| *display)
}

fn is_within(child: &Path, parent: &Path) -> bool {
    let child_parts = path_parts(child);
    let parent_parts = path_parts(parent);

    if child_parts.len() < parent_parts.len() {
        return false;
    }
    child_parts[..parent_parts.len()] == parent_parts[..]
}

fn path_parts(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

fn unique_destination(dest_dir: &Path, name: &str) -> PathBuf {
    let base = dest_dir.join(name);
    if !base.exists() {
        return base;
    }
    let (stem, ext) = match name.rfind('.') {
        Some(idx) if idx > 0 => (&name[..idx], &name[idx..]),
        _ => (name, ""),
    };
    for suffix in 1..10_000 {
        let candidate = dest_dir.join(format!("{} ({}){}", stem, suffix, ext));
        if !candidate.exists() {
            return candidate;
        }
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    dest_dir.join(format!("{} ({}){}", stem, nanos, ext))
}

#[derive(Debug, Default, PartialEq)]
struct MoveOutcome {
    moved: usize,
    renamed: usize,
    bytes: u64,
    transferred: Vec<(PathBuf, PathBuf)>,
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    fs::create_dir_all(dest).map_err(|e| format!("폴더 생성 실패 ({}): {}", dest.display(), e))?;
    for entry in fs::read_dir(source).map_err(|e| format!("폴더 읽기 실패: {}", e))? {
        let entry = entry.map_err(|e| format!("항목 읽기 실패: {}", e))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("항목 종류 확인 실패: {}", e))?;
        let target = dest.join(entry.file_name());
        if file_type.is_dir() {
            total += copy_dir_recursive(&entry.path(), &target)?;
        } else {
            let size = fs::metadata(entry.path()).map(|m| m.len()).unwrap_or(0);
            fs::copy(entry.path(), &target)
                .map_err(|e| format!("파일 복사 실패 ({}): {}", entry.path().display(), e))?;
            total += size;
        }
    }
    Ok(total)
}

fn path_size(path: &Path) -> u64 {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if !metadata.is_dir() {
        return metadata.len();
    }
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| path_size(&e.path()))
                .sum()
        })
        .unwrap_or(0)
}

fn move_entry(source: &Path, dest: &Path) -> Result<u64, String> {
    let size = path_size(source);
    match fs::rename(source, dest) {
        Ok(()) => Ok(size),
        Err(_) => {
            if source.is_dir() {
                copy_dir_recursive(source, dest)?;
                fs::remove_dir_all(source)
                    .map_err(|e| format!("원본 폴더 삭제 실패 ({}): {}", source.display(), e))?;
            } else {
                fs::copy(source, dest)
                    .map_err(|e| format!("파일 복사 실패 ({}): {}", source.display(), e))?;
                fs::remove_file(source)
                    .map_err(|e| format!("원본 파일 삭제 실패 ({}): {}", source.display(), e))?;
            }
            Ok(size)
        }
    }
}

fn move_directory_contents(source: &Path, dest: &Path) -> Result<MoveOutcome, String> {
    let mut outcome = MoveOutcome::default();
    if !source.is_dir() {
        return Ok(outcome);
    }
    fs::create_dir_all(dest).map_err(|e| format!("대상 폴더 생성 실패: {}", e))?;

    let entries: Vec<PathBuf> = fs::read_dir(source)
        .map_err(|e| format!("원본 폴더 읽기 실패: {}", e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    for entry in entries {
        let name = match entry.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };
        let target = unique_destination(dest, &name);
        let renamed = match (target.file_name(), entry.file_name()) {
            (Some(t), Some(s)) => t != s,
            _ => false,
        };
        let bytes = move_entry(&entry, &target)?;
        outcome.moved += 1;
        outcome.bytes += bytes;
        if renamed {
            outcome.renamed += 1;
        }
        outcome.transferred.push((entry, target));
    }
    Ok(outcome)
}

fn read_shell_folder_path(folders: &RegKey, key_name: &str) -> Option<PathBuf> {
    let raw: String = folders.get_value(key_name).ok()?;
    let expanded = expand_env_vars(&raw);
    if expanded.trim().is_empty() {
        return None;
    }
    Some(PathBuf::from(expanded))
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}

#[tauri::command]
pub fn get_shell_tweaks_state() -> ShellTweaksState {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // 1. Win11 classic menu
    let win11_classic_menu = hkcu.open_subkey(REG_WIN11_CLASSIC_MENU).is_ok();

    // 2. HideFileExt (값이 없으면 Windows 기본값인 "확장자 표시"로 간주)
    let show_file_extensions = if let Ok(adv) = hkcu.open_subkey(REG_EXPLORER_ADVANCED) {
        let val: Result<u32, _> = adv.get_value("HideFileExt");
        match val {
            Ok(value) => value == 0,
            Err(_) => true,
        }
    } else {
        true
    };

    // 3. Hidden
    let show_hidden_files = if let Ok(adv) = hkcu.open_subkey(REG_EXPLORER_ADVANCED) {
        let val: Result<u32, _> = adv.get_value("Hidden");
        val.map(|v| v == 1).unwrap_or(false)
    } else {
        false
    };

    // 4. ShowDriveLettersFirst
    let show_drive_letters_first = if let Ok(exp) = hkcu.open_subkey(REG_EXPLORER) {
        let val: Result<u32, _> = exp.get_value("ShowDriveLettersFirst");
        val.map(|v| v == 4).unwrap_or(false)
    } else {
        false
    };

    // 5. Hide shortcut arrow
    let hide_shortcut_arrow = if let Ok(icons) = hklm.open_subkey(REG_SHELL_ICONS) {
        icons.get_value::<String, _>("29").is_ok()
    } else {
        false
    };

    // 6. MinAnimate (창 최소화/최대화 시각 애니메이션)
    let animations_disabled = hkcu
        .open_subkey(r"Control Panel\Desktop\WindowMetrics")
        .ok()
        .and_then(|k| k.get_value::<String, _>("MinAnimate").ok())
        .is_some_and(|v| v.trim() == "0");

    // 7. SearchboxTaskbarMode
    let taskbar_search_mode = if let Ok(srch) = hkcu.open_subkey(REG_SEARCH) {
        srch.get_value::<u32, _>("SearchboxTaskbarMode")
            .unwrap_or(1)
    } else {
        1
    };

    // 8. Special folders
    let mut special_folders = Vec::new();
    if let Ok(folders) = hkcu.open_subkey(REG_USER_SHELL_FOLDERS) {
        for (k, d) in SPECIAL_FOLDER_KEYS {
            if let Ok(p) = folders.get_value::<String, _>(k) {
                special_folders.push(SpecialFolderInfo {
                    key_name: k.to_string(),
                    display_name: d.to_string(),
                    current_path: p,
                });
            }
        }
    }

    ShellTweaksState {
        win11_classic_menu,
        show_file_extensions,
        show_hidden_files,
        show_drive_letters_first,
        hide_shortcut_arrow,
        animations_disabled,
        taskbar_search_mode,
        special_folders,
    }
}

#[tauri::command]
pub fn set_win11_classic_menu(enable: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let clsid_root = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";

    if enable {
        let (key, _) = hkcu
            .create_subkey(REG_WIN11_CLASSIC_MENU)
            .map_err(|e| format!("레지스트리 생성 실패: {}", e))?;
        key.set_value("", &"")
            .map_err(|e| format!("값 설정 실패: {}", e))?;
        notify_shell_change();
        Ok("Windows 11 클래식 우클릭 메뉴가 활성화되었습니다. (탐색기 재시작 권장)".to_string())
    } else {
        match hkcu.delete_subkey_all(clsid_root) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("클래식 우클릭 메뉴 제거 실패: {}", e)),
        }
        notify_shell_change();
        Ok("Windows 11 기본 우클릭 메뉴로 복원되었습니다. (탐색기 재시작 권장)".to_string())
    }
}
#[tauri::command]
pub fn set_show_file_extensions(show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_EXPLORER_ADVANCED)
        .map_err(|e| e.to_string())?;
    let val: u32 = if show { 0 } else { 1 };
    key.set_value("HideFileExt", &val)
        .map_err(|e| e.to_string())?;
    notify_shell_change();
    Ok(if show {
        "알려진 파일 확장자가 항상 표시되도록 설정되었습니다."
    } else {
        "알려진 파일 확장자가 숨겨지도록 설정되었습니다."
    }
    .to_string())
}

#[tauri::command]
pub fn set_show_hidden_files(show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_EXPLORER_ADVANCED)
        .map_err(|e| e.to_string())?;
    let val: u32 = if show { 1 } else { 2 };
    key.set_value("Hidden", &val).map_err(|e| e.to_string())?;
    notify_shell_change();
    Ok(if show {
        "숨김 파일 및 폴더가 표시되도록 설정되었습니다."
    } else {
        "숨김 파일 및 폴더가 숨겨지도록 설정되었습니다."
    }
    .to_string())
}

#[tauri::command]
pub fn set_show_drive_letters_first(show: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(REG_EXPLORER)
        .map_err(|e| e.to_string())?;
    let val: u32 = if show { 4 } else { 0 };
    key.set_value("ShowDriveLettersFirst", &val)
        .map_err(|e| e.to_string())?;
    notify_shell_change();
    Ok(if show {
        "내 PC에서 드라이브 문자가 이름보다 먼저 표시됩니다."
    } else {
        "드라이브 문자가 기본 형태로 표시됩니다."
    }
    .to_string())
}

#[tauri::command]
pub fn set_taskbar_search_mode(mode: u32) -> Result<String, String> {
    if mode > 3 {
        return Err("작업표시줄 검색 모드는 0~3 범위여야 합니다.".to_string());
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(REG_SEARCH).map_err(|e| e.to_string())?;
    key.set_value("SearchboxTaskbarMode", &mode)
        .map_err(|e| e.to_string())?;
    notify_shell_change();
    Ok("작업표시줄 검색 모드가 변경되었습니다.".to_string())
}

#[tauri::command]
pub fn set_hide_shortcut_arrow(hide: bool) -> Result<String, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if hide {
        let (key, _) = hklm
            .create_subkey(REG_SHELL_ICONS)
            .map_err(|e| format!("관리자 권한이 필요합니다: {}", e))?;
        key.set_value("29", &r"%windir%\System32\shell32.dll,-50")
            .map_err(|e| format!("설정 실패: {}", e))?;
        notify_shell_change();
        Ok("바로가기 화살표 아이콘이 제거되었습니다. (탐색기 재시작 권장)".to_string())
    } else {
        if let Ok(key) = hklm.open_subkey_with_flags(REG_SHELL_ICONS, KEY_WRITE) {
            match key.delete_value("29") {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("바로가기 화살표 설정 제거 실패: {}", e)),
            }
        }
        notify_shell_change();
        Ok("바로가기 화살표 아이콘이 기본값으로 복원되었습니다.".to_string())
    }
}

/// 특수 폴더 경로를 레지스트리에만 반영한다(파일 이동 없음).
/// 설정 되돌리기(복원 지점 롤백)처럼 파일을 건드리지 않고 경로만 되돌려야 할 때 사용한다.
pub fn apply_special_folder_path(key_name: &str, new_path: &str) -> Result<String, String> {
    let Some(display_name) = special_folder_display_name(key_name) else {
        return Err("지원하지 않는 특수 폴더 항목입니다.".to_string());
    };

    if new_path.trim().is_empty() {
        return Err("유효한 폴더 경로를 입력해주세요.".to_string());
    }

    let target_raw = expand_env_vars(new_path.trim());
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (folders, _) = hkcu
        .create_subkey(REG_USER_SHELL_FOLDERS)
        .map_err(|e| format!("레지스트리 키 열기 실패: {}", e))?;

    folders
        .set_value(key_name, &target_raw)
        .map_err(|e| format!("'{}' 경로 변경 실패: {}", display_name, e))?;

    if let Ok(legacy) = hkcu.open_subkey_with_flags(REG_SHELL_FOLDERS, KEY_WRITE) {
        let _ = legacy.set_value(key_name, &target_raw);
    }

    notify_shell_change();
    Ok(format!(
        "'{}' 경로가 '{}'로 되돌려졌습니다.",
        display_name, target_raw
    ))
}

#[tauri::command]
pub fn set_special_folder_path(key_name: String, new_path: String) -> Result<String, String> {
    let Some(display_name) = special_folder_display_name(&key_name) else {
        return Err("지원하지 않는 특수 폴더 항목입니다.".to_string());
    };

    if new_path.trim().is_empty() {
        return Err("유효한 폴더 경로를 입력해주세요.".to_string());
    }

    let target_raw = expand_env_vars(new_path.trim());
    let target = PathBuf::from(&target_raw);
    if !target.is_absolute() {
        return Err("절대 경로만 지정할 수 있습니다.".to_string());
    }
    if target.parent().is_none() {
        return Err("드라이브 루트 경로는 지정할 수 없습니다.".to_string());
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (folders, _) = hkcu
        .create_subkey(REG_USER_SHELL_FOLDERS)
        .map_err(|e| e.to_string())?;

    let current = read_shell_folder_path(&folders, &key_name);
    if let Some(current_path) = &current {
        let current_norm = current_path
            .to_string_lossy()
            .to_lowercase()
            .trim_end_matches('\\')
            .to_string();
        let target_norm = target
            .to_string_lossy()
            .to_lowercase()
            .trim_end_matches('\\')
            .to_string();
        if current_norm == target_norm {
            return Err("현재 경로와 동일한 경로입니다.".to_string());
        }
        if is_within(&target, current_path) {
            return Err("대상 경로가 현재 폴더 안에 있어 이동할 수 없습니다.".to_string());
        }
        if is_within(current_path, &target) {
            return Err("현재 폴더가 대상 경로 안에 있어 이동할 수 없습니다.".to_string());
        }
    }

    fs::create_dir_all(&target).map_err(|e| format!("폴더 생성 실패: {}", e))?;

    let outcome = match &current {
        Some(current_path) if current_path.is_dir() => {
            move_directory_contents(current_path, &target)
                .map_err(|e| format!("폴더 내용 이동 실패: {}", e))?
        }
        _ => MoveOutcome::default(),
    };

    if let Err(e) = folders.set_value(&key_name, &target_raw) {
        for (orig, moved_to) in outcome.transferred.iter().rev() {
            if moved_to.exists() {
                let _ = move_entry(moved_to, orig);
            }
        }
        return Err(format!("레지스트리 경로 변경 실패: {}", e));
    }

    if let Ok(legacy) = hkcu.open_subkey_with_flags(REG_SHELL_FOLDERS, KEY_WRITE) {
        let _ = legacy.set_value(&key_name, &target_raw);
    }

    notify_shell_change();
    let mut message = format!(
        "'{}' 경로를 '{}'로 변경하고 {}개 항목({})을 이동했습니다.",
        display_name,
        target_raw,
        outcome.moved,
        format_bytes(outcome.bytes)
    );
    if outcome.renamed > 0 {
        message.push_str(&format!(
            " 같은 이름이 이미 있던 {}개 항목은 이름에 번호를 붙여 옮겼습니다.",
            outcome.renamed
        ));
    }
    if current.is_none() {
        message.push_str(" 등록된 기존 경로가 없어 파일 이동은 수행하지 않았습니다.");
    }
    Ok(message)
}

#[tauri::command]
pub fn restart_explorer() -> Result<String, String> {
    // Kill explorer and restart
    let _ = hidden_command("taskkill")
        .args(["/F", "/IM", "explorer.exe"])
        .status();

    // Spawn explorer.exe
    let _ = Command::new("explorer.exe").spawn();

    notify_shell_change();
    Ok("Windows 탐색기(explorer.exe)가 재시작되었습니다.".to_string())
}

#[tauri::command]
pub fn set_animation_effects(disable: bool) -> Result<String, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Control Panel\Desktop\WindowMetrics")
        .map_err(|e| e.to_string())?;

    let val = if disable { "0" } else { "1" };
    key.set_value("MinAnimate", &val)
        .map_err(|e| e.to_string())?;

    notify_shell_change();
    Ok(if disable {
        "창 최소화/최대화 시각 애니메이션이 꺼져 반응 속도가 최적화되었습니다."
    } else {
        "창 최소화/최대화 시각 애니메이션이 복원되었습니다."
    }
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("ctt_shell_{}_{}_{}", std::process::id(), tag, seq));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn known_special_folder_keys_resolve_to_display_names() {
        assert_eq!(special_folder_display_name("Desktop"), Some("바탕 화면"));
        assert_eq!(
            special_folder_display_name("{374DE290-123F-4565-9164-39C4925E467B}"),
            Some("다운로드")
        );
        assert_eq!(special_folder_display_name("Personal"), Some("내 문서"));
    }

    #[test]
    fn unknown_special_folder_keys_are_rejected() {
        assert_eq!(special_folder_display_name("Shell Folders"), None);
        assert_eq!(special_folder_display_name(""), None);
        assert_eq!(special_folder_display_name("desktop"), None);
    }

    #[test]
    fn env_vars_are_expanded() {
        unsafe { std::env::set_var("CTT_TEST_DIR", r"D:\ctt\value") };
        assert_eq!(expand_env_vars("%CTT_TEST_DIR%\\sub"), r"D:\ctt\value\sub");
        assert_eq!(expand_env_vars("plain\\path"), r"plain\path");
    }

    #[test]
    fn unknown_env_vars_are_left_intact() {
        assert_eq!(
            expand_env_vars("%CTT_NOT_DEFINED_ANYWHERE%"),
            "%CTT_NOT_DEFINED_ANYWHERE%"
        );
    }

    #[test]
    fn containment_check_detects_nesting() {
        assert!(is_within(Path::new(r"D:\a\b"), Path::new(r"D:\a")));
        assert!(!is_within(Path::new(r"D:\c"), Path::new(r"D:\a")));
    }

    #[test]
    fn containment_check_ignores_partial_component_names() {
        assert!(!is_within(Path::new(r"D:\ab"), Path::new(r"D:\a")));
        assert!(!is_within(Path::new(r"D:\a"), Path::new(r"D:\a\b")));
        assert!(is_within(Path::new(r"D:\A\B"), Path::new(r"d:\a")));
    }

    #[test]
    fn unique_destination_avoids_overwriting_existing_files() {
        let dir = temp_dir("unique");
        fs::write(dir.join("report.txt"), b"old").unwrap();

        let target = unique_destination(&dir, "report.txt");

        assert_eq!(target.file_name().unwrap(), "report (1).txt");
        assert_eq!(fs::read(dir.join("report.txt")).unwrap(), b"old");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn move_directory_contents_transfers_files_and_folders() {
        let source = temp_dir("move_src");
        let dest = temp_dir("move_dst");
        fs::write(source.join("a.txt"), b"alpha").unwrap();
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested").join("b.txt"), b"beta").unwrap();

        let outcome = move_directory_contents(&source, &dest).unwrap();

        assert_eq!(outcome.moved, 2);
        assert_eq!(outcome.renamed, 0);
        assert_eq!(outcome.bytes, 9);
        assert_eq!(fs::read(dest.join("a.txt")).unwrap(), b"alpha");
        assert_eq!(
            fs::read(dest.join("nested").join("b.txt")).unwrap(),
            b"beta"
        );
        assert!(!source.join("a.txt").exists());
        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn move_directory_contents_keeps_both_on_name_collision() {
        let source = temp_dir("collide_src");
        let dest = temp_dir("collide_dst");
        fs::write(source.join("same.txt"), b"new").unwrap();
        fs::write(dest.join("same.txt"), b"existing").unwrap();

        let outcome = move_directory_contents(&source, &dest).unwrap();

        assert_eq!(outcome.moved, 1);
        assert_eq!(outcome.renamed, 1);
        assert_eq!(fs::read(dest.join("same.txt")).unwrap(), b"existing");
        assert_eq!(fs::read(dest.join("same (1).txt")).unwrap(), b"new");
        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&dest);
    }

    #[test]
    fn move_directory_contents_on_missing_source_is_a_noop() {
        let dest = temp_dir("missing_dst");
        let outcome = move_directory_contents(Path::new(r"Z:\definitely\not\here"), &dest).unwrap();
        assert_eq!(outcome, MoveOutcome::default());
        let _ = fs::remove_dir_all(&dest);
    }
}
