export interface DiskInfo {
  name: string;
  mount_point: string;
  total_space: number;
  available_space: number;
  file_system: string;
}

export interface SystemSummary {
  os_name: string;
  os_version: string;
  hostname: string;
  cpu_name: string;
  cpu_cores: number;
  cpu_usage: number;
  total_memory: number;
  used_memory: number;
  memory_percentage: number;
  uptime_seconds: number;
  is_elevated: boolean;
  disks: DiskInfo[];
}

export interface StartupItem {
  id: string;
  name: string;
  command: string;
  location: string;
  is_enabled: boolean;
  file_path?: string | null;
}

export interface SpecialFolderInfo {
  key_name: string;
  display_name: string;
  current_path: string;
}

export interface ShellTweaksState {
  win11_classic_menu: boolean;
  show_file_extensions: boolean;
  show_hidden_files: boolean;
  show_drive_letters_first: boolean;
  hide_shortcut_arrow: boolean;
  animations_disabled: boolean;
  taskbar_search_mode: number;
  special_folders: SpecialFolderInfo[];
}

export interface CleanerCategoryScan {
  category_id: string;
  name: string;
  description: string;
  file_count: number;
  total_size_bytes: number;
  sample_files: string[];
}

export interface CleanResult {
  deleted_files_count: number;
  freed_bytes: number;
  skipped_files_count: number;
  message: string;
}

export interface WindowInfo {
  hwnd: number;
  title: string;
  class_name: string;
  pid: number;
  process_name: string;
  is_topmost: boolean;
  is_visible: boolean;
  alpha: number;
}

export interface GuardTarget {
  id: string;
  path: string;
  name: string;
  enabled: boolean;
  last_snapshot?: string | null;
  is_folder: boolean;
  snapshot_capable: boolean;
}

export interface SnapshotInfo {
  id: string;
  target_id: string;
  original_path: string;
  snapshot_path: string;
  timestamp: string;
  file_size: number;
}

export interface NetworkTrafficSummary {
  total_rx_bytes_sec: number;
  total_tx_bytes_sec: number;
  active_interface_name: string;
  is_adblock_active: boolean;
  blocked_domains_count: number;
}

export interface MacroAction {
  action_type: string;
  x: number;
  y: number;
  button?: string | null;
  key?: string | null;
  delay_ms: number;
}

export interface MacroPreset {
  id: string;
  name: string;
  description: string;
  actions: MacroAction[];
  repeat_count: number;
}

export interface RestorePoint {
  id: string;
  created_at: string;
  description: string;
  shell_state: ShellTweaksState;
}

export interface DesktopLayoutSnapshot {
  id: string;
  name: string;
  created_at: string;
  item_count: number;
}

export interface DesktopSystemIcons {
  this_pc: boolean;
  user_files: boolean;
  control_panel: boolean;
  network: boolean;
  recycle_bin: boolean;
}

export interface InputResponseSettings {
  keyboard_delay: number;
  keyboard_speed: number;
  menu_show_delay: number;
  mouse_speed_acceleration: boolean;
}

export interface SystemPowerSettings {
  hibernate_enabled: boolean;
  ultimate_performance_unlocked: boolean;
}

export interface TaskbarSettings {
  searchbox_mode: number;
  show_task_view: boolean;
  show_widgets: boolean;
}

export interface AdvancedWin11Tweaks {
  taskbar_alignment_center: boolean;
  disable_bing_search: boolean;
  prevent_auto_reboot: boolean;
  take_ownership_menu: boolean;
}

export interface RegistryJunkItem {
  id: string;
  category: string;
  key_path: string;
  value_name: string;
  target_path: string;
  description: string;
}

export interface RegistryCleanResult {
  cleaned_count: number;
  failed_count: number;
  message: string;
}

export type TabKey =
  | "dashboard"
  | "startup"
  | "tweaks"
  | "control_panel"
  | "cleaner"
  | "window"
  | "guard"
  | "network"
  | "macro"
  | "restore";
