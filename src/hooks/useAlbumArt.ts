import { useEffect, useRef, useState } from "react";

import { TrackInfo } from "@/types/track.types";
import { commands } from "@/utils/commands.utils";

/** Certaines sources envoient la pochette après le titre ; au-delà, on considère qu'il n'y en a pas. */
const MAX_ATTEMPTS = 3;

interface AlbumArt {
  key: string;
  url: string | null;
}

/** Avancement de la recherche de pochette pour un morceau (`key`). */
interface Lookup {
  key: string | null;
  attempts: number;
  found: boolean;
}

const newLookup = (key: string | null): Lookup => ({ key, attempts: 0, found: false });

/** Pochette du morceau (data URL), ou null. Chaque nouvel événement du morceau (`receivedAt`) peut déclencher une tentative. */
export const useAlbumArt = (track: TrackInfo | null, receivedAt: number) => {
  const [art, setArt] = useState<AlbumArt | null>(null);
  const lookup = useRef(newLookup(null));

  const key = track ? [track.source, track.title, track.artist, track.album].join("|") : null;

  useEffect(() => {
    if (lookup.current.key !== key) lookup.current = newLookup(key);
    if (key === null) return;

    const current = lookup.current;
    if (current.found || current.attempts >= MAX_ATTEMPTS) return;
    current.attempts += 1;

    let cancelled = false;
    commands.albumArt().then((url) => {
      if (cancelled || url === undefined) return;
      if (url) current.found = true;
      setArt({ key, url });
    });
    return () => {
      cancelled = true;
    };
  }, [key, receivedAt]);

  return art?.key === key ? art.url : null;
};
