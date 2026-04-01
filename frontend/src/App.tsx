import { useState } from "react";
import { DropZone } from "./components/DropZone/DropZone";
import { FileQueue } from "./components/FileQueue/FileQueue";
import { ProgressBar } from "./components/ProgressBar/ProgressBar";
import { ResultPanel } from "./components/ResultPanel/ResultPanel";
import { Settings } from "./components/Settings/Settings";
import { useCompression } from "./hooks/useCompression";
import { useQueueStore } from "./store";
import styles from "./App.module.css";

export default function App() {
  const [showSettings, setShowSettings] = useState(false);
  const { files, activeJobId } = useQueueStore();
  const { startCompression, cancelCompression } = useCompression();

  const pendingCount = files.filter((f) => f.status === "pending").length;
  const isRunning = activeJobId !== null;

  return (
    <div className={styles.app}>
      {/* Header */}
      <header className={styles.header}>
        <img src="/logo.jpg" alt="minikJPEG logo" className={styles.logoImg} />
        <button
          className={styles.settingsBtn}
          onClick={() => setShowSettings((v) => !v)}
          title="Ayarlar"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" width="20" height="20">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
          </svg>
        </button>
      </header>

      <main className={styles.main}>
        {/* Sol panel */}
        <div className={styles.left}>
          <DropZone />
          <ProgressBar />
          <ResultPanel />
          <FileQueue />
        </div>

        {/* Sağ panel — Ayarlar */}
        {showSettings && (
          <div className={styles.right}>
            <Settings />
          </div>
        )}
      </main>

      {/* Alt action bar */}
      <footer className={styles.footer}>
        {isRunning ? (
          <button className={styles.cancelBtn} onClick={cancelCompression}>
            İptal Et
          </button>
        ) : (
          <button
            className={styles.startBtn}
            onClick={startCompression}
            disabled={pendingCount === 0}
          >
            {pendingCount > 0
              ? `${pendingCount} Dosyayı Optimize Et`
              : "Dosya Ekleyin"}
          </button>
        )}
        <span className={styles.footerInfo}>
          SSIM tabanlı algısal kayıpsız sıkıştırma
        </span>
      </footer>
    </div>
  );
}
