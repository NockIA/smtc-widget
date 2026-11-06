import "./TrackProgress.css";

interface TrackProgressProps {
  progress: number;
}

const TrackProgress = ({ progress }: TrackProgressProps) => {
  return (
    <div className="progress">
      <div className="progress-bar" style={{ width: `${progress * 100}%` }} />
    </div>
  );
};

export default TrackProgress;
