import { useLayoutEffect, useRef } from "react";

/**
 * Publie la largeur d'un panneau superposé dans la variable CSS `--actions-width` du conteneur
 * (0 tant qu'il est caché), pour que le contenu se réduise au lieu de passer dessous.
 */
export const useReservedWidth = (visible: boolean) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const container = containerRef.current;
    const panel = panelRef.current;
    if (!container) return;

    const update = () =>
      container.style.setProperty("--actions-width", `${visible && panel ? panel.offsetWidth : 0}px`);
    update();

    if (!visible || !panel) return;
    const observer = new ResizeObserver(update);
    observer.observe(panel);
    return () => observer.disconnect();
  }, [visible]);

  return { containerRef, panelRef };
};
