import React from "react";
import ReactDOM from "react-dom/client";

import "./styles/index.css";
import App from "./App";
import { applyBackground } from "./utils/systemColors";

applyBackground(null);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
