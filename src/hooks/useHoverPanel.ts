import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Panneau affiché dès que la souris survole la zone, et replié `hideDelayMs`
 * après qu'elle l'a quittée (annulé si elle revient avant).
 */
export const useHoverPanel = (hideDelayMs: number) => {
  const [visible, setVisible] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  const onMouseEnter = useCallback(() => {
    window.clearTimeout(timer.current);
    setVisible(true);
  }, []);

  const onMouseLeave = useCallback(() => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setVisible(false), hideDelayMs);
  }, [hideDelayMs]);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  return { visible, onMouseEnter, onMouseLeave };
};
