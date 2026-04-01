import { useQueueStore } from "../../store";
import { formatBytes } from "../../utils/nanoid";
import styles from "./ResultPanel.module.css";

export function ResultPanel() {
  const { batchSummary, files } = useQueueStore();

  // Tamamlanan dosya yoksa gösterme
  const doneFiles = files.filter((f) => f.status === "done" && f.result);
  if (doneFiles.length === 0 && !batchSummary) return null;

  return (
    <div className={styles.panel}>
      {batchSummary && (
        <div className={styles.summary}>
          <div className={styles.summaryItem}>
            <span className={styles.summaryLabel}>Toplam tasarruf</span>
            <span className={styles.savingsBig}>
              {batchSummary.totalSavingsPercent.toFixed(1)}%
            </span>
          </div>
          <div className={styles.summaryItem}>
            <span className={styles.summaryLabel}>Orijinal</span>
            <span>{formatBytes(batchSummary.totalOriginalBytes)}</span>
          </div>
          <div className={styles.summaryItem}>
            <span className={styles.summaryLabel}>Sıkıştırılmış</span>
            <span className={styles.compressed}>
              {formatBytes(batchSummary.totalCompressedBytes)}
            </span>
          </div>
          <div className={styles.summaryItem}>
            <span className={styles.summaryLabel}>Başarılı</span>
            <span className={styles.success}>{batchSummary.succeeded}</span>
          </div>
          {batchSummary.failed > 0 && (
            <div className={styles.summaryItem}>
              <span className={styles.summaryLabel}>Başarısız</span>
              <span className={styles.failed}>{batchSummary.failed}</span>
            </div>
          )}
          <div className={styles.summaryItem}>
            <span className={styles.summaryLabel}>Süre</span>
            <span>{(batchSummary.elapsedMs / 1000).toFixed(1)}s</span>
          </div>
        </div>
      )}
    </div>
  );
}
