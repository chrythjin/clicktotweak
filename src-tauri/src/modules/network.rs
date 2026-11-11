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

#[tauri::command(async)]
pub fn get_network_summary() -> NetworkTrafficSummary {
    let mut networks = Networks::new_with_refreshed_list();
    std::thread::sleep(std::time::Duration::from_millis(200));
    networks.refresh(true);

    let mut total_rx = 0u64;
    let mut total_tx = 0u64;
    let mut active_iface = "None".to_string();

    for (interface_name, data) in &networks {
        let rx = data.received();
        let tx = data.transmitted();
        if rx > 0 || tx > 0 {
            total_rx += rx * 5; // 200ms -> per second
            total_tx += tx * 5;
            active_iface = interface_name.clone();
        }
    }

    let is_adblock_active = if let Ok(content) = fs::read_to_string(HOSTS_FILE_PATH) {
        content.contains(BLOCK_START_TAG)
    } else {
        false
    };

    NetworkTrafficSummary {
        total_rx_bytes_sec: total_rx,
        total_tx_bytes_sec: total_tx,
        active_interface_name: active_iface,
        is_adblock_active,
        blocked_domains_count: if is_adblock_active { AD_DOMAINS.len() } else { 0 },
    }
}

fn strip_adblock_block(original: &[u8]) -> Vec<u8> {
    let start = BLOCK_START_TAG.as_bytes();
    let end = BLOCK_END_TAG.as_bytes();

    let mut result: Vec<u8> = Vec::with_capacity(original.len());
    let mut i = 0usize;

    while i < original.len() {
        let is_start = original[i..].starts_with(start);
        if is_start {
            if let Some(rel_end) = find_subslice(&original[i..], end) {
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

fn write_hosts_atomically(path: &Path, content: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let dir = path
        .parent()
        .ok_or_else(|| "hosts 파일 경로가 올바르지 않습니다.".to_string())?;
    let original = fs::read(path).map_err(|e| format!("hosts 원본 읽기 실패: {}", e))?;
    let stamp = format!("{}_{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().ok_or("시간 조회 실패")?);
    let backup_path = dir.join(format!("hosts.clicktotweak.backup.{}", stamp));
    let tmp_path = dir.join(format!("hosts.clicktotweak.{}.tmp", stamp));
    let mut backup = fs::OpenOptions::new().write(true).create_new(true).open(&backup_path)
        .map_err(|e| format!("hosts 백업 생성 실패: {}", e))?;
    backup.write_all(&original).and_then(|_| backup.sync_all())
        .map_err(|e| format!("hosts 백업 저장 실패 ({}): {}", backup_path.display(), e))?;
    let result = (|| -> Result<(), String> {
        let mut tmp = fs::OpenOptions::new().write(true).create_new(true).open(&tmp_path)
            .map_err(|e| format!("hosts 임시 파일 생성 실패: {}", e))?;
        tmp.write_all(content).and_then(|_| tmp.sync_all())
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
            fs::remove_file(&tmp_path).map_err(|cleanup| format!("{}; 임시 파일 정리 실패: {}", error, cleanup))?;
        }
        return Err(format!("{} (백업: {})", error, backup_path.display()));
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_adblock(enable: bool) -> Result<String, String> {
    let p = Path::new(HOSTS_FILE_PATH);
    if !p.exists() {
        return Err("hosts 파일을 찾을 수 없습니다.".to_string());
    }

    let original = fs::read(p).map_err(|e| {
        format!("hosts 파일을 읽을 수 없습니다 (관리자 권한 필요): {}", e)
    })?;

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

    write_hosts_atomically(p, &new_content)?;

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
    use std::process::Command;
    let output = Command::new("ipconfig")
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

        assert!(!result.windows(BLOCK_START_TAG.len()).any(|w| w == BLOCK_START_TAG.as_bytes()));
        assert!(!result.windows(BLOCK_END_TAG.len()).any(|w| w == BLOCK_END_TAG.as_bytes()));
        assert!(result
            .windows("127.0.0.1 localhost".len())
            .any(|w| w == b"127.0.0.1 localhost"));
        assert!(result
            .windows("# user comment".len())
            .any(|w| w == b"# user comment"));
        assert!(result
            .windows("192.168.0.1 router".len())
            .any(|w| w == b"192.168.0.1 router"));
    }

    #[test]
    fn strip_block_is_idempotent_when_block_absent() {
        let original = b"127.0.0.1 localhost\n10.0.0.1 nas\n";
        let result = strip_adblock_block(original);
        assert_eq!(result, original);
    }

    #[test]
    fn strip_multiple_blocks_preserves_intervening_data() {
        let block = format!("{}\n0.0.0.0 example.test\n{}\n", BLOCK_START_TAG, BLOCK_END_TAG);
        let input = format!("first\n{}middle\n{}last\n", block, block);
        assert_eq!(strip_adblock_block(input.as_bytes()), b"first\nmiddle\nlast\n");
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
        let dir = std::env::temp_dir().join(format!("ctt_hosts_atomic_test_{}_{}", std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        fs::create_dir(&dir).unwrap();
        let hosts = dir.join("hosts");
        fs::write(&hosts, b"original\n").unwrap();

        write_hosts_atomically(&hosts, b"replaced\n").unwrap();

        assert_eq!(fs::read(&hosts).unwrap(), b"replaced\n");
        let backups: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("hosts.clicktotweak.backup."))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(backups[0].path()).unwrap(), b"original\n");

        fs::remove_dir_all(&dir).unwrap();
    }
}
