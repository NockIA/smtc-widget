import NextIcon from "@/assets/icons/next.svg?react";
import PauseIcon from "@/assets/icons/pause.svg?react";
import PlayIcon from "@/assets/icons/play.svg?react";
import PreviousIcon from "@/assets/icons/previous.svg?react";
import IconButton from "@/components/IconButton/IconButton";
import { commands } from "@/utils/commands.utils";

interface MediaControlsProps {
  isPlaying: boolean;
}

const MediaControls = ({ isPlaying }: MediaControlsProps) => (
  <>
    <IconButton label="Précédent" onClick={() => commands.mediaControl("previous")}>
      <PreviousIcon />
    </IconButton>
    <IconButton label={isPlaying ? "Pause" : "Lecture"} onClick={() => commands.mediaControl("togglePlayPause")}>
      {isPlaying ? <PauseIcon /> : <PlayIcon />}
    </IconButton>
    <IconButton label="Suivant" onClick={() => commands.mediaControl("next")}>
      <NextIcon />
    </IconButton>
  </>
);

export default MediaControls;
