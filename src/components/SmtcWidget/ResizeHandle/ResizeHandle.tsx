import { PointerEvent } from "react";

import { commands } from "@/utils/commands.utils";

import "./ResizeHandle.css";

const ResizeHandle = () => {
  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    // Garde le curseur de redimensionnement tant que le bouton est enfoncé, même hors de la zone.
    event.currentTarget.setPointerCapture(event.pointerId);
    commands.startResize();
  };

  return (
    <div
      className="resize-handle"
      title="Redimensionner (double-clic : taille d'origine)"
      onPointerDown={onPointerDown}
      onDblClick={() => commands.resetSize()}
    />
  );
};

export default ResizeHandle;
