use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[tauri::command]
pub fn shred_file(file_path: String, passes: u32) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("지정된 파일이 존재하지 않습니다.".to_string());
    }
    if !path.is_file() {
        return Err("폴더는 직접 파쇄할 수 없습니다. 파일을 지정해주세요.".to_string());
    }

    // 파일 크기 획득
    let metadata = fs::metadata(path).map_err(|e| format!("파일 메타데이터 조회 실패: {}", e))?;
    let file_size = metadata.len();

    // 읽기 전용 속성 해제
    let mut permissions = metadata.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    let _ = fs::set_permissions(path, permissions);

    // 최소 1패스, 최대 7패스
    let total_passes = passes.clamp(1, 7);

    // 버퍼 크기 (64KB 단위)
    const CHUNK_SIZE: usize = 64 * 1024;
    let zero_buffer = vec![0u8; CHUNK_SIZE];
    let ones_buffer = vec![0xFFu8; CHUNK_SIZE];
    let mut rand_buffer = vec![0u8; CHUNK_SIZE];

    let mut file = OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|e| format!("파일 열기 실패 (다른 프로그램에서 사용 중일 수 있습니다): {}", e))?;

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
                    // pseudo-random fill
                    let seed = (p as u64).wrapping_mul(6364136223846793005).wrapping_add(remaining);
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

    // 크기를 0으로 절삭
    file.set_len(0).map_err(|e| format!("파일 절삭 실패: {}", e))?;
    drop(file);

    // 파일 이름 무작위 변경 (MFT 흔적 제거)
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let temp_shred_name = parent.join(format!("_shredded_{}.tmp", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()));
    let _ = fs::rename(path, &temp_shred_name);

    // 최종 삭제
    let target_to_delete = if temp_shred_name.exists() {
        temp_shred_name.as_path()
    } else {
        path
    };

    fs::remove_file(target_to_delete)
        .map_err(|e| format!("최종 파일 삭제 실패: {}", e))?;

    Ok(format!(
        "'{}' 파일이 {}단계 덮어쓰기 파쇄 후 물리적으로 영구 삭제되었습니다.",
        path.file_name().and_then(|f| f.to_str()).unwrap_or("파일"),
        total_passes
    ))
}
