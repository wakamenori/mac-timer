import { describe, expect, it } from "vitest";
import { formatDisplay } from "../timer-ui";

describe("formatDisplay", () => {
  it("formats zero seconds", () => {
    expect(formatDisplay(0)).toBe("00:00");
  });

  it("formats seconds only", () => {
    expect(formatDisplay(5)).toBe("00:05");
  });

  it("formats minutes and seconds", () => {
    expect(formatDisplay(125)).toBe("02:05");
  });

  it("formats exactly one minute", () => {
    expect(formatDisplay(60)).toBe("01:00");
  });

  it("formats with hours when >= 3600", () => {
    expect(formatDisplay(3661)).toBe("1:01:01");
  });

  it("formats large values", () => {
    expect(formatDisplay(7200)).toBe("2:00:00");
  });
});
