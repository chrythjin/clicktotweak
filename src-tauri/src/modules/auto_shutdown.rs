use crate::modules::common::hidden_command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const MAX_SHUTDOWN_SECONDS: u32 = 315_360_000; // Windows shutdown /t 최대치 (10년)

static HIBERNATE_GENERATION: AtomicU64 = AtomicU64::new(0);
static HIBERNATE_ACTIVE: AtomicBool = AtomicBool::new(false);

fn format_after_duration(seconds: u32) -> String {
    let whole_minutes = seconds / 60;
    let remainder = seconds % 60;
    if seconds < 60 {
        format!("{}초 후", seconds)
    } else if remainder == 0 {
        format!("{}분 후", whole_minutes)
    } else {
        format!("{}분 {}초 후", whole_minutes, remainder)
    }
}

fn cancel_hibernate_timer() -> bool {
    HIBERNATE_GENERATION.fetch_add(1, Ordering::SeqCst);
    HIBERNATE_ACTIVE.swap(false, Ordering::SeqCst)
}

#[tauri::command]
pub fn schedule_shutdown(seconds: u32, action: String) -> Result<String, String> {
    if seconds == 0 || seconds > MAX_SHUTDOWN_SECONDS {
        return Err(format!(
            "예약 시간은 1초 이상 {}초(10년) 이하로 지정해 주세요.",
            MAX_SHUTDOWN_SECONDS
        ));
    }

    let action_name = match action.as_str() {
        "shutdown" => "시스템 종료",
        "restart" => "시스템 다시 시작",
        "hibernate" => "최대 절전 모드 전환",
        _ => return Err("지원하지 않는 예약 동작입니다.".to_string()),
    };

    // 기존 예약(종료/재시작 및 내부 최대 절전 타이머) 모두 취소
    cancel_hibernate_timer();
    let _ = hidden_command("shutdown").arg("/a").output();

    let after = format_after_duration(seconds);

    if action == "hibernate" {
        // Windows shutdown.exe /h 는 /t 옵션과 함께 사용할 수 없으므로 내부 타이머 후 실행한다.
        let generation = HIBERNATE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
        HIBERNATE_ACTIVE.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(seconds as u64);
            while Instant::now() < deadline {
                if HIBERNATE_GENERATION.load(Ordering::SeqCst) != generation {
                    return;
                }
                let remaining = deadline.saturating_duration_since(Instant::now());
                thread::sleep(remaining.min(Duration::from_millis(250)));
            }
            if HIBERNATE_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            HIBERNATE_ACTIVE.store(false, Ordering::SeqCst);
            let _ = hidden_command("shutdown").arg("/h").output();
        });

        return Ok(format!("{} {} 예약이 실행됩니다.", after, action_name));
    }

    let flag = if action == "restart" { "/r" } else { "/s" };
    let output = hidden_command("shutdown")
        .args([flag, "/t", &seconds.to_string()])
        .output()
        .map_err(|e| format!("shutdown 명령어 실행 실패: {}", e))?;

    if output.status.success() {
        Ok(format!("{} {} 예약이 실행됩니다.", after, action_name))
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("예약 실패: {}", err.trim()))
    }
}

#[tauri::command]
pub fn cancel_shutdown() -> Result<String, String> {
    let cancelled_hibernate = cancel_hibernate_timer();

    let output = hidden_command("shutdown")
        .arg("/a")
        .output()
        .map_err(|e| format!("shutdown /a 실행 실패: {}", e))?;

    if output.status.success() {
        Ok("시스템 종료/다시 시작 예약이 취소되었습니다.".to_string())
    } else if cancelled_hibernate {
        Ok("최대 절전 모드 전환 예약이 취소되었습니다.".to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        if output.status.code() == Some(1116) || err.contains("1116") || err.contains("진행 중인")
        {
            Ok("현재 예약된 시스템 종료 작업이 없습니다.".to_string())
        } else {
            Err(format!("취소 실패: {}", err.trim()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_or_overflow_seconds() {
        assert!(schedule_shutdown(0, "shutdown".to_string()).is_err());
        assert!(schedule_shutdown(MAX_SHUTDOWN_SECONDS + 1, "shutdown".to_string()).is_err());
    }

    #[test]
    fn rejects_unknown_action() {
        assert!(schedule_shutdown(60, "logoff_unknown".to_string()).is_err());
    }

    #[test]
    fn formats_duration_clearly() {
        assert_eq!(format_after_duration(45), "45초 후");
        assert_eq!(format_after_duration(120), "2분 후");
        assert_eq!(format_after_duration(125), "2분 5초 후");
    }

    #[test]
    fn hibernate_schedule_can_be_cancelled_without_spawning_shutdown_h_t() {
        let msg = schedule_shutdown(3600, "hibernate".to_string()).unwrap();
        assert!(msg.contains("60분 후 최대 절전 모드 전환"));
        assert!(HIBERNATE_ACTIVE.load(Ordering::SeqCst));

        let cancel_msg = cancel_shutdown().unwrap();
        assert!(cancel_msg.contains("취소되었습니다"));
        assert!(!HIBERNATE_ACTIVE.load(Ordering::SeqCst));
    }
}
