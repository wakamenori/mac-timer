import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { getNotificationMessage } from "./notification";
import { renderPomodoroTimer, resetPomodoroTimerState } from "./pomodoro-ui";
import type { TimerSnapshot } from "./timer-ui";
import { renderBasicTimer, resetBasicTimerState } from "./timer-ui";

const app = document.getElementById("app")!;

const callbacks = {
  onClose: () => hideWindow(),
  onStart: () => invoke("start_timer"),
  onPause: () => invoke("pause_timer"),
  onReset: () => invoke("reset_timer"),
  onSetDuration: (secs: number) => invoke("set_duration", { secs }),
  onSwitchMode: async () => {
    const current = (await invoke("get_snapshot")) as TimerSnapshot;
    if (current.mode === "basic") {
      await invoke("switch_to_pomodoro");
    } else {
      await invoke("switch_to_basic");
    }
    resetBasicTimerState();
    resetPomodoroTimerState();
    const updated = (await invoke("get_snapshot")) as TimerSnapshot;
    renderSnapshot(updated);
  },
};

function renderSnapshot(snapshot: TimerSnapshot) {
  if (snapshot.mode === "pomodoro") {
    renderPomodoroTimer(app, snapshot, callbacks);
  } else {
    renderBasicTimer(app, snapshot, callbacks);
  }
}

async function setupNotifications() {
  let granted = await isPermissionGranted();
  if (!granted) {
    const permission = await requestPermission();
    granted = permission === "granted";
  }
  return granted;
}

function hideWindow() {
  getCurrentWindow().hide();
}

async function init() {
  // Escape key to hide window
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") hideWindow();
  });

  // Hide window when it loses focus (click outside)
  await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
    if (!focused) hideWindow();
  });
  const notificationsGranted = await setupNotifications();

  // Initial render
  const snapshot = (await invoke("get_snapshot")) as TimerSnapshot;
  renderSnapshot(snapshot);

  // Listen for tick events
  await listen<TimerSnapshot>("timer:tick", (event) => {
    renderSnapshot(event.payload);
  });

  // Listen for phase change events (notifications)
  await listen<{ from: string; to: string }>("timer:phase-change", (event) => {
    if (notificationsGranted) {
      const msg = getNotificationMessage(event.payload.from, event.payload.to);
      if (msg) {
        sendNotification({ title: msg.title, body: msg.body });
      }
    }
  });
}

init();
