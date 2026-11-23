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
  const scale = videoRect.scale;
  const marginV = (preset.margin_v ?? 0) * scale;
  const marginLr = (preset.margin_lr ?? 0) * scale;

  const rawFontSize = Number(font?.size ?? settings.fontSize);
  const baseFontSize = Number.isFinite(rawFontSize) && rawFontSize > 0 ? rawFontSize : 20;
  const scaledFontSize = Math.max(12, baseFontSize * scale);

  console.log("renderPreview geometry", {
    videoWidth: activeVideo?.videoWidth ?? 0,
    videoHeight: activeVideo?.videoHeight ?? 0,
    stageClientWidth: stage.clientWidth,
    stageClientHeight: stage.clientHeight,
    scale,
    scaledFontSize,
  });

  const rawOutline = Number(preset.outline ?? settings.outline);
  const validOutline = Number.isFinite(rawOutline) && rawOutline >= 0 ? rawOutline : 0;
  const scaledOutline = Math.max(0, validOutline * scale);

  const shadow = preset.shadow ?? settings.shadow;

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
  container.style.padding = `0 ${marginLr}px ${marginV}px`;
  container.style.pointerEvents = "none";
  container.style.zIndex = "10";

  // --- Font & Warna Dasar ---
  container.style.fontFamily = font ? `"${font.family}", Arial, sans-serif` : settings.font;
  container.style.fontSize = `${scaledFontSize}px`;
  container.style.fontWeight = font ? (font.bold ? "700" : "400") : "";
  container.style.fontStyle = font ? (font.italic ? "italic" : "normal") : "";
  container.style.letterSpacing = font ? `${(font.letter_spacing || 0) * scale}px` : "";
  container.style.color = preset.colors?.text ?? textColor;
  container.style.textTransform = font?.uppercase ? "uppercase" : "none";

  // --- Outline & Glow (Hanya jika tidak ada kotak) ---
  if (!hasBox) {
    container.style.webkitTextStroke = `${scaledOutline}px ${preset.colors?.outline ?? outlineColor}`;
    if (glow) { container.style.textShadow = `0 0 ${Math.max(4, glow.radius * scale)}px ${glow.color}`; } else if (shadow > 0) {
      container.style.textShadow = `0 ${shadow * scale}px ${shadow * 2 * scale}px ${shadowColor}`;
    } else {
      container.style.textShadow = "none";
    }
  } else {
    container.style.webkitTextStroke = "0px";
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

  // --- Buat Konten (Kotak atau Teks Biasa) ---
  const content = document.createElement("span");
  content.className = "subtitle-content";
  content.style.display = "inline-block";
  content.style.position = "relative";
  content.style.zIndex = "2";

  if (hasBox) {
    content.style.backgroundColor = preset.box.color;
    content.style.padding = `${(preset.box.padding ?? 0) * scale}px`;
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

  return true;
}



