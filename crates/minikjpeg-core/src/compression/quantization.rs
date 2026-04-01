/// İçerik-adaptif dinamik kuantizasyon analizi.
///
/// Görüntünün frekans içeriğini analiz ederek hangi bölgelerin
/// insan gözü için önemli olduğunu hesaplar.
/// Bu bilgi, SSIM döngüsünde kalite seçimini iyileştirir.

use image::RgbImage;

const BLOCK_SIZE: usize = 8;

/// Görüntünün frekans içerik karmaşıklığını 0.0-1.0 arasında döndürür.
/// 0.0 = düz/tekdüze görüntü, 1.0 = yüksek detaylı görüntü
pub fn analyze_content_complexity(image: &RgbImage) -> f32 {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;

    if w < BLOCK_SIZE || h < BLOCK_SIZE {
        return 0.5;
    }

    // Luma kanalına dönüştür
    let raw = image.as_raw();
    let luma: Vec<f32> = raw
        .chunks_exact(3)
        .map(|px| {
            0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32
        })
        .collect();

    // Blok varyanslarını hesapla (%10 örnekleme)
    let blocks_x = w / BLOCK_SIZE;
    let blocks_y = h / BLOCK_SIZE;
    let step = (blocks_x.max(blocks_y) / 10).max(1);

    let mut total_variance = 0.0f32;
    let mut block_count = 0usize;

    let mut by = 0;
    while by < blocks_y {
        let mut bx = 0;
        while bx < blocks_x {
            let var = block_variance(&luma, bx * BLOCK_SIZE, by * BLOCK_SIZE, w);
            total_variance += var;
            block_count += 1;
            bx += step;
        }
        by += step;
    }

    if block_count == 0 {
        return 0.5;
    }

    let avg_variance = total_variance / block_count as f32;

    // Varyansı 0-1 arasına normalize et.
    // 255^2 / 4 ≈ 16256 maksimum pratik varyans değeri
    (avg_variance / 4000.0).min(1.0)
}

/// Bir 8x8 bloğun piksel varyansını hesaplar.
fn block_variance(luma: &[f32], x: usize, y: usize, width: usize) -> f32 {
    let n = (BLOCK_SIZE * BLOCK_SIZE) as f32;
    let mut sum = 0.0f32;
    let mut sum_sq = 0.0f32;

    for row in y..y + BLOCK_SIZE {
        for col in x..x + BLOCK_SIZE {
            let v = luma[row * width + col];
            sum += v;
            sum_sq += v * v;
        }
    }

    let mean = sum / n;
    (sum_sq / n) - (mean * mean)
}

/// İçerik karmaşıklığına göre önerilen kalite aralığını döndürür.
/// (min_quality, max_quality)
pub fn suggest_quality_range(complexity: f32) -> (u8, u8) {
    if complexity < 0.1 {
        // Düz/gradyan görüntü — agresif sıkıştırma mümkün
        (30, 75)
    } else if complexity < 0.3 {
        // Orta detay
        (45, 82)
    } else if complexity < 0.6 {
        // Yüksek detay (manzara, doku)
        (55, 88)
    } else {
        // Çok yüksek detay (keskin kenarlar, metin)
        (65, 92)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_image_has_low_complexity() {
        let img = RgbImage::from_pixel(64, 64, image::Rgb([128, 128, 128]));
        let c = analyze_content_complexity(&img);
        assert!(c < 0.05, "complexity = {}", c);
    }
}
