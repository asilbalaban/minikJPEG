import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  CompressionOptions,
  CompressionResult,
  FileInfo,
  BatchProgress,
  BatchComplete,
} from "../types";

export type { BatchProgress, BatchComplete };

// Rust backend ile konuşan ince sarmalayıcılar.
// camelCase → serde ile snake_case'e dönüştürülür.

export function compressSingle(
  inputPath: string,
  outputPath: string | null,
  options: CompressionOptions
): Promise<CompressionResult> {
  return invoke("compress_single", {
    request: {
      inputPath,
      outputPath,
      options,
    },
  });
}

export function getFileInfo(path: string): Promise<FileInfo> {
  return invoke("get_file_info", { path });
}

export function startBatch(
  inputPaths: string[],
  outputDir: string | null,
  suffix: string,
  options: CompressionOptions
): Promise<string> {
  return invoke("start_batch", {
    request: {
      inputPaths,
      outputDir,
      suffix,
      options,
    },
  });
}

export function cancelBatch(jobId: string): Promise<void> {
  return invoke("cancel_batch", { jobId });
}

export function onProgress(
  cb: (event: BatchProgress) => void
): Promise<UnlistenFn> {
  return listen<BatchProgress>("compression-progress", (e) => cb(e.payload));
}

export function onBatchComplete(
  cb: (event: BatchComplete) => void
): Promise<UnlistenFn> {
  return listen<BatchComplete>("batch-completed", (e) => cb(e.payload));
}
