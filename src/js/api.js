import { invoke, isTauri, listen } from "./tauri-bridge.js";

function chooseFile(accept) {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = accept;
    input.hidden = true;

    const finish = (file) => {
      input.remove();
      resolve(file);
    };

    input.addEventListener("change", () => finish(input.files?.[0] ?? null), {
      once: true,
    });
    input.addEventListener("cancel", () => finish(null), { once: true });
    document.body.append(input);
    input.click();
  });
}

function throwIfAborted(signal) {
  if (signal?.aborted) {
    throw new DOMException("Proses dibatalkan.", "AbortError");
  }
}

function pause(durationMs, signal) {
  return new Promise((resolve, reject) => {
    throwIfAborted(signal);
    const timer = window.setTimeout(() => {
      signal?.removeEventListener("abort", cancel);
      resolve();
    }, durationMs);
    const cancel = () => {
      window.clearTimeout(timer);
      reject(new DOMException("Proses dibatalkan.", "AbortError"));
    };
    signal?.addEventListener("abort", cancel, { once: true });
  });
}

// ---------- listModels ----------
export async function listModels() {
  if (isTauri()) {
    return invoke("list_models");
  }
  // MOCK (browser)
  return [
    { id: "tiny", name: "Tiny", filename: "ggml-tiny.bin", size_bytes: null, estimated_size_bytes: 75 * 1024 * 1024, installed: false },
    { id: "base", name: "Base", filename: "ggml-base.bin", size_bytes: null, estimated_size_bytes: 142 * 1024 * 1024, installed: false },
    { id: "small", name: "Small", filename: "ggml-small.bin", size_bytes: 466 * 1024 * 1024, estimated_size_bytes: 466 * 1024 * 1024, installed: true },
    { id: "medium", name: "Medium", filename: "ggml-medium.bin", size_bytes: null, estimated_size_bytes: 1530 * 1024 * 1024, installed: false },
  ];
}

export async function modelsDirectory() {
  if (isTauri()) {
    return invoke("models_directory");
  }
  return null;
}

// ---------- importModel ----------
export async function importModel() {
  if (isTauri() && window.__TAURI__ && window.__TAURI__.dialog) {
    const selected = await window.__TAURI__.dialog.open({
      multiple: false,
      filters: [{ name: "Whisper Model", extensions: ["bin"] }],
    });
    if (!selected) return null;
    return invoke("import_model", { path: selected });
  }
  // MOCK (browser)
  return chooseFile(".bin");
}

// ---------- pickInputFile ----------
export async function pickInputFile() {
  if (isTauri() && window.__TAURI__ && window.__TAURI__.dialog) {
    const selected = await window.__TAURI__.dialog.open({
      multiple: false,
      filters: [{ name: "Media", extensions: ["mp4", "mkv", "mov", "mp3", "wav", "m4a"] }],
    });
    if (!selected) return null;
    return invoke("pick_input_file", { path: selected });
  }
  // MOCK (browser)
  return chooseFile(".mp4,.mkv,.mov,.mp3,.wav,.m4a");
}

// ---------- transcribe ----------
export async function transcribe(input, modelId, language = "id", onProgress = () => {}, options = {}) {
  const { signal } = options;

  if (isTauri()) {
    // Dengarkan progress dari backend
    const unlisten = await listen("transcribe_progress", (event) => {
      onProgress(event.payload);
    });
    try {
      onProgress({ stage: "Mengekstrak audio", progress: null, status: "starting" });
      const words = await invoke("transcribe", {
        request: {
          input: input ?? null,
          model: modelId ?? null,
          language: language ?? "id",
        },
      });
      return words;
    } finally {
      unlisten();
    }
  }

  // MOCK (browser)
  const phases = [
    { stage: "Mengekstrak audio", percentages: [0, 40, 100] },
    { stage: "Transkripsi", percentages: [0, 20, 45, 70, 100] },
  ];
  for (const phase of phases) {
    for (const progress of phase.percentages) {
      throwIfAborted(signal);
      onProgress({ stage: phase.stage, progress });
      await pause(120, signal);
    }
  }
  const response = await fetch(new URL("../mock/words.json", import.meta.url));
  if (!response.ok) {
    throw new Error(`Data kata mock gagal dimuat (HTTP ${response.status}).`);
  }
  return response.json();
}

export async function cancelTranscribe() {
  if (!isTauri()) {
    throw new Error("Pembatalan transkripsi hanya tersedia di aplikasi desktop.");
  }
  return invoke("cancel_transcribe");
}

// ---------- exportSubtitleFile ----------
export async function exportSubtitleFile(options = {}) {
  if (!isTauri()) {
    throw new Error("Ekspor subtitle hanya tersedia di aplikasi desktop.");
  }
  if (typeof window.__TAURI__.dialog?.save !== "function") {
    throw new Error("Dialog simpan tidak tersedia.");
  }

  const formats = {
    ass: { name: "SubStation Alpha", extension: "ass" },
    srt: { name: "SubRip", extension: "srt" },
    vtt: { name: "WebVTT", extension: "vtt" },
    txt: { name: "Teks", extension: "txt" },
    json: { name: "JSON", extension: "json" },
  };
  const format = options.format ?? "ass";
  const formatInfo = formats[format];
  if (!formatInfo) throw new Error(`Format ekspor tidak didukung: ${format}`);

  const path = await window.__TAURI__.dialog.save({
    defaultPath: options.defaultPath ?? `caption.${formatInfo.extension}`,
    filters: [{ name: formatInfo.name, extensions: [formatInfo.extension] }],
  });
  if (!path) return null;

  return invoke("write_export_file", {
    path,
    format,
    req: {
      words: options.words ?? [],
      preset_id: options.presetId ?? "kata-tebal-putih",
      width: options.width ?? 1080,
      height: options.height ?? 1920,
      words_per_line: options.wordsPerLine ?? null,
      caption_position: options.captionPosition
        ? {
            x_percent: options.captionPosition.x,
            y_percent: options.captionPosition.y,
          }
        : null,
    },
  });
}

// ---------- burnVideo ----------
export async function burnVideo(options = {}, onProgress = () => {}) {
  if (!isTauri()) {
    throw new Error("Burn-in video hanya tersedia di aplikasi desktop.");
  }
  const unlisten = await listen("burn-progress", (event) => {
    onProgress(event.payload);
  });
  try {
    return await invoke("burn_video", {
      req: {
        input: options.input,
        words: options.words ?? [],
        preset_id: options.presetId,
        words_per_line: options.wordsPerLine,
        caption_position: options.captionPosition
          ? {
              x_percent: options.captionPosition.x,
              y_percent: options.captionPosition.y,
            }
          : null,
        output: options.output,
        quality: options.quality ?? "bagus",
      },
    });
  } finally {
    unlisten();
  }
}

export async function cancelBurn() {
  if (!isTauri()) {
    throw new Error("Pembatalan burn-in hanya tersedia di aplikasi desktop.");
  }
  return invoke("cancel_burn");
}

// ---------- listPresets ----------
export async function listPresets() {
  if (isTauri()) {
    return invoke("list_presets");
  }
  // MOCK (browser): daftar 12 preset agar galeri tetap tampil saat preview di browser
  return [
    {
      id: "kata-tebal-putih", name: "Kata Tebal Putih", category: "kata_per_kata", words_per_line: 1,
      font: { family: "Poppins ExtraBold", file: "Poppins-ExtraBold.ttf", size: 96, bold: false, italic: false, uppercase: true, letter_spacing: 0 },
      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#000000", shadow: "#00000080" },
      outline: 8, shadow: 4, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "none", emphasis_scale: null, animation: "pop",
    },
    {
      id: "kata-kotak-merah", name: "Kata Kotak Merah", category: "kata_per_kata", words_per_line: 1,
      font: { family: "Poppins ExtraBold", file: "Poppins-ExtraBold.ttf", size: 88, bold: false, italic: false, uppercase: true, letter_spacing: 0 },
      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#8B0A3C", shadow: "#00000000" },
      outline: 18, shadow: 0, box: { color: "#8B0A3C", padding: 22 }, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "none", emphasis_scale: null, animation: "none",
    },
    {
      id: "kata-anton-kuning", name: "Kata Anton Kuning", category: "kata_per_kata", words_per_line: 1,
      font: { family: "Anton", file: "Anton-Regular.ttf", size: 104, bold: false, italic: false, uppercase: true, letter_spacing: 1 },
      colors: { text: "#FFD400", highlight: "#FFFFFF", outline: "#000000", shadow: "#00000080" },
      outline: 8, shadow: 5, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "none", emphasis_scale: null, animation: "none",
    },
    {
      id: "kata-bangers", name: "Kata Bangers", category: "kata_per_kata", words_per_line: 1,
      font: { family: "Bangers", file: "Bangers-Regular.ttf", size: 100, bold: false, italic: false, uppercase: true, letter_spacing: 1 },
      colors: { text: "#FF9F1C", highlight: "#FFFFFF", outline: "#1A1A1A", shadow: "#00000080" },
      outline: 7, shadow: 4, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "none", emphasis_scale: null, animation: "none",
    },
    {
      id: "baris-kotak-hitam", name: "Baris Kotak Hitam", category: "baris_bergaya", words_per_line: 3,
      font: { family: "Poppins", file: "Poppins-Bold.ttf", size: 64, bold: true, italic: false, uppercase: false, letter_spacing: 0 },
      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#000000", shadow: "#00000000" },
      outline: 14, shadow: 0, box: { color: "#000000", padding: 20 }, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "none", emphasis_scale: null, animation: "none",
    },
    {
      id: "baris-kotak-kuning", name: "Baris Kotak Kuning", category: "baris_bergaya", words_per_line: 3,
      font: { family: "Poppins", file: "Poppins-Bold.ttf", size: 64, bold: true, italic: false, uppercase: false, letter_spacing: 0 },
      colors: { text: "#111111", highlight: "#8B0A3C", outline: "#FFE14D", shadow: "#00000000" },
      outline: 14, shadow: 0, box: { color: "#FFE14D", padding: 20 }, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "none", emphasis_scale: null, animation: "none",
    },
    {
      id: "baris-neon-mint", name: "Baris Neon Mint", category: "baris_bergaya", words_per_line: 3,
      font: { family: "Poppins ExtraBold", file: "Poppins-ExtraBold.ttf", size: 72, bold: false, italic: false, uppercase: true, letter_spacing: 2 },
      colors: { text: "#5CFFC0", highlight: "#FFFFFF", outline: "#003820", shadow: "#00000000" },
      outline: 6, shadow: 0, box: null, margin_v: 450, margin_lr: 80,
      glow: { color: "#5CFFC0", radius: 12 }, emphasis: "none", emphasis_scale: null, animation: "none",
    },
    {
      id: "baris-isi-kuning", name: "Baris Isi Kuning", category: "baris_bergaya", words_per_line: 3,
      font: { family: "Poppins", file: "Poppins-Bold.ttf", size: 68, bold: true, italic: false, uppercase: false, letter_spacing: 0 },
      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#000000", shadow: "#00000080" },
      outline: 7, shadow: 3, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "fill", emphasis_scale: null, animation: "none",
    },
    {
      id: "tekan-warna-kuning", name: "Tekan Warna Kuning", category: "penekanan_kata", words_per_line: 2,
      font: { family: "Poppins ExtraBold", file: "Poppins-ExtraBold.ttf", size: 78, bold: false, italic: false, uppercase: true, letter_spacing: 1 },
      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#000000", shadow: "#00000080" },
      outline: 7, shadow: 4, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "color_active", emphasis_scale: null, animation: "none",
    },
    {
      id: "tekan-garis-bawah", name: "Tekan Garis Bawah", category: "penekanan_kata", words_per_line: 2,
      font: { family: "Anton", file: "Anton-Regular.ttf", size: 92, bold: false, italic: false, uppercase: true, letter_spacing: 1 },
      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#000000", shadow: "#00000080" },
      outline: 6, shadow: 3, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "underline_active", emphasis_scale: null, animation: "none",
    },
    {
      id: "tekan-besar", name: "Tekan Besar", category: "penekanan_kata", words_per_line: 2,
      font: { family: "Poppins ExtraBold", file: "Poppins-ExtraBold.ttf", size: 76, bold: false, italic: false, uppercase: true, letter_spacing: 0 },
      colors: { text: "#FFFFFF", highlight: "#C6FF3D", outline: "#000000", shadow: "#00000080" },
      outline: 7, shadow: 4, box: null, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "scale_active", emphasis_scale: 118, animation: "none",
    },
    {
      id: "tekan-ungu-kotak-gelap", name: "Tekan Ungu Kotak Gelap", category: "penekanan_kata", words_per_line: 3,
      font: { family: "Poppins", file: "Poppins-Bold.ttf", size: 72, bold: true, italic: false, uppercase: false, letter_spacing: 0 },
      colors: { text: "#FFFFFF", highlight: "#B388FF", outline: "#141414", shadow: "#00000000" },
      outline: 16, shadow: 0, box: { color: "#141414", padding: 18 }, margin_v: 450, margin_lr: 80,
      glow: null, emphasis: "color_active", emphasis_scale: null, animation: "none",
    },
  ];
}
