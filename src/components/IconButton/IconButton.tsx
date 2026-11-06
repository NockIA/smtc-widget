import { ReactNode } from "react";
import "./IconButton.css";

interface IconButtonProps {
  label: string;
  onClick: () => void;
  children: ReactNode;
}

const IconButton = ({ label, onClick, children }: IconButtonProps) => (
  <button
    className="icon-button"
    title={label}
    aria-label={label}
    onClick={onClick}
  >
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="currentColor"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  </button>
);

export default IconButton;
