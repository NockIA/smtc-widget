import "./DragHandle.css";

const DOTS = [2, 7, 12].flatMap((cy) => [2, 7].map((cx) => ({ cx, cy })));

const DragHandle = () => (
  <div className="drag-handle" data-tauri-drag-region title="Déplacer">
    <svg width="9" height="14" viewBox="0 0 9 14" aria-hidden="true">
      {DOTS.map(({ cx, cy }) => (
        <circle key={`${cx}-${cy}`} cx={cx} cy={cy} r="1.5" />
      ))}
    </svg>
  </div>
);

export default DragHandle;
