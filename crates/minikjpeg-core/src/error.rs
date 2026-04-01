use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("I/O hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("Görüntü decode hatası ({path}): {source}")]
    ImageDecode {
        path: String,
        source: image::ImageError,
    },

    #[error("JPEG encode hatası: {0}")]
    JpegEncode(String),

    #[error("JPEG decode hatası: {0}")]
    JpegDecode(String),

    #[error("Metadata okuma hatası ({path}): {source}")]
    MetadataRead {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Metadata yazma hatası ({path}): {source}")]
    MetadataWrite {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Hedef SSIM eşiğine ulaşılamadı (elde edilen: {achieved:.4}, hedef: {target:.4})")]
    QualityBelowThreshold { achieved: f32, target: f32 },

    #[error("Batch işlemde kısmi başarısızlık: {succeeded} başarılı, {failed} başarısız")]
    BatchPartialFailure { succeeded: usize, failed: usize },

    #[error("Geçersiz parametre: {0}")]
    InvalidParameter(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
