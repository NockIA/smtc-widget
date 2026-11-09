import { useEffect, useState } from "react";
import IconButton from "../../IconButton/IconButton";
import VolumeIcon from "../../../assets/icons/volume.svg?react";
import VolumeMutedIcon from "../../../assets/icons/volume-muted.svg?react";
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
    return () => document.documentElement.removeEventListener("mouseleave", close);
  }, []);

  return (
    <div className="volume-control">
      <IconButton label={open ? "Masquer le volume" : "Volume"} onClick={() => setOpen((value) => !value)}>
        {volume === 0 ? <VolumeMutedIcon /> : <VolumeIcon />}
      </IconButton>
      {open && <VolumeSlider volume={volume} onChange={onChange} />}
    </div>
  );
};

export default VolumeControl;
