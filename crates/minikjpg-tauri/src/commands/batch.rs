use crate::commands::compress::CompressionOptionsDto;
use crate::state::{AppState, JobState};
use minikjpg_core::{
    batch::processor::{process_batch, BatchInput, ProgressCallback},
    make_output_path, CompressionOptions, CompressionResult,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    pub input_paths: Vec<String>,
    pub output_dir: Option<String>,
    pub suffix: Option<String>,
    pub options: CompressionOptionsDto,
}

/// Batch işlem başlatma komutu. job_id döndürür.
#[tauri::command]
pub async fn start_batch(
    state: State<'_, AppState>,
    window: Window,
    request: BatchRequest,
) -> Result<String, String> {
    let job_id = Uuid::new_v4().to_string();
    let cancel_flag = Arc::new(AtomicBool::new(false));

    let output_dir: Option<PathBuf> = request.output_dir.map(PathBuf::from);
    let suffix = request.suffix.unwrap_or_default();
    let options: CompressionOptions = request.options.into();
    let total = request.input_paths.len();

    // Çıktı klasörünü oluştur
    if let Some(ref dir) = output_dir {
        tokio::fs::create_dir_all(dir)
            .await
            .map_err(|e| format!("Çıktı klasörü oluşturulamadı: {}", e))?;
    }

    let inputs: Vec<BatchInput> = request
        .input_paths
        .into_iter()
        .map(|p| {
            let input_path = PathBuf::from(&p);
            let output_path = make_output_path(&input_path, output_dir.as_deref(), &suffix);
            BatchInput {
                input_path,
                output_path,
            }
        })
        .collect();

    // Job state'i kaydet
    {
        let mut jobs = state.jobs.lock().map_err(|e| e.to_string())?;
        jobs.insert(
            job_id.clone(),
            JobState {
                cancel_flag: Arc::clone(&cancel_flag),
                total_files: total,
                completed_files: 0,
            },
        );
    }

    // Tauri event gönderici
    let window_clone = window.clone();
    let job_id_clone = job_id.clone();

    let progress_cb: ProgressCallback = Arc::new(move |done, total, res| {
        let _ = window_clone.emit(
            "compression-progress",
            ProgressEvent {
                job_id: job_id_clone.clone(),
                file_path: res.input_path.clone(),
                current: done,
                total,
                percent: (done as f32 / total as f32) * 100.0,
                result: Some(CompressResultDto::from(res)),
            },
        );
    });

    let rt_handle = tokio::runtime::Handle::current();
    let job_id_final = job_id.clone();
    let window_final = window.clone();

    // Batch işlemi arka planda çalıştır
    tokio::task::spawn_blocking(move || {
        let batch_result = process_batch(
            inputs,
            options,
            Some(progress_cb),
            cancel_flag,
            rt_handle,
        );

        let _ = window_final.emit(
            "batch-completed",
            BatchCompleteEvent {
                job_id: job_id_final,
                succeeded: batch_result.succeeded,
                failed: batch_result.failed,
                total_original_bytes: batch_result.total_original_bytes,
                total_compressed_bytes: batch_result.total_compressed_bytes,
                total_savings_percent: batch_result.total_savings_percent(),
                elapsed_ms: batch_result.elapsed_ms,
            },
        );
    });

    Ok(job_id)
}

/// İşi iptal et.
#[tauri::command]
pub fn cancel_batch(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    let jobs = state.jobs.lock().map_err(|e| e.to_string())?;
    if let Some(job) = jobs.get(&job_id) {
        job.cancel_flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

// --- Event tipleri ---

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    job_id: String,
    file_path: String,
    current: usize,
    total: usize,
    percent: f32,
    result: Option<CompressResultDto>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CompressResultDto {
    input_path: String,
    output_path: String,
    original_size: u64,
    compressed_size: u64,
    savings_percent: f32,
    achieved_ssim: f32,
    quality_used: u8,
}

impl From<&CompressionResult> for CompressResultDto {
    fn from(r: &CompressionResult) -> Self {
        Self {
            input_path: r.input_path.clone(),
            output_path: r.output_path.clone(),
            original_size: r.original_size,
            compressed_size: r.compressed_size,
            savings_percent: r.savings_percent(),
            achieved_ssim: r.achieved_ssim,
            quality_used: r.quality_used,
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BatchCompleteEvent {
    job_id: String,
    succeeded: usize,
    failed: usize,
    total_original_bytes: u64,
    total_compressed_bytes: u64,
    total_savings_percent: f32,
    elapsed_ms: u64,
}
