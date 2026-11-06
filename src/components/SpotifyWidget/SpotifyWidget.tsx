import { TrackInfo } from "../../types/track.types";
import { useProgress } from "../../hooks/useProgress";
import TrackProgress from "./TrackProgress/TrackProgress";
import "./SpotifyWidget.css";

interface SpotifyWidgetProps {
  track: TrackInfo | null;
  receivedAt: number;
}

const SpotifyWidget = ({ track, receivedAt }: SpotifyWidgetProps) => {
  const progress = useProgress(track, receivedAt);

  let content: React.ReactNode;
  if (track === null) {
    content = <p className="track-title">Aucun morceau détecté</p>;
  } else {
    content = (
      <>
        <p className="track-title">{track.title}</p>
        <p className="track-footer">
          {[track.artist, track.album].filter(Boolean).join(" - ")}
        </p>
      </>
    );
  }
  return (
    <div className="spotify-widget">
      {content}
      {progress !== null && <TrackProgress progress={progress} />}
    </div>
  );
};

export default SpotifyWidget;
