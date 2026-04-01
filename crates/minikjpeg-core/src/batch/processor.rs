use crate::compression::pipeline::{compress_jpeg, CompressionOptions, CompressionResult};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::runtime::Handle;

/// Batch işlem girdisi.
#[derive(Debug, Clone)]
pub struct BatchInput {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

/// Batch işlem sonucu.
#[derive(Debug)]
pub struct BatchResult {
    pub results: Vec<Result<CompressionResult, (PathBuf, String)>>,
    pub total_original_bytes: u64,
    pub total_compressed_bytes: u64,
    pub succeeded: usize,
    pub failed: usize,
    pub elapsed_ms: u64,
}

impl BatchResult {
    pub fn total_savings_percent(&self) -> f32 {
        if self.total_original_bytes == 0 {
            return 0.0;
        }
        let saved = self.total_original_bytes.saturating_sub(self.total_compressed_bytes);
        (saved as f32 / self.total_original_bytes as f32) * 100.0
    }
}

/// İlerleme bildirimi callback tipi.
pub type ProgressCallback = Arc<dyn Fn(usize, usize, &CompressionResult) + Send + Sync>;

/// Birden fazla JPEG dosyasını paralel olarak sıkıştırır.
///
/// rayon thread pool kullanır. Thread sayısı = cpu_count - 1 (GUI için bir çekirdek serbest).
pub fn process_batch(
    inputs: Vec<BatchInput>,
    options: CompressionOptions,
    progress_cb: Option<ProgressCallback>,
    cancel_flag: Arc<AtomicBool>,
    rt_handle: Handle,
) -> BatchResult {
    let start = std::time::Instant::now();
    let total = inputs.len();
    let completed = Arc::new(AtomicUsize::new(0));
    let options = Arc::new(options);

    // CPU çekirdeği sayısına göre thread pool
    let num_threads = (num_cpus() - 1).max(1);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

    let results: Vec<Result<CompressionResult, (PathBuf, String)>> = pool.install(|| {
        inputs
            .into_par_iter()
            .map(|input| {
                // İptal kontrolü
                if cancel_flag.load(Ordering::Relaxed) {
                    return Err((input.input_path.clone(), "İptal edildi".to_string()));
                }

                let opts = Arc::clone(&options);
                let result = rt_handle.block_on(compress_jpeg(
                    &input.input_path,
                    &input.output_path,
                    &opts,
                ));

                let done = completed.fetch_add(1, Ordering::Relaxed) + 1;

                match result {
                    Ok(res) => {
                        if let Some(ref cb) = progress_cb {
                            cb(done, total, &res);
                        }
                        Ok(res)
                    }
                    Err(e) => Err((input.input_path, e.to_string())),
                }
            })
            .collect()
    });

    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut total_original = 0u64;
    let mut total_compressed = 0u64;

    for r in &results {
        match r {
            Ok(res) => {
                succeeded += 1;
                total_original += res.original_size;
                total_compressed += res.compressed_size;
            }
            Err(_) => failed += 1,
        }
    }

    BatchResult {
        results,
        total_original_bytes: total_original,
        total_compressed_bytes: total_compressed,
        succeeded,
        failed,
        elapsed_ms: start.elapsed().as_millis() as u64,
    }
}

/// Bir dizindeki tüm JPEG dosyalarını bul.
pub fn find_jpeg_files(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    find_jpeg_recursive(dir, recursive, &mut files);
    files
}

fn find_jpeg_recursive(dir: &Path, recursive: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && recursive {
            find_jpeg_recursive(&path, recursive, out);
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if ext_lower == "jpg" || ext_lower == "jpeg" {
                    out.push(path);
                }
            }
        }
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
