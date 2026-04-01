// Backend ile paylaşılan tipler (Rust CompressionOptions/Result ile eşleşmeli)

export interface CompressionOptions {
  ssimThreshold: number;
  psnrThreshold: number;
  maxQuality: number;
  minQuality: number;
  preserveMetadata: boolean;
  adaptiveQuality: boolean;
}

export const defaultOptions: CompressionOptions = {
  ssimThreshold: 0.92,
  psnrThreshold: 40.0,
  maxQuality: 95,
  minQuality: 20,
  preserveMetadata: true,
  adaptiveQuality: true,
};

export interface CompressionResult {
  inputPath: string;
  outputPath: string;
  originalSize: number;
  compressedSize: number;
  savingsRatio: number;
  achievedSsim: number;
  achievedPsnr: number;
  qualityUsed: number;
  elapsedMs: number;
}

export interface FileInfo {
  path: string;
  sizeBytes: number;
  width: number;
  height: number;
}

export type FileStatus =
  | "pending"
  | "processing"
  | "done"
  | "error"
  | "cancelled";

export interface QueueFile {
  id: string;
  path: string;
  name: string;
  sizeBytes: number;
  status: FileStatus;
  result?: CompressionResult;
  errorMessage?: string;
}

export interface ProgressResult {
  inputPath?: string;
  outputPath?: string;
  originalSize?: number;
  compressedSize?: number;
  savingsPercent?: number;
  achievedSsim?: number;
  qualityUsed?: number;
}

export interface BatchProgress {
  jobId: string;
  filePath: string;
  current: number;
  total: number;
  percent: number;
  result?: ProgressResult;
}

export interface BatchComplete {
  jobId: string;
  succeeded: number;
  failed: number;
  totalOriginalBytes: number;
  totalCompressedBytes: number;
  totalSavingsPercent: number;
  elapsedMs: number;
}
