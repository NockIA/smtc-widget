import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TrackInfo } from "../types/track.types";

interface AlbumArt {
  key: string;
  url: string | null;
}

/**
 * Pochette du morceau (data URL), ou null. Certaines sources l'envoient après le titre :
 * tant qu'elle manque, on retente à chaque mise à jour du morceau.
 */
export const useAlbumArt = (track: TrackInfo | null, receivedAt: number) => {
  const [art, setArt] = useState<AlbumArt | null>(null);
  const foundFor = useRef<string | null>(null);

  const key = track
    ? [track.source, track.title, track.artist, track.album].join("|")
    : null;

  useEffect(() => {
    if (key === null) {
      foundFor.current = null;
      setArt(null);
      return;
    }
    if (foundFor.current === key) return;

    let cancelled = false;
    invoke<string | null>("album_art")
      .then((url) => {
        if (cancelled) return;
        if (url) foundFor.current = key;
        setArt({ key, url });
      })
      .catch(console.error);
    return () => {
      cancelled = true;
    };
  }, [key, receivedAt]);

  return art?.key === key ? art.url : null;
};
