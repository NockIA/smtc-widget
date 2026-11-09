import { CSSProperties } from "react";

import { cx } from "@/utils/classNames.utils";

import { useMarqueeLoops } from "./useMarqueeLoops";
import { useOverflow } from "./useOverflow";
import "./MarqueeText.css";

const SPEED_PX_PER_S = 40;
const MIN_DURATION_S = 4;
const GAP_PX = 40;
// Le défilement occupe 85 % d'un cycle, le reste est la pause au début.
const SCROLL_RATIO = 0.85;

interface MarqueeTextProps {
  text: string;
  /** Tant que c'est vrai, le texte défile sans s'arrêter. */
  hovered: boolean;
  className?: string;
}

/** Fait défiler le texte qui dépasse : quelques tours quand il change, puis en continu au survol. */
const MarqueeLine = ({ text, hovered, className }: MarqueeTextProps) => {
  const { containerRef, textRef, textWidth, overflowing } = useOverflow(text);
  const { scrolling, onLoopEnd } = useMarqueeLoops(hovered);

  const active = overflowing && scrolling;
  const distance = textWidth + GAP_PX;
  const duration = Math.max(MIN_DURATION_S, distance / SPEED_PX_PER_S / SCROLL_RATIO);

  return (
    <p ref={containerRef} className={cx("marquee", !active && "idle", className)}>
      <span
        className={cx("marquee-track", active && "scrolling")}
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

/** Remonté à chaque nouveau texte : le défilement repart du début et rejoue ses tours. */
const MarqueeText = (props: MarqueeTextProps) => <MarqueeLine key={props.text} {...props} />;

export default MarqueeText;
