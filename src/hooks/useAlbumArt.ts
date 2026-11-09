import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TrackInfo } from "../types/track.types";
import { handleCommandError } from "../utils/error.utils";

/** Certaines sources envoient la pochette après le titre ; au-delà, on considère qu'il n'y en a pas. */
const MAX_ATTEMPTS = 3;

interface AlbumArt {
  key: string;
  url: string | null;
}

/** Pochette du morceau (data URL), ou null. */
export const useAlbumArt = (track: TrackInfo | null, receivedAt: number) => {
  const [art, setArt] = useState<AlbumArt | null>(null);
  const attempts = useRef<{ key: string | null; count: number; found: boolean }>({
    key: null,
    count: 0,
    found: false,
  });

  const key = track ? [track.source, track.title, track.artist, track.album].join("|") : null;

  useEffect(() => {
    if (key === null) {
      attempts.current = { key: null, count: 0, found: false };
      setArt(null);
      return;
    }
    if (attempts.current.key !== key) {
      attempts.current = { key, count: 0, found: false };
    }
    const current = attempts.current;
    if (current.found || current.count >= MAX_ATTEMPTS) return;
    current.count += 1;

    let cancelled = false;
    invoke<string | null>("album_art")
      .then((url) => {
        if (cancelled) return;
        if (url) current.found = true;
        setArt({ key, url });
      })
      .catch(handleCommandError("album_art"));
    return () => {
      cancelled = true;
    };
  }, [key, receivedAt]);

  return art?.key === key ? art.url : null;
};
