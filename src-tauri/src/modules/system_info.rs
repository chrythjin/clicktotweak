use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use std::mem::size_of;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSummary {
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    pub memory_percentage: f32,
    pub uptime_seconds: u64,
    pub is_elevated: bool,
    pub disks: Vec<DiskInfo>,
}

pub fn is_process_elevated() -> bool {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_ok() {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut returned_size = 0u32;
            let success = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned_size,
            );
            let _ = CloseHandle(token);
            if success.is_ok() {
                return elevation.TokenIsElevated != 0;
            }
        }
    }
    false
}

#[tauri::command(async)]
pub fn get_system_summary() -> SystemSummary {
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );

    // Refresh twice for accurate CPU usage
    sys.refresh_cpu_all();
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_name = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_cores = sys.cpus().len();
    let cpu_usage = sys.global_cpu_usage();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percentage = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    let disks_info = Disks::new_with_refreshed_list();
    let disks = disks_info
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_space: d.total_space(),
            available_space: d.available_space(),
            file_system: d.file_system().to_string_lossy().to_string(),
        })
        .collect();

    SystemSummary {
        os_name: System::name().unwrap_or_else(|| "Windows".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "11/10".to_string()),
        hostname: System::host_name().unwrap_or_else(|| "PC".to_string()),
        cpu_name,
        cpu_cores,
        cpu_usage,
        total_memory,
        used_memory,
        memory_percentage,
        uptime_seconds: System::uptime(),
        is_elevated: is_process_elevated(),
        disks,
    }
}

#[tauri::command(async)]
pub fn optimize_memory() -> Result<String, String> {
    #[cfg(windows)]
    unsafe {
        use sysinfo::System;
        use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
        use windows::Win32::System::Threading::{
            GetCurrentProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
        };

        let self_res = EmptyWorkingSet(GetCurrentProcess());
        if self_res.is_err() {
            return Err("메모리 최적화 실패: 현재 프로세스 워킹셋을 정리할 수 없습니다.".to_string());
        }

        // EmptyWorkingSet on another process requires PROCESS_SET_QUOTA,
        // which non-elevated callers lack, so those handles fail to open.
        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut trimmed = 0usize;
        let mut skipped = 0usize;

        for pid in sys.processes().keys() {
            let pid = pid.as_u32();
            if pid == std::process::id() {
                continue;
            }
            match OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid) {
                Ok(handle) => {
                    if EmptyWorkingSet(handle).is_ok() {
                        trimmed += 1;
                    } else {
                        skipped += 1;
                    }
                    let _ = windows::Win32::Foundation::CloseHandle(handle);
                }
                Err(_) => skipped += 1,
            }
        }

        let mut msg = format!(
            "메모리 최적화를 완료했습니다. (현재 프로세스 + {}개 프로세스 워킹셋 정리, {}개 건너뜀)",
            trimmed, skipped
        );
        if trimmed == 0 {
            msg.push_str(
                "\n다른 프로세스를 정리하려면 관리자 권한으로 실행해야 합니다. (현재는 현재 프로세스만 정리됨)",
            );
        }
        Ok(msg)
    }
    #[cfg(not(windows))]
    {
        Ok("Not Windows".to_string())
    }
}
