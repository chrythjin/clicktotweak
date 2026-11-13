use crate::modules::common::hidden_command;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use sysinfo::Networks;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkTrafficSummary {
    pub total_rx_bytes_sec: u64,
    pub total_tx_bytes_sec: u64,
    pub active_interface_name: String,
    pub is_adblock_active: bool,
    pub blocked_domains_count: usize,
}

const HOSTS_FILE_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
const BLOCK_START_TAG: &str = "# === BEGIN CLICKTOTWEAK MODERN AD BLOCK ===";
const BLOCK_END_TAG: &str = "# === END CLICKTOTWEAK MODERN AD BLOCK ===";

const AD_DOMAINS: &[&str] = &[
    "ad.daum.net",
    "adcreative.naver.com",
    "display.ad.daum.net",
    "adserver.inven.co.kr",
    "googleads.g.doubleclick.net",
    "pagead2.googlesyndication.com",
    "adservice.google.com",
    "securepubads.g.doubleclick.net",
    "adnxs.com",
    "ib.adnxs.com",
    "criteo.com",
    "static.criteo.net",
    "taboola.com",
    "outbrain.com",
    "popads.net",
    "adtrue.com",
];

fn is_adblock_active_at(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|content| content.contains(BLOCK_START_TAG))
        .unwrap_or(false)
}

#[tauri::command(async)]
pub fn get_network_summary() -> NetworkTrafficSummary {
    let mut networks = Networks::new_with_refreshed_list();
    let started = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(200));
    networks.refresh(true);
    let elapsed_secs = started.elapsed().as_secs_f64().max(0.05);

    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    let mut best_traffic = 0u64;
    let mut active_iface = "None".to_string();

    for (interface_name, data) in &networks {
        let rx = data.received();
        let tx = data.transmitted();
        let sum = rx.saturating_add(tx);
        if sum > 0 {
            total_rx = total_rx.saturating_add(rx);
            total_tx = total_tx.saturating_add(tx);
            if sum > best_traffic {
                best_traffic = sum;
                active_iface = interface_name.clone();
            }
        }
    }

    let rx_per_sec = (total_rx as f64 / elapsed_secs).round() as u64;
    let tx_per_sec = (total_tx as f64 / elapsed_secs).round() as u64;

    let is_adblock_active = is_adblock_active_at(Path::new(HOSTS_FILE_PATH));

    NetworkTrafficSummary {
        total_rx_bytes_sec: rx_per_sec,
        total_tx_bytes_sec: tx_per_sec,
        active_interface_name: active_iface,
        is_adblock_active,
        blocked_domains_count: if is_adblock_active {
            AD_DOMAINS.len()
        } else {
            0
        },
    }
}

fn strip_adblock_block(original: &[u8]) -> Vec<u8> {
    let start = BLOCK_START_TAG.as_bytes();
    let end = BLOCK_END_TAG.as_bytes();

    let mut result: Vec<u8> = Vec::with_capacity(original.len());
    let mut i = 0usize;

    while i < original.len() {
        let is_start = original[i..].starts_with(start);
        if is_start && let Some(rel_end) = find_subslice(&original[i..], end) {
            let block_end = i + rel_end + end.len();
            let mut cut_end = block_end;
            if original[block_end..].starts_with(b"\r\n") {
                cut_end += 2;
            } else if original[block_end..].starts_with(b"\n") {
                cut_end += 1;
            }
            if result.ends_with(b"\r\n\r\n") {
                result.truncate(result.len() - 2);
            } else if result.ends_with(b"\n\n") {
                result.truncate(result.len() - 1);
            }
            i = cut_end;
            continue;
        }
        result.push(original[i]);
        i += 1;
    }

    result
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

const MAX_HOSTS_BACKUPS: usize = 3;

fn prune_old_hosts_backups(dir: &Path, keep: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut backups: Vec<(std::time::SystemTime, String, std::path::PathBuf)> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.starts_with("hosts.clicktotweak.backup.") {
                return None;
            }
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            Some((modified, name, e.path()))
        })
        .collect();

    if backups.len() <= keep {
        return;
    }

    backups.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    for (_, _, old_path) in backups.into_iter().skip(keep) {
        let _ = fs::remove_file(old_path);
    }
}

fn write_hosts_atomically(path: &Path, content: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let dir = path
        .parent()
        .ok_or_else(|| "hosts 파일 경로가 올바르지 않습니다.".to_string())?;
    let original = fs::read(path).map_err(|e| format!("hosts 원본 읽기 실패: {}", e))?;
    let stamp = format!(
        "{}_{}",
        std::process::id(),
        chrono::Utc::now()
            .timestamp_nanos_opt()
            .ok_or("시간 조회 실패")?
    );
    let backup_path = dir.join(format!("hosts.clicktotweak.backup.{}", stamp));
    let tmp_path = dir.join(format!("hosts.clicktotweak.{}.tmp", stamp));
    let mut backup = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup_path)
        .map_err(|e| format!("hosts 백업 생성 실패: {}", e))?;
    backup
        .write_all(&original)
        .and_then(|_| backup.sync_all())
        .map_err(|e| format!("hosts 백업 저장 실패 ({}): {}", backup_path.display(), e))?;
    let result = (|| -> Result<(), String> {
        let mut tmp = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(|e| format!("hosts 임시 파일 생성 실패: {}", e))?;
        tmp.write_all(content)
            .and_then(|_| tmp.sync_all())
            .map_err(|e| format!("hosts 임시 파일 저장 실패: {}", e))?;
        drop(tmp);
        if fs::read(path).map_err(|e| format!("hosts 재확인 실패: {}", e))? != original {
            return Err("hosts 파일이 변경되어 교체를 중단했습니다.".to_string());
        }
        fs::rename(&tmp_path, path).map_err(|e| format!("hosts 원자적 교체 실패: {}", e))?;
        Ok(())
    })();
    if let Err(error) = result {
        if tmp_path.exists() {
            fs::remove_file(&tmp_path)
                .map_err(|cleanup| format!("{}; 임시 파일 정리 실패: {}", error, cleanup))?;
        }
        return Err(format!("{} (백업: {})", error, backup_path.display()));
    }
    prune_old_hosts_backups(dir, MAX_HOSTS_BACKUPS);
    Ok(())
}

#[tauri::command]
pub fn toggle_adblock(enable: bool) -> Result<String, String> {
    toggle_adblock_at(Path::new(HOSTS_FILE_PATH), enable)
}

fn toggle_adblock_at(path: &Path, enable: bool) -> Result<String, String> {
    if !path.exists() {
        return Err("hosts 파일을 찾을 수 없습니다.".to_string());
    }

    let original = fs::read(path)
        .map_err(|e| format!("hosts 파일을 읽을 수 없습니다 (관리자 권한 필요): {}", e))?;

    let mut new_content = strip_adblock_block(&original);

    if enable {
        if !new_content.is_empty() && !new_content.ends_with(b"\n") {
            new_content.push(b'\n');
        }
        new_content.extend_from_slice(b"\r\n");
        new_content.extend_from_slice(BLOCK_START_TAG.as_bytes());
        new_content.extend_from_slice(b"\r\n");
        for domain in AD_DOMAINS {
            new_content.extend_from_slice(format!("0.0.0.0 {}\r\n", domain).as_bytes());
        }
        new_content.extend_from_slice(BLOCK_END_TAG.as_bytes());
        new_content.extend_from_slice(b"\r\n");
    }

    write_hosts_atomically(path, &new_content)?;

    Ok(if enable {
        format!(
            "광고 및 트래커 차단 필터가 활성화되었습니다. (총 {}개 도메인)",
            AD_DOMAINS.len()
        )
    } else {
        "광고 차단 필터가 해제되었습니다.".to_string()
    })
}

#[tauri::command]
pub fn flush_dns_cache() -> Result<String, String> {
    let output = hidden_command("ipconfig")
        .arg("/flushdns")
        .output()
        .map_err(|e| format!("DNS 초기화 명령 실행 실패: {}", e))?;

    if output.status.success() {
        Ok("Windows DNS 확인자 캐시가 성공적으로 초기화(플러시)되었습니다.".to_string())
    } else {
        Err("DNS 캐시 초기화 실패".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_block_removes_managed_region_and_keeps_user_content() {
        let original = b"127.0.0.1 localhost\r\n# user comment\r\n\r\n# === BEGIN CLICKTOTWEAK MODERN AD BLOCK ===\r\n0.0.0.0 ad.daum.net\r\n# === END CLICKTOTWEAK MODERN AD BLOCK ===\r\n192.168.0.1 router\r\n";
        let result = strip_adblock_block(original);

        assert!(
            !result
                .windows(BLOCK_START_TAG.len())
                .any(|w| w == BLOCK_START_TAG.as_bytes())
        );
        assert!(
            !result
                .windows(BLOCK_END_TAG.len())
                .any(|w| w == BLOCK_END_TAG.as_bytes())
        );
        assert!(
            result
                .windows("127.0.0.1 localhost".len())
                .any(|w| w == b"127.0.0.1 localhost")
        );
        assert!(
            result
                .windows("# user comment".len())
                .any(|w| w == b"# user comment")
        );
        assert!(
            result
                .windows("192.168.0.1 router".len())
                .any(|w| w == b"192.168.0.1 router")
        );
    }

    #[test]
    fn strip_block_is_idempotent_when_block_absent() {
        let original = b"127.0.0.1 localhost\n10.0.0.1 nas\n";
        let result = strip_adblock_block(original);
        assert_eq!(result, original);
    }

    #[test]
    fn strip_multiple_blocks_preserves_intervening_data() {
        let block = format!(
            "{}\n0.0.0.0 example.test\n{}\n",
            BLOCK_START_TAG, BLOCK_END_TAG
        );
        let input = format!("first\n{}middle\n{}last\n", block, block);
        assert_eq!(
            strip_adblock_block(input.as_bytes()),
            b"first\nmiddle\nlast\n"
        );
    }

    #[test]
    fn strip_block_preserves_lf_only_line_endings() {
        let original = b"127.0.0.1 localhost\n# === BEGIN CLICKTOTWEAK MODERN AD BLOCK ===\n0.0.0.0 x\n# === END CLICKTOTWEAK MODERN AD BLOCK ===\n10.0.0.1 nas\n";
        let result = strip_adblock_block(original);
        assert_eq!(result, b"127.0.0.1 localhost\n10.0.0.1 nas\n");
    }

    #[test]
    fn strip_block_does_not_touch_similarly_named_lines() {
        let original = b"# === BEGIN CLICKTOTWEAK MODERN AD BLOCK EXTRA ===\nkeep me\n";
        let result = strip_adblock_block(original);
        assert_eq!(result, original);
    }

    #[test]
    fn atomic_write_creates_backup_and_replaces_content() {
        let dir = std::env::temp_dir().join(format!(
            "ctt_hosts_atomic_test_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir(&dir).unwrap();
        let hosts = dir.join("hosts");
        fs::write(&hosts, b"original\n").unwrap();

        write_hosts_atomically(&hosts, b"replaced\n").unwrap();

        assert_eq!(fs::read(&hosts).unwrap(), b"replaced\n");
        let backups: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("hosts.clicktotweak.backup.")
            })
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(backups[0].path()).unwrap(), b"original\n");

        fs::remove_dir_all(&dir).unwrap();
    }

    fn hosts_fixture(name: &str, content: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ctt_hosts_{}_{}_{}",
            name,
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(&dir).unwrap();
        let hosts = dir.join("hosts");
        fs::write(&hosts, content).unwrap();
        hosts
    }

    #[test]
    fn toggle_enable_writes_block_and_keeps_existing_entries() {
        let hosts = hosts_fixture("enable", b"127.0.0.1 localhost\r\n192.168.0.1 router\r\n");

        toggle_adblock_at(&hosts, true).unwrap();

        let content = fs::read_to_string(&hosts).unwrap();
        assert!(content.contains(BLOCK_START_TAG));
        assert!(content.contains(BLOCK_END_TAG));
        assert!(content.contains("127.0.0.1 localhost"));
        assert!(content.contains("192.168.0.1 router"));
        for domain in AD_DOMAINS {
            assert!(
                content.contains(&format!("0.0.0.0 {}", domain)),
                "missing {}",
                domain
            );
        }
        assert!(is_adblock_active_at(&hosts));

        fs::remove_dir_all(hosts.parent().unwrap()).unwrap();
    }

    #[test]
    fn toggle_disable_removes_block_and_restores_original_content() {
        let original = b"127.0.0.1 localhost\r\n192.168.0.1 router\r\n";
        let hosts = hosts_fixture("disable", original);

        toggle_adblock_at(&hosts, true).unwrap();
        toggle_adblock_at(&hosts, false).unwrap();

        let content = fs::read_to_string(&hosts).unwrap();
        assert!(!content.contains(BLOCK_START_TAG));
        assert!(!content.contains(BLOCK_END_TAG));
        assert!(!is_adblock_active_at(&hosts));
        assert!(content.contains("127.0.0.1 localhost"));
        assert!(content.contains("192.168.0.1 router"));

        fs::remove_dir_all(hosts.parent().unwrap()).unwrap();
    }

    #[test]
    fn toggle_enable_twice_does_not_duplicate_block() {
        let hosts = hosts_fixture("idempotent", b"127.0.0.1 localhost\r\n");

        toggle_adblock_at(&hosts, true).unwrap();
        toggle_adblock_at(&hosts, true).unwrap();

        let content = fs::read_to_string(&hosts).unwrap();
        assert_eq!(content.matches(BLOCK_START_TAG).count(), 1);
        assert_eq!(content.matches(BLOCK_END_TAG).count(), 1);

        fs::remove_dir_all(hosts.parent().unwrap()).unwrap();
    }

    #[test]
    fn toggle_on_missing_file_returns_error() {
        let missing = std::env::temp_dir().join("ctt_hosts_absent_file_should_not_exist.txt");
        assert!(toggle_adblock_at(&missing, true).is_err());
    }

    #[test]
    fn toggle_leaves_backup_of_previous_state() {
        let hosts = hosts_fixture("backup", b"127.0.0.1 localhost\r\n");

        toggle_adblock_at(&hosts, true).unwrap();

        let backups: Vec<_> = fs::read_dir(hosts.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("hosts.clicktotweak.backup.")
            })
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            fs::read(backups[0].path()).unwrap(),
            b"127.0.0.1 localhost\r\n"
        );

        fs::remove_dir_all(hosts.parent().unwrap()).unwrap();
    }

    #[test]
    fn atomic_write_prunes_old_backups_beyond_retention_limit() {
        let hosts = hosts_fixture("prune", b"v0\n");
        let dir = hosts.parent().unwrap();

        for i in 1..=5 {
            std::thread::sleep(std::time::Duration::from_millis(2));
            write_hosts_atomically(&hosts, format!("v{}\n", i).as_bytes()).unwrap();
        }

        let backups: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("hosts.clicktotweak.backup.")
            })
            .collect();
        assert_eq!(backups.len(), MAX_HOSTS_BACKUPS);

        fs::remove_dir_all(dir).unwrap();
    }
}
