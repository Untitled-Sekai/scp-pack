use std::path::Path;
use crate::error::{Result, ScpError};

/// ファイルパスを正規化し、スラッシュ区切りの相対パスに変換
pub fn normalize_path(path: &Path, base: &Path) -> Result<String> {
    let relative = path.strip_prefix(base)
        .map_err(|_| ScpError::InvalidPath(path.to_string_lossy().to_string()))?;
    
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

/// パックディレクトリが有効かチェック
pub fn validate_pack_dir(pack_dir: &Path) -> Result<()> {
    if !pack_dir.exists() {
        return Err(ScpError::InvalidPath(
            format!("Pack directory does not exist: {}", pack_dir.display())
        ));
    }
    
    let db_path = pack_dir.join("db.json");
    if !db_path.exists() {
        return Err(ScpError::InvalidFormat(
            "db.json not found in pack directory".to_string()
        ));
    }
    
    Ok(())
}

/// SCPファイルが有効かチェック
pub fn validate_scp_file(scp_file: &Path) -> Result<()> {
    if !scp_file.exists() {
        return Err(ScpError::InvalidPath(
            format!("SCP file does not exist: {}", scp_file.display())
        ));
    }
    
    if scp_file.extension().and_then(|s| s.to_str()) != Some("scp") {
        return Err(ScpError::InvalidFormat(
            "File must have .scp extension".to_string()
        ));
    }
    
    Ok(())
}

/// 出力ディレクトリを準備（存在しない場合は作成）
pub fn prepare_output_dir(output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }
    Ok(())
}