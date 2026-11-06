import { invoke } from "@tauri-apps/api/core";
import "./CloseButton.css";

const CloseButton = () => (
  <button
    className="close-button"
    title="Fermer"
    aria-label="Fermer"
    onClick={() => invoke("quit_app").catch(console.error)}
  >
    <svg
      width="8"
      height="8"
      viewBox="0 0 8 8"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M1 1l6 6" />
      <path d="M7 1L1 7" />
    </svg>
  </button>
);

export default CloseButton;
