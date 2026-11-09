import { Component, ErrorInfo, PropsWithChildren } from "react";
import MarqueeText from "../SmtcWidget/MarqueeText/MarqueeText";
import IconButton from "../IconButton/IconButton";
import RetryIcon from "../../assets/icons/retry.svg?react";
import WarningIcon from "../../assets/icons/warning.svg?react";
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
          <WarningIcon className="error-fallback-icon" />
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
            <RetryIcon />
          </IconButton>
        </div>
      </div>
    );
  }
}
