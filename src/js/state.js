export const defaultSettings = {
  wordsPerLine: 2,
  highlight: true,
  font: "Arial, sans-serif",
  fontSize: 48,
  textColor: "#ffffff",
  highlightColor: "#ffdc64",
  position: "bottom",
  outline: 3,
  shadow: 2,
};

export function createState() {
  let current = {
    words: [],
    originalWords: null,
    edited: false,
    media: null,
    generated: false,
    isPlaying: false,
    currentMs: 0,
    durationMs: 60_000,
    controller: null,
    settings: { ...defaultSettings },
  };

  const listeners = new Set();
  return {
    get() {
      return current;
    },
    update(patch) {
      current = { ...current, ...patch };
      for (const listener of listeners) {
        listener(current);
      }
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
}
