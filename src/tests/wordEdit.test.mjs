import assert from "node:assert/strict";
import test from "node:test";

import {
  deleteWord,
  editWord,
  mergeWithNext,
  resetWords,
  splitWord,
} from "../js/wordEdit.js";

const words = [
  { text: "Halo", start_ms: 100, end_ms: 1100 },
  { text: "dunia!", start_ms: 1200, end_ms: 1800 },
  { text: "lagi", start_ms: 1900, end_ms: 2400 },
];

test("mengedit teks memangkas spasi tepi tanpa mengubah masukan", () => {
  assert.deepEqual(editWord(words, 0, "  Hai!  "), [
    { ...words[0], text: "Hai!" },
    words[1],
    words[2],
  ]);
  assert.equal(words[0].text, "Halo");
});

test("menggabungkan dengan kata berikutnya tanpa spasi", () => {
  assert.deepEqual(mergeWithNext(words, 0), [
    { text: "Halodunia!", start_ms: 100, end_ms: 1800 },
    words[2],
  ]);
});

test("menghapus kata menghasilkan daftar baru", () => {
  assert.deepEqual(deleteWord(words, 1), [words[0], words[2]]);
  assert.equal(words.length, 3);
});

test("teks dengan spasi dipecah proporsional tanpa celah atau tumpang tindih", () => {
  const result = splitWord(words, 0, "a bcd");

  assert.deepEqual(
    result.slice(0, 2),
    [
      { ...words[0], text: "a", start_ms: 100, end_ms: 350 },
      { ...words[0], text: "bcd", start_ms: 350, end_ms: 1100 },
    ],
  );
  assert.equal(result[0].start_ms, words[0].start_ms);
  assert.equal(result[1].end_ms, words[0].end_ms);
  assert.equal(result[0].end_ms, result[1].start_ms);
  assert.ok(result[1].end_ms <= result[2].start_ms);
});

test("teks kosong pada edit menghapus kata", () => {
  assert.deepEqual(editWord(words, 1, " \t "), [words[0], words[2]]);
});

test("mengembalikan hasil asli dengan salinan objek baru", () => {
  const restored = resetWords(words);

  assert.deepEqual(restored, words);
  assert.notEqual(restored, words);
  assert.notEqual(restored[0], words[0]);
});

test("menolak gabung kata terakhir dan indeks yang tidak valid", () => {
  assert.throws(() => mergeWithNext(words, 2), RangeError);
  assert.throws(() => deleteWord(words, 3), RangeError);
});
