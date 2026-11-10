import { useLayoutEffect, useRef, useState } from "react";

/** Largeur du texte et indication de débordement du conteneur, remesurées à chaque redimensionnement. */
export const useOverflow = (text: string) => {
  const containerRef = useRef<HTMLParagraphElement>(null);
  const textRef = useRef<HTMLSpanElement>(null);
  const [textWidth, setTextWidth] = useState(0);
  const [overflowing, setOverflowing] = useState(false);

  useLayoutEffect(() => {
    const container = containerRef.current;
    const content = textRef.current;
    if (!container || !content) return;

    const measure = () => {
      setTextWidth(content.offsetWidth);
      setOverflowing(content.offsetWidth > container.clientWidth);
    };
    measure();

    // Mesure reportée à l'image suivante : le rendu qu'elle déclenche change la taille des éléments observés,
    // ce qui, pendant la distribution des notifications, provoque l'erreur « ResizeObserver loop ».
    let frame = 0;
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(measure);
    });
    observer.observe(container);
    observer.observe(content);
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [text]);

  return { containerRef, textRef, textWidth, overflowing };
};
