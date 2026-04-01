use crate::error::{CoreError, CoreResult};
use image::RgbImage;
use std::panic;

/// JPEG encode parametreleri.
#[derive(Debug, Clone)]
pub struct JpegEncodeParams {
    /// Kalite seviyesi (1-100). mozjpeg'e iletilir.
    pub quality: u8,
    /// Progressive JPEG üret (web için daha iyi)
    pub progressive: bool,
    /// Huffman tablosunu optimize et (daha küçük dosya, daha yavaş)
    pub optimize_coding: bool,
}

impl Default for JpegEncodeParams {
    fn default() -> Self {
        Self {
            quality: 82,
            progressive: true,
            optimize_coding: true,
        }
    }
}

/// RgbImage'ı mozjpeg ile JPEG baytlarına encode eder.
pub fn encode_jpeg(image: &RgbImage, params: &JpegEncodeParams) -> CoreResult<Vec<u8>> {
    let (width, height) = image.dimensions();
    let raw = image.as_raw().clone();
    let quality = params.quality;
    let progressive = params.progressive;

    // mozjpeg panic kullanır — catch_unwind ile sarmalıyoruz
    let result = panic::catch_unwind(move || -> std::io::Result<Vec<u8>> {
        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        compress.set_size(width as usize, height as usize);
        compress.set_quality(quality as f32);

        if progressive {
            compress.set_progressive_mode();
        }

        // 4:2:0 kroma subsampling (varsayılan, iyi sıkıştırma oranı)
        compress.set_chroma_sampling_pixel_sizes((2, 2), (2, 2));

        let output = Vec::new();
        let mut started = compress.start_compress(output)?;
        started.write_scanlines(&raw)?;
        let output = started.finish()?;
        Ok(output)
    });

    match result {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(e)) => Err(CoreError::JpegEncode(e.to_string())),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "mozjpeg panic".to_string()
            };
            Err(CoreError::JpegEncode(msg))
        }
    }
}

/// JPEG baytlarını RgbImage'a decode eder.
pub fn decode_jpeg(data: &[u8]) -> CoreResult<RgbImage> {
    let img = image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)
        .map_err(|e| CoreError::JpegDecode(e.to_string()))?
        .into_rgb8();
    Ok(img)
}

/// Ham JPEG baytlarını decode eder ve piksel boyutlarını döndürür.
pub fn decode_jpeg_dimensions(data: &[u8]) -> CoreResult<(u32, u32)> {
    use image::ImageReader;
    let reader = ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| CoreError::JpegDecode(e.to_string()))?;
    let (w, h) = reader
        .into_dimensions()
        .map_err(|e| CoreError::JpegDecode(e.to_string()))?;
    Ok((w, h))
}
