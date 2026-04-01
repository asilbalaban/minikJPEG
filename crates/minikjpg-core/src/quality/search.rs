use crate::compression::jpeg::{encode_jpeg, decode_jpeg, JpegEncodeParams};
use crate::error::{CoreError, CoreResult};
use crate::quality::{ssim::calculate_ssim, psnr::calculate_psnr};
use image::RgbImage;

/// Optimal kalite arama sonucu.
#[derive(Debug, Clone)]
pub struct OptimalEncoding {
    pub quality: u8,
    pub compressed_bytes: Vec<u8>,
    pub achieved_ssim: f32,
    pub achieved_psnr: f32,
    pub compression_ratio: f32,
}

/// SSIM tabanlı ikili arama ile hedef kaliteyi sağlayan minimum JPEG kalitesini bulur.
///
/// `target_ssim`: Hedef SSIM eşiği (örn. 0.92)
/// `min_quality`: Arama alt sınırı (örn. 20)
/// `max_quality`: Arama üst sınırı (örn. 95)
pub fn find_optimal_quality(
    image: &RgbImage,
    target_ssim: f32,
    target_psnr: f32,
    min_quality: u8,
    max_quality: u8,
) -> CoreResult<OptimalEncoding> {
    let (width, height) = image.dimensions();
    let original_bytes = image.as_raw();
    let original_size = original_bytes.len();

    let mut low = min_quality as i32;
    let mut high = max_quality as i32;
    let mut best: Option<OptimalEncoding> = None;

    // İkili arama: hedef SSIM'i sağlayan en düşük kaliteyi bul
    while low <= high {
        let mid = (low + high) / 2;
        let quality = mid as u8;

        let params = JpegEncodeParams {
            quality,
            progressive: true,
            optimize_coding: true,
        };

        let compressed = encode_jpeg(image, &params)
            .map_err(|e| CoreError::JpegEncode(e.to_string()))?;

        let decoded = decode_jpeg(&compressed)
            .map_err(|e| CoreError::JpegDecode(e.to_string()))?;

        let decoded_bytes = decoded.as_raw();
        let achieved_ssim = calculate_ssim(original_bytes, decoded_bytes, width, height);
        let achieved_psnr = calculate_psnr(original_bytes, decoded_bytes);

        let meets_threshold = achieved_ssim >= target_ssim && achieved_psnr >= target_psnr;

        tracing::debug!(
            quality,
            achieved_ssim,
            achieved_psnr,
            meets_threshold,
            "SSIM arama iterasyonu"
        );

        if meets_threshold {
            // Bu kalite yeterli — daha düşük kalite dene
            let ratio = 1.0 - (compressed.len() as f32 / original_size as f32);
            best = Some(OptimalEncoding {
                quality,
                compressed_bytes: compressed,
                achieved_ssim,
                achieved_psnr,
                compression_ratio: ratio,
            });
            high = mid - 1;
        } else {
            // Kalite yetersiz — daha yüksek kalite dene
            low = mid + 1;
        }
    }

    match best {
        Some(encoding) => Ok(encoding),
        None => {
            // Maksimum kalitede bile eşiğe ulaşılamadı — max kaliteyle encode et
            let params = JpegEncodeParams {
                quality: max_quality,
                progressive: true,
                optimize_coding: true,
            };
            let compressed = encode_jpeg(image, &params)
                .map_err(|e| CoreError::JpegEncode(e.to_string()))?;
            let decoded = decode_jpeg(&compressed)
                .map_err(|e| CoreError::JpegDecode(e.to_string()))?;
            let decoded_bytes = decoded.as_raw();
            let achieved_ssim = calculate_ssim(original_bytes, decoded_bytes, width, height);
            let achieved_psnr = calculate_psnr(original_bytes, decoded_bytes);
            let ratio = 1.0 - (compressed.len() as f32 / original_size as f32);

            tracing::warn!(
                achieved_ssim,
                target_ssim,
                "Hedef SSIM eşiğine ulaşılamadı, maksimum kalite kullanıldı"
            );

            Ok(OptimalEncoding {
                quality: max_quality,
                compressed_bytes: compressed,
                achieved_ssim,
                achieved_psnr,
                compression_ratio: ratio,
            })
        }
    }
}
