import { CSSProperties, useEffect, useLayoutEffect, useRef, useState } from "react";
import "./MarqueeText.css";

const SPEED_PX_PER_S = 40;
const MIN_DURATION_S = 4;
const GAP_PX = 40;
// Le défilement occupe 85 % d'un cycle, le reste est la pause au début.
const SCROLL_RATIO = 0.85;
/** Tours joués à chaque changement de texte, avant l'arrêt (hors survol). */
const LOOPS_ON_CHANGE = 2;

interface MarqueeTextProps {
  text: string;
  /** Tant que c'est vrai, le texte défile sans s'arrêter. */
  hovered: boolean;
  className?: string;
}

/**
 * Fait défiler le texte qui dépasse : quelques tours quand il change, puis en continu au survol.
 * Le tour en cours se termine toujours avant l'arrêt, pour éviter un saut.
 */
const MarqueeText = ({ text, hovered, className }: MarqueeTextProps) => {
  const containerRef = useRef<HTMLParagraphElement>(null);
  const textRef = useRef<HTMLSpanElement>(null);
  const [textWidth, setTextWidth] = useState(0);
  const [overflowing, setOverflowing] = useState(false);
  const [scrolling, setScrolling] = useState(true);
  const loopsLeft = useRef(LOOPS_ON_CHANGE);
  const hoveredRef = useRef(hovered);

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

  useEffect(() => {
    loopsLeft.current = LOOPS_ON_CHANGE;
    setScrolling(true);
  }, [text]);

  useEffect(() => {
    hoveredRef.current = hovered;
    if (hovered) setScrolling(true);
  }, [hovered]);

  const onLoopEnd = () => {
    loopsLeft.current = Math.max(0, loopsLeft.current - 1);
    if (loopsLeft.current === 0 && !hoveredRef.current) setScrolling(false);
  };

  const active = overflowing && scrolling;
  const distance = textWidth + GAP_PX;
  const duration = Math.max(MIN_DURATION_S, distance / SPEED_PX_PER_S / SCROLL_RATIO);

  return (
    <p ref={containerRef} className={`marquee${active ? "" : " idle"} ${className ?? ""}`}>
      <span
        key={text}
        className={`marquee-track${active ? " scrolling" : ""}`}
        onAnimationIteration={onLoopEnd}
        style={
          {
            "--marquee-gap": `${GAP_PX}px`,
            "--marquee-distance": `${distance}px`,
            "--marquee-duration": `${duration}s`,
          } as CSSProperties
        }
      >
        <span ref={textRef} className="marquee-item">
          {text}
        </span>
        {active && (
          <span className="marquee-item" aria-hidden="true">
            {text}
          </span>
        )}
      </span>
    </p>
  );
};

export default MarqueeText;
