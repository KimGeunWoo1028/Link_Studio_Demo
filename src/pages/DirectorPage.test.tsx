import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { emptyPgm } from "../lib/api";
import { studioApi } from "../lib/studioApi";

vi.mock("../lib/useReceiver", () => ({
  useReceiver: () => ({
    status: "Idle",
    error: null,
    pgm: emptyPgm(),
    streams: {},
    orientations: {},
    sendPgm: vi.fn(),
  }),
}));

vi.mock("../lib/studioApi", () => ({
  studioApi: {
    runtime: vi.fn(),
    projects: vi.fn(),
    cameras: vi.fn(),
    settings: vi.fn(),
    startSession: vi.fn(),
    createProject: vi.fn(),
    saveSettings: vi.fn(),
    allowLanFirewall: vi.fn(),
    renameCamera: vi.fn(),
  },
}));

import { DirectorPage } from "./DirectorPage";

const runtime = {
  http_base: "http://127.0.0.1:8787",
  https_base: "https://192.168.0.160:8443",
  lan_ips: ["192.168.0.160"],
  interfaces: [
    {
      id: "eth",
      name: "Ethernet",
      kind: "ethernet" as const,
      ipv4: "192.168.0.160",
      prefixLen: 24,
      gateway: null,
      physical: true,
      up: true,
      selectable: true,
      recommended: true,
    },
  ],
  recommended_ipv4: "192.168.0.160",
  bind_http: "0.0.0.0:8787",
  bind_https: "0.0.0.0:8443",
  ca_url: "http://192.168.0.160:8787/ca.crt",
  ios_profile_url: "http://192.168.0.160:8787/ios/link-studio-ca.mobileconfig",
  android_ca_url: "http://192.168.0.160:8787/ca.crt",
  http_port: 8787,
  https_port: 8443,
  active_session_id: null,
  last_project_id: "p1",
};

describe("DirectorPage setup", () => {
  beforeEach(() => {
    vi.mocked(studioApi.runtime).mockResolvedValue(runtime);
    vi.mocked(studioApi.projects).mockResolvedValue([
      { id: "p1", name: "Demo Project", description: "", created_at: "", updated_at: "" },
    ]);
    vi.mocked(studioApi.cameras).mockResolvedValue([
      { id: "c1", project_id: "p1", name: "Camera 1", role: "cam1", display_order: 1 },
      { id: "c2", project_id: "p1", name: "Camera 2", role: "cam2", display_order: 2 },
    ]);
    vi.mocked(studioApi.settings).mockResolvedValue({ last_project_id: "p1" });
  });

  it("keeps copy disabled and shows start session plus CA trust before a session exists", async () => {
    render(<DirectorPage />);

    await waitFor(() => {
      expect(screen.getByTestId("connection-network")).toBeTruthy();
    });

    expect(screen.getByRole("button", { name: "Copy camera URL" })).toBeDisabled();
    expect(screen.getByTestId("qr-start-session")).toBeTruthy();
    expect(screen.getByTestId("trust-ca")).toBeTruthy();
    expect(screen.getByText("Install iPhone profile")).toBeTruthy();
    expect(screen.getByTestId("director-workspace")).toBeTruthy();
    expect(screen.getByRole("button", { name: "+ Add Camera" })).toBeDisabled();
    expect(screen.getByText("New project")).toBeTruthy();
  });
});
