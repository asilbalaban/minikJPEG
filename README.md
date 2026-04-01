# minikJPEG — Algısal Kayıpsız JPEG Optimizer

İnsan gözünün fark edemeyeceği kalite kaybıyla JPEG dosyalarının boyutunu dinamik olarak küçülten masaüstü yazılımı.

## Özellikler

- **SSIM tabanlı kalite arama** — İkili arama ile hedef SSIM eşiğini sağlayan minimum kaliteyi bulur
- **İçerik-adaptif kuantizasyon** — Görüntü karmaşıklığına göre kalite aralığını otomatik ayarlar
- **EXIF/IPTC/XMP metadata koruması** — Piksel verisi değişirken metadata korunur
- **EXIF orientation düzeltmesi** — Döndürülmüş fotoğraflar otomatik düzeltilir
- **Multithreading** — Tüm CPU çekirdeklerini kullanarak toplu işlem
- **Tauri GUI** — Sürükle-bırak arayüzü, ilerleme çubuğu, sonuç özeti
- **CLI aracı** — Otomasyon için komut satırı desteği

## İndirme (Windows)

Derlemeden kullanmak isteyenler için hazır kurulum dosyaları:

| Dosya | Tür | Yol |
|-------|-----|-----|
| `minikJPEG_0.1.0_x64-setup.exe` | NSIS Installer | `target/release/bundle/nsis/` |
| `minikJPEG_0.1.0_x64_en-US.msi` | MSI Installer | `target/release/bundle/msi/` |
| `minikJPEG.exe` | Taşınabilir (kurulum gerektirmez) | `target/release/` |

> Kaynak koddan derlemek için aşağıdaki gereksinimlere bakın.

## Gereksinimler

- Rust 1.75+
- Visual Studio Build Tools 2019/2022 (C++ workload)
- CMake 3.25+
- NASM 2.16+
- Node.js 18+

## Kurulum

```bash
# Bağımlılıkları kur
cd frontend && npm install

# CLI aracı derle
cargo build --release -p minikjpeg-cli

# GUI geliştirme sunucusu başlat
cargo tauri dev
```

## CLI Kullanımı

```bash
# Tek dosya optimize et
minikjpeg compress resim.jpg --ssim 0.92

# Çıktıyı farklı konuma kaydet
minikjpeg compress resim.jpg --output resim_opt.jpg

# Klasördeki tüm JPEG'leri optimize et
minikjpeg batch ./fotograflar --output-dir ./optimize --recursive

# Agresif sıkıştırma (web için)
minikjpeg compress resim.jpg --ssim 0.87 --suffix _web
```

## Mimari

```
crates/
  minikjpeg-core/     # Çekirdek kütüphane
    compression/     # mozjpeg entegrasyonu, pipeline
    quality/         # SSIM, PSNR, kalite arama döngüsü
    metadata/        # EXIF/ICC segment yönetimi
    batch/           # rayon tabanlı paralel işlem
    io/              # Async dosya okuma/yazma
  minikjpeg-cli/      # CLI binary (clap)
  minikjpeg-tauri/    # Tauri masaüstü uygulaması
frontend/            # React/TypeScript GUI
```

## SSIM Eşiği Rehberi

| Değer | Kullanım | Açıklama |
|-------|----------|----------|
| 0.87  | Web      | En küçük dosya, web kalitesi yeterli |
| 0.92  | Genel    | **Varsayılan** — ideal denge |
| 0.96  | Baskı    | Yüksek kalite, minimal kayıp |
