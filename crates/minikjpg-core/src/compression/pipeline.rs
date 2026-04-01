use crate::{
    compression::{
        jpeg::decode_jpeg,
        quantization::{analyze_content_complexity, suggest_quality_range},
    },
    error::CoreResult,
    io::{reader::read_file, writer::write_atomic},
    metadata::exif::{extract_app_segments, inject_app_segments, read_orientation},
    quality::search::find_optimal_quality,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Sıkıştırma seçenekleri.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOptions {
    /// Hedef SSIM eşiği (0.0-1.0). Varsayılan: 0.92
    pub ssim_threshold: f32,
    /// Hedef PSNR eşiği (dB). Varsayılan: 40.0
    pub psnr_threshold: f32,
    /// Kalite arama üst sınırı. Varsayılan: 95
    pub max_quality: u8,
    /// Kalite arama alt sınırı. Varsayılan: 20
    pub min_quality: u8,
    /// EXIF/IPTC/XMP metadata'yı koru. Varsayılan: true
    pub preserve_metadata: bool,
    /// İçerik analizi ile kalite aralığını otomatik ayarla. Varsayılan: true
    pub adaptive_quality: bool,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            ssim_threshold: 0.92,
            psnr_threshold: 40.0,
            max_quality: 95,
            min_quality: 20,
            preserve_metadata: true,
            adaptive_quality: true,
        }
    }
}

/// Tek dosya sıkıştırma sonucu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub input_path: String,
    pub output_path: String,
    pub original_size: u64,
    pub compressed_size: u64,
    /// Tasarruf oranı (0.0-1.0). Örn. 0.45 = %45 küçüldü
    pub savings_ratio: f32,
    pub achieved_ssim: f32,
    pub achieved_psnr: f32,
    pub quality_used: u8,
    pub elapsed_ms: u64,
}

impl CompressionResult {
    pub fn savings_percent(&self) -> f32 {
        self.savings_ratio * 100.0
    }
}

/// Bir JPEG dosyasını sıkıştırır.
///
/// # Akış
/// 1. Dosyayı oku
/// 2. APP segmentlerini (EXIF/ICC) çıkar
/// 3. Piksel verisini decode et + orientation düzelt
/// 4. İçerik karmaşıklığı analizi
/// 5. SSIM tabanlı ikili arama ile optimal kaliteyi bul
/// 6. Metadata'yı geri ekle
/// 7. Atomik yaz
pub async fn compress_jpeg(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    options: &CompressionOptions,
) -> CoreResult<CompressionResult> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    let start = Instant::now();

    tracing::info!(
        input = %input_path.display(),
        output = %output_path.display(),
        "Sıkıştırma başlıyor"
    );

    // 1. Dosyayı oku
    let raw_data = read_file(input_path).await?;
    let original_size = raw_data.len() as u64;

    // 2. APP segmentlerini çıkar
    let app_segments = if options.preserve_metadata {
        extract_app_segments(&raw_data)
    } else {
        vec![]
    };

    // 3. Piksel verisini decode et
    let mut image = decode_jpeg(&raw_data)?;

    // EXIF orientation düzeltmesi
    if options.preserve_metadata {
        if let Some(orientation) = read_orientation(&app_segments) {
            image = apply_orientation(image, orientation);
        }
    }

    // 4. İçerik karmaşıklığı analizi → kalite aralığını ayarla
    let (min_q, max_q) = if options.adaptive_quality {
        let complexity = analyze_content_complexity(&image);
        let (suggested_min, suggested_max) = suggest_quality_range(complexity);
        tracing::debug!(complexity, suggested_min, suggested_max, "İçerik analizi");
        (
            suggested_min.max(options.min_quality),
            suggested_max.min(options.max_quality),
        )
    } else {
        (options.min_quality, options.max_quality)
    };

    // 5. SSIM tabanlı ikili arama
    let encoding = find_optimal_quality(
        &image,
        options.ssim_threshold,
        options.psnr_threshold,
        min_q,
        max_q,
    )?;

    // 6. Metadata'yı geri ekle
    let final_bytes = if options.preserve_metadata && !app_segments.is_empty() {
        inject_app_segments(&encoding.compressed_bytes, &app_segments)
    } else {
        encoding.compressed_bytes
    };

    let compressed_size = final_bytes.len() as u64;

    // Eğer sıkıştırma sonucu büyüdüyse orijinali koru
    let (output_bytes, final_compressed_size) = if compressed_size >= original_size {
        tracing::warn!(
            original_size,
            compressed_size,
            "Sıkıştırma orijinalden büyük, orijinal korunuyor"
        );
        (raw_data, original_size)
    } else {
        (final_bytes, compressed_size)
    };

    // 7. Atomik yaz
    write_atomic(output_path, &output_bytes).await?;

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let savings_ratio = 1.0 - (final_compressed_size as f32 / original_size as f32);

    tracing::info!(
        quality_used = encoding.quality,
        achieved_ssim = encoding.achieved_ssim,
        savings_percent = savings_ratio * 100.0,
        elapsed_ms,
        "Sıkıştırma tamamlandı"
    );

    Ok(CompressionResult {
        input_path: input_path.to_string_lossy().into_owned(),
        output_path: output_path.to_string_lossy().into_owned(),
        original_size,
        compressed_size: final_compressed_size,
        savings_ratio,
        achieved_ssim: encoding.achieved_ssim,
        achieved_psnr: encoding.achieved_psnr,
        quality_used: encoding.quality,
        elapsed_ms,
    })
}

/// Çıktı yolunu oluşturur.
/// suffix = "" ise üzerine yazar; suffix = "_opt" ise dosya_opt.jpg
pub fn make_output_path(
    input_path: &Path,
    output_dir: Option<&Path>,
    suffix: &str,
) -> PathBuf {
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");

    let new_filename = format!("{}{}.{}", stem, suffix, ext);

    match output_dir {
        Some(dir) => dir.join(new_filename),
        None => input_path.with_file_name(new_filename),
    }
}

/// EXIF orientation değerine göre görüntüyü döndürür.
fn apply_orientation(image: image::RgbImage, orientation: u16) -> image::RgbImage {
    use image::imageops;
    match orientation {
        3 => imageops::rotate180(&image),
        6 => imageops::rotate90(&image),
        8 => imageops::rotate270(&image),
        _ => image,
    }
}
