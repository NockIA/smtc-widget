import { MouseEvent } from "react";

import { useAlbumArt } from "@/hooks/useAlbumArt";
import { useCurrentTrack } from "@/hooks/useCurrentTrack";
import { useHoverPanel } from "@/hooks/useHoverPanel";
import { useReservedWidth } from "@/hooks/useReservedWidth";
import { useVolume } from "@/hooks/useVolume";
import { useWidgetHidden } from "@/hooks/useWidgetHidden";
import { cx } from "@/utils/classNames.utils";
import { commands } from "@/utils/commands.utils";

import DragHandle from "./DragHandle/DragHandle";
import MediaControls from "./MediaControls/MediaControls";
import ResizeHandle from "./ResizeHandle/ResizeHandle";
import TrackProgress from "./TrackProgress/TrackProgress";
import TrackText from "./TrackText/TrackText";
import VolumeControl from "./VolumeControl/VolumeControl";
import "./SmtcWidget.css";

const PANELS_HIDE_DELAY_MS = 3000;

const SmtcWidget = () => {
  const hidden = useWidgetHidden();
  const { track, receivedAt } = useCurrentTrack(hidden);
  const { volume, setVolume } = useVolume(track?.source);
  const panels = useHoverPanel(PANELS_HIDE_DELAY_MS);
  const albumArt = useAlbumArt(track, receivedAt);
  const { containerRef, panelRef } = useReservedWidth(panels.visible && track !== null);

  const onMouseEnter = () => {
    panels.onMouseEnter();
    commands.refreshSystemColors();
  };

  const onContextMenu = (event: MouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    commands.showContextMenu(event.clientX);
  };

  return (
    <div
      ref={containerRef}
      className="smtc-widget"
      onMouseEnter={onMouseEnter}
      onMouseLeave={panels.onMouseLeave}
      onContextMenu={onContextMenu}
    >
      <div className={cx("widget-handle", panels.visible && "visible")}>
        <DragHandle />
        <ResizeHandle />
      </div>
      <div className={cx("widget-content", panels.visible && "shifted")}>
        {albumArt && <img className="album-art" src={albumArt} alt="" />}
        <TrackText track={track} hovered={panels.visible} />
      </div>
      {track !== null && (
        <>
          <div ref={panelRef} className={cx("widget-actions", panels.visible && "visible")}>
            <MediaControls isPlaying={track.isPlaying} />
            {volume !== null && <VolumeControl volume={volume} onChange={setVolume} />}
          </div>
          {track.durationMs > 0 && <TrackProgress track={track} receivedAt={receivedAt} hidden={hidden} />}
        </>
      )}
    </div>
  );
};

export default SmtcWidget;
