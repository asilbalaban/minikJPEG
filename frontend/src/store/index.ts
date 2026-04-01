import { create } from "zustand";
import {
  QueueFile,
  CompressionOptions,
  CompressionResult,
  defaultOptions,
  BatchComplete,
} from "../types";
import { nanoid } from "../utils/nanoid";

interface QueueStore {
  files: QueueFile[];
  activeJobId: string | null;
  batchSummary: BatchComplete | null;

  addFiles: (paths: string[]) => void;
  removeFile: (id: string) => void;
  clearAll: () => void;
  clearCompleted: () => void;
  setFileProcessing: (id: string) => void;
  setFileDone: (path: string, result: CompressionResult) => void;
  setFileError: (path: string, msg: string) => void;
  setActiveJob: (jobId: string | null) => void;
  setBatchSummary: (summary: BatchComplete | null) => void;
}

export const useQueueStore = create<QueueStore>((set) => ({
  files: [],
  activeJobId: null,
  batchSummary: null,

  addFiles: (paths) =>
    set((state) => {
      const existing = new Set(state.files.map((f) => f.path));
      const newFiles: QueueFile[] = paths
        .filter((p) => !existing.has(p))
        .map((p) => ({
          id: nanoid(),
          path: p,
          name: p.split(/[\\/]/).pop() ?? p,
          sizeBytes: 0,
          status: "pending",
        }));
      return { files: [...state.files, ...newFiles] };
    }),

  removeFile: (id) =>
    set((state) => ({ files: state.files.filter((f) => f.id !== id) })),

  clearAll: () => set({ files: [], batchSummary: null }),

  clearCompleted: () =>
    set((state) => ({
      files: state.files.filter((f) => f.status !== "done"),
    })),

  setFileProcessing: (id) =>
    set((state) => ({
      files: state.files.map((f) =>
        f.id === id ? { ...f, status: "processing" } : f
      ),
    })),

  setFileDone: (path, result) =>
    set((state) => ({
      files: state.files.map((f) =>
        f.path === path
          ? {
              ...f,
              status: "done",
              sizeBytes: result.originalSize,
              result,
            }
          : f
      ),
    })),

  setFileError: (path, errorMessage) =>
    set((state) => ({
      files: state.files.map((f) =>
        f.path === path ? { ...f, status: "error", errorMessage } : f
      ),
    })),

  setActiveJob: (jobId) => set({ activeJobId: jobId }),
  setBatchSummary: (summary) => set({ batchSummary: summary }),
}));

interface SettingsStore {
  options: CompressionOptions;
  outputDir: string | null;
  suffix: string;
  setOption: <K extends keyof CompressionOptions>(
    key: K,
    value: CompressionOptions[K]
  ) => void;
  setOutputDir: (dir: string | null) => void;
  setSuffix: (suffix: string) => void;
}

export const useSettingsStore = create<SettingsStore>((set) => ({
  options: defaultOptions,
  outputDir: null,
  suffix: "",

  setOption: (key, value) =>
    set((state) => ({ options: { ...state.options, [key]: value } })),

  setOutputDir: (outputDir) => set({ outputDir }),
  setSuffix: (suffix) => set({ suffix }),
}));
