use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class_name: String,
    pub pid: u32,
    pub process_name: String,
    pub is_topmost: bool,
    pub is_visible: bool,
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

#[tauri::command]
pub fn get_open_windows() -> Vec<WindowInfo> {
    let mut windows_list = Vec::new();

    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW,
            GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE, WS_EX_TOPMOST,
        };

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let list = &mut *(lparam.0 as *mut Vec<WindowInfo>);

                let len = GetWindowTextLengthW(hwnd);
                if len <= 0 {
                    return BOOL(1);
                }

                let mut title_buf = vec![0u16; (len + 1) as usize];
                GetWindowTextW(hwnd, &mut title_buf);
                let title = String::from_utf16_lossy(&title_buf[..len as usize]);

                let mut class_buf = [0u16; 256];
                let class_len = GetClassNameW(hwnd, &mut class_buf);
                let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

                // Filter out system shell / tooltips / cloaked windows
                if class_name == "Progman" || class_name == "Shell_TrayWnd" || title.trim().is_empty() {
                    return BOOL(1);
                }

                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                let process_name = get_process_name(pid);

                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                let is_topmost = (ex_style & WS_EX_TOPMOST.0) != 0;

                list.push(WindowInfo {
                    hwnd: hwnd.0 as usize as isize,
                    title,
                    class_name,
                    pid,
                    process_name,
                    is_topmost,
                    is_visible: IsWindowVisible(hwnd).as_bool(),
                });

                BOOL(1)
            }
        }

        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(&mut windows_list as *mut _ as isize),
        );
    }

    windows_list
}

#[tauri::command]
pub fn get_window_at_cursor() -> Result<WindowInfo, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::POINT;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetAncestor, GetClassNameW, GetCursorPos, GetWindowLongW, GetWindowTextLengthW,
            GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, WindowFromPoint, GA_ROOT,
            GWL_EXSTYLE, WS_EX_TOPMOST,
        };

        let mut pt = POINT::default();
        if !GetCursorPos(&mut pt).is_ok() {
            return Err("커서 위치를 가져올 수 없습니다.".to_string());
        }

        let mut target_hwnd = WindowFromPoint(pt);
        if target_hwnd.0.is_null() {
            return Err("커서 아래에 창이 없습니다.".to_string());
        }

        // Get the root top-level window
        let root_hwnd = GetAncestor(target_hwnd, GA_ROOT);
        if !root_hwnd.0.is_null() {
            target_hwnd = root_hwnd;
        }

        let len = GetWindowTextLengthW(target_hwnd);
        let title = if len > 0 {
            let mut buf = vec![0u16; (len + 1) as usize];
            GetWindowTextW(target_hwnd, &mut buf);
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            "(제목 없음)".to_string()
        };

        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(target_hwnd, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

        let mut pid = 0u32;
        GetWindowThreadProcessId(target_hwnd, Some(&mut pid));
        let process_name = get_process_name(pid);

        let ex_style = GetWindowLongW(target_hwnd, GWL_EXSTYLE) as u32;
        let is_topmost = (ex_style & WS_EX_TOPMOST.0) != 0;
        let is_visible = IsWindowVisible(target_hwnd).as_bool();

        Ok(WindowInfo {
            hwnd: target_hwnd.0 as usize as isize,
            title,
            class_name,
            pid,
            process_name,
            is_topmost,
            is_visible,
        })
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn set_window_transparency(hwnd: isize, alpha: u8) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::{COLORREF, HWND};
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
            HWND_TOP, LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
            WS_EX_LAYERED,
        };

        let win = HWND(hwnd as *mut _);
        let mut ex_style = GetWindowLongW(win, GWL_EXSTYLE) as u32;

        ex_style |= WS_EX_LAYERED.0;
        SetWindowLongW(win, GWL_EXSTYLE, ex_style as i32);
        SetLayeredWindowAttributes(win, COLORREF(0), alpha, LWA_ALPHA).map_err(|e| e.to_string())?;

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

        Ok(format!("창 투명도가 {}% 로 설정되었습니다.", (alpha as f32 / 255.0 * 100.0) as u32))
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn set_window_topmost(hwnd: isize, enable: bool) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE,
        };

        let win = HWND(hwnd as *mut _);
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
        use windows::core::HSTRING;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;

        let win = HWND(hwnd as *mut _);
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
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE, SW_SHOW};

        let win = HWND(hwnd as *mut _);
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

#[tauri::command]
pub fn suspend_process(pid: u32) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::core::s;
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};

        type NtSuspendProcessFn = unsafe extern "system" fn(HANDLE) -> i32;

        let ntdll = GetModuleHandleA(s!("ntdll.dll")).map_err(|e| e.to_string())?;
        let proc = GetProcAddress(ntdll, s!("NtSuspendProcess"))
            .ok_or_else(|| "NtSuspendProcess 함수를 찾을 수 없습니다.".to_string())?;
        let nt_suspend: NtSuspendProcessFn = std::mem::transmute(proc);

        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid).map_err(|e| e.to_string())?;
        let status = nt_suspend(handle);
        let _ = CloseHandle(handle);

        if status == 0 {
            Ok(format!("프로세스 (PID: {})가 일시 중지되었습니다.", pid))
        } else {
            Err(format!("프로세스 일시 중지 실패 (NTSTATUS: 0x{:08X})", status))
        }
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}

#[tauri::command]
pub fn resume_process(pid: u32) -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use windows::core::s;
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};

        type NtResumeProcessFn = unsafe extern "system" fn(HANDLE) -> i32;

        let ntdll = GetModuleHandleA(s!("ntdll.dll")).map_err(|e| e.to_string())?;
        let proc = GetProcAddress(ntdll, s!("NtResumeProcess"))
            .ok_or_else(|| "NtResumeProcess 함수를 찾을 수 없습니다.".to_string())?;
        let nt_resume: NtResumeProcessFn = std::mem::transmute(proc);

        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid).map_err(|e| e.to_string())?;
        let status = nt_resume(handle);
        let _ = CloseHandle(handle);

        if status == 0 {
            Ok(format!("프로세스 (PID: {})가 다시 실행되었습니다.", pid))
        } else {
            Err(format!("프로세스 재개 실패 (NTSTATUS: 0x{:08X})", status))
        }
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
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

        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA,
            false,
            pid,
        )
        .map_err(|e| e.to_string())?;
        let res = EmptyWorkingSet(handle);
        let _ = CloseHandle(handle);

        if res.is_ok() {
            Ok(format!("프로세스 (PID: {})의 워킹셋 메모리가 최적화되었습니다.", pid))
        } else {
            Err(format!("프로세스 메모리 최적화 실패: {:?}", res))
        }
    }
    #[cfg(not(windows))]
    Err("Windows 전용 기능입니다.".to_string())
}
