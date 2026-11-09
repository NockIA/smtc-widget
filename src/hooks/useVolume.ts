import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { handleCommandError } from "../utils/error.utils";

/**
 * Volume (0..1) de la source audio courante, ou null si aucune session audio ne correspond.
 * Relu quand la source change et à la demande (`refresh`), car le volume peut aussi
 * être modifié depuis le mélangeur Windows.
 */
export const useVolume = (source: string | undefined) => {
  const [volume, setVolumeState] = useState<number | null>(null);

  const refresh = useCallback(() => {
    invoke<number | null>("get_volume")
      .then(setVolumeState)
      .catch((error) => {
        handleCommandError("get_volume")(error);
        setVolumeState(null);
      });
  }, []);

  useEffect(refresh, [source, refresh]);

  const setVolume = useCallback((value: number) => {
    setVolumeState(value);
    invoke("set_volume", { volume: value }).catch(handleCommandError("set_volume"));
  }, []);

  return { volume, setVolume, refresh };
};
