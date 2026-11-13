use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class_name: String,
    pub pid: u32,
    pub process_name: String,
    pub is_topmost: bool,
    pub is_visible: bool,
    pub alpha: u8,
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> String {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::GetProcessImageFileNameW;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut buf = [0u16; 512];
            let len = GetProcessImageFileNameW(handle, &mut buf);
            let _ = CloseHandle(handle);
            if len > 0 {
                let full = String::from_utf16_lossy(&buf[..len as usize]);
                if let Some(name) = full.rsplit('\\').next() {
                    return name.to_string();
                }
                return full;
            }
        }
    }
    format!("PID: {}", pid)
}

#[cfg(windows)]
unsafe fn read_window_alpha(hwnd: windows::Win32::Foundation::HWND, ex_style: u32) -> u8 {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetLayeredWindowAttributes, LWA_ALPHA, WS_EX_LAYERED,
    };

    if (ex_style & WS_EX_LAYERED.0) == 0 {
        return 255;
    }
    let mut alpha = 255u8;
    let mut flags = windows::Win32::UI::WindowsAndMessaging::LAYERED_WINDOW_ATTRIBUTES_FLAGS(0);
    if unsafe { GetLayeredWindowAttributes(hwnd, None, Some(&mut alpha), Some(&mut flags)) }.is_ok()
        && (flags.0 & LWA_ALPHA.0) != 0
    {
        return alpha;
    }
    255
}

#[cfg(windows)]
unsafe fn ensure_valid_window(hwnd: isize) -> Result<windows::Win32::Foundation::HWND, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    let win = HWND(hwnd as *mut _);
    if win.0.is_null() || unsafe { !IsWindow(win).as_bool() } {
        return Err("대상 창이 더 이상 존재하지 않습니다. 목록을 새로고침해 주세요.".to_string());
    }
    Ok(win)
}

#[tauri::command(async)]
pub fn get_open_windows() -> Vec<WindowInfo> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GWL_EXSTYLE, GetClassNameW, GetWindowLongW, GetWindowTextLengthW,
            GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, WS_EX_TOPMOST,
        };

        struct EnumContext {
            list: Vec<WindowInfo>,
            pid_names: HashMap<u32, String>,
        }

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let ctx = &mut *(lparam.0 as *mut EnumContext);

                let len = GetWindowTextLengthW(hwnd);
                if len <= 0 {
                    return BOOL(1);
                }

                let mut title_buf = vec![0u16; (len + 1) as usize];
                let copied = GetWindowTextW(hwnd, &mut title_buf);
                let title = String::from_utf16_lossy(&title_buf[..copied.max(0) as usize]);

                let mut class_buf = [0u16; 256];
                let class_len = GetClassNameW(hwnd, &mut class_buf);
                let class_name = String::from_utf16_lossy(&class_buf[..class_len.max(0) as usize]);

                if class_name == "Progman"
                    || class_name == "Shell_TrayWnd"
                    || title.trim().is_empty()
                {
                    return BOOL(1);
                }

                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                let process_name = ctx
                    .pid_names
                    .entry(pid)
                    .or_insert_with(|| get_process_name(pid))
                    .clone();

                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                let is_topmost = (ex_style & WS_EX_TOPMOST.0) != 0;
                let alpha = read_window_alpha(hwnd, ex_style);

                ctx.list.push(WindowInfo {
                    hwnd: hwnd.0 as usize as isize,
                    title,
                    class_name,
                    pid,
                    process_name,
                    is_topmost,
                    is_visible: IsWindowVisible(hwnd).as_bool(),
                    alpha,
                });

                BOOL(1)
            }
        }

        let mut ctx = EnumContext {
            list: Vec::new(),
            pid_names: HashMap::new(),
        };
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut ctx as *mut _ as isize));
        ctx.list
    }

    #[cfg(not(windows))]
    Vec::new()
}

#[tauri::command]
pub fn get_window_at_cursor() -> Result<WindowInfo, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::{
            GA_ROOT, GWL_EXSTYLE, GetAncestor, GetClassNameW, GetCursorPos, GetWindowLongW,
            GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
            WS_EX_TOPMOST, WindowFromPoint,
        };

        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_err() {
            return Err("커서 위치를 가져올 수 없습니다.".to_string());
        }

        let mut target_hwnd = WindowFromPoint(pt);
        if target_hwnd.0.is_null() {
            return Err("커서 아래에 창이 없습니다.".to_string());
        }

        let root_hwnd = GetAncestor(target_hwnd, GA_ROOT);
        if !root_hwnd.0.is_null() {
            target_hwnd = root_hwnd;
        }

        let len = GetWindowTextLengthW(target_hwnd);
        let title = if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            let copied = GetWindowTextW(target_hwnd, &mut buf);
            String::from_utf16_lossy(&buf[..copied.max(0) as usize])
        } else {
            "(제목 없음)".to_string()
        };

        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(target_hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len.max(0) as usize]);

        let mut pid = 0u32;
        GetWindowThreadProcessId(target_hwnd, Some(&mut pid));
        let process_name = get_process_name(pid);

        let ex_style = GetWindowLongW(target_hwnd, GWL_EXSTYLE) as u32;
        let is_topmost = (ex_style & WS_EX_TOPMOST.0) != 0;
        let is_visible = IsWindowVisible(target_hwnd).as_bool();
        let alpha = read_window_alpha(target_hwnd, ex_style);

        Ok(WindowInfo {
            hwnd: target_hwnd.0 as usize as isize,
            title,
            class_name,
            pid,
            process_name,
            is_topmost,
            is_visible,
            alpha,
        })
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn set_window_transparency(hwnd: isize, alpha: u8) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::COLORREF;
        use windows::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GetWindowLongW, HWND_TOP, LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOMOVE,
            SWP_NOSIZE, SWP_NOZORDER, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos,
            WS_EX_LAYERED,
        };

        let win = ensure_valid_window(hwnd)?;
        let mut ex_style = GetWindowLongW(win, GWL_EXSTYLE) as u32;

        ex_style |= WS_EX_LAYERED.0;
        SetWindowLongW(win, GWL_EXSTYLE, ex_style as i32);
        SetLayeredWindowAttributes(win, COLORREF(0), alpha, LWA_ALPHA)
            .map_err(|e| e.to_string())?;

        if alpha == 255 {
            ex_style &= !WS_EX_LAYERED.0;
            SetWindowLongW(win, GWL_EXSTYLE, ex_style as i32);
            let _ = SetWindowPos(
                win,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
            );
        }

        Ok(format!(
            "창 투명도가 {}% 로 설정되었습니다.",
            ((alpha as f32 / 255.0) * 100.0).round() as u32
        ))
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn set_window_topmost(hwnd: isize, enable: bool) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
        };

        let win = ensure_valid_window(hwnd)?;
        let target = if enable { HWND_TOPMOST } else { HWND_NOTOPMOST };

        let res = SetWindowPos(win, target, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        if res.is_ok() {
            Ok(if enable {
                "창이 항상 위로 고정되었습니다."
            } else {
                "창의 항상 위 설정이 해제되었습니다."
            }
            .to_string())
        } else {
            Err("창 위치 속성 변경 실패".to_string())
        }
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn set_window_title(hwnd: isize, title: String) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
        use windows::core::HSTRING;

        let win = ensure_valid_window(hwnd)?;
        let h_title = HSTRING::from(&title);
        let res = SetWindowTextW(win, windows::core::PCWSTR(h_title.as_ptr()));
        if res.is_ok() {
            Ok(format!("창 제목이 '{}'로 변경되었습니다.", title))
        } else {
            Err("창 제목 변경 실패".to_string())
        }
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn set_window_visibility(hwnd: isize, show: bool) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindow};

        let win = ensure_valid_window(hwnd)?;
        let _ = ShowWindow(win, if show { SW_SHOW } else { SW_HIDE });
        Ok(if show {
            "창이 화면에 표시되었습니다."
        } else {
            "창이 숨겨졌습니다."
        }
        .to_string())
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

fn validate_suspend_target_pid(pid: u32, current_pid: u32) -> Result<(), String> {
    if pid == 0 || pid == 4 {
        return Err("Windows 시스템 핵심 프로세스는 일시 중지할 수 없습니다.".to_string());
    }
    if pid == current_pid {
        return Err("현재 실행 중인 ClickToTweak 프로세스는 일시 중지할 수 없습니다.".to_string());
    }
    Ok(())
}

#[cfg(windows)]
fn control_process_nt(pid: u32, proc_name: &CStr) -> Result<i32, String> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};
    use windows::core::{PCSTR, s};

    type NtProcessControlFn = unsafe extern "system" fn(HANDLE) -> i32;

    unsafe {
        let ntdll = GetModuleHandleA(s!("ntdll.dll")).map_err(|e| e.to_string())?;
        let proc = GetProcAddress(ntdll, PCSTR(proc_name.as_ptr() as *const u8))
            .ok_or_else(|| format!("{} 함수를 찾을 수 없습니다.", proc_name.to_string_lossy()))?;
        let nt_fn: NtProcessControlFn = std::mem::transmute(proc);

        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid).map_err(|e| e.to_string())?;
        let status = nt_fn(handle);
        let _ = CloseHandle(handle);
        Ok(status)
    }
}

#[tauri::command]
pub fn suspend_process(pid: u32) -> Result<String, String> {
    validate_suspend_target_pid(pid, std::process::id())?;
    #[cfg(windows)]
    {
        let status = control_process_nt(pid, c"NtSuspendProcess")?;
        if status == 0 {
            Ok(format!("프로세스 (PID: {})가 일시 중지되었습니다.", pid))
        } else {
            Err(format!(
                "프로세스 일시 중지 실패 (NTSTATUS: 0x{:08X})",
                status
            ))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Err("Windows 전용 기능입니다.".to_string())
    }
}

#[tauri::command]
pub fn resume_process(pid: u32) -> Result<String, String> {
    #[cfg(windows)]
    {
        let status = control_process_nt(pid, c"NtResumeProcess")?;
        if status == 0 {
            Ok(format!("프로세스 (PID: {})가 다시 실행되었습니다.", pid))
        } else {
            Err(format!("프로세스 재개 실패 (NTSTATUS: 0x{:08X})", status))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        Err("Windows 전용 기능입니다.".to_string())
    }
}

#[tauri::command]
pub fn trim_process_memory(pid: u32) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
        };

        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA, false, pid)
            .map_err(|e| e.to_string())?;
        let res = EmptyWorkingSet(handle);
        let _ = CloseHandle(handle);

        if res.is_ok() {
            Ok(format!(
                "프로세스 (PID: {})의 워킹셋 메모리가 최적화되었습니다.",
                pid
            ))
        } else {
            Err(format!("프로세스 메모리 최적화 실패: {:?}", res))
        }
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suspend_rejects_self_and_system_pids() {
        assert!(validate_suspend_target_pid(0, 1234).is_err());
        assert!(validate_suspend_target_pid(4, 1234).is_err());
        assert!(validate_suspend_target_pid(1234, 1234).is_err());
        assert!(validate_suspend_target_pid(5678, 1234).is_ok());
    }
}
