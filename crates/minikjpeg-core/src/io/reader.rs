use crate::error::CoreResult;
use std::path::Path;
use tokio::fs;

/// Dosyayı asenkron olarak okur ve ham baytları döndürür.
pub async fn read_file(path: impl AsRef<Path>) -> CoreResult<Vec<u8>> {
    let bytes = fs::read(path).await?;
    Ok(bytes)
}

/// Dosya boyutunu döndürür (bayt).
pub async fn file_size(path: impl AsRef<Path>) -> CoreResult<u64> {
    let meta = fs::metadata(path).await?;
    Ok(meta.len())
}
