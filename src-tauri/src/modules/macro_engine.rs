use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

static IS_PLAYING: AtomicBool = AtomicBool::new(false);
static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);
static FAILED_INPUTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStatus {
    pub running: bool,
    pub failed_inputs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroAction {
    pub action_type: String, // "click", "move", "key", "delay"
    pub x: i32,
    pub y: i32,
    pub button: Option<String>, // "left", "right", "double"
    pub key: Option<String>,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub actions: Vec<MacroAction>,
    pub repeat_count: u32,
}

#[tauri::command]
pub fn get_sample_macros() -> Vec<MacroPreset> {
    vec![
        MacroPreset {
            id: "auto_clicker".to_string(),
            name: "초고속 자동 클릭 (Auto Clicker)".to_string(),
            description: "현재 마우스 위치에서 50ms 간격으로 좌클릭 10회 반복".to_string(),
            actions: vec![MacroAction {
                action_type: "click".to_string(),
                x: 0,
                y: 0,
                button: Some("left".to_string()),
                key: None,
                delay_ms: 50,
            }],
            repeat_count: 10,
        },
        MacroPreset {
            id: "win_refresh".to_string(),
            name: "바탕화면 새로고침 매크로 (F5)".to_string(),
            description: "F5 키를 2회 연타하여 탐색기 캐시 갱신".to_string(),
            actions: vec![
                MacroAction {
                    action_type: "key".to_string(),
                    x: 0,
                    y: 0,
                    button: None,
                    key: Some("F5".to_string()),
                    delay_ms: 100,
                },
                MacroAction {
                    action_type: "delay".to_string(),
                    x: 0,
                    y: 0,
                    button: None,
                    key: None,
                    delay_ms: 200,
                },
                MacroAction {
                    action_type: "key".to_string(),
                    x: 0,
                    y: 0,
                    button: None,
                    key: Some("F5".to_string()),
                    delay_ms: 100,
                },
            ],
            repeat_count: 1,
        },
    ]
}

struct PlayingGuard;

impl Drop for PlayingGuard {
    fn drop(&mut self) {
        IS_PLAYING.store(false, Ordering::SeqCst);
    }
}

#[tauri::command]
pub fn play_macro(actions: Vec<MacroAction>, repeat_count: u32) -> Result<String, String> {
    if IS_PLAYING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("이미 다른 매크로가 실행 중입니다.".to_string());
    }

    STOP_REQUESTED.store(false, Ordering::SeqCst);
    FAILED_INPUTS.store(0, Ordering::SeqCst);

    thread::spawn(move || {
        let _guard = PlayingGuard;
        let count = if repeat_count == 0 { 1 } else { repeat_count };
        let mut failed_inputs = 0usize;

        for _ in 0..count {
            if STOP_REQUESTED.load(Ordering::SeqCst) {
                break;
            }

            for action in &actions {
                if STOP_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }

                match action.action_type.as_str() {
                    "move" => {
                        #[cfg(windows)]
                        unsafe {
                            use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

                            if SetCursorPos(action.x, action.y).is_err() {
                                failed_inputs += 1;
                            }
                        }
                    }
                    "click" => {
                        #[cfg(windows)]
                        unsafe {
                            use windows::Win32::UI::Input::KeyboardAndMouse::{
                                INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN,
                                MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
                                MOUSEINPUT, SendInput,
                            };

                            let btn = action.button.as_deref().unwrap_or("left");
                            let (down_flag, up_flag) = if btn == "right" {
                                (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)
                            } else {
                                (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)
                            };

                            let inputs = [
                                INPUT {
                                    r#type: INPUT_MOUSE,
                                    Anonymous: INPUT_0 {
                                        mi: MOUSEINPUT {
                                            dx: 0,
                                            dy: 0,
                                            mouseData: 0,
                                            dwFlags: down_flag,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                },
                                INPUT {
                                    r#type: INPUT_MOUSE,
                                    Anonymous: INPUT_0 {
                                        mi: MOUSEINPUT {
                                            dx: 0,
                                            dy: 0,
                                            mouseData: 0,
                                            dwFlags: up_flag,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                },
                            ];
                            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                            if sent as usize != inputs.len() {
                                failed_inputs += 1;
                            }
                        }
                    }
                    "key" => {
                        #[cfg(windows)]
                        unsafe {
                            use windows::Win32::UI::Input::KeyboardAndMouse::{
                                INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
                                KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_DOWN, VK_ESCAPE, VK_F5,
                                VK_LEFT, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
                            };

                            let vk = match action.key.as_deref() {
                                Some("F5") => VK_F5,
                                Some("Enter") | Some("Return") => VK_RETURN,
                                Some("Space") => VK_SPACE,
                                Some("Tab") => VK_TAB,
                                Some("Esc") | Some("Escape") => VK_ESCAPE,
                                Some("Left") => VK_LEFT,
                                Some("Right") => VK_RIGHT,
                                Some("Up") => VK_UP,
                                Some("Down") => VK_DOWN,
                                _ => VIRTUAL_KEY(0),
                            };

                            if vk.0 != 0 {
                                let is_extended = matches!(
                                    action.key.as_deref(),
                                    Some("Left") | Some("Right") | Some("Up") | Some("Down")
                                );
                                let base_flags = if is_extended {
                                    KEYEVENTF_EXTENDEDKEY
                                } else {
                                    Default::default()
                                };
                                let inputs = [
                                    INPUT {
                                        r#type: INPUT_KEYBOARD,
                                        Anonymous: INPUT_0 {
                                            ki: KEYBDINPUT {
                                                wVk: vk,
                                                wScan: 0,
                                                dwFlags: base_flags,
                                                time: 0,
                                                dwExtraInfo: 0,
                                            },
                                        },
                                    },
                                    INPUT {
                                        r#type: INPUT_KEYBOARD,
                                        Anonymous: INPUT_0 {
                                            ki: KEYBDINPUT {
                                                wVk: vk,
                                                wScan: 0,
                                                dwFlags: base_flags | KEYEVENTF_KEYUP,
                                                time: 0,
                                                dwExtraInfo: 0,
                                            },
                                        },
                                    },
                                ];
                                let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                                if sent as usize != inputs.len() {
                                    failed_inputs += 1;
                                }
                            } else {
                                failed_inputs += 1;
                            }
                        }
                    }
                    _ => {}
                }

                if action.delay_ms > 0 {
                    thread::sleep(Duration::from_millis(action.delay_ms));
                }
            }
        }

        FAILED_INPUTS.store(failed_inputs, Ordering::SeqCst);
    });

    Ok("매크로 실행을 시작했습니다.".to_string())
}

#[tauri::command]
pub fn stop_macro() -> Result<String, String> {
    STOP_REQUESTED.store(true, Ordering::SeqCst);
    Ok("매크로 중지를 요청했습니다.".to_string())
}

#[tauri::command]
pub fn get_macro_status() -> MacroStatus {
    MacroStatus {
        running: IS_PLAYING.load(Ordering::SeqCst),
        failed_inputs: FAILED_INPUTS.load(Ordering::SeqCst),
    }
}
