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
    { id: "base", name: "Base", installed: false },
    { id: "small", name: "Small", installed: true },
    { id: "medium", name: "Medium", installed: false },
  ];
}

// ---------- importModel ----------
export async function importModel() {
  if (isTauri()) {
    // Buka dialog file via HTML (bisa diganti plugin-dialog nanti)
    const file = await chooseFile(".bin");
    if (!file) return null;
    // Tauri 2: File object punya path via file.path (butuh plugin-fs). Fallback nama.
    const path = file.path ?? file.name;
    return invoke("import_model", { path });
  }
  // MOCK (browser)
  return chooseFile(".bin");
}

// ---------- pickInputFile ----------
export async function pickInputFile() {
  if (isTauri()) {
    const file = await chooseFile(".mp4,.mkv,.mov,.mp3,.wav,.m4a");
    if (!file) return null;
    const path = file.path ?? file.name;
    return invoke("pick_input_file", { path });
  }
  // MOCK (browser)
  return chooseFile(".mp4,.mkv,.mov,.mp3,.wav,.m4a");
}

// ---------- transcribe ----------
export async function transcribe(options = {}, onProgress = () => {}) {
  const { signal } = options;

  if (isTauri()) {
    // Dengarkan progress dari backend
    const unlisten = await listen("transcribe_progress", (event) => {
      onProgress(event.payload);
    });
    try {
      const words = await invoke("transcribe", {
        request: {
          input: options.input ?? null,
          model: options.model ?? null,
          language: options.language ?? "id",
          threads: options.threads ?? null,
          prompt: options.prompt ?? null,
          words_per_line: options.wordsPerLine ?? null,
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

// ---------- exportAss ----------
export async function exportAss(options = {}) {
  if (isTauri()) {
    // Render ASS via command backend. `options` berisi { words, presetId, width, height }
    return invoke("render_ass_from_words", {
      req: {
        words: options.words ?? [],
        preset_id: options.presetId ?? "kata-tebal-putih",
        width: options.width ?? 1080,
        height: options.height ?? 1920,
        words_per_line: options.wordsPerLine ?? null,
      },
    });
  }
  // MOCK (browser)
  const message = "Ekspor .ass nyata akan dihubungkan di Fase 3B.";
  window.alert(message);
  return { exported: false, message };
}
