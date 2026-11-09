import { useCallback, useEffect, useState } from "react";

import { commands } from "@/utils/commands.utils";

/**
 * Volume (0..1) de la source audio courante, ou null si aucune session audio ne correspond.
 * Relu quand la source change et à la demande (`refresh`), car le volume peut aussi
 * être modifié depuis le mélangeur Windows.
 */
export const useVolume = (source: string | undefined) => {
  const [volume, setVolumeState] = useState<number | null>(null);

  const refresh = useCallback(() => {
    commands.getVolume().then((value) => setVolumeState(value ?? null));
  }, []);

  useEffect(refresh, [source, refresh]);

  const setVolume = useCallback((value: number) => {
    setVolumeState(value);
    commands.setVolume(value);
  }, []);

  return { volume, setVolume, refresh };
};
