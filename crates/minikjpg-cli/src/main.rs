use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use minikjpg_core::{
    batch::processor::{find_jpeg_files, process_batch, BatchInput, ProgressCallback},
    compress_jpeg, make_output_path, CompressionOptions,
};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(
    name = "minikjpg",
    about = "Algısal kayıpsız JPEG optimizasyon aracı",
    version = env!("CARGO_PKG_VERSION"),
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Tek bir JPEG dosyasını optimize et
    Compress {
        /// Girdi JPEG dosyası
        #[arg(value_name = "DOSYA")]
        input: PathBuf,

        /// Çıktı dosyası (belirtilmezse girdiyle aynı konuma kaydedilir)
        #[arg(short, long, value_name = "ÇIKTI")]
        output: Option<PathBuf>,

        /// Çıktı dosya soneki (örn. "_opt" → dosya_opt.jpg)
        #[arg(long, default_value = "")]
        suffix: String,

        /// SSIM eşiği [0.80-0.99]
        #[arg(long, default_value = "0.92")]
        ssim: f32,

        /// PSNR eşiği (dB)
        #[arg(long, default_value = "40.0")]
        psnr: f32,

        /// Metadata'yı koru
        #[arg(long, default_value = "true")]
        preserve_metadata: bool,

        /// Ayrıntılı çıktı
        #[arg(short, long)]
        verbose: bool,
    },

    /// Klasördeki tüm JPEG dosyalarını toplu optimize et
    Batch {
        /// Girdi klasörü
        #[arg(value_name = "KLASÖR")]
        input_dir: PathBuf,

        /// Çıktı klasörü (belirtilmezse girdi klasöründe güncellenir)
        #[arg(short, long, value_name = "ÇIKTI_KLASÖR")]
        output_dir: Option<PathBuf>,

        /// Alt klasörleri de işle
        #[arg(short, long)]
        recursive: bool,

        /// Çıktı dosya soneki
        #[arg(long, default_value = "")]
        suffix: String,

        /// SSIM eşiği
        #[arg(long, default_value = "0.92")]
        ssim: f32,

        /// PSNR eşiği
        #[arg(long, default_value = "40.0")]
        psnr: f32,

        /// Metadata'yı koru
        #[arg(long, default_value = "true")]
        preserve_metadata: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let rt = Runtime::new().context("Tokio runtime oluşturulamadı")?;

    match cli.command {
        Commands::Compress {
            input,
            output,
            suffix,
            ssim,
            psnr,
            preserve_metadata,
            verbose,
        } => {
            if verbose {
                tracing_subscriber::fmt()
                    .with_env_filter("minikjpg=debug")
                    .init();
            }

            validate_ssim(ssim)?;

            let output_path = match output {
                Some(p) => p,
                None => make_output_path(&input, None, &suffix),
            };

            let options = CompressionOptions {
                ssim_threshold: ssim,
                psnr_threshold: psnr,
                preserve_metadata,
                ..Default::default()
            };

            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );
            pb.set_message(format!("Sıkıştırılıyor: {}", input.display()));
            pb.enable_steady_tick(std::time::Duration::from_millis(80));

            let result = rt.block_on(compress_jpeg(&input, &output_path, &options))
                .with_context(|| format!("Sıkıştırma başarısız: {}", input.display()))?;

            pb.finish_and_clear();

            print_single_result(&result);
        }

        Commands::Batch {
            input_dir,
            output_dir,
            recursive,
            suffix,
            ssim,
            psnr,
            preserve_metadata,
        } => {
            validate_ssim(ssim)?;

            let files = find_jpeg_files(&input_dir, recursive);
            if files.is_empty() {
                println!("{}", "Belirtilen klasörde JPEG dosyası bulunamadı.".yellow());
                return Ok(());
            }

            println!(
                "{} {} JPEG dosyası bulundu",
                "→".cyan(),
                files.len().to_string().bold()
            );

            if let Some(ref out_dir) = output_dir {
                std::fs::create_dir_all(out_dir)
                    .context("Çıktı klasörü oluşturulamadı")?;
            }

            let inputs: Vec<BatchInput> = files
                .into_iter()
                .map(|p| {
                    let out = make_output_path(&p, output_dir.as_deref(), &suffix);
                    BatchInput {
                        input_path: p,
                        output_path: out,
                    }
                })
                .collect();

            let total = inputs.len();
            let multi_pb = Arc::new(MultiProgress::new());
            let global_pb = multi_pb.add(ProgressBar::new(total as u64));
            global_pb.set_style(
                ProgressStyle::with_template(
                    "{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}] {msg}",
                )
                .unwrap()
                .progress_chars("█▓░"),
            );

            let pb_clone = global_pb.clone();
            let progress_cb: ProgressCallback = Arc::new(move |done, _total, res| {
                pb_clone.inc(1);
                pb_clone.set_message(format!(
                    "Son: {} → {:.1}% küçüldü",
                    PathBuf::from(&res.input_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    res.savings_percent()
                ));
            });

            let options = CompressionOptions {
                ssim_threshold: ssim,
                psnr_threshold: psnr,
                preserve_metadata,
                ..Default::default()
            };

            let cancel = Arc::new(AtomicBool::new(false));
            let rt_handle = rt.handle().clone();

            let batch_result = process_batch(
                inputs,
                options,
                Some(progress_cb),
                cancel,
                rt_handle,
            );

            global_pb.finish_and_clear();

            print_batch_result(&batch_result);
        }
    }

    Ok(())
}

fn print_single_result(result: &minikjpg_core::CompressionResult) {
    println!();
    println!("{}", "✓ Tamamlandı".green().bold());
    println!(
        "  Orijinal boyut  : {}",
        format_bytes(result.original_size).yellow()
    );
    println!(
        "  Sıkıştırılmış   : {}",
        format_bytes(result.compressed_size).green()
    );
    println!(
        "  Tasarruf        : {}",
        format!("{:.1}%", result.savings_percent()).cyan().bold()
    );
    println!("  SSIM            : {:.4}", result.achieved_ssim);
    println!("  PSNR            : {:.1} dB", result.achieved_psnr);
    println!("  Kullanılan kalite: {}", result.quality_used);
    println!("  Süre            : {} ms", result.elapsed_ms);
    println!("  Çıktı           : {}", result.output_path.dimmed());
}

fn print_batch_result(result: &minikjpg_core::batch::processor::BatchResult) {
    println!();
    println!("{}", "─".repeat(50));
    println!("{}", "  Batch İşlem Özeti".bold());
    println!("{}", "─".repeat(50));
    println!(
        "  Başarılı : {}",
        result.succeeded.to_string().green().bold()
    );
    if result.failed > 0 {
        println!("  Başarısız: {}", result.failed.to_string().red().bold());
    }
    println!(
        "  Toplam orijinal  : {}",
        format_bytes(result.total_original_bytes).yellow()
    );
    println!(
        "  Toplam sıkıştırılmış: {}",
        format_bytes(result.total_compressed_bytes).green()
    );
    println!(
        "  Toplam tasarruf  : {}",
        format!("{:.1}%", result.total_savings_percent()).cyan().bold()
    );
    println!("  Toplam süre      : {} ms", result.elapsed_ms);

    // Başarısız dosyalar
    for r in &result.results {
        if let Err((path, err)) = r {
            println!(
                "  {} {}: {}",
                "✗".red(),
                path.display(),
                err.dimmed()
            );
        }
    }
}

fn validate_ssim(ssim: f32) -> Result<()> {
    if !(0.5..=0.999).contains(&ssim) {
        anyhow::bail!("SSIM eşiği 0.50 ile 0.999 arasında olmalıdır, verilen: {}", ssim);
    }
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
