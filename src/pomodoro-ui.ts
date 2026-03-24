import type { TimerCallbacks, TimerSnapshot } from "./timer-ui";
import { updateProgressRing, progressRingSvg } from "./timer-ui";

let lastPomodoroState: {
  isRunning: boolean;
  isIdle: boolean;
  phase: string | null;
  remainingSecs: number;
} | null = null;

export function renderPomodoroTimer(
  container: HTMLElement,
  snapshot: TimerSnapshot,
  callbacks: TimerCallbacks,
): void {
  const showAdjust = snapshot.is_idle && snapshot.phase === "Work";

  // If already mounted and state hasn't changed, just update text
  const existing = container.querySelector(".timer-display");
  if (
    existing &&
    lastPomodoroState &&
    lastPomodoroState.isRunning === snapshot.is_running &&
    lastPomodoroState.isIdle === snapshot.is_idle &&
    lastPomodoroState.phase === snapshot.phase &&
    lastPomodoroState.remainingSecs === snapshot.remaining_secs
  ) {
    existing.textContent = snapshot.display;
    updateProgressRing(container, snapshot.remaining_secs, snapshot.total_secs);
    const dots = container.querySelector(".session-dots");
    if (dots) dots.textContent = snapshot.session_display || "";
    return;
  }

  lastPomodoroState = {
    isRunning: snapshot.is_running,
    isIdle: snapshot.is_idle,
    phase: snapshot.phase,
    remainingSecs: snapshot.remaining_secs,
  };

  container.innerHTML = `
    <div class="timer-container" data-tauri-drag-region>
      <div class="mode-label" data-tauri-drag-region>Pomodoro</div>
      <div class="timer-ring-wrapper">
        ${progressRingSvg(snapshot.remaining_secs, snapshot.total_secs)}
        <div class="timer-ring-content">
          <div class="timer-display">${snapshot.display}</div>
        </div>
      </div>
      ${
        showAdjust
          ? `<div class="duration-adjust">
              <button id="btn-minus" class="btn btn-adjust">−5m</button>
              <button id="btn-plus" class="btn btn-adjust">+5m</button>
            </div>`
          : `<div class="session-dots">${snapshot.session_display || ""}</div>`
      }
      <div class="timer-controls">
        ${
          snapshot.is_running
            ? `<button id="btn-pause" class="btn">Pause</button>`
            : `<button id="btn-start" class="btn btn-primary">Start</button>`
        }
        <button id="btn-reset" class="btn">Reset</button>
      </div>
      <button id="btn-switch" class="btn btn-mode">Switch to Basic</button>
    </div>
  `;

  container
    .querySelector("#btn-start")
    ?.addEventListener("click", callbacks.onStart);
  container
    .querySelector("#btn-pause")
    ?.addEventListener("click", callbacks.onPause);
  container
    .querySelector("#btn-reset")
    ?.addEventListener("click", callbacks.onReset);
  container
    .querySelector("#btn-minus")
    ?.addEventListener("click", () => callbacks.onAdjustDuration(-5 * 60));
  container
    .querySelector("#btn-plus")
    ?.addEventListener("click", () => callbacks.onAdjustDuration(5 * 60));
  container
    .querySelector("#btn-switch")
    ?.addEventListener("click", callbacks.onSwitchMode);
}

export function resetPomodoroTimerState(): void {
  lastPomodoroState = null;
}
