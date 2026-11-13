use std::path::PathBuf;
use std::process::Command;

/// `%LOCALAPPDATA%\ClickToTweakModern\<subdir>` 경로를 반환하고 없으면 생성한다.
pub fn app_data_subdir(subdir: &str) -> PathBuf {
    let dir = if let Ok(local) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local).join("ClickToTweakModern").join(subdir)
    } else {
        PathBuf::from(".").join(subdir)
    };
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// 스냅샷·복원 지점 식별자에 경로 탐색 문자가 포함되지 않았는지 검증한다.
pub fn validate_safe_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("유효하지 않은 식별자입니다.".to_string());
    }
    Ok(())
}

/// 단일 파일 이름(경로 구분자 및 상위 디렉터리 참조 불허)인지 검증한다.
pub fn validate_single_filename(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('\0')
    {
        return Err("유효하지 않은 파일 이름입니다.".to_string());
    }
    Ok(())
}

/// `%VAR%` 형태의 Windows 환경 변수를 최대 4단계까지 확장한다.
pub fn expand_env_vars(input: &str) -> String {
    let mut expanded = input.to_string();
    for _ in 0..4 {
        let mut changed = false;
        let mut result = String::with_capacity(expanded.len());
        let mut rest = expanded.as_str();
        while let Some(start) = rest.find('%') {
            result.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            match after.find('%') {
                Some(end) => {
                    let name = &after[..end];
                    if name.is_empty() {
                        result.push('%');
                    } else {
                        let resolved = std::env::var(name)
                            .or_else(|_| std::env::var(name.to_uppercase().as_str()));
                        match resolved {
                            Ok(value) => {
                                result.push_str(&value);
                                changed = true;
                            }
                            Err(_) => {
                                result.push('%');
                                result.push_str(name);
                                result.push('%');
                            }
                        }
                    }
                    rest = &after[end + 1..];
                }
                None => {
                    result.push_str(&rest[start..]);
                    rest = "";
                }
            }
        }
        result.push_str(rest);
        expanded = result;
        if !changed {
            break;
        }
    }
    expanded
}

/// Windows 탐색기에 쉘/레지스트리 변경 알림을 전파한다.
pub fn notify_shell_change() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
}

/// GUI 서브시스템에서 콘솔 창 깜빡임(`CREATE_NO_WINDOW`) 없이 외부 명령을 생성한다.
pub fn hidden_command(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_id_rejects_traversal_and_special_chars() {
        assert!(validate_safe_id("rp_20261113_120000_123").is_ok());
        assert!(validate_safe_id("layout-1_ok").is_ok());
        assert!(validate_safe_id("").is_err());
        assert!(validate_safe_id("..\\secret").is_err());
        assert!(validate_safe_id("a/b").is_err());
        assert!(validate_safe_id("has space").is_err());
    }

    #[test]
    fn single_filename_rejects_separators_and_dot_segments() {
        assert!(validate_single_filename("app.lnk").is_ok());
        assert!(validate_single_filename("my app (1).lnk").is_ok());
        assert!(validate_single_filename("").is_err());
        assert!(validate_single_filename(".").is_err());
        assert!(validate_single_filename("..").is_err());
        assert!(validate_single_filename("..\\evil.lnk").is_err());
        assert!(validate_single_filename("sub/evil.lnk").is_err());
        assert!(validate_single_filename("C:evil.lnk").is_err());
    }

    #[test]
    fn env_vars_expand_both_exact_and_uppercase_names() {
        unsafe { std::env::set_var("CTT_COMMON_TEST", r"D:\ctt\common") };
        assert_eq!(
            expand_env_vars("%CTT_COMMON_TEST%\\bin"),
            r"D:\ctt\common\bin"
        );
        assert_eq!(
            expand_env_vars("%ctt_common_test%\\bin"),
            r"D:\ctt\common\bin"
        );
        assert_eq!(expand_env_vars("%CTT_MISSING_VAR%"), "%CTT_MISSING_VAR%");
    }
}
