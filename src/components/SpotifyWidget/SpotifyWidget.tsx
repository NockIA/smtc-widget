import { invoke } from "@tauri-apps/api/core";
import { TrackInfo } from "../../types/track.types";
import { useProgress } from "../../hooks/useProgress";
import TrackProgress from "./TrackProgress/TrackProgress";
import MediaControls from "./MediaControls/MediaControls";
import DragHandle from "./DragHandle/DragHandle";
import MarqueeText from "./MarqueeText/MarqueeText";
import VolumeControl from "./VolumeControl/VolumeControl";
import { useVolume } from "../../hooks/useVolume";
import { useHoverPanel } from "../../hooks/useHoverPanel";
import { useAlbumArt } from "../../hooks/useAlbumArt";
import "./SpotifyWidget.css";

interface SpotifyWidgetProps {
  track: TrackInfo | null;
  receivedAt: number;
}

const SpotifyWidget = ({ track, receivedAt }: SpotifyWidgetProps) => {
  const progress = useProgress(track, receivedAt);
  const { volume, setVolume, refresh } = useVolume(track?.source);
  const panels = useHoverPanel(3000);
  const albumArt = useAlbumArt(track, receivedAt);

  return (
    <div
      className="spotify-widget"
      onMouseEnter={() => {
        refresh();
        panels.onMouseEnter();
      }}
      onMouseLeave={panels.onMouseLeave}
      onContextMenu={(event) => {
        event.preventDefault();
        invoke("show_context_menu", { x: event.clientX }).catch(console.error);
      }}
    >
      <div className={`widget-handle${panels.visible ? " visible" : ""}`}>
        <DragHandle />
      </div>
      <div className={`widget-content${panels.visible ? " shifted" : ""}`}>
        {albumArt && <img className="album-art" src={albumArt} alt="" />}
        <div className="track-text">
          {track === null ? (
            <p className="track-title">Aucun morceau détecté</p>
          ) : (
            <>
              <MarqueeText className="track-title" text={track.title} />
              <MarqueeText
                className="track-footer"
                text={[track.artist, track.album].filter(Boolean).join(" - ")}
              />
            </>
          )}
        </div>
      </div>
      {track !== null && (
        <div className={`widget-actions${panels.visible ? " visible" : ""}`}>
          <MediaControls isPlaying={track.isPlaying} />
          {volume !== null && (
            <VolumeControl volume={volume} onChange={setVolume} />
          )}
        </div>
      )}
      {progress !== null && <TrackProgress progress={progress} />}
    </div>
  );
};

export default SpotifyWidget;
