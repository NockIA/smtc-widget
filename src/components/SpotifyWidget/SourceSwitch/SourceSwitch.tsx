import { invoke } from "@tauri-apps/api/core";
import IconButton from "../../IconButton/IconButton";

/** Bouton qui passe à la source audio suivante. */
const SourceSwitch = () => (
  <IconButton
    label="Changer de source audio"
    onClick={() => invoke("next_session").catch(console.error)}
  >
    <g fill="none">
      <path d="M17 3l4 4-4 4" />
      <path d="M3 7h18" />
      <path d="M7 21l-4-4 4-4" />
      <path d="M21 17H3" />
    </g>
  </IconButton>
);

export default SourceSwitch;
