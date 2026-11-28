const previewFontFamilies = ["Poppins ExtraBold", "Poppins", "Anton", "Bangers"];
const pendingFontLoads = new Map();
let fontChecksPromise = null;
let fontChecksLogged = false;

function loadFontDescriptor(descriptor) {
  const pending = pendingFontLoads.get(descriptor);
  if (pending) return pending;

  const loading = Promise.resolve()
    .then(() => document.fonts.load(descriptor))
    .finally(() => pendingFontLoads.delete(descriptor));
  pendingFontLoads.set(descriptor, loading);
  return loading;
}

export function loadPreviewFont(preset) {
  const family = preset?.font?.family;
  const size = Number(preset?.font?.size);
  if (!family || !Number.isFinite(size) || size <= 0) return Promise.resolve();

  return loadFontDescriptor(`${size}px "${family}"`).then(() => {
    if (fontChecksLogged) return;
    if (!fontChecksPromise) {
      const descriptors = previewFontFamilies
        .filter((name) => name !== family)
        .map((name) => `${size}px "${name}"`);
      fontChecksPromise = Promise.all(descriptors.map(loadFontDescriptor))
        .then(() => {
          const availability = Object.fromEntries(
            previewFontFamilies.map((name) => [
              name,
              document.fonts.check(`${size}px "${name}"`),
            ]),
          );
          console.log("Preview font checks", availability);
          fontChecksLogged = true;
        })
        .catch((error) => {
          fontChecksPromise = null;
          throw error;
        });
    }
    return fontChecksPromise;
  });
}

export function getVideoRect(stage, video) {
  const stageWidth = stage.clientWidth;
  const stageHeight = stage.clientHeight;
  const hasVideoDimensions = video?.videoWidth > 0 && video?.videoHeight > 0;
  const videoWidth = hasVideoDimensions ? video.videoWidth : 1080;
  const videoHeight = hasVideoDimensions ? video.videoHeight : 1920;
  const scale = Math.min(stageWidth / videoWidth, stageHeight / videoHeight);
  const width = videoWidth * scale;
  const height = videoHeight * scale;

  return {
    left: (stageWidth - width) / 2,
    top: (stageHeight - height) / 2,
    width,
    height,
    scale,
  };
}

export const POP_START_SCALE = 85;
export const POP_OVERSHOOT_SCALE = 108;
export const POP_UP_MS = 90;
export const POP_TOTAL_MS = 160;
export const POP_MIN_DURATION_MS = 40;

// samakan dengan POP_* di preset_render.rs
export function computePopScale(elapsedMs, durationMs) {
  if (!Number.isFinite(elapsedMs) || !Number.isFinite(durationMs)) return 1;
  if (durationMs < POP_MIN_DURATION_MS) return 1;

  const totalMs = Math.min(POP_TOTAL_MS, durationMs);
  const upMs = Math.round((totalMs * POP_UP_MS) / POP_TOTAL_MS);

  if (elapsedMs <= 0) return POP_START_SCALE / 100;
  if (elapsedMs >= totalMs) return 1;
  if (elapsedMs <= upMs) {
    return (
      POP_START_SCALE +
      ((POP_OVERSHOOT_SCALE - POP_START_SCALE) * elapsedMs) / upMs
    ) / 100;
  }
  return (
    POP_OVERSHOOT_SCALE -
    ((POP_OVERSHOOT_SCALE - 100) * (elapsedMs - upMs)) / (totalMs - upMs)
  ) / 100;
}

export function getLibassCanvasSize(rect, devicePixelRatio = 1) {
  const dpr = Number.isFinite(devicePixelRatio) && devicePixelRatio > 0
    ? Math.min(devicePixelRatio, 2)
    : 1;

  return {
    width: Math.round(rect.width * dpr),
    height: Math.round(rect.height * dpr),
  };
}

export function buildLibassFontOptions(baseUrl) {
  const fontEntries = [
    ["poppins extrabold", "Poppins-ExtraBold.ttf"],
    ["poppins", "Poppins-Bold.ttf"],
    ["anton", "Anton-Regular.ttf"],
    ["bangers", "Bangers-Regular.ttf"],
  ];
  const fonts = [];
  const availableFonts = {};

  for (const [family, filename] of fontEntries) {
    const url = new URL(filename, baseUrl).href;
    fonts.push(url);
    availableFonts[family] = url;
  }

  return { fonts, availableFonts };
}

function validateCaptionRect(videoRect, blockRect) {
  if (
    !Number.isFinite(videoRect?.left) ||
    !Number.isFinite(videoRect?.top) ||
    !Number.isFinite(videoRect?.width) ||
    !Number.isFinite(videoRect?.height) ||
    videoRect.width <= 0 ||
    videoRect.height <= 0 ||
    !Number.isFinite(blockRect?.width) ||
    !Number.isFinite(blockRect?.height) ||
    !Number.isFinite(blockRect?.left) ||
    !Number.isFinite(blockRect?.top) ||
    blockRect.width < 0 ||
    blockRect.height < 0
  ) {
    throw new RangeError("Ukuran area video atau blok caption tidak valid.");
  }
}

export function getCaptionPosition(videoRect, blockRect) {
  validateCaptionRect(videoRect, blockRect);
  return {
    x: ((blockRect.left + blockRect.width / 2 - videoRect.left) / videoRect.width) * 100,
    y: ((blockRect.top + blockRect.height / 2 - videoRect.top) / videoRect.height) * 100,
  };
}

export function getCaptionVisualOutset(metrics) {
  return Math.max(metrics.outline, metrics.shadow * 2, metrics.glowRadius * 2);
}

export function clampCaptionPosition(position, videoRect, blockRect, outset = 0) {
  validateCaptionRect(videoRect, blockRect);
  if (
    !Number.isFinite(position?.x) ||
    !Number.isFinite(position?.y) ||
    !Number.isFinite(outset) ||
    outset < 0
  ) {
    throw new RangeError("Posisi atau batas visual caption tidak valid.");
  }

  const minX = ((blockRect.width / 2 + outset) / videoRect.width) * 100;
  const minY = ((blockRect.height / 2 + outset) / videoRect.height) * 100;
  const maxX = 100 - minX;
  const maxY = 100 - minY;
  return {
    x: minX > maxX ? 50 : Math.max(minX, Math.min(maxX, position.x)),
    y: minY > maxY ? 50 : Math.max(minY, Math.min(maxY, position.y)),
  };
}

export function moveCaptionPosition(position, deltaX, deltaY, videoRect, blockRect, outset = 0) {
  validateCaptionRect(videoRect, blockRect);
  if (!Number.isFinite(deltaX) || !Number.isFinite(deltaY)) {
    throw new RangeError("Perpindahan caption tidak valid.");
  }
  return clampCaptionPosition(
    {
      x: position.x + (deltaX / videoRect.width) * 100,
      y: position.y + (deltaY / videoRect.height) * 100,
    },
    videoRect,
    blockRect,
    outset,
  );
}

export function applyCaptionPosition(content, position) {
  content.style.position = "absolute";
  content.style.left = `${position.x}%`;
  content.style.top = `${position.y}%`;
  content.style.transform = "translate(-50%, -50%)";
  content.style.margin = "0";
}

export function computePreviewMetrics({ rect, videoWidth, videoHeight, preset, settings = {} }) {
  const hasVideoDimensions =
    Number.isFinite(videoWidth) &&
    videoWidth > 0 &&
    Number.isFinite(videoHeight) &&
    videoHeight > 0;
  const vw = hasVideoDimensions ? videoWidth : 1080;
  const vh = hasVideoDimensions ? videoHeight : 1920;
  const assScale = Math.min(vw, vh) / 1080;
  const previewScale = rect.scale * assScale;

  const rawFontSize = Number(preset.font?.size ?? settings.fontSize);
  const fontSize = Number.isFinite(rawFontSize) && rawFontSize > 0 ? rawFontSize : 20;
  const rawOutline = Number(preset.outline ?? settings.outline);
  const outline = Number.isFinite(rawOutline) && rawOutline >= 0 ? rawOutline : 0;
  const rawShadow = Number(preset.shadow ?? settings.shadow);
  const shadow = Number.isFinite(rawShadow) && rawShadow > 0 ? rawShadow * previewScale : 0;

  return {
    assScale,
    previewScale,
    fontSize: fontSize * previewScale,
    outline: outline * previewScale,
    shadow,
    letterSpacing: Number(preset.font?.letter_spacing ?? 0) * previewScale,
    boxPadding: Number(preset.box?.padding ?? 0) * previewScale,
    marginLr: Number(preset.margin_lr ?? 0) * previewScale,
    marginV:
      vh > vw
        ? Number(preset.margin_v ?? 0) * previewScale
        : rect.height * 0.08,
    glowRadius: Number(preset.glow?.radius ?? 0) * previewScale,
  };
}

export function groupWords(words, n) {
  if (![1, 2, 3, "sentence"].includes(n)) {
    throw new RangeError("Jumlah kata per baris harus 1, 2, 3, atau sentence.");
  }

  if (n === "sentence") {
    const groups = [];
    let current = [];
    words.forEach((word, index) => {
      current.push(word);
      const endsWithPunctuation = /[.?!]$/.test(word.text.trimEnd());
      const next = words[index + 1];
      const hasLargeGap = next && next.start_ms - word.end_ms >= 700;
      if (endsWithPunctuation || hasLargeGap || current.length >= 8) {
        groups.push(current);
        current = [];
      }
    });
    if (current.length > 0) groups.push(current);
    return groups;
  }

  const groups = [];
  for (let index = 0; index < words.length; index += n) {
    groups.push(words.slice(index, index + n));
  }
  return groups;
}

export function renderPreview(container, words, currentMs, settings) {
  const preset = settings.preset;
  if (!preset) return false;

  const font = preset.font;
  const textColor = preset.colors?.text ?? settings.textColor;
  const highlightColor = preset.colors?.highlight ?? settings.highlightColor;
  const outlineColor = preset.colors?.outline ?? "#000000";
  const shadowColor = preset.colors?.shadow ?? "#000000";
  const glow = preset.glow;
  const emphasis = preset.emphasis;
  const hasBox = Boolean(preset.box);

  const stage = document.getElementById('preview-stage');
  const video = document.getElementById('video-player');
  const activeVideo = video && !video.hidden ? video : null;
  const videoRect = getVideoRect(stage, activeVideo);
  const metrics = computePreviewMetrics({
    rect: videoRect,
    videoWidth: activeVideo?.videoWidth,
    videoHeight: activeVideo?.videoHeight,
    preset,
    settings,
  });

  // --- Ukuran dan posisi area video ---
  container.style.position = "absolute";
  container.style.left = `${videoRect.left}px`;
  container.style.top = `${videoRect.top}px`;
  container.style.width = `${videoRect.width}px`;
  container.style.height = `${videoRect.height}px`;
  container.style.right = "auto";
  container.style.bottom = "auto";
  container.style.transform = "none";
  container.style.boxSizing = "border-box";
  container.style.display = "flex";
  container.style.justifyContent = "center";
  container.style.alignItems = "flex-end";
  container.style.textAlign = "center";
  container.style.padding = `0 ${metrics.marginLr}px ${metrics.marginV}px`;
  container.style.pointerEvents = "none";
  container.style.zIndex = "10";

  // --- Font & Warna Dasar ---
  container.style.fontFamily = font ? `"${font.family}", Arial, sans-serif` : settings.font;
  container.style.fontSize = `${metrics.fontSize}px`;
  container.style.fontWeight = font ? (font.bold ? "700" : "400") : "";
  container.style.fontStyle = font ? (font.italic ? "italic" : "normal") : "";
  container.style.letterSpacing = font ? `${metrics.letterSpacing}px` : "";
  container.style.color = preset.colors?.text ?? textColor;
  container.style.textTransform = font?.uppercase ? "uppercase" : "none";

  // --- Outline & Glow (Hanya jika tidak ada kotak) ---
  if (!hasBox) {
    container.style.webkitTextStroke = `${metrics.outline * 2}px ${preset.colors?.outline ?? outlineColor}`;
    container.style.paintOrder = "stroke fill";
    if (glow) { container.style.textShadow = `0 0 ${metrics.glowRadius}px ${glow.color}`; } else if (metrics.shadow > 0) {
      container.style.textShadow = `${metrics.shadow}px ${metrics.shadow}px 0 ${shadowColor}`;
    } else {
      container.style.textShadow = "none";
    }
  } else {
    container.style.webkitTextStroke = "0px";
    container.style.paintOrder = "normal";
    container.style.textShadow = "none";
  }

  // --- Cari Grup Kata Aktif ---
  const wpl = preset.words_per_line || 1;
  let activeGroup = null;

  if (wpl === 1) {
    // Mode kata per kata
    activeGroup = words.find(w => currentMs >= w.start_ms && currentMs <= w.end_ms);
  } else {
    // Mode baris: cari grup yang mencakup currentMs
    for (let i = 0; i < words.length; i += wpl) {
      const group = words.slice(i, i + wpl);
      if (group.length > 0 && currentMs >= group[0].start_ms && currentMs <= group[group.length - 1].end_ms) {
        activeGroup = group;
        break;
      }
    }
  }

  if (!activeGroup) {
    container.innerHTML = '';
    return false;
  }

  const displayWords = Array.isArray(activeGroup) ? activeGroup : [activeGroup];
  const popScale =
    preset.animation === "pop" &&
    emphasis !== "scale_active" &&
    displayWords.length === 1
      ? computePopScale(
          currentMs - displayWords[0].start_ms,
          displayWords[0].end_ms - displayWords[0].start_ms,
        )
      : null;

  // --- Buat Konten (Kotak atau Teks Biasa) ---
  const content = document.createElement("span");
  content.className = "subtitle-content";
  content.style.display = "inline-block";
  content.style.position = "relative";
  content.style.zIndex = "2";
  if (popScale !== null) {
    content.style.transform = `scale(${popScale})`;
    content.style.transformOrigin = "50% 100%";
  }
  content.classList.add("is-draggable");
  content.style.setProperty(
    "max-width",
    `${Math.max(0, videoRect.width - metrics.marginLr * 2)}px`,
    "important",
  );

  if (hasBox) {
    content.style.backgroundColor = preset.box.color;
    content.style.padding = `${metrics.boxPadding}px`;
    content.style.borderRadius = "0";
  }

  container.innerHTML = '';
  container.appendChild(content);

  // --- Render Setiap Kata ---
  displayWords.forEach((word, index) => {
    if (index > 0) content.append(document.createTextNode(" "));
    const span = document.createElement("span");
    span.className = "karaoke-word";

    let wordText = word.text;
    if (font?.uppercase) wordText = wordText.toUpperCase();
    span.textContent = wordText;

    span.style.display = "inline-block";
    span.style.position = "relative";
    span.style.zIndex = "3";

    const isActive = currentMs >= word.start_ms && currentMs <= word.end_ms;

    // --- Logika Emphasis (Penekanan) ---
    if (isActive) {
      if (emphasis === 'fill') {
        span.style.color = preset.colors?.highlight ?? highlightColor;
        span.style.webkitTextFillColor = highlightColor;
      } else if (emphasis === 'scale_active') {
        const scaleFactor = (preset.emphasis_scale || 118) / 100;
        span.style.transform = `scale(${scaleFactor})`;
        span.style.color = preset.colors?.highlight ?? highlightColor;
        span.style.webkitTextFillColor = highlightColor;
      } else if (emphasis === 'underline_active') {
        span.style.textDecoration = "underline";
        span.style.textDecorationColor = highlightColor;
        span.style.color = preset.colors?.highlight ?? highlightColor;
        span.style.webkitTextFillColor = highlightColor;
      } else if (emphasis === 'color_active') {
        span.style.color = preset.colors?.highlight ?? highlightColor;
        span.style.webkitTextFillColor = highlightColor;
      } else {
        span.style.color = textColor;
        span.style.webkitTextFillColor = textColor;
      }
    } else {
      span.style.color = textColor;
      span.style.webkitTextFillColor = textColor;
    }

    content.appendChild(span);
  });

  if (settings.captionPosition) {
    const position = clampCaptionPosition(
      settings.captionPosition,
      { left: 0, top: 0, width: videoRect.width, height: videoRect.height },
      content.getBoundingClientRect(),
      getCaptionVisualOutset(metrics),
    );
    applyCaptionPosition(content, position);
    if (popScale !== null) {
      content.style.transform = `translate(-50%, -50%) scale(${popScale})`;
    }
  }

  return true;
}




// ============================================================
// libass-wasm Preview Renderer
// ============================================================

let libassRenderer = null;
let libassReady = false;
let libassInitPromise = null;
let currentAssContent = null;

function deactivateLibassPreview() {
  libassReady = false;
  const canvas = document.getElementById("preview-canvas");
  if (!canvas) return;
  canvas.classList.remove("is-active");
  canvas.dispatchEvent(new Event("libass-error"));
}

export function initLibassPreview(canvasId = "preview-canvas") {
  if (libassInitPromise) return libassInitPromise;

  libassInitPromise = new Promise((resolve, reject) => {
    try {
      const canvas = document.getElementById(canvasId);
      if (!canvas) throw new Error("Canvas tidak ditemukan: " + canvasId);

      if (typeof window.SubtitlesOctopus !== "function") {
        throw new Error("SubtitlesOctopus belum dimuat. Cek <script> libass-wasm di index.html.");
      }

      const workerUrl = new URL("../vendor/libass-wasm/subtitles-octopus-worker.js", import.meta.url).href;
      const legacyWorkerUrl = new URL("../vendor/libass-wasm/subtitles-octopus-worker-legacy.js", import.meta.url).href;
      const fontsBaseUrl = new URL("../assets/fonts/", import.meta.url);
      const { fonts, availableFonts } = buildLibassFontOptions(fontsBaseUrl);
      const emptyAssTrack = `[Script Info]
ScriptType: v4.00+
PlayResX: 1080
PlayResY: 1920

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Poppins ExtraBold,104,&H00FFFFFF,&H00FFFFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,8,4,2,80,80,450,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text`;

      libassRenderer = new window.SubtitlesOctopus({
        canvas,
        workerUrl,
        legacyWorkerUrl,
        subContent: emptyAssTrack,
        fonts,
        availableFonts,
        onReady: () => {
          try {
            libassReady = true;
            if (currentAssContent) {
              libassRenderer.setTrack(currentAssContent);
            }

            const stage = document.getElementById("preview-stage");
            const video = document.getElementById("video-player");
            if (
              stage &&
              !resizeLibassCanvas(getVideoRect(stage, video && !video.hidden ? video : null))
            ) {
              reject(new Error("Gagal mengubah ukuran canvas libass-wasm."));
              return;
            }

            canvas.classList.add("is-active");
            console.log("libass-wasm siap.");
            resolve(libassRenderer);
          } catch (error) {
            deactivateLibassPreview();
            console.error("Gagal menyiapkan preview libass-wasm:", error);
            reject(error);
          }
        },
        onError: (event) => {
          console.error("libass-wasm error:", {
            type: event?.type,
            message: event?.message,
            filename: event?.filename,
            lineno: event?.lineno,
            colno: event?.colno,
            stack: event?.error?.stack,
          });
          deactivateLibassPreview();
          reject(event instanceof Error ? event : new Error(event?.message || "Worker libass-wasm gagal."));
        },
      });
    } catch (error) {
      console.error("Gagal inisialisasi libass-wasm:", error);
      deactivateLibassPreview();
      reject(error);
    }
  });

  return libassInitPromise;
}

export function isLibassReady() {
  return libassReady;
}

export function setAssContent(assText) {
  currentAssContent = assText;
  if (!libassRenderer || !libassReady) return;
  try {
    libassRenderer.setTrack(assText);
  } catch (err) {
    deactivateLibassPreview();
    console.error("Gagal setSubtitles:", err);
  }
}

export function setCurrentTime(seconds) {
  if (!libassRenderer || !libassReady) return;
  try {
    libassRenderer.setCurrentTime(seconds);
  } catch (error) {
    deactivateLibassPreview();
    console.error("Gagal menyinkronkan waktu libass-wasm:", error);
  }
}

export function setIsPaused(isPaused, currentTime) {
  if (!libassRenderer || !libassReady || typeof libassRenderer.setIsPaused !== "function") return;
  try {
    libassRenderer.setIsPaused(isPaused, currentTime);
  } catch (error) {
    deactivateLibassPreview();
    console.error("Gagal mengubah status pause libass-wasm:", error);
  }
}

export function setRate(rate) {
  if (!libassRenderer || !libassReady || typeof libassRenderer.setRate !== "function") return;
  try {
    libassRenderer.setRate(rate);
  } catch (error) {
    deactivateLibassPreview();
    console.error("Gagal mengubah playback rate libass-wasm:", error);
  }
}

export function resizeLibassCanvas(rect) {
  const canvas = document.getElementById("preview-canvas");
  if (
    !canvas ||
    !Number.isFinite(rect?.left) ||
    !Number.isFinite(rect?.top) ||
    !Number.isFinite(rect?.width) ||
    !Number.isFinite(rect?.height) ||
    rect.width <= 0 ||
    rect.height <= 0
  ) {
    return true;
  }

  const { width, height } = getLibassCanvasSize(rect, window.devicePixelRatio || 1);
  if (width <= 0 || height <= 0) return true;

  canvas.style.left = `${rect.left}px`;
  canvas.style.top = `${rect.top}px`;
  canvas.style.width = `${rect.width}px`;
  canvas.style.height = `${rect.height}px`;

  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }

  if (libassRenderer && libassReady && typeof libassRenderer.resize === "function") {
    try {
      libassRenderer.resize(width, height, rect.top, rect.left);
    } catch (error) {
      deactivateLibassPreview();
      console.error("Gagal mengubah ukuran canvas libass-wasm:", error);
      return false;
    }
  }
  return true;
}

export function disposeLibass() {
  if (libassRenderer && libassRenderer.dispose) {
    try { libassRenderer.dispose(); } catch (e) {}
  }
  libassRenderer = null;
  libassReady = false;
  libassInitPromise = null;
  currentAssContent = null;
}