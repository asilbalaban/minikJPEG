import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { useQueueStore } from "../../store";
import styles from "./DropZone.module.css";

export function DropZone() {
  const addFiles = useQueueStore((s) => s.addFiles);
  const [isDragActive, setIsDragActive] = useState(false);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    // Tauri'nin native drag-drop event sistemini kullan
    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "over") {
          setIsDragActive(true);
        } else if (event.payload.type === "drop") {
          setIsDragActive(false);
          const paths: string[] = event.payload.paths.filter((p: string) =>
            /\.(jpg|jpeg|png)$/i.test(p)
          );
          if (paths.length > 0) {
            addFiles(paths);
          }
        } else if (event.payload.type === "leave") {
          setIsDragActive(false);
        }
      })
      .then((unlisten) => {
        unlistenRef.current = unlisten;
      });

    return () => {
      unlistenRef.current?.();
    };
  }, [addFiles]);

  // Tıklayarak dosya seçimi için Tauri dialog kullan
  const handleClick = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "Görüntü (JPEG / PNG)", extensions: ["jpg", "jpeg", "png"] }],
      });
      console.log("dialog selected:", selected);
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        console.log("addFiles paths:", paths);
        addFiles(paths);
      }
    } catch (err) {
      console.error("dialog error:", err);
    }
  }, [addFiles]);

  return (
    <div
      onClick={handleClick}
      className={`${styles.zone} ${isDragActive ? styles.active : ""}`}
    >
      <div className={styles.content}>
        <svg
          className={styles.icon}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5"
          />
        </svg>
        {isDragActive ? (
          <p className={styles.text}>Dosyaları bırakın...</p>
        ) : (
          <>
            <p className={styles.text}>JPEG veya PNG dosyalarını buraya sürükleyin</p>
            <p className={styles.sub}>veya seçmek için tıklayın</p>
          </>
        )}
      </div>
    </div>
  );
}
