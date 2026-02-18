import { getCurrentWindow } from "@tauri-apps/api/window";
import { getNotificationMessage } from "./notification";

const AUTO_CLOSE_MS = 5000;

const style = document.createElement("style");
style.textContent = `
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }
  html, body {
    background: transparent;
    overflow: hidden;
    user-select: none;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Helvetica Neue", sans-serif;
  }
  #notification {
    width: 300px;
    height: 120px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    background: rgba(30, 30, 30, 0.9);
    -webkit-backdrop-filter: blur(20px);
    backdrop-filter: blur(20px);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #f0f0f0;
    cursor: pointer;
  }
  .notif-title {
    font-size: 18px;
    font-weight: 600;
  }
  .notif-body {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.6);
  }
`;
document.head.appendChild(style);

const container = document.getElementById("notification")!;

function dismiss() {
  getCurrentWindow().close();
}

// Click anywhere to dismiss
container.addEventListener("click", dismiss);

// Escape to dismiss
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") dismiss();
});

// Read from/to from URL query params and show immediately
const params = new URLSearchParams(window.location.search);
const from = params.get("from");
const to = params.get("to");

if (from && to) {
  const msg = getNotificationMessage(from, to);
  if (msg) {
    container.innerHTML = `
      <div class="notif-title">${msg.title}</div>
      <div class="notif-body">${msg.body}</div>
    `;
  }
}

setTimeout(dismiss, AUTO_CLOSE_MS);
