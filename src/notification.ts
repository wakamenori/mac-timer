export function getNotificationMessage(
  from: string,
  to: string,
): { title: string; body: string } | null {
  if (from === "timer" && to === "finished") {
    return { title: "Timer Finished!", body: "Your timer has completed." };
  }
  if (from === "Work" && to === "ShortBreak") {
    return { title: "Break Time!", body: "Take a short break." };
  }
  if (from === "Work" && to === "LongBreak") {
    return { title: "Long Break!", body: "Great work! Take a longer break." };
  }
  if (
    (from === "ShortBreak" || from === "LongBreak") &&
    to === "Work"
  ) {
    return { title: "Back to Work!", body: "Time to focus." };
  }
  return null;
}
