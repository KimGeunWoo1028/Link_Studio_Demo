export type ViewportOrientation = "portrait" | "landscape";

export function isLandscapeViewport(
  win: Pick<Window, "matchMedia" | "innerWidth" | "innerHeight"> & { orientation?: number } = window,
): boolean {
  const angled = win.orientation;
  if (angled === 90 || angled === -90) return true;
  if (angled === 0 || angled === 180) return false;
  if (typeof win.matchMedia === "function") {
    try {
      if (win.matchMedia("(orientation: landscape)").matches) return true;
      if (win.matchMedia("(orientation: portrait)").matches) return false;
    } catch {
      /* jsdom or incomplete matchMedia */
    }
  }
  return win.innerWidth > win.innerHeight;
}

export function viewportOrientation(
  win: Pick<Window, "matchMedia" | "innerWidth" | "innerHeight"> & { orientation?: number } = window,
): ViewportOrientation {
  return isLandscapeViewport(win) ? "landscape" : "portrait";
}

export function videoConstraintsForViewport(
  deviceId: string,
  orientation: ViewportOrientation,
): MediaTrackConstraints {
  const sized: MediaTrackConstraints =
    orientation === "landscape"
      ? { width: { ideal: 1920 }, height: { ideal: 1080 } }
      : { width: { ideal: 1080 }, height: { ideal: 1920 } };
  if (deviceId) {
    return { ...sized, deviceId: { exact: deviceId } };
  }
  return { ...sized, facingMode: { ideal: "environment" } };
}

export function trackMatchesOrientation(
  settings: Pick<MediaTrackSettings, "width" | "height">,
  orientation: ViewportOrientation,
): boolean {
  const width = settings.width ?? 0;
  const height = settings.height ?? 0;
  if (!width || !height) return false;
  const landscape = width >= height;
  return orientation === "landscape" ? landscape : !landscape;
}
