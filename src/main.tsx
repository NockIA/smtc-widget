import React from "react";
import ReactDOM from "react-dom/client";

import "./styles/index.css";
import App from "./App";
import ErrorBoundary from "./components/ErrorBoundary/ErrorBoundary";
import { useSystemColors } from "./hooks/useSystemColors";
import { logger } from "./utils/logger";
import { applyBackground } from "./utils/systemColors";

applyBackground(null);

// L'Error Boundary ne voit que les erreurs de rendu : le reste est journalisé ici.
window.addEventListener("error", (event) => logger.error("erreur non interceptée", event.error ?? event.message));
window.addEventListener("unhandledrejection", (event) =>
  logger.error("promesse rejetée non interceptée", event.reason),
);

/** Les couleurs système vivent au-dessus de l'Error Boundary : le repli garde le fond du thème. */
const Root = () => {
  useSystemColors();
  return (
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  );
};

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
