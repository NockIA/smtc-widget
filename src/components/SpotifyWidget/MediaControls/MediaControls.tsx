import { invoke } from "@tauri-apps/api/core";
import IconButton from "../../IconButton/IconButton";

type MediaAction = "previous" | "togglePlayPause" | "next";

const sendAction = (action: MediaAction) =>
  invoke("media_control", { action }).catch(console.error);

interface MediaControlsProps {
  isPlaying: boolean;
}

/** Précédent, lecture / pause, suivant. */
const MediaControls = ({ isPlaying }: MediaControlsProps) => (
  <>
    <IconButton label="Précédent" onClick={() => sendAction("previous")}>
      <path d="M19 20L9 12l10-8v16z" />
      <path d="M5 4v16" fill="none" />
    </IconButton>
    <IconButton
      label={isPlaying ? "Pause" : "Lecture"}
      onClick={() => sendAction("togglePlayPause")}
    >
      {isPlaying ? (
        <>
          <rect x="6" y="4" width="4" height="16" />
          <rect x="14" y="4" width="4" height="16" />
        </>
      ) : (
        <path d="M6 4l14 8-14 8V4z" />
      )}
    </IconButton>
    <IconButton label="Suivant" onClick={() => sendAction("next")}>
      <path d="M5 4l10 8-10 8V4z" />
      <path d="M19 4v16" fill="none" />
    </IconButton>
  </>
);

export default MediaControls;
