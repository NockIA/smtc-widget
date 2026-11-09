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

    const observer = new ResizeObserver(measure);
    observer.observe(container);
    observer.observe(content);
    return () => observer.disconnect();
  }, [text]);

  return { containerRef, textRef, textWidth, overflowing };
};
