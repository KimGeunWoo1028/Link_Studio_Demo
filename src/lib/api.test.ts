import { describe, expect, it } from "vitest";
import { escapeCaption, emptyPgm, wsUrl } from "./api";

describe("caption overlay helpers", () => {
  it("trims and caps caption length", () => {
    expect(escapeCaption("  hello   world  ")).toBe("hello world");
    expect(escapeCaption("x".repeat(200)).length).toBe(120);
  });

  it("starts with captions hidden", () => {
    expect(emptyPgm().captionVisible).toBe(false);
    expect(emptyPgm().pipEnabled).toBe(false);
  });

  it("uses wss on https origins", () => {
    const previous = window.location;
    Object.defineProperty(window, "location", {
      configurable: true,
      value: { protocol: "https:", host: "10.0.0.8:8443", port: "8443", hostname: "10.0.0.8" },
    });
    expect(wsUrl()).toBe("wss://10.0.0.8:8443/ws");
    Object.defineProperty(window, "location", { configurable: true, value: previous });
  });
});
