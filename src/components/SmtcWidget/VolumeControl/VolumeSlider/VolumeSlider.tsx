import { CSSProperties } from "react";
import "./VolumeSlider.css";

interface VolumeSliderProps {
  /** 0 à 1. */
  volume: number;
  onChange: (volume: number) => void;
}

const VolumeSlider = ({ volume, onChange }: VolumeSliderProps) => (
  <input
    className="volume-slider"
    type="range"
    min={0}
    max={1}
    step={0.01}
    value={volume}
    style={{ "--fill": `${volume * 100}%` } as CSSProperties}
    title={`Volume : ${Math.round(volume * 100)} %`}
    aria-label="Volume"
    onInput={(event) => onChange(Number(event.currentTarget.value))}
  />
);

export default VolumeSlider;
