export interface UserError {
  message: string;
  detail?: string;
}

export interface UserErrorContext {
  name?: string | null;
}

function rawMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function subjectName(context?: UserErrorContext): string {
  return context?.name?.trim() || "this file";
}

export function formatUserError(
  err: unknown,
  context?: UserErrorContext,
): UserError {
  const raw = rawMessage(err);
  const lower = raw.toLowerCase();

  if (lower.startsWith("mastered preview is still preparing")) {
    return { message: raw };
  }

  if (lower.startsWith("decode error:")) {
    return {
      message: `Couldn't read ${subjectName(context)}. The file may be moved, unsupported, or in use. Re-import it or use Re-analyze.`,
      detail: raw,
    };
  }

  if (lower.startsWith("invalid path:")) {
    return {
      message:
        "Couldn't use that location. Choose a different folder or file name, then try again.",
      detail: raw,
    };
  }

  if (lower.startsWith("io error:")) {
    return {
      message:
        "Couldn't access the file. It may be moved, locked, or in use. Check the file and try again.",
      detail: raw,
    };
  }

  if (lower.includes("timeout")) {
    return {
      message:
        "That operation is taking longer than expected. Wait a moment and try again.",
      detail: raw,
    };
  }

  // Backend `CommandError::Render` ("render error: …", types.rs). The write
  // is atomic (tmp + rename), so a failed render leaves no partial file.
  if (lower.startsWith("render error:")) {
    return {
      message:
        "The render failed and no file was written. Try again; if it repeats, save a diagnostics report from Help.",
      detail: raw,
    };
  }

  // audio.rs raises both "audio device unavailable: …" (default device) and
  // "audio output device unavailable (<name>): …" (a chosen device).
  if (
    lower.includes("audio device unavailable") ||
    lower.includes("audio output device unavailable")
  ) {
    return {
      message:
        "No audio output device is available. Choose an output in Settings, then press Play.",
      detail: raw,
    };
  }

  return {
    message: "Something went wrong.",
    detail: raw,
  };
}

export function userErrorMessage(
  err: unknown,
  context?: UserErrorContext,
): string {
  const formatted = formatUserError(err, context);
  return formatted.detail
    ? `${formatted.message} Details: ${formatted.detail}`
    : formatted.message;
}
