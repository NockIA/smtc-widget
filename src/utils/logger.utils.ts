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

const createLogger =
  (write: (message: string) => Promise<void>, fallback: (message: string) => void) =>
  (message: string, cause?: unknown) => {
    const text = cause === undefined ? message : `${message} : ${describe(cause)}`;
    write(`[front] ${text}`).catch(() => fallback(text));
  };

/** Écrit dans le fichier de log de l'app. */
export const logger = {
  error: createLogger(logError, (text) => console.error(text)),
  warn: createLogger(logWarn, (text) => console.warn(text)),
};
