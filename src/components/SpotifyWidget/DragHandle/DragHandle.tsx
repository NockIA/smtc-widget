import "./DragHandle.css";

const DOTS = [1.5, 5.5, 9.5].flatMap((cy) => [1.5, 5.5].map((cx) => ({ cx, cy })));

const DragHandle = () => (
  <div className="drag-handle" data-tauri-drag-region title="Déplacer">
    <svg width="7" height="11" viewBox="0 0 7 11" aria-hidden="true">
      {DOTS.map(({ cx, cy }) => (
        <circle key={`${cx}-${cy}`} cx={cx} cy={cy} r="1" />
      ))}
    </svg>
  </div>
);

export default DragHandle;
