import { useRef, useState } from "react";

/** Tours joués à l'apparition d'un texte, avant l'arrêt (hors survol). */
const LOOPS_ON_CHANGE = 2;

/**
 * Indique si le défilement est actif : quelques tours à l'apparition du texte, puis en continu tant que `hovered`.
 * `onLoopEnd` est à brancher sur la fin de chaque tour ; le tour en cours se termine toujours avant l'arrêt.
 * À utiliser dans un composant remonté à chaque nouveau texte (`key`), ce qui remet les tours à zéro.
 */
export const useMarqueeLoops = (hovered: boolean) => {
  const [scrolling, setScrolling] = useState(true);
  const [wasHovered, setWasHovered] = useState(hovered);
  const loopsLeft = useRef(LOOPS_ON_CHANGE);

  if (hovered !== wasHovered) {
    setWasHovered(hovered);
    if (hovered) setScrolling(true);
  }

  const onLoopEnd = () => {
    loopsLeft.current = Math.max(0, loopsLeft.current - 1);
    if (loopsLeft.current === 0 && !hovered) setScrolling(false);
  };

  return { scrolling, onLoopEnd };
};
