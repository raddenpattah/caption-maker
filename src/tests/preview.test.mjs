import assert from "node:assert/strict";
import test from "node:test";

import { groupWords } from "../js/preview.js";

const words = ["satu", "dua", "tiga", "empat", "lima"].map((text, index) => ({
  text,
  start_ms: index * 100,
  end_ms: (index + 1) * 100,
}));

test("mengelompokkan satu kata per baris", () => {
  assert.deepEqual(
    groupWords(words, 1).map((group) => group.map((word) => word.text)),
    [["satu"], ["dua"], ["tiga"], ["empat"], ["lima"]],
  );
});

test("mengelompokkan dua kata per baris", () => {
  assert.deepEqual(
    groupWords(words, 2).map((group) => group.map((word) => word.text)),
    [["satu", "dua"], ["tiga", "empat"], ["lima"]],
  );
});

test("mengelompokkan tiga kata per baris", () => {
  assert.deepEqual(
    groupWords(words, 3).map((group) => group.map((word) => word.text)),
    [["satu", "dua", "tiga"], ["empat", "lima"]],
  );
});

test("mengembalikan array kosong untuk masukan kosong", () => {
  assert.deepEqual(groupWords([], 2), []);
});

test("mempertahankan urutan kata dan tidak mengubah masukan", () => {
  const original = structuredClone(words);
  const groups = groupWords(words, 2);

  assert.deepEqual(groups.flat(), words);
  assert.deepEqual(words, original);
});

test("menolak jumlah kata per baris di luar 1 sampai 3", () => {
  assert.throws(() => groupWords(words, 4), RangeError);
});
