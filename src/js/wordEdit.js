function assertWordIndex(words, index) {
  if (!Array.isArray(words)) {
    throw new TypeError("Daftar kata harus berupa array.");
  }
  if (!Number.isInteger(index) || index < 0 || index >= words.length) {
    throw new RangeError("Indeks kata di luar daftar.");
  }
}

function copyWord(word) {
  return { ...word };
}

function splitParts(word, parts) {
  const start = word.start_ms;
  const end = word.end_ms;
  if (!Number.isFinite(start) || !Number.isFinite(end) || end < start) {
    throw new RangeError("Rentang waktu kata tidak valid.");
  }

  const weights = parts.map((part) => {
    const characters = Array.from(part);
    const letters = characters.filter((character) => /\p{L}/u.test(character)).length;
    return letters || characters.length;
  });
  const totalWeight = weights.reduce((sum, weight) => sum + weight, 0);
  const duration = end - start;
  let accumulatedWeight = 0;
  let partStart = start;

  return parts.map((text, index) => {
    accumulatedWeight += weights[index];
    const partEnd =
      index === parts.length - 1
        ? end
        : start + Math.floor((duration * accumulatedWeight) / totalWeight);
    const part = {
      ...word,
      text,
      start_ms: partStart,
      end_ms: partEnd,
    };
    partStart = partEnd;
    return part;
  });
}

export function splitWord(words, index, text) {
  assertWordIndex(words, index);
  if (typeof text !== "string") {
    throw new TypeError("Teks kata harus berupa string.");
  }
  const normalized = text.trim();
  if (!normalized) return deleteWord(words, index);

  const parts = normalized.split(/\s+/u);
  const replacements =
    parts.length === 1
      ? [{ ...words[index], text: parts[0] }]
      : splitParts(words[index], parts);
  return [...words.slice(0, index).map(copyWord), ...replacements, ...words.slice(index + 1).map(copyWord)];
}

export function editWord(words, index, text) {
  return splitWord(words, index, text);
}

export function mergeWithNext(words, index) {
  assertWordIndex(words, index);
  if (index + 1 >= words.length) {
    throw new RangeError("Kata terakhir tidak memiliki kata berikutnya.");
  }
  const first = words[index];
  const next = words[index + 1];
  const merged = {
    ...first,
    text: `${first.text}${next.text}`,
    end_ms: next.end_ms,
  };
  return [
    ...words.slice(0, index).map(copyWord),
    merged,
    ...words.slice(index + 2).map(copyWord),
  ];
}

export function deleteWord(words, index) {
  assertWordIndex(words, index);
  return [...words.slice(0, index).map(copyWord), ...words.slice(index + 1).map(copyWord)];
}

export function resetWords(originalWords) {
  if (!Array.isArray(originalWords)) {
    throw new TypeError("Salinan kata asli tidak valid.");
  }
  return originalWords.map(copyWord);
}
