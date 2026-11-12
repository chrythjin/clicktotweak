use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[tauri::command(async)]
pub fn shred_file(file_path: String, passes: u32) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("지정된 파일이 존재하지 않습니다.".to_string());
    }
    if !path.is_file() {
        return Err("폴더는 직접 파쇄할 수 없습니다. 파일을 지정해주세요.".to_string());
    }

    let total_passes = passes.clamp(1, 7);
    overwrite_with_patterns(path, total_passes)?;

    let file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("파일 절삭 준비 실패: {}", e))?;
    file.set_len(0)
        .map_err(|e| format!("파일 절삭 실패: {}", e))?;
    drop(file);

    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let temp_shred_name = parent.join(format!(
        "_shredded_{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));
    let renamed = fs::rename(path, &temp_shred_name).is_ok();
    let target_to_delete = if renamed {
        temp_shred_name.as_path()
    } else {
        path
    };

    fs::remove_file(target_to_delete).map_err(|e| format!("최종 파일 삭제 실패: {}", e))?;

    Ok(format!(
        "'{}' 파일이 {}단계 덮어쓰기 파쇄 후 물리적으로 영구 삭제되었습니다.",
        path.file_name().and_then(|f| f.to_str()).unwrap_or("파일"),
        total_passes
    ))
}

fn overwrite_with_patterns(path: &Path, total_passes: u32) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|e| format!("파일 메타데이터 조회 실패: {}", e))?;
    let file_size = metadata.len();

    let mut permissions = metadata.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    let _ = fs::set_permissions(path, permissions);

    const CHUNK_SIZE: usize = 64 * 1024;
    let zero_buffer = vec![0u8; CHUNK_SIZE];
    let ones_buffer = vec![0xFFu8; CHUNK_SIZE];
    let mut rand_buffer = vec![0u8; CHUNK_SIZE];

    let mut file = OpenOptions::new().write(true).open(path).map_err(|e| {
        format!(
            "파일 열기 실패 (다른 프로그램에서 사용 중일 수 있습니다): {}",
            e
        )
    })?;

    for p in 1..=total_passes {
        file.seek(SeekFrom::Start(0))
            .map_err(|e| format!("시크 실패: {}", e))?;

        let mut remaining = file_size;
        let pattern_type = p % 3;

        while remaining > 0 {
            let write_len = std::cmp::min(remaining as usize, CHUNK_SIZE);
            let buf = match pattern_type {
                1 => &zero_buffer[..write_len],
                2 => &ones_buffer[..write_len],
                _ => {
                    let seed = (p as u64)
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(remaining);
                    for (i, b) in rand_buffer[..write_len].iter_mut().enumerate() {
                        *b = ((seed.wrapping_add(i as u64)) & 0xFF) as u8;
                    }
                    &rand_buffer[..write_len]
                }
            };

            file.write_all(buf)
                .map_err(|e| format!("데이터 덮어쓰기 실패 (Pass {}): {}", p, e))?;
            remaining -= write_len as u64;
        }

        file.flush().map_err(|e| format!("플러시 실패: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str, content: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ctt_shred_{}_{}_{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join(name);
        fs::write(&file, content).unwrap();
        file
    }

    #[test]
    fn overwrite_single_pass_writes_zero_pattern_and_keeps_length() {
        let file = fixture("single.txt", b"SECRET DATA");
        let original_len = fs::metadata(&file).unwrap().len();

        overwrite_with_patterns(&file, 1).unwrap();

        let content = fs::read(&file).unwrap();
        assert_eq!(content.len() as u64, original_len);
        assert!(content.iter().all(|b| *b == 0x00));
        assert_ne!(content, b"SECRET DATA");

        fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }

    #[test]
    fn overwrite_second_pass_writes_ones_pattern() {
        let file = fixture("second.txt", b"SECRET DATA");

        overwrite_with_patterns(&file, 2).unwrap();

        let content = fs::read(&file).unwrap();
        assert!(content.iter().all(|b| *b == 0xFF));

        fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }

    #[test]
    fn shred_file_removes_target_and_reports_clamped_pass_count() {
        let file = fixture("shred.txt", b"SECRET DATA");

        let message = shred_file(file.to_string_lossy().to_string(), 0).unwrap();

        assert!(message.contains("1단계"));
        assert!(!file.exists());

        fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }

    #[test]
    fn shred_file_clamps_high_pass_count_to_seven() {
        let file = fixture("shred7.txt", b"SECRET DATA");

        let message = shred_file(file.to_string_lossy().to_string(), 99).unwrap();

        assert!(message.contains("7단계"));
        assert!(!file.exists());

        fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }

    #[test]
    fn shred_file_leaves_no_renamed_residue_in_directory() {
        let file = fixture("residue.txt", b"SECRET DATA");
        let dir = file.parent().unwrap().to_path_buf();

        shred_file(file.to_string_lossy().to_string(), 3).unwrap();

        let residue: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("_shredded_"))
            .collect();
        assert!(residue.is_empty());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn shred_file_rejects_directory_and_missing_path() {
        let dir = std::env::temp_dir().join(format!(
            "ctt_shred_dir_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();

        assert!(shred_file(dir.to_string_lossy().to_string(), 3).is_err());
        assert!(shred_file(dir.join("nope.txt").to_string_lossy().to_string(), 3).is_err());

        fs::remove_dir_all(&dir).unwrap();
    }
}
