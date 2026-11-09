import MarqueeText from "@/components/SmtcWidget/MarqueeText/MarqueeText";
import { TrackInfo } from "@/types/track.types";

interface TrackTextProps {
  track: TrackInfo | null;
  hovered: boolean;
}

const TrackText = ({ track, hovered }: TrackTextProps) => (
  <div className="track-text">
    {track === null ? (
      <p className="track-title">Aucun morceau détecté</p>
    ) : (
      <>
        <MarqueeText className="track-title" text={track.title} hovered={hovered} />
        <MarqueeText
          className="track-footer"
          text={[track.artist, track.album].filter(Boolean).join(" - ")}
          hovered={hovered}
        />
      </>
    )}
  </div>
);

export default TrackText;
