use std::process::Command;

#[tauri::command]
pub fn schedule_shutdown(seconds: u32, action: String) -> Result<String, String> {
    // 먼저 기존 예약 취소
    let _ = Command::new("shutdown").arg("/a").output();

    let flag = match action.as_str() {
        "restart" => "/r",
        "hibernate" => "/h",
        _ => "/s",
    };

    let output = Command::new("shutdown")
        .args(&[flag, "/t", &seconds.to_string()])
        .output()
        .map_err(|e| format!("shutdown 명령어 실행 실패: {}", e))?;

    if output.status.success() {
        let mins = seconds / 60;
        let action_name = match action.as_str() {
            "restart" => "시스템 다시 시작",
            "hibernate" => "최대 절전 모드 전환",
            _ => "시스템 종료",
        };
        Ok(format!("{}분이 지나면 {} 예약이 실행됩니다.", mins, action_name))
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("예약 실패: {}", err))
    }
}

#[tauri::command]
pub fn cancel_shutdown() -> Result<String, String> {
    let output = Command::new("shutdown")
        .arg("/a")
        .output()
        .map_err(|e| format!("shutdown /a 실행 실패: {}", e))?;

    if output.status.success() {
        Ok("시스템 종료/다시 시작 예약이 취소되었습니다.".to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("1116") || err.contains("진행 중인") {
            Ok("현재 예약된 시스템 종료 작업이 없습니다.".to_string())
        } else {
            Err(format!("취소 실패: {}", err))
        }
    }
}

