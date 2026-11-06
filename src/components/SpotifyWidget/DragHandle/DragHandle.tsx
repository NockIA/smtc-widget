import "./DragHandle.css";

const DOTS = [1.5, 6.5, 11.5, 16.5].flatMap((cy) =>
  [1.5, 6.5].map((cx) => ({ cx, cy })),
);

const DragHandle = () => (
  <div className="drag-handle" data-tauri-drag-region title="Déplacer">
    <svg width="8" height="18" viewBox="0 0 8 18" aria-hidden="true">
      {DOTS.map(({ cx, cy }) => (
        <circle key={`${cx}-${cy}`} cx={cx} cy={cy} r="1.25" />
      ))}
    </svg>
  </div>
);

export default DragHandle;
