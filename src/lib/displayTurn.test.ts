import { describe, expect, it } from "vitest";
import { turnForDisplay } from "./displayTurn";

describe("director display turn", () => {
  it("does not turn when the frame already matches the phone", () => {
    expect(turnForDisplay(1920, 1080, "landscape")).toBe(0);
    expect(turnForDisplay(1080, 1920, "portrait")).toBe(0);
  });

  it("turns 90 when the phone is landscape but the track stayed portrait", () => {
    expect(turnForDisplay(1080, 1920, "landscape")).toBe(90);
  });

  it("turns 90 when the phone is portrait but the track stayed landscape", () => {
    expect(turnForDisplay(1920, 1080, "portrait")).toBe(90);
  });

  it("does not turn until the frame size and phone orientation are known", () => {
    expect(turnForDisplay(1080, 1920, undefined)).toBe(0);
    expect(turnForDisplay(0, 0, "landscape")).toBe(0);
  });
});
