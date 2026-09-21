import { describe, expect, it } from "vitest";
import {
  isLandscapeViewport,
  trackMatchesOrientation,
  videoConstraintsForViewport,
  viewportOrientation,
} from "./cameraOrientation";

function fakeWindow(width: number, height: number, orientation?: "landscape" | "portrait") {
  return {
    innerWidth: width,
    innerHeight: height,
    matchMedia: (query: string) => ({
      matches:
        orientation === "landscape"
          ? query.includes("landscape")
          : orientation === "portrait"
            ? query.includes("portrait") && !query.includes("landscape")
            : query.includes("landscape")
              ? width > height
              : width <= height,
      media: query,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    }),
  } as unknown as Pick<Window, "matchMedia" | "innerWidth" | "innerHeight">;
}

describe("camera viewport orientation", () => {
  it("detects landscape from matchMedia", () => {
    expect(viewportOrientation(fakeWindow(844, 390, "landscape"))).toBe("landscape");
    expect(isLandscapeViewport(fakeWindow(390, 844, "portrait"))).toBe(false);
  });

  it("prefers window.orientation on iOS after a rotation round-trip", () => {
    const win = fakeWindow(390, 844, "portrait") as typeof window & { orientation?: number };
    win.orientation = 90;
    expect(viewportOrientation(win)).toBe("landscape");
  });

  it("requests landscape-sized capture in landscape and portrait-sized capture in portrait", () => {
    const landscape = videoConstraintsForViewport("", "landscape");
    const portrait = videoConstraintsForViewport("cam-1", "portrait");
    expect(landscape.width).toEqual({ ideal: 1920 });
    expect(landscape.height).toEqual({ ideal: 1080 });
    expect(landscape.facingMode).toEqual({ ideal: "environment" });
    expect(portrait.width).toEqual({ ideal: 1080 });
    expect(portrait.height).toEqual({ ideal: 1920 });
    expect(portrait.deviceId).toEqual({ exact: "cam-1" });
  });

  it("compares track size to viewport orientation", () => {
    expect(trackMatchesOrientation({ width: 1920, height: 1080 }, "landscape")).toBe(true);
    expect(trackMatchesOrientation({ width: 1080, height: 1920 }, "landscape")).toBe(false);
    expect(trackMatchesOrientation({ width: 1080, height: 1920 }, "portrait")).toBe(true);
  });
});
