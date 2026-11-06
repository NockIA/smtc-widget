import { TrackInfo } from "../../types/track.types";
import { useProgress } from "../../hooks/useProgress";
import TrackProgress from "./TrackProgress/TrackProgress";
import SourceSwitch from "./SourceSwitch/SourceSwitch";
import MediaControls from "./MediaControls/MediaControls";
import DragHandle from "./DragHandle/DragHandle";
import CloseButton from "./CloseButton/CloseButton";
import VolumeControl from "./VolumeControl/VolumeControl";
import { useVolume } from "../../hooks/useVolume";
import { useHoverPanel } from "../../hooks/useHoverPanel";
import "./SpotifyWidget.css";

interface SpotifyWidgetProps {
  track: TrackInfo | null;
  receivedAt: number;
}

const SpotifyWidget = ({ track, receivedAt }: SpotifyWidgetProps) => {
  const progress = useProgress(track, receivedAt);
  const { volume, setVolume, refresh } = useVolume(track?.source);
  const panels = useHoverPanel(3000);

  return (
    <div
      className="spotify-widget"
      onMouseEnter={() => {
        refresh();
        panels.onMouseEnter();
      }}
      onMouseLeave={panels.onMouseLeave}
    >
      <div className={`widget-handle${panels.visible ? " visible" : ""}`}>
        <CloseButton />
        <DragHandle />
      </div>
      <div className={`widget-content${panels.visible ? " shifted" : ""}`}>
        {track === null ? (
          <p className="track-title">Aucun morceau détecté</p>
        ) : (
          <>
            <p className="track-title">{track.title}</p>
            <p className="track-footer">
              {[track.artist, track.album].filter(Boolean).join(" - ")}
            </p>
          </>
        )}
      </div>
      {track !== null && (
        <div className={`widget-actions${panels.visible ? " visible" : ""}`}>
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
