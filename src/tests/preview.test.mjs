import assert from "node:assert/strict";
import test from "node:test";

import { getVideoRect, groupWords, loadPreviewFont } from "../js/preview.js";

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

test("menghitung rect video portrait di stage landscape", () => {
  assert.deepEqual(
    getVideoRect(
      { clientWidth: 1600, clientHeight: 900 },
      { videoWidth: 1080, videoHeight: 1920 },
    ),
    { left: 546.875, top: 0, width: 506.25, height: 900, scale: 0.46875 },
  );
});

test("menghitung rect video landscape", () => {
  assert.deepEqual(
    getVideoRect(
      { clientWidth: 1200, clientHeight: 900 },
      { videoWidth: 1920, videoHeight: 1080 },
    ),
    { left: 0, top: 112.5, width: 1200, height: 675, scale: 0.625 },
  );
});

test("memakai ukuran referensi portrait jika videoWidth nol", () => {
  assert.deepEqual(
    getVideoRect(
      { clientWidth: 1600, clientHeight: 900 },
      { videoWidth: 0, videoHeight: 0 },
    ),
    { left: 546.875, top: 0, width: 506.25, height: 900, scale: 0.46875 },
  );
});

test("memuat preset font dan memeriksa semua family sebelum menyelesaikan", async () => {
  const originalDocument = globalThis.document;
  const originalLog = console.log;
  const loaded = [];
  const checked = [];
  const logs = [];
  globalThis.document = {
    fonts: {
      load: async (descriptor) => {
        loaded.push(descriptor);
        return [{}];
      },
      check: (descriptor) => {
        checked.push(descriptor);
        return true;
      },
    },
  };
  console.log = (...args) => logs.push(args);

  try {
    await loadPreviewFont({ font: { family: "Poppins ExtraBold", size: 42 } });

    const descriptors = [
      '42px "Poppins ExtraBold"',
      '42px "Poppins"',
      '42px "Anton"',
      '42px "Bangers"',
    ];
    assert.deepEqual(loaded, descriptors);
    assert.deepEqual(checked, descriptors);
    assert.deepEqual(logs, [["Preview font checks", {
      "Poppins ExtraBold": true,
      Poppins: true,
      Anton: true,
      Bangers: true,
    }]]);
  } finally {
    console.log = originalLog;
    if (originalDocument === undefined) {
      delete globalThis.document;
    } else {
      globalThis.document = originalDocument;
    }
  }
});
