import { useQueueStore } from "../../store";
import styles from "./ProgressBar.module.css";

export function ProgressBar() {
  const files = useQueueStore((s) => s.files);
  const activeJobId = useQueueStore((s) => s.activeJobId);

  if (!activeJobId) return null;

  const total = files.length;
  const done = files.filter(
    (f) => f.status === "done" || f.status === "error"
  ).length;
  const percent = total > 0 ? (done / total) * 100 : 0;

  const current = files.find((f) => f.status === "processing");

  return (
    <div className={styles.wrapper}>
      <div className={styles.header}>
        <span className={styles.label}>
          {done} / {total} dosya işlendi
        </span>
        <span className={styles.percent}>{percent.toFixed(0)}%</span>
      </div>
      <div className={styles.track}>
        <div className={styles.fill} style={{ width: `${percent}%` }} />
      </div>
      {current && (
        <p className={styles.current}>{current.name}</p>
      )}
    </div>
  );
}
