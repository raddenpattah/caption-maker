// tauri-bridge.js — jembatan ke API Tauri via window.__TAURI__ (withGlobalTauri).
// Tidak ada import bare module; aman dijalankan tanpa bundler.

export function isTauri() {
  return typeof window !== "undefined" && !!window.__TAURI__;
}

function requireTauri() {
  if (!isTauri()) {
    throw new Error("Bukan lingkungan Tauri: window.__TAURI__ tidak tersedia.");
  }
}

export async function invoke(cmd, args = {}) {
  requireTauri();
  return window.__TAURI__.core.invoke(cmd, args);
}

export async function listen(event, handler) {
  requireTauri();
  return window.__TAURI__.event.listen(event, handler);
}
