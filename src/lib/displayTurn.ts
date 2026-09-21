import type { ViewportOrientation } from "./cameraOrientation";

export function turnForDisplay(
  frameWidth: number,
  frameHeight: number,
  phone?: ViewportOrientation,
): 0 | 90 {
  if (!phone || !frameWidth || !frameHeight) return 0;
  const frameLandscape = frameWidth >= frameHeight;
  const phoneLandscape = phone === "landscape";
  return frameLandscape === phoneLandscape ? 0 : 90;
}

export function bindDisplayTurn(
  el: HTMLVideoElement | null,
  phone?: ViewportOrientation,
): () => void {
  if (!el) return () => undefined;
  const sync = () => {
    el.dataset.turn = String(turnForDisplay(el.videoWidth, el.videoHeight, phone));
  };
  el.addEventListener("loadedmetadata", sync);
  el.addEventListener("resize", sync);
  sync();
  return () => {
    el.removeEventListener("loadedmetadata", sync);
    el.removeEventListener("resize", sync);
  };
}
