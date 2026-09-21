import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router";
import { CameraPage } from "./CameraPage";

function renderCamera(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/camera/:sessionId" element={<CameraPage />} />
        <Route path="/camera" element={<CameraPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

function stubOrientation(orientation: "portrait" | "landscape") {
  Object.defineProperty(window, "innerWidth", {
    configurable: true,
    value: orientation === "landscape" ? 844 : 390,
  });
  Object.defineProperty(window, "innerHeight", {
    configurable: true,
    value: orientation === "landscape" ? 390 : 844,
  });
  window.matchMedia = ((query: string) => ({
    matches: query.includes(orientation),
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  })) as typeof window.matchMedia;
}

describe("CameraPage route states", () => {
  beforeEach(() => {
    stubOrientation("portrait");
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("renders an explicit invalid session state without a session id", () => {
    renderCamera("/camera");
    expect(screen.getByTestId("camera-state")).toHaveAttribute("data-state", "invalid-session");
    expect(screen.getByRole("heading", { name: "Invalid session" })).toBeTruthy();
  });

  it("renders session not found instead of a blank screen", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(JSON.stringify({ error: "Session not found." }), { status: 404 })),
    );
    Object.defineProperty(window.navigator, "mediaDevices", {
      configurable: true,
      value: { getUserMedia: vi.fn(), enumerateDevices: vi.fn() },
    });
    Object.defineProperty(window, "isSecureContext", { configurable: true, value: true });

    renderCamera("/camera/missing-session");
    await waitFor(() => {
      expect(screen.getByTestId("camera-state")).toHaveAttribute("data-state", "not-found");
    });
    expect(screen.getByRole("heading", { name: "Session not found" })).toBeTruthy();
  });

  it("renders the camera controls for a valid session", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(
          JSON.stringify({ sessionId: "sess-ok", projectId: "proj-ok" }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      ),
    );
    Object.defineProperty(window.navigator, "mediaDevices", {
      configurable: true,
      value: { getUserMedia: vi.fn(), enumerateDevices: vi.fn() },
    });
    Object.defineProperty(window, "isSecureContext", { configurable: true, value: true });

    renderCamera("/camera/sess-ok");
    await waitFor(() => {
      expect(screen.getByTestId("camera-state")).toHaveAttribute("data-state", "ready");
    });
    expect(screen.getByRole("heading", { name: "Link Studio Camera" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Start Camera" })).toBeTruthy();
  });

  it("uses camera names from the session lookup", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(
          JSON.stringify({
            sessionId: "sess-ok",
            projectId: "proj-ok",
            cameras: [
              { role: "cam1", name: "Stage left" },
              { role: "cam2", name: "Stage right" },
            ],
          }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      ),
    );
    Object.defineProperty(window.navigator, "mediaDevices", {
      configurable: true,
      value: { getUserMedia: vi.fn(), enumerateDevices: vi.fn() },
    });
    Object.defineProperty(window, "isSecureContext", { configurable: true, value: true });

    renderCamera("/camera/sess-ok");
    await waitFor(() => {
      expect(screen.getByTestId("camera-state")).toHaveAttribute("data-state", "ready");
    });
    expect(screen.getByRole("option", { name: "Stage left" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Stage right" })).toBeTruthy();
  });

  it("marks landscape orientation on the camera page", async () => {
    stubOrientation("landscape");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(
          JSON.stringify({ sessionId: "sess-ok", projectId: "proj-ok" }),
          { status: 200, headers: { "Content-Type": "application/json" } },
        ),
      ),
    );
    Object.defineProperty(window.navigator, "mediaDevices", {
      configurable: true,
      value: { getUserMedia: vi.fn(), enumerateDevices: vi.fn() },
    });
    Object.defineProperty(window, "isSecureContext", { configurable: true, value: true });

    renderCamera("/camera/sess-ok");
    await waitFor(() => {
      expect(screen.getByTestId("camera-state")).toHaveAttribute("data-state", "ready");
    });
    expect(screen.getByTestId("camera-state")).toHaveAttribute("data-orientation", "landscape");
  });

  it("keeps the working capture track when the phone rotates", async () => {
    stubOrientation("landscape");
    const getUserMedia = vi.fn(async () => {
      const video = {
        kind: "video",
        stop: vi.fn(),
        getSettings: () => ({ width: 1920, height: 1080 }),
        applyConstraints: vi.fn(async () => undefined),
      };
      const audio = { kind: "audio", stop: vi.fn(), getSettings: () => ({}) };
      return {
        getTracks: () => [video, audio],
        getVideoTracks: () => [video],
        getAudioTracks: () => [audio],
        addTrack: vi.fn(),
        removeTrack: vi.fn(),
      };
    });
    vi.stubGlobal(
      "fetch",
      vi.fn(async () =>
        new Response(JSON.stringify({ sessionId: "sess-ok", projectId: "proj-ok" }), { status: 200 }),
      ),
    );
    vi.stubGlobal(
      "WebSocket",
      class {
        readyState = 1;
        send = vi.fn();
        close = vi.fn();
        addEventListener = vi.fn();
        removeEventListener = vi.fn();
      },
    );
    Object.defineProperty(window.navigator, "mediaDevices", {
      configurable: true,
      value: { getUserMedia, enumerateDevices: vi.fn(async () => []) },
    });
    Object.defineProperty(window, "isSecureContext", { configurable: true, value: true });

    renderCamera("/camera/sess-ok");
    await waitFor(() => {
      expect(screen.getByTestId("camera-state")).toHaveAttribute("data-state", "ready");
    });
    fireEvent.click(screen.getByRole("button", { name: "Start Camera" }));
    await waitFor(() => {
      expect(getUserMedia).toHaveBeenCalledTimes(1);
    });

    stubOrientation("portrait");
    window.dispatchEvent(new Event("orientationchange"));
    window.dispatchEvent(new Event("resize"));
    await waitFor(() => {
      expect(screen.getByTestId("camera-state")).toHaveAttribute("data-orientation", "portrait");
    });
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(getUserMedia).toHaveBeenCalledTimes(1);
  });
});
