import { useEffect, useState } from "react";
import IconButton from "../../IconButton/IconButton";
import VolumeSlider from "../VolumeSlider/VolumeSlider";
import "./VolumeControl.css";

interface VolumeControlProps {
  /** 0 à 1. */
  volume: number;
  onChange: (volume: number) => void;
}

const VolumeControl = ({ volume, onChange }: VolumeControlProps) => {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const close = () => setOpen(false);
    document.documentElement.addEventListener("mouseleave", close);
    return () =>
      document.documentElement.removeEventListener("mouseleave", close);
  }, []);

  return (
    <div className="volume-control">
      <IconButton
        label={open ? "Masquer le volume" : "Volume"}
        onClick={() => setOpen((value) => !value)}
      >
        <path d="M11 5L6 9H2v6h4l5 4V5z" />
        <g fill="none">
          {volume === 0 ? (
            <>
              <path d="M23 9l-6 6" />
              <path d="M17 9l6 6" />
            </>
          ) : (
            <path d="M15.5 8.5a5 5 0 0 1 0 7" />
          )}
        </g>
      </IconButton>
      {open && <VolumeSlider volume={volume} onChange={onChange} />}
    </div>
  );
};

export default VolumeControl;
