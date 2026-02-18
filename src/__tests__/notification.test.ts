import { describe, expect, it } from "vitest";
import { getNotificationMessage } from "../notification";

describe("getNotificationMessage", () => {
  it("returns message for basic timer finish", () => {
    const msg = getNotificationMessage("timer", "finished");
    expect(msg).toEqual({
      title: "Timer Finished!",
      body: "Your timer has completed.",
    });
  });

  it("returns message for work to short break", () => {
    const msg = getNotificationMessage("Work", "ShortBreak");
    expect(msg).toEqual({
      title: "Break Time!",
      body: "Take a short break.",
    });
  });

  it("returns message for work to long break", () => {
    const msg = getNotificationMessage("Work", "LongBreak");
    expect(msg).toEqual({
      title: "Long Break!",
      body: "Great work! Take a longer break.",
    });
  });

  it("returns message for break to work", () => {
    const msg = getNotificationMessage("ShortBreak", "Work");
    expect(msg).toEqual({
      title: "Back to Work!",
      body: "Time to focus.",
    });
  });

  it("returns message for long break to work", () => {
    const msg = getNotificationMessage("LongBreak", "Work");
    expect(msg).toEqual({
      title: "Back to Work!",
      body: "Time to focus.",
    });
  });

  it("returns null for unknown transition", () => {
    const msg = getNotificationMessage("unknown", "unknown");
    expect(msg).toBeNull();
  });
});
