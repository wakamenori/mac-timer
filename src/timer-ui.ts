export function formatDisplay(remainingSecs: number): string {
  const h = Math.floor(remainingSecs / 3600);
  const m = Math.floor((remainingSecs % 3600) / 60);
  const s = remainingSecs % 60;
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  if (h > 0) {
    return `${h}:${mm}:${ss}`;
  }
  return `${mm}:${ss}`;
}

export interface TimerSnapshot {
  mode: string;
  display: string;
  remaining_secs: number;
  is_running: boolean;
  is_finished: boolean;
  phase: string | null;
  session_display: string | null;
  tray_title: string;
}

export interface TimerCallbacks {
  onStart: () => void;
  onPause: () => void;
  onReset: () => void;
  onSetDuration: (secs: number) => void;
  onSwitchMode: () => void;
  onClose: () => void;
}

let lastBasicIsRunning: boolean | null = null;

export function renderBasicTimer(
  container: HTMLElement,
  snapshot: TimerSnapshot,
  callbacks: TimerCallbacks,
): void {
  // If already mounted and running state hasn't changed, just update text
  const existing = container.querySelector(".timer-display");
  if (existing && lastBasicIsRunning === snapshot.is_running) {
    existing.textContent = snapshot.display;
    return;
  }

  lastBasicIsRunning = snapshot.is_running;

  container.innerHTML = `
    <div class="timer-container">
      <button id="btn-close" class="btn-close" aria-label="Close">&times;</button>
      <div class="mode-label">Basic Timer</div>
      <div class="timer-display">${snapshot.display}</div>
      <div class="timer-controls">
        ${
          snapshot.is_running
            ? `<button id="btn-pause" class="btn">Pause</button>`
            : `<button id="btn-start" class="btn btn-primary">Start</button>`
        }
        <button id="btn-reset" class="btn">Reset</button>
      </div>
      <div class="presets">
        <button class="btn btn-preset" data-secs="300">5m</button>
        <button class="btn btn-preset" data-secs="600">10m</button>
        <button class="btn btn-preset" data-secs="900">15m</button>
        <button class="btn btn-preset" data-secs="1800">30m</button>
      </div>
      <button id="btn-switch" class="btn btn-mode">Switch to Pomodoro</button>
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

  container.querySelectorAll(".btn-preset").forEach((btn) => {
    btn.addEventListener("click", () => {
      const secs = parseInt((btn as HTMLElement).dataset.secs || "300", 10);
      callbacks.onSetDuration(secs);
    });
  });
}

export function resetBasicTimerState(): void {
  lastBasicIsRunning = null;
}
