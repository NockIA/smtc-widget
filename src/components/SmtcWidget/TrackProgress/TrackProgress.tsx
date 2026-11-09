import { useEffect, useRef } from "react";
import { TrackInfo } from "../../../types/track.types";
import "./TrackProgress.css";

// La barre avance d'environ 1 px par seconde : une image par seconde suffit.
const UPDATE_INTERVAL_MS = 1000;

interface TrackProgressProps {
  track: TrackInfo;
  /** Instant (performance.now) de réception du morceau, base de l'interpolation. */
  receivedAt: number;
}

const TrackProgress = ({ track, receivedAt }: TrackProgressProps) => {
  const barRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const bar = barRef.current;
    if (!bar) return;

    const render = () => {
      const elapsed = track.isPlaying ? performance.now() - receivedAt : 0;
      const progress = Math.min(1, (track.positionMs + elapsed) / track.durationMs);
      bar.style.transform = `scaleX(${progress})`;
    };
    render();

    if (!track.isPlaying) return;
    const id = window.setInterval(() => {
      // Widget caché par une application plein écran : rien à dessiner.
      if (!document.documentElement.classList.contains("widget-hidden")) render();
    }, UPDATE_INTERVAL_MS);
    return () => window.clearInterval(id);
  }, [track, receivedAt]);

  return (
    <div className="progress">
      <div ref={barRef} className="progress-bar" />
    </div>
  );
};

export default TrackProgress;
