import { invoke } from "@tauri-apps/api/core";
import IconButton from "../../IconButton/IconButton";
import NextIcon from "../../../assets/icons/next.svg?react";
import PauseIcon from "../../../assets/icons/pause.svg?react";
import PlayIcon from "../../../assets/icons/play.svg?react";
import PreviousIcon from "../../../assets/icons/previous.svg?react";
import { handleCommandError } from "../../../utils/error.utils";

type MediaAction = "previous" | "togglePlayPause" | "next";

const sendAction = (action: MediaAction) =>
  invoke("media_control", { action }).catch(handleCommandError("media_control"));

interface MediaControlsProps {
  isPlaying: boolean;
}

const MediaControls = ({ isPlaying }: MediaControlsProps) => (
  <>
    <IconButton label="Précédent" onClick={() => sendAction("previous")}>
      <PreviousIcon />
    </IconButton>
    <IconButton label={isPlaying ? "Pause" : "Lecture"} onClick={() => sendAction("togglePlayPause")}>
      {isPlaying ? <PauseIcon /> : <PlayIcon />}
    </IconButton>
    <IconButton label="Suivant" onClick={() => sendAction("next")}>
      <NextIcon />
    </IconButton>
  </>
);

export default MediaControls;
