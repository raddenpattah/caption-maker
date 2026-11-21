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

// MOCK - diganti invoke Tauri di 3B
export async function listModels() {
  return [
    { id: "base", name: "Base", installed: false },
    { id: "small", name: "Small", installed: true },
    { id: "medium", name: "Medium", installed: false },
  ];
}

// MOCK - diganti invoke Tauri di 3B
export async function importModel() {
  return chooseFile(".bin");
}

// MOCK - diganti invoke Tauri di 3B
export async function pickInputFile() {
  return chooseFile(".mp4,.mkv,.mov,.mp3,.wav,.m4a");
}

// MOCK - diganti invoke Tauri di 3B
export async function transcribe(options = {}, onProgress = () => {}) {
  const { signal } = options;
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

// MOCK - diganti invoke Tauri di 3B
export async function exportAss() {
  const message = "Ekspor .ass nyata akan dihubungkan di Fase 3B.";
  window.alert(message);
  return { exported: false, message };
}
