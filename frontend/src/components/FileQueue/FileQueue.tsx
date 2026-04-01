import { useQueueStore } from "../../store";
import { FileQueueItem } from "./FileQueueItem";
import styles from "./FileQueue.module.css";

export function FileQueue() {
  const { files, clearAll, clearCompleted } = useQueueStore();

  if (files.length === 0) return null;

  const doneCount = files.filter((f) => f.status === "done").length;

  return (
    <div className={styles.queue}>
      <div className={styles.header}>
        <span className={styles.title}>
          Kuyruk ({files.length} dosya)
        </span>
        <div className={styles.actions}>
          {doneCount > 0 && (
            <button className={styles.btn} onClick={clearCompleted}>
              Tamamlananları temizle
            </button>
          )}
          <button className={`${styles.btn} ${styles.btnDanger}`} onClick={clearAll}>
            Tümünü temizle
          </button>
        </div>
      </div>
      <div className={styles.list}>
        {files.map((file) => (
          <FileQueueItem key={file.id} file={file} />
        ))}
      </div>
    </div>
  );
}
