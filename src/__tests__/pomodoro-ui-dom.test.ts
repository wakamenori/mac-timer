// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderPomodoroTimer, resetPomodoroTimerState } from "../pomodoro-ui";
import type { TimerCallbacks, TimerSnapshot } from "../timer-ui";

function makeSnapshot(overrides: Partial<TimerSnapshot> = {}): TimerSnapshot {
  return {
    mode: "pomodoro",
    display: "25:00",
    remaining_secs: 1500,
    total_secs: 1500,
    is_running: false,
    is_finished: false,
    phase: "Work",
    session_display: "○ ○ ○ ○",
    tray_title: "🍅 25:00",
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

describe("renderPomodoroTimer", () => {
  let container: HTMLElement;
  let callbacks: TimerCallbacks;

  beforeEach(() => {
    container = document.createElement("div");
    callbacks = makeCallbacks();
    resetPomodoroTimerState();
  });

  it('renders "Pomodoro" mode label', () => {
    renderPomodoroTimer(container, makeSnapshot(), callbacks);
    expect(container.querySelector(".mode-label")?.textContent).toBe(
      "Pomodoro",
    );
  });

  it("renders session dots from snapshot.session_display", () => {
    renderPomodoroTimer(
      container,
      makeSnapshot({ session_display: "● ○ ○ ○" }),
      callbacks,
    );
    expect(container.querySelector(".session-dots")?.textContent).toBe(
      "● ○ ○ ○",
    );
  });

  it("renders Start button when not running", () => {
    renderPomodoroTimer(
      container,
      makeSnapshot({ is_running: false }),
      callbacks,
    );
    expect(container.querySelector("#btn-start")).not.toBeNull();
    expect(container.querySelector("#btn-pause")).toBeNull();
  });

  it("renders Pause button when running", () => {
    renderPomodoroTimer(
      container,
      makeSnapshot({ is_running: true }),
      callbacks,
    );
    expect(container.querySelector("#btn-pause")).not.toBeNull();
    expect(container.querySelector("#btn-start")).toBeNull();
  });

  it('shows "Switch to Basic" button', () => {
    renderPomodoroTimer(container, makeSnapshot(), callbacks);
    expect(container.querySelector("#btn-switch")?.textContent).toBe(
      "Switch to Basic",
    );
  });

  it("wires all callback buttons correctly", () => {
    renderPomodoroTimer(
      container,
      makeSnapshot({ is_running: false }),
      callbacks,
    );
    (container.querySelector("#btn-start") as HTMLElement).click();
    expect(callbacks.onStart).toHaveBeenCalledOnce();

    // Re-render with running to get pause button
    resetPomodoroTimerState();
    renderPomodoroTimer(
      container,
      makeSnapshot({ is_running: true }),
      callbacks,
    );
    (container.querySelector("#btn-pause") as HTMLElement).click();
    expect(callbacks.onPause).toHaveBeenCalledOnce();

    (container.querySelector("#btn-reset") as HTMLElement).click();
    expect(callbacks.onReset).toHaveBeenCalledOnce();

    (container.querySelector("#btn-switch") as HTMLElement).click();
    expect(callbacks.onSwitchMode).toHaveBeenCalledOnce();

  });

  it("updates display and session dots on second render", () => {
    renderPomodoroTimer(
      container,
      makeSnapshot({
        display: "25:00",
        session_display: "○ ○ ○ ○",
      }),
      callbacks,
    );
    renderPomodoroTimer(
      container,
      makeSnapshot({
        display: "24:55",
        session_display: "● ○ ○ ○",
      }),
      callbacks,
    );
    expect(container.querySelector(".timer-display")?.textContent).toBe(
      "24:55",
    );
    expect(container.querySelector(".session-dots")?.textContent).toBe(
      "● ○ ○ ○",
    );
  });

  it("reflects phase change correctly in UI", () => {
    renderPomodoroTimer(
      container,
      makeSnapshot({ phase: "Work", is_running: true }),
      callbacks,
    );
    expect(container.querySelector("#btn-pause")).not.toBeNull();

    renderPomodoroTimer(
      container,
      makeSnapshot({ phase: "ShortBreak", is_running: true }),
      callbacks,
    );
    // Full re-render happened due to phase change — buttons should still work
    expect(container.querySelector("#btn-pause")).not.toBeNull();
    (container.querySelector("#btn-pause") as HTMLElement).click();
    expect(callbacks.onPause).toHaveBeenCalled();
  });
});
