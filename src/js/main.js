import {
  exportSubtitleFile,
  burnVideo,
  cancelBurn,
  cancelTranscribe,
  importModel,
  listModels,
  modelsDirectory,
  listPresets,
  pickInputFile,
  transcribe,
} from "./api.js";
import {
  applyCaptionPosition,
  clampCaptionPosition,
  computePreviewMetrics,
  getCaptionPosition,
  getCaptionVisualOutset,
  getVideoRect,
  groupWords,
  initLibassPreview,
  isLibassReady,
  setAssContent,
  setCurrentTime,
  setIsPaused,
  setRate,
  resizeLibassCanvas,
  loadPreviewFont,
  moveCaptionPosition,
  renderPreview,
} from "./preview.js";
import { createState } from "./state.js";
import { invoke, isTauri } from "./tauri-bridge.js";
import {
  applyGroupText,
  deleteWord,
  mergeWithNext,
  resetWords,
} from "./wordEdit.js";

function updateStageAspectRatio(videoElement) {
  const stage = document.getElementById("preview-stage");
  if (!stage || !videoElement || !videoElement.videoWidth || !videoElement.videoHeight) {
    return;
  }
  const aspect = videoElement.videoWidth / videoElement.videoHeight;
  stage.style.aspectRatio = aspect;
  stage.style.height = "100%";
  stage.style.width = "auto";
  stage.style.maxWidth = "100%";
  stage.style.margin = "auto";
}

const state = createState();

// Render ASS dari backend dan kirim ke libass-wasm
async function renderAssForLibass(words) {
  if (!words || words.length === 0) return;
  const preset = selectedPreset();
  if (!preset) return;
  try {
    const result = await invoke("render_ass_from_words", {
      req: {
        words: words.map((w) => ({
          text: w.text,
          start_ms: w.start_ms,
          end_ms: w.end_ms,
        })),
        preset_id: preset.id,
        width: 1080,
        height: 1920,
        words_per_line: effectiveWordsPerLine(),
        caption_position: null,
      },
    });
    if (result?.ass) {
      setAssContent(result.ass);
    }
  } catch (err) {
    console.warn("[main] Gagal generate ASS untuk libass:", err);
  }
}

const byId = (id) => document.getElementById(id);
const video = byId("video-player");
const audio = byId("audio-player");
const previewStage = byId("preview-stage");
const previewCanvas = byId("preview-canvas");
const previewSubtitle = byId("preview-subtitle");
const wordEditorList = byId("word-editor-list");
const progressBar = byId("progress-bar");
const supportedExtensions = new Set(["mp4", "mkv", "mov", "mp3", "wav", "m4a"]);
let activeObjectUrl = null;
let sourcePlaybackSrc = null;
let preparingVideo = false;
let burnActive = false;
let burnCancelRequested = false;
let burnProgressStatus = null;
let transcribeCancelRequested = false;
let transcribeProgressStatus = null;
let burnedPreviewActive = false;
let burnedVideoPath = null;
let internalTimer = null;
let libassPlaybackFrame = null;
let mediaPlayPending = false;
let installedModelIds = new Set();
let presetCache = [];
let selectedPresetId = null;
let previewStageResizeObserver = null;
let loadedPreviewFontKey = null;
let pendingPreviewFontKey = null;
let pendingPreviewFont = null;
let editorMode = "1";
let editorNeedsRender = false;
let editorNeedsRenderWordIndex = null;
let rowPlayback = null;
const captionPositions = new Map();
let captionDrag = null;

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

function setIndeterminateProgress(stage) {
  byId("progress-label").textContent = stage;
  byId("progress-percent").textContent = "…";
  progressBar.removeAttribute("value");
  progressBar.textContent = stage;
}

function errorMessage(error) {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message || error.name || "Kesalahan tidak diketahui.";
  if (error && typeof error === "object" && typeof error.message === "string") {
    return error.message;
  }
  return error == null ? "Kesalahan tidak diketahui." : String(error);
}

function taskBusy() {
  return Boolean(state.get().controller) || preparingVideo || burnActive;
}

function updatePreviewTag() {
  document.querySelector(".preview-tag").textContent = isTauri() ? "PRATINJAU" : "SIMULASI";
}

function clearGeneratedWords(message) {
  const current = state.get();
  if (current.generated || current.words.length) {
    showSourcePreview();
  }
  state.update({ words: [], originalWords: null, edited: false, generated: false});
  renderPreviewText();
  renderWordEditor();
  updatePreviewTag();
  setProgress(message, 0);
  updateAvailability();
}

function selectedPreset() {
  return presetCache.find((preset) => preset.id === selectedPresetId);
}

function effectiveWordsPerLine() {
  return selectedPreset()?.words_per_line ?? state.get().settings.wordsPerLine;
}

function syncWordsPerLineControl() {
  const presetWordsPerLine = selectedPreset()?.words_per_line;
  const isPresetControlled = presetWordsPerLine != null;
  const fieldset = byId("words-per-line-field");
  fieldset.disabled = isPresetControlled;
  byId("words-per-line-note").hidden = !isPresetControlled;

  const value = isPresetControlled
    ? String(presetWordsPerLine)
    : state.get().settings.wordsPerLine === "sentence"
      ? "4"
      : String(state.get().settings.wordsPerLine);
  for (const input of document.querySelectorAll('input[name="words-per-line"]')) {
    input.checked = input.value === value;
  }
}

function selectPreset(id) {
  if (taskBusy()) return;
  showSourcePreview();
  selectedPresetId = id;
  document.querySelectorAll("#preset-gallery .preset-item").forEach((el) => {
    el.classList.toggle("active", el.dataset.presetId === id);
  });
  syncWordsPerLineControl();
  renderPreviewText();
}

function renderPresetGallery(presets) {
  const gallery = byId("preset-gallery");
  if (!gallery) return;
  if (!presets || presets.length === 0) {
    gallery.innerHTML = '<p class="preset-loading">Tidak ada preset.</p>';
    return;
  }
  gallery.innerHTML = "";
  for (const p of presets) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "preset-item";
    btn.dataset.presetId = p.id;
    btn.setAttribute("role", "radio");
    btn.setAttribute("aria-checked", "false");
    btn.innerHTML = `
      <span class="preset-preview">${p.name}</span>
      <span class="preset-name">${p.name}</span>
      <span class="preset-category">${p.category.replace(/_/g, " ")}</span>
    `;
    btn.addEventListener("click", () => selectPreset(p.id));
    gallery.appendChild(btn);
  }
  // Pilih preset pertama secara default
  if (presets.length > 0) selectPreset(presets[0].id);
  updateAvailability();
}

async function loadPresets() {
  try {
    presetCache = await listPresets();
    renderPresetGallery(presetCache);
  } catch (err) {
    console.error("Gagal memuat preset:", err);
    const gallery = byId("preset-gallery");
    if (gallery) {
      const message = document.createElement("p");
      message.className = "preset-loading";
      message.textContent = `Gagal memuat preset: ${errorMessage(err)}`;
      gallery.replaceChildren(message);
    }
  }
}

function currentMediaElement() {
  if (burnedPreviewActive) return video;
  const media = state.get().media;
  return media?.kind === "video" ? video : media?.kind === "audio" ? audio : null;
}

function resizeLibassPreviewCanvas() {
  if (!previewStage) return;
  const activeVideo = video && !video.hidden ? video : null;
  resizeLibassCanvas(getVideoRect(previewStage, activeVideo));
}

function renderPreviewText() {
  if (burnedPreviewActive) {
    previewSubtitle.hidden = true;
    return;
  }

  const { words, currentMs, settings } = state.get();
  const preset = selectedPreset();
  const font = preset?.font;
  const fontKey = font ? `${preset.id}:${font.size}px "${font.family}"` : null;
  if (fontKey && loadedPreviewFontKey !== fontKey) {
    if (pendingPreviewFontKey !== fontKey) {
      const fontLoad = loadPreviewFont(preset);
      pendingPreviewFontKey = fontKey;
      pendingPreviewFont = fontLoad;
      fontLoad.then(
        () => {
          if (pendingPreviewFont !== fontLoad || pendingPreviewFontKey !== fontKey) return;
          pendingPreviewFont = null;
          pendingPreviewFontKey = null;
          loadedPreviewFontKey = fontKey;
          if (!burnedPreviewActive && selectedPreset()?.id === preset.id) {
            renderPreviewText();
          }
        },
        (error) => {
          console.error("Gagal memuat font pratinjau:", error);
          if (pendingPreviewFont !== fontLoad || pendingPreviewFontKey !== fontKey) return;
          pendingPreviewFont = null;
          pendingPreviewFontKey = null;
          loadedPreviewFontKey = fontKey;
          if (!burnedPreviewActive && selectedPreset()?.id === preset.id) {
            renderPreviewText();
          }
        },
      );
    }
    return;
  }

  const hasActiveGroup = renderPreview(previewSubtitle, words, currentMs, {
    ...settings,
    preset,
    wordsPerLine: effectiveWordsPerLine(),
    captionPosition: preset ? captionPositions.get(preset.id) ?? null : null,
  });
  if (hasActiveGroup && preset) syncRenderedCaptionPosition(preset, settings);
  if (!hasActiveGroup) {
    const placeholder = document.createElement("span");
    placeholder.className = "preview-placeholder";
    placeholder.textContent = words.length
      ? "Putar atau geser waktu untuk melihat subtitle"
      : "Pratinjau subtitle akan tampil di sini";
    previewSubtitle.append(placeholder);
  }
}

function getPreviewVideoBounds() {
  const { left, top, width, height } = previewSubtitle.getBoundingClientRect();
  return { left, top, width, height };
}

function getCaptionMetrics(preset, settings) {
  const activeVideo = video && !video.hidden ? video : null;
  const videoRect = getVideoRect(previewStage, activeVideo);
  return computePreviewMetrics({
    rect: videoRect,
    videoWidth: activeVideo?.videoWidth,
    videoHeight: activeVideo?.videoHeight,
    preset,
    settings,
  });
}

function syncRenderedCaptionPosition(preset, settings) {
  const content = previewSubtitle.querySelector(".subtitle-content");
  if (!content) return;

  const videoRect = getPreviewVideoBounds();
  const blockRect = content.getBoundingClientRect();
  const savedPosition = captionPositions.get(preset.id);
  const position = clampCaptionPosition(
    savedPosition ?? getCaptionPosition(videoRect, blockRect),
    videoRect,
    blockRect,
    getCaptionVisualOutset(getCaptionMetrics(preset, settings)),
  );
  captionPositions.set(preset.id, position);
  applyCaptionPosition(content, position);
}

function beginCaptionDrag(event) {
  if (
    event.button !== 0 ||
    !event.isPrimary ||
    burnedPreviewActive ||
    !selectedPresetId
  ) {
    return;
  }
  const content = event.target.closest?.("#preview-subtitle .subtitle-content");
  if (!content || !previewSubtitle.contains(content)) return;

  const preset = selectedPreset();
  if (!preset) return;
  const bounds = getPreviewVideoBounds();
  const blockRect = content.getBoundingClientRect();
  const position = captionPositions.get(preset.id) ??
    clampCaptionPosition(
      getCaptionPosition(bounds, blockRect),
      bounds,
      blockRect,
      getCaptionVisualOutset(getCaptionMetrics(preset, state.get().settings)),
    );
  captionPositions.set(preset.id, position);
  captionDrag = {
    pointerId: event.pointerId,
    presetId: preset.id,
    startX: event.clientX,
    startY: event.clientY,
    startPosition: position,
    dragging: false,
  };
  event.preventDefault();
  previewStage.setPointerCapture(event.pointerId);
}

function moveCaptionDrag(event) {
  if (!captionDrag || event.pointerId !== captionDrag.pointerId) return;
  const deltaX = event.clientX - captionDrag.startX;
  const deltaY = event.clientY - captionDrag.startY;
  if (!captionDrag.dragging && Math.hypot(deltaX, deltaY) < 4) return;

  captionDrag.dragging = true;
  previewStage.classList.add("is-caption-dragging");
  const preset = presetCache.find((item) => item.id === captionDrag.presetId);
  const content = previewSubtitle.querySelector(".subtitle-content");
  if (!preset || !content) return;

  const videoRect = getPreviewVideoBounds();
  const position = moveCaptionPosition(
    captionDrag.startPosition,
    deltaX,
    deltaY,
    videoRect,
    content.getBoundingClientRect(),
    getCaptionVisualOutset(getCaptionMetrics(preset, state.get().settings)),
  );
  captionPositions.set(preset.id, position);
  applyCaptionPosition(content, position);
  event.preventDefault();
}

function endCaptionDrag(event) {
  if (!captionDrag || (event && event.pointerId !== captionDrag.pointerId)) return;
  const { pointerId } = captionDrag;
  captionDrag = null;
  previewStage.classList.remove("is-caption-dragging");
  if (previewStage.hasPointerCapture(pointerId)) {
    previewStage.releasePointerCapture(pointerId);
  }
}

function observePreviewStageResize() {
  if (previewStageResizeObserver) return;
  previewStageResizeObserver = new ResizeObserver(() => {
    resizeLibassPreviewCanvas();
    if (!burnedPreviewActive && selectedPreset()) renderPreviewText();
  });
  previewStageResizeObserver.observe(byId("preview-stage"));
}

function updateClock(milliseconds) {
  const { durationMs } = state.get();
  byId("current-time").textContent = formatTime(milliseconds);
  byId("total-time").textContent = formatTime(durationMs);
  byId("seek-slider").value = String(Math.min(milliseconds, durationMs) / 1000);
  if (!burnedPreviewActive) renderPreviewText();
  updateEditorPlayback(milliseconds);
}

function updatePlayButton(playing) {
  const button = byId("play-pause");
  button.textContent = playing ? "Ⅱ" : "▶";
  button.setAttribute("aria-label", playing ? "Jeda pratinjau" : "Putar pratinjau");
  button.setAttribute("aria-pressed", String(playing));
}

function seekActiveMedia(offsetSeconds) {
  const element = currentMediaElement();
  if (!element || element.readyState < 1 || !Number.isFinite(element.currentTime)) return;

  stopRowPlayback();
  const duration = Number.isFinite(element.duration) ? element.duration : Number.POSITIVE_INFINITY;
  const nextTime = Math.max(0, Math.min(duration, element.currentTime + offsetSeconds));
  element.currentTime = nextTime;
  const currentMs = Math.round(nextTime * 1000);
  state.update({ currentMs });
  updateClock(currentMs);
}

function stepVideoFrame(direction) {
  if (currentMediaElement() !== video || video.readyState < 1) return;
  seekActiveMedia(direction / 30);
}

function updateAvailability() {
  const modelInstalled = installedModelIds.has(byId("model-select").value);
  const { words, generated } = state.get();
  const presetEnabled = generated && words.length > 0;
  const subtitleTaskActive = Boolean(state.get().controller);
  const videoOperationActive = preparingVideo || burnActive;
  const busy = subtitleTaskActive || videoOperationActive;
  const activeMedia = currentMediaElement();
  const mediaReady = Boolean(activeMedia && activeMedia.readyState >= 1);
  const presetGallery = byId("preset-gallery");
  presetGallery.classList.toggle("is-disabled", !presetEnabled);
  presetGallery.setAttribute("aria-disabled", String(!presetEnabled));
  byId("seek-back-5").disabled = busy || !mediaReady;
  byId("seek-forward-5").disabled = busy || !mediaReady;
  byId("frame-backward").disabled = busy || activeMedia !== video || !mediaReady;
  byId("frame-forward").disabled = busy || activeMedia !== video || !mediaReady;
  byId("generate-subtitles").disabled =
    !state.get().media ||
    !modelInstalled ||
    busy;
  byId("edit-text").disabled = !words.length || busy;
  updateEditorAvailability(busy, mediaReady);
  byId("cancel-task").disabled = burnActive
    ? burnCancelRequested || burnProgressStatus !== "running"
    : subtitleTaskActive
      ? transcribeCancelRequested || (isTauri() && transcribeProgressStatus !== "running")
      : true;
  byId("choose-file").disabled = busy;
  byId("remove-file").disabled = busy || !state.get().media;
  byId("export-ass").disabled = busy || !state.get().generated || !words.length;
  byId("export-format").disabled = busy;
  byId("create-video").disabled = busy;
  byId("model-select").disabled = busy;
  byId("import-model").disabled = busy;
  byId("guidance-import").disabled = busy;
  byId("menu-import-model").disabled = busy;
  document.querySelectorAll("#preset-gallery .preset-item").forEach((button) => {
    button.disabled = busy || !presetEnabled;
  });
}

function updateSettingsFromControls() {
  const selectedWordsPerLine = document.querySelector('input[name="words-per-line"]:checked').value;
  const settings = {
    ...state.get().settings,
    highlight: byId("highlight-toggle").checked,
    font: byId("font-select").value,
    fontSize: Number(byId("font-size").value),
    textColor: byId("text-color").value,
    highlightColor: byId("highlight-color").value,
    position: byId("position-select").value,
    outline: Number(byId("outline-size").value),
    shadow: Number(byId("shadow-size").value),
  };
  if (selectedPreset()?.words_per_line == null) {
    settings.wordsPerLine =
      selectedWordsPerLine === "4" ? "sentence" : Number(selectedWordsPerLine);
  }
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
  if (element !== currentMediaElement()) {
    return;
  }
  updateAvailability();
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
  syncLibassCurrentTime(element);
}

function stopLibassPlaybackSync() {
  if (libassPlaybackFrame !== null) {
    cancelAnimationFrame(libassPlaybackFrame);
    libassPlaybackFrame = null;
  }
}

function syncLibassCurrentTime(mediaElement) {
  if (mediaElement !== currentMediaElement() || !isLibassReady()) return;
  setCurrentTime(mediaElement.currentTime);
}

function syncLibassPlaybackState(mediaElement) {
  if (!isLibassReady()) {
    stopLibassPlaybackSync();
    return;
  }
  if (mediaElement !== currentMediaElement()) return;

  setIsPaused(mediaElement.paused, mediaElement.currentTime);
  setRate(mediaElement.playbackRate);
  syncLibassCurrentTime(mediaElement);
  if (!isLibassReady()) return;

  if (mediaElement.paused) {
    stopLibassPlaybackSync();
    return;
  }

  if (libassPlaybackFrame !== null) return;

  const syncFrame = () => {
    libassPlaybackFrame = null;
    const activeMedia = currentMediaElement();
    if (!isLibassReady() || !activeMedia || activeMedia.paused) {
      stopLibassPlaybackSync();
      return;
    }

    setCurrentTime(activeMedia.currentTime);
    if (!isLibassReady()) return;
    libassPlaybackFrame = requestAnimationFrame(syncFrame);
  };

  libassPlaybackFrame = requestAnimationFrame(syncFrame);
}

function handleLibassError() {
  stopLibassPlaybackSync();
  if (!burnedPreviewActive) {
    previewSubtitle.hidden = false;
    renderPreviewText();
  }
}

function pauseInternalTimer() {
  if (internalTimer !== null) {
    window.clearInterval(internalTimer);
    internalTimer = null;
  }
}

function showSourcePreview() {
  if (!burnedPreviewActive) return;

  const media = state.get().media;
  video.pause();
  audio.pause();
  burnedPreviewActive = false;
  burnedVideoPath = null;
  video.removeAttribute("src");
  video.load();
  previewSubtitle.hidden = false;
  document.querySelector(".preview-tag").hidden = false;
  byId("show-source-video").hidden = true;

  if (media?.kind === "video") {
    video.hidden = false;
    audio.hidden = true;
    video.src = sourcePlaybackSrc;
    video.load();
  } else {
    video.removeAttribute("src");
    video.load();
    video.hidden = true;
    audio.hidden = !media;
    if (media?.kind === "audio") {
      audio.load();
    }
  }

  state.update({ currentMs: 0, isPlaying: false });
  updatePlayButton(false);
  const sourceElement = currentMediaElement();
  if (sourceElement?.readyState >= 1) updateMediaDuration(sourceElement);
  else updateClock(0);
}

function showBurnedPreview(path) {
  const convertFileSrc = window.__TAURI__?.core?.convertFileSrc;
  if (typeof convertFileSrc !== "function") return false;

  let source;
  try {
    source = convertFileSrc(path);
  } catch (error) {
    console.error("Video tersimpan, tetapi path pratinjau tidak dapat dikonversi:", error);
    return false;
  }

  pauseInternalTimer();
  video.pause();
  audio.pause();
  burnedPreviewActive = true;
  burnedVideoPath = path;
  video.hidden = false;
  audio.hidden = true;
  previewSubtitle.hidden = true;
  document.querySelector(".preview-tag").hidden = true;
  byId("show-source-video").hidden = false;
  video.src = source;
  video.load();
  state.update({ currentMs: 0, isPlaying: false });
  updatePlayButton(false);
  updateClock(0);
  return true;
}

async function loadLocalFile(fileOrPath) {
  if (taskBusy()) return;
  let file;
  let extension;
  let src;

  if (typeof fileOrPath === "string") {
    // Path dari dialog Tauri (asset protocol)
    const fileName = fileOrPath.split(/[/\\]/).pop() ?? "media";
    file = { name: fileName };
    extension = fileName.split(".").pop()?.toLowerCase();
    src = window.__TAURI__.core.convertFileSrc(fileOrPath);
  } else {
    // Objek File dari drag-drop browser
    file = fileOrPath;
    extension = file.name.split(".").pop()?.toLowerCase();
    src = URL.createObjectURL(file);
  }

  if (!extension || !supportedExtensions.has(extension)) {
    setProgress("Format file tidak didukung.", 0);
    return;
  }

  showSourcePreview();
  pauseInternalTimer();
  video.pause();
  audio.pause();
  if (activeObjectUrl && activeObjectUrl.startsWith("blob:")) URL.revokeObjectURL(activeObjectUrl);
  activeObjectUrl = src;
  sourcePlaybackSrc = src;
  burnedPreviewActive = false;
  burnedVideoPath = null;

  const kind = ["mp4", "mkv", "mov"].includes(extension) ? "video" : "audio";
  video.hidden = kind !== "video";
  audio.hidden = kind !== "audio";
  video.removeAttribute("src");
  audio.removeAttribute("src");
  const element = kind === "video" ? video : audio;
  element.src = src;
  element.load();

  state.update({
    media: { file, kind, path: typeof fileOrPath === "string" ? fileOrPath : null },
    words: [],
    originalWords: null,
    edited: false,
    generated: false,
    isPlaying: false,
    currentMs: 0,
    durationMs: 60_000,
  });
  byId("file-name").textContent = file.name;
  byId("file-duration").textContent = "Durasi: memuat...";
  byId("remove-file").disabled = false;
  byId("export-ass").disabled = true;
  byId("show-source-video").hidden = true;
  previewSubtitle.hidden = false;
  document.querySelector(".preview-tag").hidden = false;
  byId("seek-slider").max = "60";
  byId("seek-slider").value = "0";
  byId("total-time").textContent = "00:00";
  updatePlayButton(false);
  updateAvailability();
  renderPreviewText();
  renderWordEditor();
  resetProgress(
    isTauri()
      ? "Media lokal siap — buat subtitle untuk mulai mentranskripsi."
      : "Media lokal siap — transkripsi masih berupa simulasi",
  );
}

function clearMedia() {
  if (taskBusy()) return;
  stopRowPlayback();
  pauseInternalTimer();
  video.pause();
  audio.pause();
  video.removeAttribute("src");
  audio.removeAttribute("src");
  video.load();
  audio.load();
  if (activeObjectUrl) URL.revokeObjectURL(activeObjectUrl);
  activeObjectUrl = null;
  sourcePlaybackSrc = null;
  burnedPreviewActive = false;
  burnedVideoPath = null;
  video.hidden = true;
  audio.hidden = true;
  previewSubtitle.hidden = false;
  document.querySelector(".preview-tag").hidden = false;
  byId("show-source-video").hidden = true;
  state.update({
    media: null,
    words: [],
    originalWords: null,
    edited: false,
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
  renderWordEditor();
  resetProgress();
}

async function chooseMedia() {
  if (taskBusy()) return;
  const file = await pickInputFile();
  if (file) await loadLocalFile(file);
}

async function importSelectedModel() {
  if (taskBusy()) return;
  try {
    const result = await importModel();
    if (!result) return;
    if (typeof File !== "undefined" && result instanceof File) {
      byId("model-status").textContent =
        `${result.name} dipilih; pemasangan model nyata hanya tersedia di aplikasi desktop.`;
      return;
    }

    const models = await loadModels(result.id);
    if (!models) return;
    byId("model-status").textContent =
      `Model ${result.name} berhasil diimpor (${formatModelSize(result.size_bytes)}).`;
  } catch (error) {
    byId("model-status").textContent = `Gagal mengimpor model: ${errorMessage(error)}`;
  }
}

function formatModelSize(bytes) {
  if (!Number.isFinite(bytes) || bytes <= 0) return "ukuran tidak diketahui";
  return `${Math.round(bytes / (1024 * 1024))} MiB`;
}

async function loadModels(preferredModelId = null) {
  try {
    const models = await listModels();
    const modelDirectory = await modelsDirectory();
    installedModelIds = new Set(models.filter((model) => model.installed).map((model) => model.id));
    const select = byId("model-select");
    const previousModelId = select.value;
    select.replaceChildren();
    for (const model of models) {
      const option = document.createElement("option");
      option.value = model.id;
      const displayedSize = model.installed ? model.size_bytes : model.estimated_size_bytes;
      const sizeLabel = displayedSize ? ` · ${formatModelSize(displayedSize)}` : "";
      option.textContent = `${model.name} (${model.installed ? "terpasang" : "belum terpasang"}${sizeLabel})`;
      option.disabled = !model.installed;
      select.append(option);
    }

    const installedModel =
      models.find((model) => model.id === preferredModelId && model.installed) ??
      models.find((model) => model.id === previousModelId && model.installed) ??
      models.find((model) => model.installed);
    byId("model-guidance").hidden = Boolean(installedModel);
    byId("model-status").textContent = installedModel
      ? isTauri()
        ? `Model ${installedModel.name} terpasang (${formatModelSize(installedModel.size_bytes)}).`
        : `Model ${installedModel.name.toLowerCase()} tersedia dalam simulasi.`
      : "Impor model untuk mengaktifkan transkripsi.";
    if (installedModel) select.value = installedModel.id;
    if (select.value !== previousModelId && previousModelId) {
      clearGeneratedWords("Model berubah — buat subtitle kembali.");
    }
    byId("model-location").textContent = modelDirectory
      ? `Model disimpan di: ${modelDirectory}`
      : "";
    updateAvailability();
    return models;
  } catch (error) {
    byId("model-status").textContent = `Gagal memuat daftar model: ${errorMessage(error)}`;
    setProgress("Daftar model tidak dapat dimuat.", 0);
    return null;
  }
}

async function generateSubtitles() {
  if (state.get().controller || burnActive) return false;
  const media = state.get().media;
  if (!media) {
    setProgress("Pilih media terlebih dahulu sebelum membuat subtitle.", 0);
    return false;
  }
  const modelId = byId("model-select").value;
  if (isTauri()) {
    if (media.kind !== "video") {
      setProgress("Transkripsi desktop memerlukan file video, bukan file audio.", 0);
      return false;
    }
    if (!media.path) {
      setProgress("Pilih media melalui tombol Pilih File agar path lokal tersedia.", 0);
      return false;
    }
    if (!installedModelIds.has(modelId)) {
      setProgress("Impor model Whisper dulu (tombol Impor)", 0);
      return false;
    }
  }

  if (
    state.get().edited &&
    !window.confirm("Menjalankan Buat Subtitle lagi akan menghapus semua suntingan. Lanjutkan?")
  ) {
    return false;
  }

  const controller = new AbortController();
  stopRowPlayback();
  state.update({ controller, originalWords: null });
  transcribeCancelRequested = false;
  transcribeProgressStatus = "starting";
  updateAvailability();
  const startedAt = performance.now();
  let elapsedMs = null;
  setIndeterminateProgress("Mengekstrak audio: Menyiapkan...");
  try {
    const words = await transcribe(
      media.path,
      modelId,
      "id",
      (progress) => {
        if (progress.status) transcribeProgressStatus = progress.status;
        updateAvailability();
        if (progress.status === "completed") {
          elapsedMs = progress.elapsed_ms ?? null;
          setProgress("Transkripsi selesai", 100);
          return;
        }
        if (progress.status === "cancelled") {
          setProgress("Dibatalkan", 0);
          return;
        }
        if (progress.status === "starting") {
          setIndeterminateProgress(`${progress.stage}: Menyiapkan...`);
          return;
        }
        if (progress.status !== "running") return;

        const stage = progress.stage === "Transkripsi" ? "Mentranskripsi" : progress.stage;
        if (progress.progress == null) {
          setIndeterminateProgress(`${stage}: Memproses...`);
        } else {
          setProgress(`${stage}: ${progress.progress}%`, progress.progress);
        }
      },
      { signal: controller.signal },
    );
    if (transcribeCancelRequested || controller.signal.aborted) {
      setProgress("Dibatalkan", 0);
      return false;
    }
    if (!words.length) {
      state.update({ words: [], originalWords: null, edited: false, generated: false});
      renderPreviewText();
      renderWordEditor();
      updatePreviewTag();
      updateAvailability();
      setProgress("Tidak ada ucapan terdeteksi di video ini", 0);
      return false;
    }

    state.update({
      words,
      originalWords: resetWords(words),
      edited: false,
      generated: true});
    renderPreviewText();
    // Update libass-wasm saat transkripsi selesai
    renderAssForLibass(words).catch((err) =>
      console.warn("[main] Gagal render ASS ke libass:", err),
    );
    renderWordEditor();
    updatePreviewTag();
    updateAvailability();
    const durationSeconds = Math.round((elapsedMs ?? performance.now() - startedAt) / 1000);
    setProgress(`${words.length} kata terdeteksi dalam ${durationSeconds} detik`, 100);
    return true;
  } catch (error) {
    const message = errorMessage(error);
    if (
      error?.name === "AbortError" ||
      transcribeCancelRequested ||
      /dibatalkan/i.test(message)
    ) {
      setProgress("Dibatalkan", 0);
    } else {
      setProgress(`Gagal transkripsi: ${message}`, 0);
    }
    return false;
  } finally {
    state.update({ controller: null });
    transcribeCancelRequested = false;
    transcribeProgressStatus = null;
    updateAvailability();
  }
}

function formatWordTime(milliseconds) {
  const totalMs = Math.max(0, Math.floor(milliseconds));
  const minutes = Math.floor(totalMs / 60_000);
  const seconds = Math.floor((totalMs % 60_000) / 1000);
  const tenths = Math.floor((totalMs % 1000) / 100);
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}.${tenths}`;
}

function getEditorGroups(words = state.get().words) {
  const grouped = editorMode === "sentence"
    ? groupWords(words, "sentence")
    : Array.from({ length: Math.ceil(words.length / Number(editorMode)) }, (_, index) =>
        words.slice(index * Number(editorMode), (index + 1) * Number(editorMode)),
      );
  let nextStartIndex = 0;

  return grouped.map((group) => {
    const startIdx = words.indexOf(group[0], nextStartIndex);
    const endIdx = startIdx + group.length - 1;
    nextStartIndex = endIdx + 1;
    return {
      startIdx,
      endIdx,
      words: group,
      startMs: group[0].start_ms,
      endMs: group[group.length - 1].end_ms,
      text: group.map((word) => word.text).join(" "),
    };
  });
}

function scrollEditorRowIntoView(row) {
  if (!row) return;
  const listRect = wordEditorList.getBoundingClientRect();
  const rowRect = row.getBoundingClientRect();
  if (rowRect.top < listRect.top) {
    wordEditorList.scrollTop -= listRect.top - rowRect.top;
  } else if (rowRect.bottom > listRect.bottom) {
    wordEditorList.scrollTop += rowRect.bottom - listRect.bottom;
  }
}

function renderWordEditor({ wordIndex = null, focus = false, focusControl = null } = {}) {
  const words = state.get().words;
  const groups = getEditorGroups(words);
  wordEditorList.replaceChildren();
  editorNeedsRender = false;
  editorNeedsRenderWordIndex = null;

  groups.forEach((group, groupIndex) => {
    const row = document.createElement("div");
    row.className = "word-editor-row";
    row.dataset.editorRowIndex = String(groupIndex);
    row.dataset.startIndex = String(group.startIdx);
    row.setAttribute("role", "listitem");
    row.tabIndex = 0;

    const heading = document.createElement("div");
    heading.className = "word-editor-row-heading";

    const time = document.createElement("button");
    time.type = "button";
    time.className = "button button-secondary word-editor-time";
    time.textContent = formatWordTime(group.startMs);
    time.setAttribute("aria-label", `Lompat ke baris ${groupIndex + 1} pada ${time.textContent}`);
    time.dataset.editorControl = "time";
    time.addEventListener("click", () => {
      seekEditorGroup(group);
      if (editorNeedsRender) {
        renderWordEditor({ wordIndex: group.startIdx, focus: true, focusControl: "time" });
      }
    });

    const play = document.createElement("button");
    play.type = "button";
    play.className = "button button-secondary word-editor-play";
    play.textContent = "Putar baris";
    play.dataset.editorControl = "play";
    play.setAttribute("aria-label", `Putar baris ${groupIndex + 1}`);
    play.addEventListener("click", () => {
      playEditorGroup(group);
      if (editorNeedsRender) {
        renderWordEditor({ wordIndex: group.startIdx, focus: true, focusControl: "play" });
      }
    });
    heading.append(time, play);

    const input = document.createElement("textarea");
    input.className = "word-editor-input";
    input.rows = 1;
    input.value = group.text;
    input.dataset.originalText = group.text;
    input.dataset.startIndex = String(group.startIdx);
    input.dataset.endIndex = String(group.endIdx);
    input.setAttribute("aria-label", `Teks baris ${groupIndex + 1}`);
    input.addEventListener("input", () => resizeEditorTextarea(input));
    input.addEventListener("blur", (event) => {
      if (input.dataset.commitInProgress === "true") return;
      const result = commitEditorTextarea(input);
      if (result.changed) {
        if (event.relatedTarget?.matches?.(".word-editor-input")) {
          renderWordEditor({
            wordIndex: Number(event.relatedTarget.dataset.startIndex),
            focus: true,
          });
        } else if (!wordEditorList.contains(event.relatedTarget)) {
          renderWordEditor({ wordIndex: result.focusWordIndex });
        }
      }
    });
    input.addEventListener("keydown", (event) => {
      event.stopPropagation();
      if (event.key === "Escape") {
        event.preventDefault();
        input.value = input.dataset.originalText;
        resizeEditorTextarea(input);
        return;
      }
      if (event.key !== "Enter") return;
      event.preventDefault();
      if (event.ctrlKey || event.metaKey) {
        input.dataset.commitInProgress = "true";
        const result = commitEditorTextarea(input);
        renderWordEditor({ wordIndex: result.focusWordIndex, focus: true });
        const updatedGroup = findCurrentEditorGroup(group);
        if (updatedGroup) playEditorGroup(updatedGroup);
        return;
      }

      input.dataset.commitInProgress = "true";
      const result = commitEditorTextarea(input);
      const nextWordIndex = result.startIdx + result.replacementCount;
      const groups = getEditorGroups();
      const nextGroup = groups.find((candidate) =>
        nextWordIndex >= candidate.startIdx && nextWordIndex <= candidate.endIdx,
      ) ?? groups.find((candidate) => candidate.startIdx >= nextWordIndex);
      if (nextGroup) {
        renderWordEditor({ wordIndex: nextGroup.startIdx, focus: true });
        seekEditorGroup(nextGroup);
      } else {
        renderWordEditor({ wordIndex: result.focusWordIndex, focus: true });
      }
    });

    row.addEventListener("click", (event) => {
      if (event.target.closest("textarea, button")) return;
      seekEditorGroup(group);
    });
    row.append(heading, input);

    if (editorMode === "1") {
      const actions = document.createElement("div");
      actions.className = "word-editor-row-actions";
      if (group.startIdx < words.length - 1) {
        const merge = document.createElement("button");
        merge.type = "button";
        merge.className = "button button-secondary";
        merge.textContent = "Gabung dengan berikutnya";
        merge.dataset.editorControl = "merge";
        merge.addEventListener("click", () => performWordEditorAction("merge", group.startIdx));
        actions.append(merge);
      }

      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "button button-secondary";
      remove.textContent = "Hapus";
      remove.setAttribute("aria-label", `Hapus kata ${groupIndex + 1}`);
      remove.dataset.editorControl = "delete";
      remove.addEventListener("click", () => performWordEditorAction("delete", group.startIdx));
      actions.append(remove);
      row.append(actions);
    }

    wordEditorList.append(row);
    resizeEditorTextarea(input);
  });

  const targetGroup = wordIndex == null
    ? groups.findIndex((group) => state.get().currentMs >= group.startMs && state.get().currentMs <= group.endMs)
    : groups.findIndex((group) => wordIndex >= group.startIdx && wordIndex <= group.endIdx);
  const targetRow = wordEditorList.querySelector(
    `[data-editor-row-index="${targetGroup >= 0 ? targetGroup : 0}"]`,
  );
  if (focus && targetRow) {
    const focusTarget = targetRow.querySelector(
      focusControl ? `[data-editor-control="${focusControl}"]` : "textarea",
    );
    focusTarget?.focus();
  } else if (focus && !groups.length && state.get().originalWords) {
    byId("reset-original-words").focus();
  }
  scrollEditorRowIntoView(targetRow);
  updateEditorPlayback(state.get().currentMs);
  updateEditorAvailability(taskBusy(), Boolean(currentMediaElement()?.readyState >= 1));
}

function resizeEditorTextarea(textarea) {
  textarea.style.height = "auto";
  textarea.style.height = `${Math.max(36, textarea.scrollHeight)}px`;
}

function commitEditorTextarea(textarea) {
  if (taskBusy()) {
    return { changed: false, startIdx: 0, replacementCount: 0, focusWordIndex: 0 };
  }
  const words = state.get().words;
  const startIdx = Number(textarea.dataset.startIndex);
  const endIdx = Number(textarea.dataset.endIndex);
  if (!Number.isInteger(startIdx) || !Number.isInteger(endIdx) || !words[startIdx]) {
    return { changed: false, startIdx, replacementCount: 0, focusWordIndex: startIdx };
  }

  const updatedWords = applyGroupText(words, startIdx, endIdx, textarea.value);
  const replacementCount = updatedWords.length - (words.length - (endIdx - startIdx + 1));
  const changed =
    updatedWords.length !== words.length ||
    updatedWords.some((word, index) =>
      word.text !== words[index]?.text ||
      word.start_ms !== words[index]?.start_ms ||
      word.end_ms !== words[index]?.end_ms,
    );
  const focusWordIndex = Math.min(startIdx, Math.max(0, updatedWords.length - 1));
  if (!changed) {
    return { changed: false, startIdx, replacementCount, focusWordIndex };
  }

  state.update({ words: updatedWords, edited: true, generated: true });
  renderPreviewText();
  updateAvailability();
  editorNeedsRender = true;
  editorNeedsRenderWordIndex = focusWordIndex;
  return { changed: true, startIdx, replacementCount, focusWordIndex };
}

function performWordEditorAction(control, index) {
  if (taskBusy()) return;
  const words = state.get().words;
  if (!words[index]) return;

  const updatedWords = control === "merge"
    ? index + 1 < words.length
      ? mergeWithNext(words, index)
      : words
    : deleteWord(words, index);
  if (updatedWords === words) return;

  state.update({ words: updatedWords, edited: true, generated: true });
  renderPreviewText();
  updateAvailability();
  renderWordEditor({ wordIndex: Math.min(index, updatedWords.length - 1), focus: true });
}

function findCurrentEditorGroup(group) {
  const groups = getEditorGroups();
  return groups.find((candidate) => candidate.startIdx === group.startIdx) ??
    groups.find((candidate) => candidate.startIdx >= group.startIdx);
}

function seekEditorGroup(group) {
  if (taskBusy()) return;
  const currentGroup = findCurrentEditorGroup(group);
  if (!currentGroup) return;
  stopRowPlayback();
  const media = currentMediaElement();
  if (media?.readyState >= 1) {
    media.currentTime = currentGroup.startMs / 1000;
  }
  state.update({ currentMs: currentGroup.startMs });
  updateClock(currentGroup.startMs);
}

function playEditorGroup(group) {
  if (taskBusy()) return;
  const currentGroup = findCurrentEditorGroup(group);
  const media = currentMediaElement();
  if (!currentGroup || !media || media.readyState < 1) return;

  stopRowPlayback();
  const startMs = Math.max(0, currentGroup.startMs - 300);
  media.currentTime = startMs / 1000;
  rowPlayback = { media, endMs: currentGroup.endMs };
  state.update({ currentMs: startMs });
  updateClock(startMs);
  Promise.resolve(media.play()).catch((error) => {
    if (rowPlayback?.media !== media) return;
    stopRowPlayback();
    console.warn("Playback baris:", error);
    setProgress(`Tidak dapat memutar baris: ${errorMessage(error)}`, 0);
  });
}

function stopRowPlayback() {
  rowPlayback = null;
}

function updateEditorPlayback(milliseconds) {
  if (editorNeedsRender && !wordEditorList.contains(document.activeElement)) {
    renderWordEditor({ wordIndex: editorNeedsRenderWordIndex });
    return;
  }

  const groups = getEditorGroups();
  const activeIndex = groups.findIndex(
    (group) => milliseconds >= group.startMs && milliseconds <= group.endMs,
  );
  wordEditorList.querySelectorAll(".word-editor-row").forEach((row, index) => {
    row.classList.toggle("is-active", index === activeIndex);
  });

  if (
    activeIndex >= 0 &&
    activeIndex !== Number(wordEditorList.dataset.followedRowIndex) &&
    byId("editor-follow-playback").checked &&
    !document.activeElement?.matches(".word-editor-input")
  ) {
    scrollEditorRowIntoView(
      wordEditorList.querySelector(`[data-editor-row-index="${activeIndex}"]`),
    );
    wordEditorList.dataset.followedRowIndex = String(activeIndex);
  }
  wordEditorList.dataset.activeRowIndex = String(activeIndex);

  if (rowPlayback && rowPlayback.media === currentMediaElement() && milliseconds >= rowPlayback.endMs) {
    const media = rowPlayback.media;
    stopRowPlayback();
    media.pause();
  }
}

function updateEditorAvailability(busy, mediaReady) {
  const { words, originalWords, edited } = state.get();
  byId("word-editor-empty").hidden = words.length > 0;
  wordEditorList.hidden = words.length === 0;
  wordEditorList.tabIndex = words.length ? 0 : -1;
  byId("reset-original-words").disabled = busy || !originalWords || !edited;
  document.querySelectorAll(".editor-mode-button").forEach((button) => {
    button.disabled = busy;
  });
  wordEditorList.querySelectorAll(".word-editor-input").forEach((input) => {
    input.readOnly = busy;
  });
  wordEditorList.querySelectorAll("[data-editor-control]").forEach((button) => {
    const needsMedia = button.dataset.editorControl === "time" ||
      button.dataset.editorControl === "play";
    button.disabled = busy || (needsMedia && !mediaReady);
  });
}

function activateSettingsTab(tab, { focusList = false } = {}) {
  const isText = tab === "text";
  byId("settings-panel-style").hidden = isText;
  byId("settings-panel-text").hidden = !isText;
  byId("settings-tab-style").classList.toggle("is-active", !isText);
  byId("settings-tab-text").classList.toggle("is-active", isText);
  byId("settings-tab-style").setAttribute("aria-selected", String(!isText));
  byId("settings-tab-text").setAttribute("aria-selected", String(isText));
  byId("settings-panel").dataset.activeTab = tab;
  if (isText) {
    renderWordEditor();
    if (focusList && state.get().words.length) {
      wordEditorList.focus();
      updateEditorPlayback(state.get().currentMs);
    }
  }
}

function openWordEditor() {
  if (!state.get().words.length || taskBusy()) return;
  activateSettingsTab("text", { focusList: true });
}

function resetEditedWords() {
  const originalWords = state.get().originalWords;
  if (!originalWords || !state.get().edited || taskBusy()) return;
  if (!window.confirm("Kembalikan semua kata ke hasil transkripsi asli?")) return;
  state.update({ words: resetWords(originalWords), edited: false, generated: true });
  renderPreviewText();
  renderWordEditor();
  updateAvailability();
}

byId("settings-tab-style").addEventListener("click", () => activateSettingsTab("style"));
byId("settings-tab-text").addEventListener("click", () => activateSettingsTab("text"));
byId("reset-original-words").addEventListener("click", resetEditedWords);
byId("editor-follow-playback").addEventListener("change", () => {
  updateEditorPlayback(state.get().currentMs);
});
document.querySelectorAll(".editor-mode-button").forEach((button) => {
  button.addEventListener("click", () => {
    if (taskBusy()) return;
    editorMode = button.dataset.editorMode;
    document.querySelectorAll(".editor-mode-button").forEach((modeButton) => {
      const selected = modeButton === button;
      modeButton.classList.toggle("is-active", selected);
      modeButton.setAttribute("aria-pressed", String(selected));
    });
    renderWordEditor();
  });
});

async function exportSubtitles() {
  if (!state.get().generated || taskBusy()) return;
  if (!selectedPresetId) {
    setProgress("Pilih gaya caption terlebih dahulu.", 0);
    return;
  }
  const mediaName = state.get().media?.file?.name ?? "caption";
  const baseName = mediaName.replace(/\.[^.]+$/, "") || "caption";
  const format = byId("export-format").value;
  try {
    const path = await exportSubtitleFile({
      format,
      words: state.get().words,
      presetId: selectedPresetId,
      width: 1080,
      height: 1920,
      wordsPerLine: effectiveWordsPerLine(),
      captionPosition: captionPositions.get(selectedPresetId) ?? null,
      defaultPath: `${baseName}.${format}`,
    });
    if (!path) {
      setProgress(`Ekspor .${format} dibatalkan.`, 0);
      return;
    }
    setProgress(`File .${format} berhasil disimpan: ${path}`, 100);
  } catch (error) {
    setProgress(`Gagal mengekspor file .${format}: ${errorMessage(error)}`, 0);
  }
}

function onBurnProgress(progress) {
  burnProgressStatus = progress.status;
  if (progress.status === "running") {
    if (progress.indeterminate || progress.progress == null) {
      setIndeterminateProgress("Merender video: Memproses...");
    } else {
      const percent = Math.max(0, Math.min(100, progress.progress));
      setProgress(`Merender video: ${percent}%`, percent);
    }
  } else if (progress.status === "completed") {
    setProgress("Merender video: 100%", 100);
  } else if (progress.status === "cancelled") {
    setProgress("Dibatalkan", 0);
  }
  updateAvailability();
}

async function createVideo() {
  if (preparingVideo || burnActive || state.get().controller) return;
  const media = state.get().media;
  if (!media) {
    setProgress("Pilih media terlebih dahulu sebelum membuat video.", 0);
    return;
  }
  if (!window.__TAURI__ || typeof window.__TAURI__.dialog?.save !== "function") {
    setProgress("Buat Video hanya tersedia di aplikasi desktop.", 0);
    return;
  }
  if (media.kind !== "video") {
    setProgress("Buat Video memerlukan file video, bukan file audio.", 0);
    return;
  }
  if (!media.path) {
    setProgress("Pilih video melalui tombol Pilih File agar dapat diproses.", 0);
    return;
  }
  if (!selectedPresetId) {
    setProgress("Pilih gaya caption terlebih dahulu.", 0);
    return;
  }

  stopRowPlayback();
  preparingVideo = true;
  updateAvailability();
  try {
    if (!state.get().generated) {
      const generated = await generateSubtitles();
      if (!generated) return;
    }
    if (!state.get().words.length) {
      setProgress("Belum ada kata subtitle untuk dirender.", 0);
      return;
    }

    const baseName = media.file.name.replace(/\.[^.]+$/, "") || "caption";
    const output = await window.__TAURI__.dialog.save({
      defaultPath: `${baseName}.mp4`,
      filters: [{ name: "Video MP4", extensions: ["mp4"] }],
    });
    if (!output) {
      setProgress("Pembuatan video dibatalkan.", 0);
      return;
    }

    if (burnedPreviewActive) showSourcePreview();
    burnActive = true;
    burnCancelRequested = false;
    burnProgressStatus = "starting";
    updateAvailability();
    setIndeterminateProgress("Merender video: Memproses...");
    const savedPath = await burnVideo(
      {
        input: media.path,
        words: state.get().words,
        presetId: selectedPresetId,
        wordsPerLine: effectiveWordsPerLine(),
        captionPosition: captionPositions.get(selectedPresetId) ?? null,
        output,
        quality: "cepat",
      },
      onBurnProgress,
    );
    const previewLoaded = showBurnedPreview(savedPath);
    setProgress(
      previewLoaded
        ? `Video disimpan: ${savedPath}`
        : `Video disimpan: ${savedPath} (pratinjau tidak dapat dimuat)`,
      100,
    );
  } catch (error) {
    const message = errorMessage(error);
    if (burnCancelRequested || burnProgressStatus === "cancelled" || /dibatalkan/i.test(message)) {
      setProgress("Dibatalkan", 0);
    } else {
      setProgress(`Gagal membuat video: ${message}`, 0);
    }
  } finally {
    preparingVideo = false;
    burnActive = false;
    burnCancelRequested = false;
    burnProgressStatus = null;
    updateAvailability();
  }
}

async function cancelCurrentTask() {
  const controller = state.get().controller;
  if (controller) {
    if (transcribeCancelRequested) return;
    if (!isTauri()) {
      controller.abort();
      return;
    }
    transcribeCancelRequested = true;
    updateAvailability();
    setIndeterminateProgress("Membatalkan transkripsi...");
    try {
      await cancelTranscribe();
    } catch (error) {
      if (
        !state.get().controller ||
        ["completed", "failed", "cancelled"].includes(transcribeProgressStatus)
      ) {
        return;
      }
      transcribeCancelRequested = false;
      updateAvailability();
      setProgress(`Gagal membatalkan transkripsi: ${errorMessage(error)}`, 0);
    }
    return;
  }
  if (!burnActive) {
    return;
  }
  if (
    burnCancelRequested ||
    ["completed", "failed", "cancelled"].includes(burnProgressStatus)
  ) {
    return;
  }
  burnCancelRequested = true;
  updateAvailability();
  setIndeterminateProgress("Membatalkan render video...");
  try {
    await cancelBurn();
  } catch (error) {
    if (!burnActive || ["completed", "failed", "cancelled"].includes(burnProgressStatus)) {
      return;
    }
    burnCancelRequested = false;
    updateAvailability();
    setProgress(`Gagal membatalkan video: ${errorMessage(error)}`, 0);
  }
}

async function togglePlayback() {
  const element = currentMediaElement();
  if (element) {
    if (element.paused) {
      if (mediaPlayPending || element.readyState < 2 || element.seeking) return;
      mediaPlayPending = true;
      element.play()
        .catch((error) => console.warn("Playback:", error))
        .finally(() => {
          mediaPlayPending = false;
        });
    } else {
      if (mediaPlayPending) return;
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
  mediaElement.addEventListener("loadedmetadata", () => {
    if (mediaElement.tagName === "VIDEO") {
      updateStageAspectRatio(mediaElement);
      resizeLibassPreviewCanvas();
    }
    updateMediaDuration(mediaElement);
  });
  mediaElement.addEventListener("durationchange", () => updateMediaDuration(mediaElement));
  mediaElement.addEventListener("timeupdate", onMediaTimeUpdate);
  mediaElement.addEventListener("play", () => {
    state.update({ isPlaying: true });
    updatePlayButton(true);
    syncLibassPlaybackState(mediaElement);
  });
  mediaElement.addEventListener("pause", () => {
    if (rowPlayback?.media === mediaElement) stopRowPlayback();
    state.update({ isPlaying: false });
    updatePlayButton(false);
    if (mediaElement === currentMediaElement()) {
      stopLibassPlaybackSync();
      if (isLibassReady()) {
        setIsPaused(true, mediaElement.currentTime);
        syncLibassCurrentTime(mediaElement);
      }
    }
  });
  mediaElement.addEventListener("seeking", () => {
    syncLibassCurrentTime(mediaElement);
  });
  mediaElement.addEventListener("seeked", () => {
    syncLibassCurrentTime(mediaElement);
  });
  mediaElement.addEventListener("ratechange", () => {
    if (mediaElement !== currentMediaElement() || !isLibassReady()) return;
    setRate(mediaElement.playbackRate);
    syncLibassCurrentTime(mediaElement);
  });
  mediaElement.addEventListener("ended", () => {
    if (rowPlayback?.media === mediaElement) stopRowPlayback();
    state.update({ isPlaying: false });
    updatePlayButton(false);
    if (mediaElement === currentMediaElement()) {
      stopLibassPlaybackSync();
      if (isLibassReady()) {
        setIsPaused(true, mediaElement.currentTime);
        syncLibassCurrentTime(mediaElement);
      }
    }
  });
  mediaElement.addEventListener("error", () => {
    if (mediaElement === video && burnedPreviewActive) {
      setProgress(
        `Video disimpan, tetapi tidak dapat diputar: ${burnedVideoPath}`,
        100,
      );
      return;
    }
    byId("file-duration").textContent = "Durasi: tidak dapat dibaca";
    setProgress("Format media tidak dapat diputar oleh browser ini.", 0);
  });
}

byId("choose-file").addEventListener("click", chooseMedia);
byId("remove-file").addEventListener("click", clearMedia);
byId("play-pause").addEventListener("click", togglePlayback);
byId("seek-back-5").addEventListener("click", () => seekActiveMedia(-5));
byId("seek-forward-5").addEventListener("click", () => seekActiveMedia(5));
byId("frame-backward").addEventListener("click", () => stepVideoFrame(-1));
byId("frame-forward").addEventListener("click", () => stepVideoFrame(1));
byId("show-source-video").addEventListener("click", showSourcePreview);
byId("generate-subtitles").addEventListener("click", generateSubtitles);
byId("edit-text").addEventListener("click", openWordEditor);
byId("cancel-task").addEventListener("click", cancelCurrentTask);
byId("export-ass").addEventListener("click", exportSubtitles);
byId("create-video").addEventListener("click", createVideo);
byId("import-model").addEventListener("click", importSelectedModel);
byId("menu-import-model").addEventListener("click", importSelectedModel);
byId("guidance-import").addEventListener("click", importSelectedModel);
byId("model-select").addEventListener("change", () => {
  if (taskBusy()) return;
  clearGeneratedWords("Model berubah — buat subtitle kembali.");
});

byId("seek-slider").addEventListener("input", (event) => {
  stopRowPlayback();
  const currentMs = Math.round(Number(event.target.value) * 1000);
  const element = currentMediaElement();
  if (element) element.currentTime = currentMs / 1000;
  state.update({ currentMs });
  updateClock(currentMs);
});

previewStage.addEventListener("pointerdown", beginCaptionDrag);
previewStage.addEventListener("pointermove", moveCaptionDrag);
previewStage.addEventListener("pointerup", endCaptionDrag);
previewStage.addEventListener("pointercancel", endCaptionDrag);
previewStage.addEventListener("lostpointercapture", endCaptionDrag);
previewCanvas.addEventListener("libass-error", handleLibassError);

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
updatePreviewTag();
observePreviewStageResize();
byId("model-menu-status").textContent = isTauri()
  ? "Model Whisper disimpan secara lokal di aplikasi desktop."
  : "Pengelolaan model masih berupa simulasi.";
byId("settings-disclaimer").textContent = isTauri()
  ? "Transkripsi menggunakan model Whisper terpasang; overlay pratinjau adalah perkiraan CSS."
  : "Transkripsi dan pratinjau di halaman ini menggunakan data tiruan.";
updateClock(0);
loadModels();
loadPresets();

// Inisialisasi libass-wasm untuk preview (fallback ke CSS jika gagal)
initLibassPreview()
  .then(() => {
    console.log('[main] libass-wasm siap untuk preview');
    resizeLibassPreviewCanvas();
    const mediaElement = currentMediaElement();
    if (mediaElement) syncLibassPlaybackState(mediaElement);
  })
  .catch((err) => console.warn('[main] libass-wasm gagal, fallback ke CSS:', err));


