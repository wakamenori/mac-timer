import { invoke } from "@tauri-apps/api/core";

const style = document.createElement("style");
style.textContent = `
  @import url('https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700&display=swap');
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }
  html, body {
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    user-select: none;
    font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
    -webkit-font-smoothing: antialiased;
  }
  #overlay {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 32px;
    background: rgba(10, 8, 6, 0.92);
  }
  .overlay-emoji {
    font-size: 64px;
    line-height: 1;
  }
  .overlay-title {
    font-size: 32px;
    font-weight: 700;
    background: linear-gradient(135deg, #F59E0B, #FB923C);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .overlay-body {
    font-size: 18px;
    font-weight: 400;
    color: rgba(250, 245, 240, 0.5);
  }
  .overlay-button {
    margin-top: 16px;
    padding: 14px 48px;
    font-size: 18px;
    font-weight: 600;
    font-family: inherit;
    color: #1a1510;
    background: linear-gradient(135deg, #F59E0B, #FB923C);
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: transform 0.1s ease, opacity 0.15s ease;
  }
  .overlay-button:hover {
    opacity: 0.9;
    transform: scale(1.03);
  }
  .overlay-button:active {
    transform: scale(0.97);
  }
`;
document.head.appendChild(style);

const container = document.getElementById("overlay")!;

const params = new URLSearchParams(window.location.search);
const to = params.get("to");

const isLong = to === "LongBreak";
const emoji = isLong ? "🎉" : "☕";
const title = "集中お疲れさまでした！";
const body = isLong ? "長めの休憩を取りましょう" : "少し休憩しましょう";

container.innerHTML = `
  <div class="overlay-emoji">${emoji}</div>
  <div class="overlay-title">${title}</div>
  <div class="overlay-body">${body}</div>
  <button class="overlay-button">休憩を始める</button>
`;

container.querySelector(".overlay-button")!.addEventListener("click", () => {
  invoke("dismiss_overlay");
});

function playNotificationSound() {
  const ctx = new AudioContext();
  const oscillator = ctx.createOscillator();
  const gain = ctx.createGain();
  oscillator.connect(gain);
  gain.connect(ctx.destination);

  oscillator.type = "sine";
  oscillator.frequency.setValueAtTime(880, ctx.currentTime);
  oscillator.frequency.setValueAtTime(1047, ctx.currentTime + 0.15);
  oscillator.frequency.setValueAtTime(1319, ctx.currentTime + 0.3);

  gain.gain.setValueAtTime(0.5, ctx.currentTime);
  gain.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + 0.6);

  oscillator.start(ctx.currentTime);
  oscillator.stop(ctx.currentTime + 0.6);
}

playNotificationSound();
