import type { TimerCallbacks, TimerSnapshot } from "./timer-ui";
import { updateProgressRing, progressRingSvg } from "./timer-ui";

let lastPomodoroState: { isRunning: boolean; phase: string | null } | null =
  null;

export function renderPomodoroTimer(
  container: HTMLElement,
  snapshot: TimerSnapshot,
  callbacks: TimerCallbacks,
): void {
  // If already mounted and state hasn't changed, just update text
  const existing = container.querySelector(".timer-display");
  if (
    existing &&
    lastPomodoroState &&
    lastPomodoroState.isRunning === snapshot.is_running &&
    lastPomodoroState.phase === snapshot.phase
  ) {
    existing.textContent = snapshot.display;
    updateProgressRing(container, snapshot.remaining_secs, snapshot.total_secs);
    const dots = container.querySelector(".session-dots");
    if (dots) dots.textContent = snapshot.session_display || "";
    return;
  }

  lastPomodoroState = {
    isRunning: snapshot.is_running,
    phase: snapshot.phase,
  };

  container.innerHTML = `
    <div class="timer-container" data-tauri-drag-region>
      <button id="btn-close" class="btn-close" aria-label="Close">&times;</button>
      <div class="mode-label" data-tauri-drag-region>Pomodoro</div>
      <div class="timer-ring-wrapper">
        ${progressRingSvg(snapshot.remaining_secs, snapshot.total_secs)}
        <div class="timer-ring-content">
          <div class="timer-display">${snapshot.display}</div>
        </div>
      </div>
      <div class="session-dots">${snapshot.session_display || ""}</div>
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
    .querySelector("#btn-switch")
    ?.addEventListener("click", callbacks.onSwitchMode);
  container
    .querySelector("#btn-close")
    ?.addEventListener("click", callbacks.onClose);
}

export function resetPomodoroTimerState(): void {
  lastPomodoroState = null;
}
