import { invoke, InvokeArgs } from "@tauri-apps/api/core";

import { SystemColors } from "@/types/systemColors.types";
import { TrackInfo } from "@/types/track.types";

import { logger } from "./logger.utils";

type MediaAction = "previous" | "togglePlayPause" | "next";

/**
 * Appelle une commande Tauri ; en cas d'échec, le résultat vaut `undefined`.
 * Le back journalise déjà le détail de l'erreur : on ne garde ici que son code (`AppError::code`).
 */
const call = <T = void>(command: string, args?: InvokeArgs): Promise<T | undefined> =>
  invoke<T>(command, args).catch((error: unknown) => {
    const code = (error as { code?: unknown } | null)?.code ?? "inconnu";
    logger.error(`échec de la commande ${command} : ${String(code)}`);
    return undefined;
  });

export const commands = {
  currentTrack: () => call<TrackInfo | null>("current_track"),
  albumArt: () => call<string | null>("album_art"),
  mediaControl: (action: MediaAction) => call("media_control", { action }),
  getVolume: () => call<number | null>("get_volume"),
  setVolume: (volume: number) => call("set_volume", { volume }),
  getSystemColors: () => call<SystemColors>("get_system_colors"),
  refreshSystemColors: () => call("refresh_system_colors"),
  showContextMenu: (x: number) => call("show_context_menu", { x }),
  startResize: () => call("start_resize"),
  resetSize: () => call("reset_size"),
};
