import {
  exportAss,
  importModel,
  listModels,
  pickInputFile,
  transcribe,
} from "./api.js";
import { createState } from "./state.js";

const state = createState();
const byId = (id) => document.getElementById(id);
const video = byId("video-player");
const audio = byId("audio-player");
const previewSubtitle = byId("preview-subtitle");
const progressBar = byId("progress-bar");
const supportedExtensions = new Set(["mp4", "mkv", "mov", "mp3", "wav", "m4a"]);
let activeObjectUrl = null;
let internalTimer = null;
let installedModelIds = new Set();

function formatTime(milliseconds) {
  const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) {
    return [hours, minutes, seconds].map((part) => String(part).padStart(2, "0")).join(":");
  }
  return [minutes, seconds].map((part) => String(part).padStart(2, "0")).join(":");
}

function setProgress(stage, progress) {
  const safeProgress = Math.max(0, Math.min(100, progress));
  byId("progress-label").textContent = stage;
  byId("progress-percent").textContent = `${Math.round(safeProgress)}%`;
  progressBar.value = safeProgress;
  progressBar.textContent = `${Math.round(safeProgress)}%`;
}

function currentMediaElement() {
  const media = state.get().media;
  return media?.kind === "video" ? video : media?.kind === "audio" ? audio : null;
}

function renderPreviewText() {
  const { words, currentMs, settings } = state.get();
  previewSubtitle.dataset.position = settings.position;
  previewSubtitle.style.fontFamily = settings.font;
  previewSubtitle.style.fontSize = `${settings.fontSize}px`;
  previewSubtitle.style.color = settings.textColor;
  previewSubtitle.style.webkitTextStroke = `${settings.outline}px #000`;
  previewSubtitle.style.textShadow = settings.shadow
    ? `0 ${settings.shadow}px ${settings.shadow * 2}px #000`
    : "none";

  const activeWord = words.find(
    (word) => currentMs >= word.start_ms && currentMs <= word.end_ms,
  );
  previewSubtitle.replaceChildren();
  if (!activeWord) {
    const placeholder = document.createElement("span");
    placeholder.className = "preview-placeholder";
    placeholder.textContent = words.length
      ? "Putar atau geser waktu untuk melihat subtitle"
      : "Pratinjau subtitle akan tampil di sini";
    previewSubtitle.append(placeholder);
    return;
  }

  const text = document.createElement("span");
  text.textContent = activeWord.text;
  if (settings.highlight) {
    text.style.color = settings.highlightColor;
  }
  previewSubtitle.append(text);
}

function updateClock(milliseconds) {
  const { durationMs } = state.get();
  byId("current-time").textContent = formatTime(milliseconds);
  byId("total-time").textContent = formatTime(durationMs);
  byId("seek-slider").value = String(Math.min(milliseconds, durationMs) / 1000);
  renderPreviewText();
}

function updatePlayButton(playing) {
  const button = byId("play-pause");
  button.textContent = playing ? "Ⅱ" : "▶";
  button.setAttribute("aria-label", playing ? "Jeda pratinjau" : "Putar pratinjau");
  button.setAttribute("aria-pressed", String(playing));
}

function updateAvailability() {
  const modelInstalled = installedModelIds.has(byId("model-select").value);
  byId("generate-subtitles").disabled =
    !state.get().media || !modelInstalled || Boolean(state.get().controller);
  byId("cancel-task").disabled = !state.get().controller;
}

function updateSettingsFromControls() {
  const settings = {
    wordsPerLine: Number(document.querySelector('input[name="words-per-line"]:checked').value),
    highlight: byId("highlight-toggle").checked,
    font: byId("font-select").value,
    fontSize: Number(byId("font-size").value),
    textColor: byId("text-color").value,
    highlightColor: byId("highlight-color").value,
    position: byId("position-select").value,
    outline: Number(byId("outline-size").value),
    shadow: Number(byId("shadow-size").value),
  };
  state.update({ settings });
  byId("font-size-value").value = `${settings.fontSize} px`;
  byId("outline-value").value = `${settings.outline} px`;
  byId("shadow-value").value = `${settings.shadow} px`;
  byId("text-color").nextElementSibling.textContent = settings.textColor.toUpperCase();
  byId("highlight-color").nextElementSibling.textContent =
    settings.highlightColor.toUpperCase();
  renderPreviewText();
}

function resetProgress(message = "Siap — pilih media untuk memulai") {
  setProgress(message, 0);
}

function updateMediaDuration(element) {
  if (!Number.isFinite(element.duration) || element.duration <= 0) {
    byId("file-duration").textContent = "Durasi: tidak tersedia";
    return;
  }
  const durationMs = Math.round(element.duration * 1000);
  state.update({ durationMs, currentMs: 0 });
  byId("file-duration").textContent = `Durasi: ${formatTime(durationMs)}`;
  byId("seek-slider").max = String(durationMs / 1000);
  updateClock(0);
}

function onMediaTimeUpdate() {
  const element = currentMediaElement();
  if (!element) return;
  const currentMs = Math.round(element.currentTime * 1000);
  state.update({ currentMs });
  updateClock(currentMs);
}

function pauseInternalTimer() {
  if (internalTimer !== null) {
    window.clearInterval(internalTimer);
    internalTimer = null;
  }
}

async function loadLocalFile(file) {
  const extension = file.name.split(".").pop()?.toLowerCase();
  if (!extension || !supportedExtensions.has(extension)) {
    setProgress("Format file tidak didukung.", 0);
    return;
  }

  pauseInternalTimer();
  video.pause();
  audio.pause();
  if (activeObjectUrl) URL.revokeObjectURL(activeObjectUrl);
  activeObjectUrl = URL.createObjectURL(file);

  const kind = ["mp4", "mkv", "mov"].includes(extension) ? "video" : "audio";
  video.hidden = kind !== "video";
  audio.hidden = kind !== "audio";
  video.removeAttribute("src");
  audio.removeAttribute("src");
  const element = kind === "video" ? video : audio;
  element.src = activeObjectUrl;
  element.load();

  state.update({
    media: { file, kind },
    words: [],
    generated: false,
    isPlaying: false,
    currentMs: 0,
    durationMs: 60_000,
  });
  byId("file-name").textContent = file.name;
  byId("file-duration").textContent = "Durasi: memuat...";
  byId("remove-file").disabled = false;
  byId("export-ass").disabled = true;
  byId("seek-slider").max = "60";
  byId("seek-slider").value = "0";
  byId("total-time").textContent = "00:00";
  updatePlayButton(false);
  updateAvailability();
  renderPreviewText();
  resetProgress("Media lokal siap — transkripsi masih berupa simulasi");
}

function clearMedia() {
  pauseInternalTimer();
  video.pause();
  audio.pause();
  video.removeAttribute("src");
  audio.removeAttribute("src");
  video.load();
  audio.load();
  if (activeObjectUrl) URL.revokeObjectURL(activeObjectUrl);
  activeObjectUrl = null;
  video.hidden = true;
  audio.hidden = true;
  state.update({
    media: null,
    words: [],
    generated: false,
    isPlaying: false,
    currentMs: 0,
    durationMs: 60_000,
  });
  byId("file-name").textContent = "Belum ada file dipilih";
  byId("file-duration").textContent = "Durasi: —";
  byId("remove-file").disabled = true;
  byId("export-ass").disabled = true;
  byId("seek-slider").max = "60";
  byId("seek-slider").value = "0";
  updatePlayButton(false);
  updateAvailability();
  updateClock(0);
  resetProgress();
}

async function chooseMedia() {
  const file = await pickInputFile();
  if (file) await loadLocalFile(file);
}

async function importSelectedModel() {
  const file = await importModel();
  if (!file) return;
  byId("model-status").textContent =
    `${file.name} dipilih; pemasangan model nyata tersedia di Fase 3B.`;
}

async function loadModels() {
  try {
    const models = await listModels();
    installedModelIds = new Set(models.filter((model) => model.installed).map((model) => model.id));
    const select = byId("model-select");
    select.replaceChildren();
    for (const model of models) {
      const option = document.createElement("option");
      option.value = model.id;
      option.textContent = model.installed
        ? `${model.name} (terpasang)`
        : `${model.name} (belum terpasang)`;
      option.disabled = !model.installed;
      select.append(option);
    }

    const installedModel = models.find((model) => model.installed);
    byId("model-guidance").hidden = Boolean(installedModel);
    byId("model-status").textContent = installedModel
      ? `Model ${installedModel.name.toLowerCase()} tersedia dalam simulasi.`
      : "Impor model untuk mengaktifkan transkripsi.";
    if (installedModel) select.value = installedModel.id;
    updateAvailability();
  } catch (error) {
    byId("model-status").textContent = `Gagal memuat daftar model: ${error.message}`;
    setProgress("Daftar model tidak dapat dimuat.", 0);
  }
}

async function generateSubtitles() {
  if (!state.get().media || !installedModelIds.has(byId("model-select").value)) return;
  const controller = new AbortController();
  state.update({ controller });
  byId("export-ass").disabled = true;
  updateAvailability();
  setProgress("Menyiapkan simulasi transkripsi", 0);

  try {
    const words = await transcribe(
      {
        signal: controller.signal,
        model: byId("model-select").value,
        file: state.get().media.file,
        settings: state.get().settings,
      },
      ({ stage, progress }) => setProgress(stage, progress),
    );
    state.update({ words, generated: true });
    byId("export-ass").disabled = false;
    setProgress(`Simulasi selesai — ${words.length} kata`, 100);
    renderPreviewText();
  } catch (error) {
    if (error.name === "AbortError") {
      resetProgress("Simulasi dibatalkan");
    } else {
      setProgress(`Transkripsi gagal: ${error.message}`, 0);
    }
  } finally {
    state.update({ controller: null });
    updateAvailability();
  }
}

async function exportSubtitles() {
  if (!state.get().generated) return;
  const result = await exportAss({
    words: state.get().words,
    settings: state.get().settings,
  });
  setProgress(result.message, 100);
}

async function togglePlayback() {
  const element = currentMediaElement();
  if (element) {
    if (element.paused) {
      try {
        await element.play();
      } catch (error) {
        setProgress(`Media tidak dapat diputar: ${error.message}`, 0);
      }
    } else {
      element.pause();
    }
    return;
  }

  if (internalTimer !== null) {
    pauseInternalTimer();
    state.update({ isPlaying: false });
    updatePlayButton(false);
    return;
  }

  state.update({ isPlaying: true });
  updatePlayButton(true);
  internalTimer = window.setInterval(() => {
    const nextMs = state.get().currentMs + 100;
    const durationMs = state.get().durationMs;
    const currentMs = nextMs >= durationMs ? 0 : nextMs;
    state.update({ currentMs });
    updateClock(currentMs);
  }, 100);
}

for (const mediaElement of [video, audio]) {
  mediaElement.addEventListener("loadedmetadata", () => updateMediaDuration(mediaElement));
  mediaElement.addEventListener("durationchange", () => updateMediaDuration(mediaElement));
  mediaElement.addEventListener("timeupdate", onMediaTimeUpdate);
  mediaElement.addEventListener("play", () => {
    state.update({ isPlaying: true });
    updatePlayButton(true);
  });
  mediaElement.addEventListener("pause", () => {
    state.update({ isPlaying: false });
    updatePlayButton(false);
  });
  mediaElement.addEventListener("ended", () => {
    state.update({ isPlaying: false });
    updatePlayButton(false);
  });
  mediaElement.addEventListener("error", () => {
    byId("file-duration").textContent = "Durasi: tidak dapat dibaca";
    setProgress("Format media tidak dapat diputar oleh browser ini.", 0);
  });
}

byId("choose-file").addEventListener("click", chooseMedia);
byId("remove-file").addEventListener("click", clearMedia);
byId("play-pause").addEventListener("click", togglePlayback);
byId("generate-subtitles").addEventListener("click", generateSubtitles);
byId("cancel-task").addEventListener("click", () => state.get().controller?.abort());
byId("export-ass").addEventListener("click", exportSubtitles);
byId("import-model").addEventListener("click", importSelectedModel);
byId("menu-import-model").addEventListener("click", importSelectedModel);
byId("guidance-import").addEventListener("click", importSelectedModel);

byId("seek-slider").addEventListener("input", (event) => {
  const currentMs = Math.round(Number(event.target.value) * 1000);
  const element = currentMediaElement();
  if (element) element.currentTime = currentMs / 1000;
  state.update({ currentMs });
  updateClock(currentMs);
});

for (const control of document.querySelectorAll(
  'input[name="words-per-line"], #highlight-toggle, #font-select, #font-size, #text-color, #highlight-color, #position-select, #outline-size, #shadow-size',
)) {
  control.addEventListener("input", updateSettingsFromControls);
  control.addEventListener("change", updateSettingsFromControls);
}

const dropZone = byId("drop-zone");
dropZone.addEventListener("dragover", (event) => {
  event.preventDefault();
  dropZone.classList.add("drag-over");
});
dropZone.addEventListener("dragleave", () => dropZone.classList.remove("drag-over"));
dropZone.addEventListener("drop", (event) => {
  event.preventDefault();
  dropZone.classList.remove("drag-over");
  const file = event.dataTransfer?.files[0];
  if (file) loadLocalFile(file);
});

window.addEventListener("beforeunload", () => {
  pauseInternalTimer();
  if (activeObjectUrl) URL.revokeObjectURL(activeObjectUrl);
});

updateSettingsFromControls();
updateClock(0);
loadModels();
