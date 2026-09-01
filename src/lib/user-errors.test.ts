import { describe, expect, it } from "vitest";

import { formatUserError, userErrorMessage } from "./user-errors";

describe("formatUserError", () => {
  it("maps decode errors to action-oriented copy with raw detail", () => {
    expect(formatUserError("decode error: no decodable track", { name: "bad.wav" })).toEqual({
      message:
        "Couldn't read bad.wav. The file may be moved, unsupported, or in use. Re-import it or use Re-analyze.",
      detail: "decode error: no decodable track",
    });
  });

  it("maps invalid paths, I/O failures, and timeouts", () => {
    expect(formatUserError("invalid path: empty path").message).toBe(
      "Couldn't use that location. Choose a different folder or file name, then try again.",
    );
    expect(formatUserError("io error: access denied").message).toBe(
      "Couldn't access the file. It may be moved, locked, or in use. Check the file and try again.",
    );
    expect(formatUserError("audio seek reply timeout").message).toBe(
      "That operation is taking longer than expected. Wait a moment and try again.",
    );
  });

  it("maps render failures to plain language and keeps the raw detail", () => {
    // S6.3 (2026-09-01): CommandError::Render surfaces as "render error: …".
    expect(formatUserError("render error: rendered samples contain non-finite value at index 3")).toEqual({
      message:
        "The render failed and no file was written. Try again; if it repeats, save a diagnostics report from Help.",
      detail: "render error: rendered samples contain non-finite value at index 3",
    });
  });

  it("maps both audio-device-unavailable shapes to the Settings hint", () => {
    // audio.rs: default-device form and chosen-device form.
    for (const raw of [
      "audio device unavailable: no default output device",
      "audio output device unavailable (Speakers (Realtek)): device disconnected",
    ]) {
      expect(formatUserError(raw)).toEqual({
        message:
          "No audio output device is available. Choose an output in Settings, then press Play.",
        detail: raw,
      });
    }
  });

  it("keeps unknown raw errors available", () => {
    expect(userErrorMessage("backend exploded")).toBe(
      "Something went wrong. Details: backend exploded",
    );
  });
});
