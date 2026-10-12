# minikJPEG — Algısal Kayıpsız JPEG Optimizer

İnsan gözünün fark edemeyeceği kalite kaybıyla JPEG dosyalarının boyutunu dinamik olarak küçülten masaüstü yazılımı.

## Özellikler

- **SSIM tabanlı kalite arama** — İkili arama ile hedef SSIM eşiğini sağlayan minimum kaliteyi bulur
- **İçerik-adaptif kuantizasyon** — Görüntü karmaşıklığına göre kalite aralığını otomatik ayarlar
- **EXIF/IPTC/XMP metadata koruması** — Piksel verisi değişirken metadata korunur
- **EXIF orientation düzeltmesi** — Döndürülmüş fotoğraflar otomatik düzeltilir
- **PNG'den JPG** — Bırakılan PNG, aynı klasöre aynı adla JPG olarak kaydedilir; saydam alanlar beyaz zemine oturur
- **Multithreading** — Tüm CPU çekirdeklerini kullanarak toplu işlem
- **Tauri GUI** — Sürükle-bırak arayüzü, ilerleme çubuğu, sonuç özeti
- **CLI aracı** — Otomasyon için komut satırı desteği

## İndirme (Windows)

Derlemeden kullanmak isteyenler için hazır dosyalar [Releases](https://github.com/asilbalaban/minikJPEG/releases/latest) sayfasında:

| Dosya | Tür |
|-------|-----|
| `minikJPEG_<sürüm>_x64-setup.exe` | Kurulum (NSIS) |
| `minikJPEG_<sürüm>_x64_en-US.msi` | Kurulum (MSI) |
| `minikjpeg.exe` | Taşınabilir komut satırı aracı (kurulum gerektirmez) |

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

Masaüstü uygulamasında kalite eşiği kaydırıcısı PSNR eşiğini de birlikte ayarlar (0.87 → 35 dB, 0.92 → 40 dB, 0.96 → 44 dB). Çıktı klasörü ve dosya soneki boş bırakılırsa asıl dosyanın üzerine yazılır.

## Lisans

[MIT](LICENSE)
