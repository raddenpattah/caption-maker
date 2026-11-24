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

function findMatchingTokens(oldTokens, newTokens) {
  const rows = oldTokens.length + 1;
  const columns = newTokens.length + 1;
  const lcs = Array.from({ length: rows }, () => new Uint16Array(columns));

  for (let oldIndex = oldTokens.length - 1; oldIndex >= 0; oldIndex -= 1) {
    for (let newIndex = newTokens.length - 1; newIndex >= 0; newIndex -= 1) {
      lcs[oldIndex][newIndex] =
        oldTokens[oldIndex] === newTokens[newIndex]
          ? lcs[oldIndex + 1][newIndex + 1] + 1
          : Math.max(lcs[oldIndex + 1][newIndex], lcs[oldIndex][newIndex + 1]);
    }
  }

  const matches = [];
  let oldIndex = 0;
  let newIndex = 0;
  while (oldIndex < oldTokens.length && newIndex < newTokens.length) {
    if (oldTokens[oldIndex] === newTokens[newIndex]) {
      matches.push({ oldIndex, newIndex });
      oldIndex += 1;
      newIndex += 1;
    } else if (lcs[oldIndex + 1][newIndex] >= lcs[oldIndex][newIndex + 1]) {
      oldIndex += 1;
    } else {
      newIndex += 1;
    }
  }
  return matches;
}

function splitInsertedTokens(tokens, leftWord, rightWord, fallbackWord) {
  const template = leftWord ?? rightWord ?? fallbackWord;
  const start = leftWord?.end_ms ?? rightWord?.start_ms ?? fallbackWord.start_ms;
  const end = rightWord?.start_ms ?? start;
  if (end < start) {
    throw new RangeError("Tidak ada rentang waktu kosong untuk menyisipkan kata.");
  }
  return splitParts({ ...template, start_ms: start, end_ms: end }, tokens);
}

function assertNonOverlapping(words) {
  for (let index = 0; index < words.length; index += 1) {
    const word = words[index];
    if (
      !Number.isFinite(word.start_ms) ||
      !Number.isFinite(word.end_ms) ||
      word.start_ms > word.end_ms ||
      (index > 0 &&
        (words[index - 1].start_ms > word.start_ms ||
          words[index - 1].end_ms > word.start_ms))
    ) {
      throw new RangeError("Hasil edit menghasilkan rentang kata yang tidak berurutan.");
    }
  }
}

export function applyGroupText(words, startIdx, endIdx, newText) {
  if (!Array.isArray(words)) {
    throw new TypeError("Daftar kata harus berupa array.");
  }
  if (
    !Number.isInteger(startIdx) ||
    !Number.isInteger(endIdx) ||
    startIdx < 0 ||
    endIdx < startIdx ||
    endIdx >= words.length
  ) {
    throw new RangeError("Rentang kelompok kata tidak valid.");
  }
  if (typeof newText !== "string") {
    throw new TypeError("Teks kelompok harus berupa string.");
  }

  const oldWords = words.slice(startIdx, endIdx + 1);
  const oldTokens = oldWords.map((word) => word.text);
  const newTokens = newText.trim().split(/\s+/u).filter(Boolean);
  if (!newTokens.length) {
    const result = [
      ...words.slice(0, startIdx).map(copyWord),
      ...words.slice(endIdx + 1).map(copyWord),
    ];
    assertNonOverlapping(result);
    return result;
  }

  const matches = findMatchingTokens(oldTokens, newTokens);
  const replacements = [];
  let oldCursor = 0;
  let newCursor = 0;
  let leftMatch = null;

  const appendEditBlock = (oldEnd, newEnd, rightMatch) => {
    const oldCount = oldEnd - oldCursor;
    const newBlock = newTokens.slice(newCursor, newEnd);
    if (oldCount && newBlock.length) {
      const firstOld = oldWords[oldCursor];
      const lastOld = oldWords[oldEnd - 1];
      replacements.push(
        ...splitParts(
          { ...firstOld, start_ms: firstOld.start_ms, end_ms: lastOld.end_ms },
          newBlock,
        ),
      );
    } else if (newBlock.length) {
      const leftWord = leftMatch ? oldWords[leftMatch.oldIndex] : null;
      const rightWord = rightMatch ? oldWords[rightMatch.oldIndex] : null;
      replacements.push(...splitInsertedTokens(newBlock, leftWord, rightWord, oldWords[0]));
    }
  };

  for (const match of matches) {
    appendEditBlock(match.oldIndex, match.newIndex, match);
    replacements.push(copyWord(oldWords[match.oldIndex]));
    oldCursor = match.oldIndex + 1;
    newCursor = match.newIndex + 1;
    leftMatch = match;
  }
  appendEditBlock(oldWords.length, newTokens.length, null);

  const result = [
    ...words.slice(0, startIdx).map(copyWord),
    ...replacements,
    ...words.slice(endIdx + 1).map(copyWord),
  ];
  assertNonOverlapping(result);
  return result;
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
