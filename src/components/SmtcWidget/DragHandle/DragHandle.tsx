import DragHandleIcon from "@/assets/icons/drag-handle.svg?react";
import "./DragHandle.css";

const DragHandle = () => (
  <div className="drag-handle" data-tauri-drag-region title="Déplacer">
    <DragHandleIcon />
  </div>
);

export default DragHandle;
