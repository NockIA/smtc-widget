import { TrackInfo } from "../../types/track.types";
import { useProgress } from "../../hooks/useProgress";
import TrackProgress from "./TrackProgress/TrackProgress";
import SourceSwitch from "./SourceSwitch/SourceSwitch";
import MediaControls from "./MediaControls/MediaControls";
import DragHandle from "./DragHandle/DragHandle";
import VolumeControl from "./VolumeControl/VolumeControl";
import { useVolume } from "../../hooks/useVolume";
import "./SpotifyWidget.css";

interface SpotifyWidgetProps {
  track: TrackInfo | null;
  receivedAt: number;
}

const SpotifyWidget = ({ track, receivedAt }: SpotifyWidgetProps) => {
  const progress = useProgress(track, receivedAt);
  const { volume, setVolume, refresh } = useVolume(track?.source);

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
    <div className="spotify-widget" onMouseEnter={refresh}>
      <DragHandle />
      {content}
      {track !== null && (
        <div className="widget-actions">
          <MediaControls isPlaying={track.isPlaying} />
          {volume !== null && (
            <VolumeControl volume={volume} onChange={setVolume} />
          )}
          {track.sessionCount > 1 && <SourceSwitch />}
        </div>
      )}
      {progress !== null && <TrackProgress progress={progress} />}
    </div>
  );
};

export default SpotifyWidget;
