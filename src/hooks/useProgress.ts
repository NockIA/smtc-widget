import { useEffect, useState } from "react";
import { TrackInfo } from "../types/track.types";

export const useProgress = (track: TrackInfo | null, receivedAt: number): number | null => {
  const [now, setNow] = useState(performance.now());

  useEffect(() => {
    if (!track?.isPlaying) return;
    const id = setInterval(() => setNow(performance.now()), 250);
    return () => clearInterval(id);
  }, [track]);

  if (!track || track.durationMs <= 0) return null;
  const elapsed = track.isPlaying ? Math.max(0, now - receivedAt) : 0;
  return Math.min(1, (track.positionMs + elapsed) / track.durationMs);
};
