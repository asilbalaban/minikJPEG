/// PSNR (Peak Signal-to-Noise Ratio) hesaplama modülü.
///
/// 40+ dB = algısal olarak kayıpsız
/// 35-40 dB = iyi kalite
/// 30-35 dB = orta kalite
/// <30 dB = görünür kalite kaybı

/// İki görüntü arasındaki PSNR'ı hesaplar (dB cinsinden).
pub fn calculate_psnr(original: &[u8], compressed: &[u8]) -> f32 {
    if original.len() != compressed.len() || original.is_empty() {
        return 0.0;
    }

    let mse: f64 = original
        .iter()
        .zip(compressed.iter())
        .map(|(&a, &b)| {
            let diff = a as f64 - b as f64;
            diff * diff
        })
        .sum::<f64>()
        / original.len() as f64;

    if mse == 0.0 {
        return f32::INFINITY;
    }

    let max_val = 255.0f64;
    (10.0 * (max_val * max_val / mse).log10()) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_infinite_psnr() {
        let img = vec![128u8; 100];
        let psnr = calculate_psnr(&img, &img);
        assert!(psnr.is_infinite());
    }

    #[test]
    fn psnr_above_40_is_lossless() {
        // Küçük gürültü ile 40+ dB beklenir
        let original: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let compressed: Vec<u8> = original.iter().map(|&x| x.saturating_add(1)).collect();
        let psnr = calculate_psnr(&original, &compressed);
        assert!(psnr > 40.0, "PSNR = {}", psnr);
    }
}
