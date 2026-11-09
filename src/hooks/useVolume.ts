import { useCallback, useEffect, useState } from "react";

import { EVENTS } from "@/constants/EVENTS";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { commands } from "@/utils/commands.utils";

/**
 * Volume (0..1) de la source audio courante, ou null si aucune session audio ne correspond.
 * Lu quand la source change, puis poussé par le back à chaque changement (y compris depuis le mélangeur Windows).
 */
export const useVolume = (source: string | undefined) => {
  const [volume, setVolumeState] = useState<number | null>(null);

  useEffect(() => {
    commands.getVolume().then((value) => setVolumeState(value ?? null));
  }, [source]);

  useTauriEvent<number | null>(EVENTS.volumeChanged, setVolumeState);

  const setVolume = useCallback((value: number) => {
    setVolumeState(value);
    commands.setVolume(value);
  }, []);

  return { volume, setVolume };
};
