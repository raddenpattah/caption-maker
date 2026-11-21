export function groupWords(words, n) {
  if (![1, 2, 3].includes(n)) {
    throw new RangeError("Jumlah kata per baris harus 1, 2, atau 3.");
  }

  const groups = [];
  for (let index = 0; index < words.length; index += n) {
    groups.push(words.slice(index, index + n));
  }
  return groups;
}

export function renderPreview(container, words, currentMs, settings) {
  container.dataset.position = settings.position;
  container.style.fontFamily = settings.font;
  container.style.fontSize = `${settings.fontSize}px`;
  container.style.color = settings.textColor;
  container.style.webkitTextStroke = `${settings.outline}px #000`;
  container.style.textShadow = settings.shadow
    ? `0 ${settings.shadow}px ${settings.shadow * 2}px #000`
    : "none";
  container.replaceChildren();

  const activeGroup = groupWords(words, settings.wordsPerLine).find((group) => {
    const first = group[0];
    const last = group[group.length - 1];
    return currentMs >= first.start_ms && currentMs <= last.end_ms;
  });
  if (!activeGroup) return false;

  activeGroup.forEach((word, index) => {
    if (index > 0) container.append(document.createTextNode(" "));
    const span = document.createElement("span");
    span.className = "karaoke-word";
    span.textContent = word.text;

    if (settings.highlight) {
      const duration = Math.max(1, word.end_ms - word.start_ms);
      const progress = Math.max(0, Math.min(1, (currentMs - word.start_ms) / duration));
      const percent = progress * 100;
      span.style.backgroundImage = `linear-gradient(to right, ${settings.highlightColor} ${percent}%, ${settings.textColor} ${percent}%)`;
      span.style.backgroundClip = "text";
      span.style.webkitBackgroundClip = "text";
      span.style.webkitTextFillColor = "transparent";
    } else {
      span.style.color = settings.textColor;
    }

    container.append(span);
  });
  return true;
}
