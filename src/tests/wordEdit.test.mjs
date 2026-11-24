import assert from "node:assert/strict";
import test from "node:test";

import {
  applyGroupText,
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

const groupedWords = [
  { text: "satu", start_ms: 100, end_ms: 300 },
  { text: "dua", start_ms: 400, end_ms: 700 },
  { text: "tiga", start_ms: 800, end_ms: 1100 },
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

test("mengganti satu kata dengan kata lain tanpa mengubah waktunya", () => {
  assert.deepEqual(applyGroupText(groupedWords, 0, 0, "baru"), [
    { ...groupedWords[0], text: "baru" },
    ...groupedWords.slice(1),
  ]);
});

test("mempertahankan waktu token yang sama saat jumlah kata tidak berubah", () => {
  const result = applyGroupText(groupedWords, 0, 2, "satu dua baru");

  assert.deepEqual(result, [
    groupedWords[0],
    groupedWords[1],
    { ...groupedWords[2], text: "baru" },
  ]);
});

test("menyisipkan kata di tengah dengan membagi jeda tetangga", () => {
  const result = applyGroupText(groupedWords, 0, 2, "satu tambah dua tiga");

  assert.deepEqual(result[0], groupedWords[0]);
  assert.deepEqual(result[1], {
    ...groupedWords[0],
    text: "tambah",
    start_ms: 300,
    end_ms: 400,
  });
  assert.deepEqual(result.slice(2), groupedWords.slice(1));
});

test("menyisipkan kata di ujung kelompok tanpa mengubah waktu token yang sama", () => {
  const result = applyGroupText(groupedWords, 0, 2, "awal satu dua tiga akhir");

  assert.equal(result[0].text, "awal");
  assert.equal(result[0].start_ms, groupedWords[0].start_ms);
  assert.equal(result[0].end_ms, groupedWords[0].start_ms);
  assert.deepEqual(result.slice(1, 4), groupedWords);
  assert.equal(result[4].text, "akhir");
  assert.equal(result[4].start_ms, groupedWords[2].end_ms);
  assert.equal(result[4].end_ms, groupedWords[2].end_ms);
});

test("menghapus kata tengah atau ujung tanpa menggeser waktu kata yang tersisa", () => {
  assert.deepEqual(applyGroupText(groupedWords, 0, 2, "satu tiga"), [
    groupedWords[0],
    groupedWords[2],
  ]);
  assert.deepEqual(applyGroupText(groupedWords, 0, 2, "satu dua"), groupedWords.slice(0, 2));
});

test("mengganti beberapa kata dengan satu kata dalam rentang waktu yang sama", () => {
  const result = applyGroupText(groupedWords, 0, 1, "gabungan");

  assert.deepEqual(result[0], {
    ...groupedWords[0],
    text: "gabungan",
    start_ms: 100,
    end_ms: 700,
  });
  assert.deepEqual(result[1], groupedWords[2]);
});

test("mengganti satu kata dengan beberapa kata secara proporsional", () => {
  const result = applyGroupText(groupedWords, 0, 0, "a bcd");

  assert.deepEqual(result.slice(0, 2), [
    { ...groupedWords[0], text: "a", start_ms: 100, end_ms: 150 },
    { ...groupedWords[0], text: "bcd", start_ms: 150, end_ms: 300 },
  ]);
});

test("membagi waktu pengganti menurut jumlah huruf Unicode", () => {
  const result = applyGroupText(groupedWords, 0, 0, "猫 abc");

  assert.deepEqual(result.slice(0, 2), [
    { ...groupedWords[0], text: "猫", start_ms: 100, end_ms: 150 },
    { ...groupedWords[0], text: "abc", start_ms: 150, end_ms: 300 },
  ]);
});

test("teks kelompok kosong menghapus semua kata kelompok", () => {
  assert.deepEqual(applyGroupText(groupedWords, 1, 2, " \t "), [groupedWords[0]]);
});

test("tokenisasi mengabaikan spasi ganda dan spasi tepi", () => {
  assert.deepEqual(
    applyGroupText(groupedWords, 0, 1, "  satu   kata\tbaru  "),
    [
      groupedWords[0],
      { ...groupedWords[1], text: "kata", start_ms: 400, end_ms: 550 },
      { ...groupedWords[1], text: "baru", start_ms: 550, end_ms: 700 },
      groupedWords[2],
    ],
  );
});

test("rentang kelompok pengganti mempertahankan waktu total dan tidak tumpang tindih", () => {
  const result = applyGroupText(groupedWords, 0, 2, "alpha beta gamma delta");

  assert.equal(result[0].start_ms, groupedWords[0].start_ms);
  assert.equal(result[3].end_ms, groupedWords[2].end_ms);
  assert.ok(result.every((word, index) =>
    word.start_ms <= word.end_ms &&
    (index === 0 || result[index - 1].end_ms <= word.start_ms),
  ));
});

test("applyGroupText tidak memutasi daftar kata masukan", () => {
  const original = structuredClone(groupedWords);

  applyGroupText(groupedWords, 0, 2, "satu kata baru tiga");

  assert.deepEqual(groupedWords, original);
});

test("menolak gabung kata terakhir dan indeks yang tidak valid", () => {
  assert.throws(() => mergeWithNext(words, 2), RangeError);
  assert.throws(() => deleteWord(words, 3), RangeError);
});
