import { describe, expect, it } from "vitest";
import {
  androidCaUrl,
  cameraHttpsUrl,
  iosInstallUrl,
  iosProfileUrl,
  programLocalUrl,
  resolveSelectedIpv4,
  type LanInterface,
} from "./connectionUrls";

function iface(partial: Partial<LanInterface> & Pick<LanInterface, "ipv4" | "kind">): LanInterface {
  return {
    id: partial.ipv4,
    name: partial.kind === "wifi" ? "Wi-Fi" : "Ethernet",
    prefixLen: 24,
    gateway: "192.168.0.1",
    physical: true,
    up: true,
    selectable: true,
    recommended: false,
    ...partial,
  };
}

describe("connection URL contract", () => {
  it("rebuilds camera/CA URLs from the selected host without changing session id", () => {
    const sessionId = "abc123";
    expect(cameraHttpsUrl("192.168.0.137", 8443, sessionId)).toBe(
      "https://192.168.0.137:8443/camera/abc123",
    );
    expect(cameraHttpsUrl("192.168.0.160", 8443, sessionId)).toBe(
      "https://192.168.0.160:8443/camera/abc123",
    );
    expect(iosProfileUrl("192.168.0.137", 8787)).toBe(
      "http://192.168.0.137:8787/ios/link-studio-ca.mobileconfig",
    );
    expect(iosInstallUrl("192.168.0.160", 8787)).toBe("http://192.168.0.160:8787/install-ca");
    expect(androidCaUrl("192.168.0.160", 8787)).toBe("http://192.168.0.160:8787/ca.crt");
  });

  it("keeps PGM on localhost when the camera network changes", () => {
    expect(programLocalUrl(8787, "abc123")).toBe("http://127.0.0.1:8787/program/abc123");
  });

  it("keeps a valid preferred IP and falls back when that adapter disappears", () => {
    const ifaces = [
      iface({ ipv4: "192.168.0.137", kind: "wifi", recommended: true }),
      iface({ ipv4: "192.168.0.160", kind: "ethernet" }),
    ];
    expect(resolveSelectedIpv4(ifaces, "192.168.0.160")).toBe("192.168.0.160");
    expect(resolveSelectedIpv4(ifaces, "10.0.0.9")).toBe("192.168.0.137");
  });

  it("does not select disconnected or virtual adapters", () => {
    const ifaces = [
      iface({
        ipv4: "192.168.0.160",
        kind: "ethernet",
        up: false,
        selectable: false,
        reason: "Disconnected",
      }),
      iface({ ipv4: "192.168.192.1", kind: "other", physical: false, selectable: false }),
      iface({ ipv4: "192.168.0.137", kind: "wifi", recommended: true }),
    ];
    expect(resolveSelectedIpv4(ifaces, "192.168.0.160")).toBe("192.168.0.137");
  });
});
