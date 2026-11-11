pub mod modules;

use modules::auto_shutdown::*;
use modules::backup_guard::*;
use modules::backup_manager::*;
use modules::cleaner::*;
use modules::control_panel::*;
use modules::desktop_icons::*;
use modules::file_shredder::*;
use modules::macro_engine::*;
use modules::network::*;
use modules::registry_cleaner::*;
use modules::shell_tweaks::*;
use modules::startup::*;
use modules::system_info::*;
use modules::window_tool::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // System
            get_system_summary,
            optimize_memory,
            // Startup
            get_startup_items,
            toggle_startup_item,
            delete_startup_item,
            add_startup_item,
            // Shell Tweaks
            get_shell_tweaks_state,
            set_win11_classic_menu,
            set_show_file_extensions,
            set_show_hidden_files,
            set_show_drive_letters_first,
            set_taskbar_search_mode,
            set_hide_shortcut_arrow,
            set_special_folder_path,
            set_animation_effects,
            restart_explorer,
            // Desktop Icons (DesktopFixer)
            backup_desktop_icon_layout,
            list_desktop_icon_layouts,
            restore_desktop_icon_layout,
            // Cleaner
            scan_cleaner_targets,
            clean_selected_targets,
            // Window & Process Tool
            get_open_windows,
            get_window_at_cursor,
            set_window_transparency,
            set_window_topmost,
            set_window_title,
            set_window_visibility,
            suspend_process,
            resume_process,
            trim_process_memory,
            // Backup Guard
            get_guard_targets,
            add_guard_target,
            remove_guard_target,
            create_target_snapshot,
            list_snapshots,
            restore_snapshot,
            // Network & Filter
            get_network_summary,
            toggle_adblock,
            flush_dns_cache,
            // Macro
            get_sample_macros,
            play_macro,
            stop_macro,
            is_macro_running,
            get_macro_status,
            // Restore Points
            create_restore_point,
            get_restore_points,
            restore_from_point,
            // Control Panel & System Tools
            get_desktop_system_icons,
            set_desktop_system_icon,
            get_input_response_settings,
            set_input_response_settings,
            get_system_power_settings,
            set_hibernate_enabled,
            unlock_ultimate_performance,
            get_taskbar_settings,
            set_taskbar_setting,
            launch_control_panel_applet,
            get_advanced_win11_tweaks,
            set_advanced_win11_tweak,
            // Registry Cleaner
            scan_registry_junk,
            clean_registry_junk,
            // File Shredder
            shred_file,
            // Auto Shutdown
            schedule_shutdown,
            cancel_shutdown,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
