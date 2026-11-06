import { TrackInfo } from "../../types/track.types";
import "./SpotifyWidget.css";

interface SpotifyWidgetProps {
  track: TrackInfo | null;
}

const SpotifyWidget = ({ track }: SpotifyWidgetProps) => {
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
  return <div className="spotify-widget">{content}</div>;
};

export default SpotifyWidget;
