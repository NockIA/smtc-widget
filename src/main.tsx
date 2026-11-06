import React from "react";
import ReactDOM from "react-dom/client";

import "./styles/index.css";
import SpotifyWidget from "./components/SpotifyWidget/SpotifyWidget";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SpotifyWidget />
  </React.StrictMode>,
);
