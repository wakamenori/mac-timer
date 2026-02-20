import { describe, expect, it } from "vitest";
import { formatDisplay, progressRingSvg } from "../timer-ui";

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

describe("progressRingSvg", () => {
  const RING_RADIUS = 72;
  const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;

  function extractDashoffset(svg: string): number {
    const match = svg.match(/stroke-dashoffset="([^"]+)"/);
    return match ? parseFloat(match[1]) : NaN;
  }

  it("full ring (remaining == total) has offset close to 0", () => {
    const svg = progressRingSvg(100, 100);
    expect(extractDashoffset(svg)).toBeCloseTo(0, 5);
  });

  it("empty ring (remaining == 0) has offset close to circumference", () => {
    const svg = progressRingSvg(0, 100);
    expect(extractDashoffset(svg)).toBeCloseTo(RING_CIRCUMFERENCE, 5);
  });

  it("half ring (50%) has correct offset", () => {
    const svg = progressRingSvg(50, 100);
    expect(extractDashoffset(svg)).toBeCloseTo(RING_CIRCUMFERENCE * 0.5, 5);
  });

  it("handles total == 0 without division by zero", () => {
    const svg = progressRingSvg(0, 0);
    expect(extractDashoffset(svg)).toBeCloseTo(RING_CIRCUMFERENCE, 5);
  });

  it("contains expected SVG structure", () => {
    const svg = progressRingSvg(50, 100);
    expect(svg).toContain("linearGradient");
    expect(svg).toContain("progress-ring-bg");
    expect(svg).toContain("progress-ring-fill");
    expect(svg).toContain(`stroke-dasharray="${RING_CIRCUMFERENCE}"`);
  });
});
