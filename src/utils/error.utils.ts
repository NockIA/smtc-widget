import { logger } from "./logger";
import { FALLBACK_ERROR_CODE, KNOWN_ERROR_CODES } from "../constants/ERROR_CODES";
import { AppErrorCode } from "../types/error.types";

/**
 * Handler de `.catch` pour les `invoke` : journalise le code reçu, sans jamais exposer
 * de message brut. L'absence de session n'est pas une erreur (le back répond `null`/`false`).
 */
export const handleCommandError = (command: string) => (error: unknown) => {
  logger.error(`commande ${command} échouée (${parseAppError(error)})`);
};

/** Normalise ce que `invoke` rejette : tout ce qui n'est pas `{ code }` connu devient `internal`. */
export const parseAppError = (error: unknown): AppErrorCode => {
  const code = (error as { code?: unknown } | null)?.code;
  return KNOWN_ERROR_CODES.find((known) => known === code) ?? FALLBACK_ERROR_CODE;
};
