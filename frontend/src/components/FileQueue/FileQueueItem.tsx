import { QueueFile } from "../../types";
import { formatBytes } from "../../utils/nanoid";
import { useQueueStore } from "../../store";
import styles from "./FileQueue.module.css";

interface Props {
  file: QueueFile;
}

export function FileQueueItem({ file }: Props) {
  const removeFile = useQueueStore((s) => s.removeFile);

  const statusIcon = {
    pending: <span className={styles.badge} data-status="pending">Bekliyor</span>,
    processing: (
      <span className={styles.badge} data-status="processing">
        <span className={styles.spinner} /> İşleniyor
      </span>
    ),
    done: <span className={styles.badge} data-status="done">Tamamlandı</span>,
    error: <span className={styles.badge} data-status="error">Hata</span>,
    cancelled: <span className={styles.badge} data-status="cancelled">İptal</span>,
  }[file.status];

  return (
    <div className={styles.item}>
      <div className={styles.itemInfo}>
        <span className={styles.fileName} title={file.path}>
          {file.name}
        </span>
        {file.sizeBytes > 0 && (
          <span className={styles.fileSize}>{formatBytes(file.sizeBytes)}</span>
        )}
      </div>

      <div className={styles.itemRight}>
        {file.result && (
          <div className={styles.resultBadges}>
            <span className={styles.saving}>
              -{(file.result.savingsRatio * 100).toFixed(1)}%
            </span>
            <span className={styles.ssim}>
              SSIM {file.result.achievedSsim.toFixed(3)}
            </span>
            <span className={styles.quality}>
              Q{file.result.qualityUsed}
            </span>
          </div>
        )}
        {file.errorMessage && (
          <span className={styles.errorMsg} title={file.errorMessage}>
            {file.errorMessage.slice(0, 40)}
          </span>
        )}
        {statusIcon}
        {file.status === "pending" && (
          <button
            className={styles.removeBtn}
            onClick={() => removeFile(file.id)}
            title="Kaldır"
          >
            ×
          </button>
        )}
      </div>
    </div>
  );
}
