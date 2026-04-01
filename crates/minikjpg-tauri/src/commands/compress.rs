use minikjpg_core::{compress_jpeg, make_output_path, CompressionOptions, CompressionResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use crate::state::AppState;

/// Frontend'den gelen sıkıştırma isteği.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressRequest {
    pub input_path: String,
    pub output_path: Option<String>,
    pub suffix: Option<String>,
    pub options: CompressionOptionsDto,
}

/// Frontend ile paylaşılan seçenekler (camelCase JSON).
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompressionOptionsDto {
    pub ssim_threshold: f32,
    pub psnr_threshold: f32,
    pub max_quality: u8,
    pub min_quality: u8,
    pub preserve_metadata: bool,
    pub adaptive_quality: bool,
}

impl From<CompressionOptionsDto> for CompressionOptions {
    fn from(dto: CompressionOptionsDto) -> Self {
        Self {
            ssim_threshold: dto.ssim_threshold,
            psnr_threshold: dto.psnr_threshold,
            max_quality: dto.max_quality,
            min_quality: dto.min_quality,
            preserve_metadata: dto.preserve_metadata,
            adaptive_quality: dto.adaptive_quality,
        }
    }
}

/// Tekli JPEG dosyası sıkıştırma Tauri komutu.
#[tauri::command]
pub async fn compress_single(
    _state: State<'_, AppState>,
    request: CompressRequest,
) -> Result<CompressionResult, String> {
    let input = PathBuf::from(&request.input_path);
    let suffix = request.suffix.as_deref().unwrap_or("");

    let output = match request.output_path {
        Some(ref p) => PathBuf::from(p),
        None => make_output_path(&input, None, suffix),
    };

    let options: CompressionOptions = request.options.into();

    compress_jpeg(&input, &output, &options)
        .await
        .map_err(|e| e.to_string())
}

/// Dosya metadata bilgisi sorgulama.
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfoDto, String> {
    let p = PathBuf::from(&path);

    let size = tokio::fs::metadata(&p)
        .await
        .map_err(|e| e.to_string())?
        .len();

    let img: (u32, u32) = tokio::task::spawn_blocking(move || {
        image::open(&p)
            .map(|img| (img.width(), img.height()))
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e: tokio::task::JoinError| e.to_string())??;

    Ok(FileInfoDto {
        path,
        size_bytes: size,
        width: img.0,
        height: img.1,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfoDto {
    pub path: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
}
