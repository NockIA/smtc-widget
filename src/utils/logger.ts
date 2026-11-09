import { error as logError, warn as logWarn } from "@tauri-apps/plugin-log";

const describe = (value: unknown): string => {
  if (value instanceof Error) return value.stack ?? value.message;
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
};

/** Écrit dans le fichier de log de l'app */
export const logger = {
  error: (message: string, cause?: unknown) => {
    const text = cause === undefined ? message : `${message} : ${describe(cause)}`;
    logError(`[front] ${text}`).catch(() => console.error(text));
  },
  warn: (message: string, cause?: unknown) => {
    const text = cause === undefined ? message : `${message} : ${describe(cause)}`;
    logWarn(`[front] ${text}`).catch(() => console.warn(text));
  },
};
