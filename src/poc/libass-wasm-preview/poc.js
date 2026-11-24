import { invoke, isTauri } from "../../js/tauri-bridge.js";

const canvas = document.getElementById("ass-canvas");
const renderButton = document.getElementById("render-ass");
const renderTime = document.getElementById("render-time");
const renderTimeLabel = document.getElementById("render-time-label");
const renderStatus = document.getElementById("render-status");
const workerUrl = new URL(
  "./vendor/jellyfin-4.2.4/subtitles-octopus-worker.js",
  import.meta.url,
).href;
const fontUrl = new URL("../../assets/fonts/Poppins-ExtraBold.ttf", import.meta.url).href;

let renderer = null;
let rendererReady = false;

function updateRenderTime() {
  const seconds = Number(renderTime.value);
  renderTimeLabel.value = `${seconds.toFixed(2)} s`;
  if (rendererReady) renderer.setCurrentTime(seconds);
}

async function loadAssContent() {
  if (isTauri()) {
    const result = await invoke("render_ass_from_words", {
      req: {
        words: [
          { text: "LIBASS", start_ms: 200, end_ms: 1300 },
          { text: "WASM", start_ms: 1400, end_ms: 2800 },
        ],
        preset_id: "kata-tebal-putih",
        width: canvas.width,
        height: canvas.height,
        words_per_line: 1,
        caption_position: null,
      },
    });
    if (!result?.ass) {
      throw new Error("Backend tidak mengembalikan konten ASS.");
    }
    return { ass: result.ass, source: "backend Caption Maker" };
  }

  const response = await fetch("./fixture.ass");
  if (!response.ok) {
    throw new Error(`Fixture ASS gagal dimuat (HTTP ${response.status}).`);
  }
  return { ass: await response.text(), source: "fixture lokal (bukan backend Tauri)" };
}

function disposeRenderer() {
  rendererReady = false;
  if (renderer) {
    renderer.dispose();
    renderer = null;
  }
}

async function renderAss() {
  renderButton.disabled = true;
  renderStatus.textContent = isTauri()
    ? "Meminta ASS dari backend dan memulai worker WASM..."
    : "Memuat fixture ASS lokal dan memulai worker WASM...";
  disposeRenderer();

  try {
    const { ass, source } = await loadAssContent();
    const SubtitlesOctopus = window.SubtitlesOctopus;
    if (typeof SubtitlesOctopus !== "function") {
      throw new Error("Entry point SubtitlesOctopus tidak tersedia.");
    }

    await new Promise((resolve, reject) => {
      let timeoutId;
      try {
        renderer = new SubtitlesOctopus({
          canvas,
          subContent: ass,
          workerUrl,
          fonts: [fontUrl],
          debug: true,
          onReady: () => {
            window.clearTimeout(timeoutId);
            rendererReady = true;
            renderer.setCurrentTime(Number(renderTime.value));
            renderStatus.textContent = `Renderer siap; ASS dimuat dari ${source}.`;
            resolve();
          },
          onError: (error) => {
            window.clearTimeout(timeoutId);
            reject(error instanceof Error ? error : new Error(String(error)));
          },
        });
        timeoutId = window.setTimeout(
          () => reject(new Error("Worker belum siap setelah 15 detik.")),
          15000,
        );
      } catch (error) {
        window.clearTimeout(timeoutId);
        reject(error);
      }
    });
  } catch (error) {
    console.error("POC libass WASM gagal:", error);
    disposeRenderer();
    renderStatus.textContent = `Gagal: ${error.message}`;
  } finally {
    renderButton.disabled = false;
  }
}

renderButton.addEventListener("click", renderAss);
renderTime.addEventListener("input", updateRenderTime);
window.addEventListener("beforeunload", disposeRenderer, { once: true });
updateRenderTime();
