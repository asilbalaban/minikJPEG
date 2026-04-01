import { useEffect, useRef } from "react";
import {
  startBatch,
  cancelBatch,
  onProgress,
  onBatchComplete,
} from "../ipc/commands";
import { useQueueStore, useSettingsStore } from "../store";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { BatchProgress } from "../types";

export function useCompression() {
  const {
    files,
    activeJobId,
    setFileDone,
    setActiveJob,
    setBatchSummary,
  } = useQueueStore();
  const { options, outputDir, suffix } = useSettingsStore();

  const unlistenRef = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    // Komponent unmount edildiğinde event listener'ları temizle
    return () => {
      unlistenRef.current.forEach((fn) => fn());
    };
  }, []);

  async function startCompression() {
    const pendingFiles = files.filter((f) => f.status === "pending");
    if (pendingFiles.length === 0) return;

    const paths = pendingFiles.map((f) => f.path);

    // Önceki listener'ları temizle
    unlistenRef.current.forEach((fn) => fn());
    unlistenRef.current = [];

    // Progress ve tamamlanma event'lerini dinle
    const unlistenProgress = await onProgress((event: BatchProgress) => {
      if (event.result) {
        setFileDone(event.filePath, {
          inputPath: event.filePath,
          outputPath: event.result.outputPath ?? "",
          originalSize: event.result.originalSize ?? 0,
          compressedSize: event.result.compressedSize ?? 0,
          savingsRatio: (event.result.savingsPercent ?? 0) / 100,
          achievedSsim: event.result.achievedSsim ?? 0,
          achievedPsnr: 0,
          qualityUsed: event.result.qualityUsed ?? 0,
          elapsedMs: 0,
        });
      }
    });

    const unlistenComplete = await onBatchComplete((summary) => {
      setActiveJob(null);
      setBatchSummary(summary);
    });

    unlistenRef.current = [unlistenProgress, unlistenComplete];

    const jobId = await startBatch(paths, outputDir, suffix, options);
    setActiveJob(jobId);
  }

  async function cancelCompression() {
    if (activeJobId) {
      await cancelBatch(activeJobId);
      setActiveJob(null);
    }
  }

  return { startCompression, cancelCompression };
}
