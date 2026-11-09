import { useCallback, useEffect, useState } from "react";

import { EVENTS } from "@/constants/EVENTS";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { TrackInfo } from "@/types/track.types";
import { commands } from "@/utils/commands.utils";

interface TrackState {
  track: TrackInfo | null;
  /** performance.now() à la réception, pour interpoler la progression. */
  receivedAt: number;
}

/** Morceau courant, relu au démarrage et au retour du widget (`hidden` repasse à faux) pour se recaler. */
export const useCurrentTrack = (hidden: boolean) => {
  const [state, setState] = useState<TrackState>(() => ({ track: null, receivedAt: performance.now() }));

  const update = useCallback((track: TrackInfo | null) => setState({ track, receivedAt: performance.now() }), []);

  useEffect(() => {
    if (hidden) return;
    commands.currentTrack().then((track) => {
      if (track !== undefined) update(track);
    });
  }, [hidden, update]);

  useTauriEvent<TrackInfo | null>(EVENTS.mediaChanged, update);

  return state;
};
