/** Codes d'erreur renvoyés par les commandes Tauri (`AppError::code` côté Rust). */
export const KNOWN_ERROR_CODES = ["smtc_unavailable", "audio_unavailable", "internal"] as const;

/** Code utilisé quand l'erreur reçue n'est pas reconnue. */
export const FALLBACK_ERROR_CODE = "internal";
