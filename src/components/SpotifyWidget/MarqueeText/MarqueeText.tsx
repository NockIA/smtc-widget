import { CSSProperties, useLayoutEffect, useRef, useState } from "react";
import "./MarqueeText.css";

const SPEED_PX_PER_S = 40;
const MIN_DURATION_S = 4;
const GAP_PX = 40;
// Le défilement occupe 85 % d'un cycle, le reste est la pause au début.
const SCROLL_RATIO = 0.85;

interface MarqueeTextProps {
  text: string;
  className?: string;
}

const MarqueeText = ({ text, className }: MarqueeTextProps) => {
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

  const distance = textWidth + GAP_PX;
  const duration = Math.max(
    MIN_DURATION_S,
    distance / SPEED_PX_PER_S / SCROLL_RATIO,
  );

  return (
    <p ref={containerRef} className={className}>
      <span
        key={text}
        className={`marquee-track${overflowing ? " scrolling" : ""}`}
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
        {overflowing && (
          <span className="marquee-item" aria-hidden="true">
            {text}
          </span>
        )}
      </span>
    </p>
  );
};

export default MarqueeText;
