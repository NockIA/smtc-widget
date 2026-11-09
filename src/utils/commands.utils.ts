import { invoke, InvokeArgs } from "@tauri-apps/api/core";

import { MediaAction } from "@/types/mediaAction.types";
import { SystemColors } from "@/types/systemColors.types";
import { TrackInfo } from "@/types/track.types";

import { handleCommandError } from "./error.utils";

/** Appelle une commande Tauri ; en cas d'échec, l'erreur est journalisée et le résultat vaut `undefined`. */
const call = <T = void>(command: string, args?: InvokeArgs): Promise<T | undefined> =>
  invoke<T>(command, args).catch((error) => {
    handleCommandError(command)(error);
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
};
