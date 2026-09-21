import { describe, expect, it } from "vitest";
import { parseSessionLookup, parseSessionResponse, isUsableSessionId } from "./sessionContract";

describe("session creation contract", () => {
  it("accepts camelCase session creation JSON", () => {
    const parsed = parseSessionResponse({
      sessionId: "sess_live",
      cameraUrl: "https://10.0.0.8:8443/camera/sess_live",
      programUrl: "https://10.0.0.8:8443/program/sess_live",
      programHttpUrl: "http://127.0.0.1:8787/program/sess_live",
    });
    expect(parsed.sessionId).toBe("sess_live");
    expect(parsed.cameraUrl).toContain("/camera/sess_live");
  });

  it("accepts snake_case session creation JSON", () => {
    const parsed = parseSessionResponse({
      session_id: "sess_snake",
      camera_url: "https://host/camera/sess_snake",
      program_url: "https://host/program/sess_snake",
      program_http_url: "http://127.0.0.1:8787/program/sess_snake",
    });
    expect(parsed.sessionId).toBe("sess_snake");
    expect(parsed.programHttpUrl).toContain("sess_snake");
  });

  it("rejects missing or placeholder session IDs", () => {
    expect(() => parseSessionResponse({})).toThrow(/missing session ID/i);
    expect(() => parseSessionResponse({ session_id: "" })).toThrow(/missing session ID/i);
    expect(() => parseSessionResponse({ sessionId: "undefined" })).toThrow(/missing session ID/i);
  });

  it("parses session lookup from either naming style", () => {
    expect(parseSessionLookup({ sessionId: "a", projectId: "p" })).toEqual({
      sessionId: "a",
      projectId: "p",
      cameras: [
        { role: "cam1", name: "Camera 1" },
        { role: "cam2", name: "Camera 2" },
      ],
    });
    expect(parseSessionLookup({ id: "a", project_id: "p" })).toEqual({
      sessionId: "a",
      projectId: "p",
      cameras: [
        { role: "cam1", name: "Camera 1" },
        { role: "cam2", name: "Camera 2" },
      ],
    });
  });

  it("parses camera slot names from session lookup", () => {
    expect(
      parseSessionLookup({
        sessionId: "a",
        projectId: "p",
        cameras: [
          { role: "cam1", name: "Stage left" },
          { role: "cam2", name: "Stage right" },
        ],
      }).cameras,
    ).toEqual([
      { role: "cam1", name: "Stage left" },
      { role: "cam2", name: "Stage right" },
    ]);
  });

  it("treats undefined/null path segments as unusable", () => {
    expect(isUsableSessionId(undefined)).toBe(false);
    expect(isUsableSessionId("undefined")).toBe(false);
    expect(isUsableSessionId("real-id")).toBe(true);
  });
});
