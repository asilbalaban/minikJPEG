import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../../store";
import styles from "./Settings.module.css";

const SSIM_PRESETS = [
  { label: "Agresif (web)", value: 0.87, desc: "En küçük dosya, web için yeterli" },
  { label: "Dengeli", value: 0.92, desc: "Varsayılan — ideal denge" },
  { label: "Muhafazakar", value: 0.96, desc: "Baskı kalitesi, minimal kayıp" },
];

export function Settings() {
  const { options, outputDir, suffix, setOption, setOutputDir, setSuffix } =
    useSettingsStore();

  async function pickOutputDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      setOutputDir(selected);
    }
  }

  return (
    <div className={styles.settings}>
      <h2 className={styles.title}>Ayarlar</h2>

      {/* SSIM Eşiği */}
      <div className={styles.group}>
        <label className={styles.label}>
          Kalite Eşiği (SSIM)
          <span className={styles.value}>{options.ssimThreshold.toFixed(2)}</span>
        </label>
        <input
          type="range"
          min={0.80}
          max={0.99}
          step={0.01}
          value={options.ssimThreshold}
          onChange={(e) =>
            setOption("ssimThreshold", parseFloat(e.target.value))
          }
          className={styles.slider}
        />
        <div className={styles.presets}>
          {SSIM_PRESETS.map((p) => (
            <button
              key={p.value}
              className={`${styles.preset} ${
                Math.abs(options.ssimThreshold - p.value) < 0.005
                  ? styles.presetActive
                  : ""
              }`}
              onClick={() => setOption("ssimThreshold", p.value)}
              title={p.desc}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      {/* Metadata */}
      <div className={styles.group}>
        <label className={styles.checkLabel}>
          <input
            type="checkbox"
            checked={options.preserveMetadata}
            onChange={(e) => setOption("preserveMetadata", e.target.checked)}
          />
          EXIF/IPTC metadata koru
        </label>
        <label className={styles.checkLabel}>
          <input
            type="checkbox"
            checked={options.adaptiveQuality}
            onChange={(e) => setOption("adaptiveQuality", e.target.checked)}
          />
          İçerik-adaptif kalite aralığı
        </label>
      </div>

      {/* Çıktı Klasörü */}
      <div className={styles.group}>
        <label className={styles.label}>Çıktı Klasörü</label>
        <div className={styles.row}>
          <span className={styles.dirPath}>
            {outputDir ?? "Orijinal dosyayla aynı konum"}
          </span>
          <button className={styles.browseBtn} onClick={pickOutputDir}>
            Seç
          </button>
          {outputDir && (
            <button
              className={styles.clearBtn}
              onClick={() => setOutputDir(null)}
            >
              ×
            </button>
          )}
        </div>
      </div>

      {/* Dosya Soneki */}
      <div className={styles.group}>
        <label className={styles.label}>Dosya Soneki</label>
        <input
          type="text"
          value={suffix}
          onChange={(e) => setSuffix(e.target.value)}
          placeholder="örn. _opt (boş bırakılırsa üzerine yazar)"
          className={styles.input}
        />
      </div>
    </div>
  );
}
