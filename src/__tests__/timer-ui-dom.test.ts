// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  updateProgressRing,
  renderBasicTimer,
  resetBasicTimerState,
} from "../timer-ui";
import type { TimerSnapshot, TimerCallbacks } from "../timer-ui";

const RING_RADIUS = 72;
const RING_CIRCUMFERENCE = 2 * Math.PI * RING_RADIUS;

function makeSnapshot(overrides: Partial<TimerSnapshot> = {}): TimerSnapshot {
  return {
    mode: "basic",
    display: "25:00",
    remaining_secs: 1500,
    total_secs: 1500,
    is_running: false,
    is_finished: false,
    phase: null,
    session_display: null,
    tray_title: "⏱ 25:00",
    ...overrides,
  };
}

function makeCallbacks(): TimerCallbacks {
  return {
    onStart: vi.fn(),
    onPause: vi.fn(),
    onReset: vi.fn(),
    onSetDuration: vi.fn(),
    onSwitchMode: vi.fn(),
    onClose: vi.fn(),
  };
}

describe("updateProgressRing", () => {
  let container: HTMLElement;

  beforeEach(() => {
    container = document.createElement("div");
    container.innerHTML = `
      <svg>
        <circle class="progress-ring-fill" />
      </svg>
    `;
  });

  it("sets correct stroke-dashoffset on fill element", () => {
    updateProgressRing(container, 750, 1500);
    const fill = container.querySelector(
      ".progress-ring-fill",
    ) as SVGCircleElement;
    expect(parseFloat(fill.style.strokeDashoffset)).toBeCloseTo(
      RING_CIRCUMFERENCE * 0.5,
      5,
    );
  });

  it("no-ops when fill element is missing", () => {
    const empty = document.createElement("div");
    expect(() => updateProgressRing(empty, 750, 1500)).not.toThrow();
  });

  it("handles total == 0 gracefully", () => {
    updateProgressRing(container, 0, 0);
    const fill = container.querySelector(
      ".progress-ring-fill",
    ) as SVGCircleElement;
    expect(parseFloat(fill.style.strokeDashoffset)).toBeCloseTo(
      RING_CIRCUMFERENCE,
      5,
    );
  });
});

describe("renderBasicTimer", () => {
  let container: HTMLElement;
  let callbacks: TimerCallbacks;

  beforeEach(() => {
    container = document.createElement("div");
    callbacks = makeCallbacks();
    resetBasicTimerState();
  });

  it("renders Start button when not running", () => {
    renderBasicTimer(container, makeSnapshot({ is_running: false }), callbacks);
    expect(container.querySelector("#btn-start")).not.toBeNull();
    expect(container.querySelector("#btn-pause")).toBeNull();
  });

  it("renders Pause button when running", () => {
    renderBasicTimer(container, makeSnapshot({ is_running: true }), callbacks);
    expect(container.querySelector("#btn-pause")).not.toBeNull();
    expect(container.querySelector("#btn-start")).toBeNull();
  });

  it("displays timer value in .timer-display", () => {
    renderBasicTimer(
      container,
      makeSnapshot({ display: "10:30" }),
      callbacks,
    );
    expect(container.querySelector(".timer-display")?.textContent).toBe(
      "10:30",
    );
  });

  it("calls onStart on Start button click", () => {
    renderBasicTimer(container, makeSnapshot({ is_running: false }), callbacks);
    (container.querySelector("#btn-start") as HTMLElement).click();
    expect(callbacks.onStart).toHaveBeenCalledOnce();
  });

  it("calls onPause on Pause button click", () => {
    renderBasicTimer(container, makeSnapshot({ is_running: true }), callbacks);
    (container.querySelector("#btn-pause") as HTMLElement).click();
    expect(callbacks.onPause).toHaveBeenCalledOnce();
  });

  it("calls onReset on Reset button click", () => {
    renderBasicTimer(container, makeSnapshot(), callbacks);
    (container.querySelector("#btn-reset") as HTMLElement).click();
    expect(callbacks.onReset).toHaveBeenCalledOnce();
  });

  it("calls onSwitchMode on Switch button click", () => {
    renderBasicTimer(container, makeSnapshot(), callbacks);
    (container.querySelector("#btn-switch") as HTMLElement).click();
    expect(callbacks.onSwitchMode).toHaveBeenCalledOnce();
  });

  it("calls onSetDuration(300) for 5m preset button click", () => {
    renderBasicTimer(container, makeSnapshot(), callbacks);
    const presetBtn = container.querySelector(
      '.btn-preset[data-secs="300"]',
    ) as HTMLElement;
    presetBtn.click();
    expect(callbacks.onSetDuration).toHaveBeenCalledWith(300);
  });

  it("updates display correctly on second render without is_running change", () => {
    renderBasicTimer(
      container,
      makeSnapshot({ is_running: false, display: "25:00" }),
      callbacks,
    );
    renderBasicTimer(
      container,
      makeSnapshot({ is_running: false, display: "24:55" }),
      callbacks,
    );
    expect(container.querySelector(".timer-display")?.textContent).toBe(
      "24:55",
    );
  });

  it("switches Start/Pause button when is_running changes", () => {
    renderBasicTimer(container, makeSnapshot({ is_running: false }), callbacks);
    expect(container.querySelector("#btn-start")).not.toBeNull();

    renderBasicTimer(container, makeSnapshot({ is_running: true }), callbacks);
    expect(container.querySelector("#btn-pause")).not.toBeNull();
    expect(container.querySelector("#btn-start")).toBeNull();
  });
});
