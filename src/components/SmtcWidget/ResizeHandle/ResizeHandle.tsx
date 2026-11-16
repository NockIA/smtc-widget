import { PointerEvent } from "react";

import { commands, ResizeHandleKind } from "@/utils/commands.utils";

import "./ResizeHandle.css";

interface ResizeHandleProps {
  kind: ResizeHandleKind;
}

const TITLES: Record<ResizeHandleKind, string> = {
  corner: "Agrandir (double-clic : taille d'origine)",
  left: "Élargir (double-clic : taille d'origine)",
  right: "Élargir (double-clic : taille d'origine)",
};

const ResizeHandle = ({ kind }: ResizeHandleProps) => {
  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    // Garde le curseur de redimensionnement tant que le bouton est enfoncé, même hors de la zone.
    event.currentTarget.setPointerCapture(event.pointerId);
    commands.startResize(kind);
  };

  return (
    <div
      className={`resize-handle resize-handle-${kind}`}
      title={TITLES[kind]}
      onPointerDown={onPointerDown}
      onDblClick={() => commands.resetSize()}
    />
  );
};

export default ResizeHandle;
