import { ReactNode } from "react";
import "./IconButton.css";

interface IconButtonProps {
  label: string;
  onClick: () => void;
  children: ReactNode;
}

const IconButton = ({ label, onClick, children }: IconButtonProps) => (
  <button className="icon-button" title={label} aria-label={label} onClick={onClick}>
    {children}
  </button>
);

export default IconButton;
