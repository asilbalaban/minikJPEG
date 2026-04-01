/// SSIM (Structural Similarity Index) hesaplama modülü.
///
/// İnsan görsel sistemini modelleyen bu metrik, iki görüntünün algısal
/// benzerliğini 0.0-1.0 arasında bir değerle ölçer.
/// 1.0 = tam özdeş, 0.92+ = algısal olarak kayıpsız.

const C1: f64 = 6.5025;   // (0.01 * 255)^2
const C2: f64 = 58.5225;  // (0.03 * 255)^2
const WINDOW_SIZE: usize = 8;

/// İki görüntü için SSIM hesaplar.
/// Her iki görüntü de aynı boyutta RGB bayt dizisi olmalıdır.
pub fn calculate_ssim(original: &[u8], compressed: &[u8], width: u32, height: u32) -> f32 {
    let w = width as usize;
    let h = height as usize;

    // Görüntüleri luma (Y) kanalına dönüştür
    let luma_orig = rgb_to_luma(original);
    let luma_comp = rgb_to_luma(compressed);

    // Örneklemeli SSIM: görüntüyü 4x4 bölgeye ayır, her bölgeden pencereler al
    let grid_x = 4usize;
    let grid_y = 4usize;
    let windows_per_cell = 4usize;

    let mut total_ssim = 0.0f64;
    let mut window_count = 0usize;

    for gy in 0..grid_y {
        for gx in 0..grid_x {
            let cell_x_start = (gx * w) / grid_x;
            let cell_x_end = ((gx + 1) * w) / grid_x;
            let cell_y_start = (gy * h) / grid_y;
            let cell_y_end = ((gy + 1) * h) / grid_y;

            if cell_x_end < cell_x_start + WINDOW_SIZE
                || cell_y_end < cell_y_start + WINDOW_SIZE
            {
                continue;
            }

            // Her hücreden düzenli aralıklarla pencere al
            let step_x = (cell_x_end - cell_x_start - WINDOW_SIZE) / windows_per_cell + 1;
            let step_y = (cell_y_end - cell_y_start - WINDOW_SIZE) / windows_per_cell + 1;

            let mut wx = cell_x_start;
            while wx + WINDOW_SIZE <= cell_x_end {
                let mut wy = cell_y_start;
                while wy + WINDOW_SIZE <= cell_y_end {
                    let s = window_ssim(&luma_orig, &luma_comp, wx, wy, w);
                    total_ssim += s;
                    window_count += 1;
                    wy += step_y.max(1);
                }
                wx += step_x.max(1);
            }
        }
    }

    if window_count == 0 {
        return 1.0;
    }

    (total_ssim / window_count as f64) as f32
}

/// Tek bir 8x8 pencere için SSIM hesaplar.
fn window_ssim(luma_a: &[f64], luma_b: &[f64], x: usize, y: usize, width: usize) -> f64 {
    let mut sum_a = 0.0f64;
    let mut sum_b = 0.0f64;
    let n = (WINDOW_SIZE * WINDOW_SIZE) as f64;

    for row in y..y + WINDOW_SIZE {
        for col in x..x + WINDOW_SIZE {
            let idx = row * width + col;
            sum_a += luma_a[idx];
            sum_b += luma_b[idx];
        }
    }

    let mean_a = sum_a / n;
    let mean_b = sum_b / n;

    let mut var_a = 0.0f64;
    let mut var_b = 0.0f64;
    let mut cov_ab = 0.0f64;

    for row in y..y + WINDOW_SIZE {
        for col in x..x + WINDOW_SIZE {
            let idx = row * width + col;
            let da = luma_a[idx] - mean_a;
            let db = luma_b[idx] - mean_b;
            var_a += da * da;
            var_b += db * db;
            cov_ab += da * db;
        }
    }

    var_a /= n - 1.0;
    var_b /= n - 1.0;
    cov_ab /= n - 1.0;

    let numerator = (2.0 * mean_a * mean_b + C1) * (2.0 * cov_ab + C2);
    let denominator = (mean_a * mean_a + mean_b * mean_b + C1) * (var_a + var_b + C2);

    if denominator == 0.0 {
        1.0
    } else {
        (numerator / denominator).clamp(0.0, 1.0)
    }
}

/// RGB bayt dizisini luma (parlaklık) kanalına dönüştürür.
/// BT.709 katsayıları kullanılır.
fn rgb_to_luma(rgb: &[u8]) -> Vec<f64> {
    rgb.chunks_exact(3)
        .map(|px| {
            let r = px[0] as f64;
            let g = px[1] as f64;
            let b = px[2] as f64;
            0.2126 * r + 0.7152 * g + 0.0722 * b
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_ssim_is_one() {
        let img = vec![128u8; 64 * 64 * 3];
        let ssim = calculate_ssim(&img, &img, 64, 64);
        assert!((ssim - 1.0).abs() < 0.001, "SSIM = {}", ssim);
    }

    #[test]
    fn very_different_images_ssim_is_low() {
        let img_a = vec![0u8; 64 * 64 * 3];
        let img_b = vec![255u8; 64 * 64 * 3];
        let ssim = calculate_ssim(&img_a, &img_b, 64, 64);
        assert!(ssim < 0.5, "SSIM = {}", ssim);
    }
}
