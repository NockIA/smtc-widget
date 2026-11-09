import { Component, ErrorInfo, PropsWithChildren } from "react";
import MarqueeText from "../SmtcWidget/MarqueeText/MarqueeText";
import IconButton from "../IconButton/IconButton";
import { logger } from "../../utils/logger";
import "./ErrorBoundary.css";

const RETRY_DELAY_MS = 5000;

interface ErrorBoundaryState {
  failed: boolean;
  hovered: boolean;
}

export default class ErrorBoundary extends Component<PropsWithChildren, ErrorBoundaryState> {
  state: ErrorBoundaryState = { failed: false, hovered: false };
  private retryTimer: number | undefined;

  static getDerivedStateFromError(): Partial<ErrorBoundaryState> {
    return { failed: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logger.error("erreur de rendu", `${error.stack ?? error.message}${info.componentStack ?? ""}`);
    this.retryTimer = window.setTimeout(this.retry, RETRY_DELAY_MS);
  }

  componentWillUnmount() {
    window.clearTimeout(this.retryTimer);
  }

  private retry = () => {
    window.clearTimeout(this.retryTimer);
    this.setState({ failed: false, hovered: false });
  };

  render() {
    if (!this.state.failed) return this.props.children;
    const { hovered } = this.state;
    return (
      <div
        className="smtc-widget error-fallback"
        role="alert"
        onMouseEnter={() => this.setState({ hovered: true })}
        onMouseLeave={() => this.setState({ hovered: false })}
      >
        <div className="widget-content">
          <svg className="error-fallback-icon" width="26" height="26" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M12 2.5 1.5 21h21L12 2.5Z"
              fill="currentColor"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinejoin="round"
            />
            <path d="M12 9.5v5" stroke="#fff" strokeWidth="2" strokeLinecap="round" />
            <circle cx="12" cy="17.5" r="1.2" fill="#fff" />
          </svg>
          <div className="track-text">
            <MarqueeText className="track-title" text="Une erreur est survenue dans le widget" hovered={hovered} />
            <MarqueeText
              className="track-footer"
              text="Nouvelle tentative automatique dans quelques secondes"
              hovered={hovered}
            />
          </div>
        </div>
        <div className="error-fallback-action">
          <IconButton label="Réessayer" onClick={this.retry}>
            <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" fill="none" />
            <path d="M21 3v5h-5" fill="none" />
          </IconButton>
        </div>
      </div>
    );
  }
}
