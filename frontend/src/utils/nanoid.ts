/** Basit benzersiz ID üretici (nanoid bağımlılığı olmadan) */
export function nanoid(): string {
  return Math.random().toString(36).slice(2, 11);
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}
