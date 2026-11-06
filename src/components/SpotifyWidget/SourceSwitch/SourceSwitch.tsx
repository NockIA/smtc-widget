import { invoke } from "@tauri-apps/api/core";
import "./SourceSwitch.css";

/** Bouton qui passe à la source audio suivante. */
const SourceSwitch = () => (
  <button
    className="source-switch"
    title="Changer de source audio"
    aria-label="Changer de source audio"
    onClick={() => invoke("next_session").catch(console.error)}
  >
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M17 3l4 4-4 4" />
      <path d="M3 7h18" />
      <path d="M7 21l-4-4 4-4" />
      <path d="M21 17H3" />
    </svg>
  </button>
);

export default SourceSwitch;
