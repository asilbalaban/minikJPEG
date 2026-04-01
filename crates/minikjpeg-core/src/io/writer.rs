use crate::error::CoreResult;
use std::path::Path;
use tokio::fs;

/// Veriyi atomik olarak yazar: önce geçici dosyaya yazar, ardından rename ile taşır.
/// Bu, yarım dosya oluşmasını önler.
pub async fn write_atomic(path: impl AsRef<Path>, data: &[u8]) -> CoreResult<()> {
    let path = path.as_ref();
    let tmp_path = path.with_extension("tmp_minik");

    fs::write(&tmp_path, data).await?;
    fs::rename(&tmp_path, path).await?;

    Ok(())
}
