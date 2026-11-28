import assert from "node:assert/strict";
import test from "node:test";

import {
  buildLibassFontOptions,
  clampCaptionPosition,
  computePreviewMetrics,
  computePopScale,
  getCaptionPosition,
  getLibassCanvasSize,
  getVideoRect,
  groupWords,
  loadPreviewFont,
  moveCaptionPosition,
} from "../js/preview.js";

test("menghitung skala animasi pop sesuai keyframe", () => {
  assert.equal(computePopScale(0, 300), 0.85);
  assert.equal(computePopScale(45, 300), 0.965);
  assert.equal(computePopScale(90, 300), 1.08);
  assert.equal(computePopScale(160, 300), 1);
  assert.equal(computePopScale(500, 300), 1);
  assert.equal(computePopScale(-10, 300), 0.85);
  assert.equal(computePopScale(0, 30), 1);
});

test("menginterpolasi animasi pop untuk durasi 100 ms", () => {
  const expected = 0.85 + ((1.08 - 0.85) * 50) / 56;
  assert.ok(Math.abs(computePopScale(50, 100) - expected) < 1e-12);
});

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

test("menghitung bitmap canvas untuk rect portrait", () => {
  assert.deepEqual(
    getLibassCanvasSize({ width: 506.25, height: 900 }, 1),
    { width: 506, height: 900 },
  );
});

test("menghitung bitmap canvas untuk rect landscape", () => {
  assert.deepEqual(
    getLibassCanvasSize({ width: 1200, height: 675 }, 1),
    { width: 1200, height: 675 },
  );
});

test("menghitung bitmap canvas dengan DPR 2", () => {
  assert.deepEqual(
    getLibassCanvasSize({ width: 506.25, height: 900 }, 2),
    { width: 1013, height: 1800 },
  );
});

test("memetakan setiap family libass ke file font yang sesuai", () => {
  assert.deepEqual(
    buildLibassFontOptions(new URL("https://example.test/assets/fonts/")),
    {
      fonts: [
        "https://example.test/assets/fonts/Poppins-ExtraBold.ttf",
        "https://example.test/assets/fonts/Poppins-Bold.ttf",
        "https://example.test/assets/fonts/Anton-Regular.ttf",
        "https://example.test/assets/fonts/Bangers-Regular.ttf",
      ],
      availableFonts: {
        "poppins extrabold": "https://example.test/assets/fonts/Poppins-ExtraBold.ttf",
        "poppins": "https://example.test/assets/fonts/Poppins-Bold.ttf",
        "anton": "https://example.test/assets/fonts/Anton-Regular.ttf",
        "bangers": "https://example.test/assets/fonts/Bangers-Regular.ttf",
      },
    },
  );
});

test("menghitung anchor tengah caption relatif terhadap videoRect", () => {
  assert.deepEqual(
    getCaptionPosition(
      { left: 100, top: 50, width: 400, height: 200 },
      { left: 250, top: 125, width: 100, height: 50 },
    ),
    { x: 50, y: 50 },
  );
});

test("membatasi anchor agar seluruh blok dan efeknya berada di area video", () => {
  const position = clampCaptionPosition(
    { x: -10, y: 110 },
    { left: 0, top: 0, width: 400, height: 200 },
    { left: 0, top: 0, width: 100, height: 40 },
    5,
  );

  assert.ok(Math.abs(position.x - 13.75) < 0.0001);
  assert.equal(position.y, 87.5);
});

test("mengubah drag pixel menjadi persen dan membatasi posisi akhir", () => {
  assert.deepEqual(
    moveCaptionPosition(
      { x: 50, y: 50 },
      200,
      -120,
      { left: 0, top: 0, width: 400, height: 200 },
      { left: 0, top: 0, width: 100, height: 40 },
      5,
    ),
    { x: 86.25, y: 12.5 },
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

const metricPreset = {
  font: { size: 72, letter_spacing: 2 },
  outline: 6,
  shadow: 3,
  box: { padding: 18, padding_vertical: 8 },
  margin_v: 450,
  margin_lr: 80,
  glow: { radius: 12 },
};

test("menghitung preview metrics pada geometri 1080x1920", () => {
  const metrics = computePreviewMetrics({
    rect: { scale: 0.5, height: 960 },
    videoWidth: 1080,
    videoHeight: 1920,
    preset: metricPreset,
  });

  assert.equal(metrics.assScale, 1);
  assert.equal(metrics.previewScale, 0.5);
  assert.equal(metrics.fontSize, 36);
  assert.equal(metrics.outline, 3);
  assert.equal(metrics.shadow, 1.5);
  assert.equal(metrics.letterSpacing, 1);
  assert.equal(metrics.boxPadding, 9);
  assert.equal(metrics.boxPaddingVertical, 4);
  assert.equal(metrics.marginLr, 40);
  assert.equal(metrics.marginV, 225);
  assert.equal(metrics.glowRadius, 6);
});

test("memakai ukuran referensi portrait untuk mode audio", () => {
  const metrics = computePreviewMetrics({
    rect: { scale: 0.5, height: 960 },
    videoWidth: 0,
    videoHeight: 0,
    preset: metricPreset,
  });

  assert.equal(metrics.assScale, 1);
  assert.equal(metrics.previewScale, 0.5);
  assert.equal(metrics.marginV, 225);
});

test("menghitung assScale 0.6667 pada video 720x1280", () => {
  const metrics = computePreviewMetrics({
    rect: { scale: 0.5, height: 640 },
    videoWidth: 720,
    videoHeight: 1280,
    preset: metricPreset,
  });

  assert.ok(Math.abs(metrics.assScale - 0.6667) < 0.0001);
  assert.ok(Math.abs(metrics.previewScale - (0.5 * (720 / 1080))) < 0.0001);
  assert.ok(Math.abs(metrics.marginV - 150) < 0.0001);
});

test("marginV landscape memakai 8 persen tinggi rect", () => {
  const metrics = computePreviewMetrics({
    rect: { scale: 0.5, height: 540 },
    videoWidth: 1920,
    videoHeight: 1080,
    preset: metricPreset,
  });

  assert.equal(metrics.assScale, 1);
  assert.equal(metrics.marginV, 540 * 0.08);
});

test("video persegi memakai aturan marginV landscape", () => {
  const metrics = computePreviewMetrics({
    rect: { scale: 0.5, height: 400 },
    videoWidth: 1080,
    videoHeight: 1080,
    preset: metricPreset,
  });

  assert.equal(metrics.assScale, 1);
  assert.equal(metrics.marginV, 400 * 0.08);
});
