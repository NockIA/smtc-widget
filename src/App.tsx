import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import SmtcWidget from "./components/SmtcWidget/SmtcWidget";
import { TrackInfo, TrackState } from "./types/track.types";
import { useSystemColors } from "./hooks/useSystemColors";

const App = () => {
  useSystemColors();

  const [state, setState] = useState<TrackState>({
    track: null,
    receivedAt: performance.now(),
  });

  useEffect(() => {
    const update = (track: TrackInfo | null) =>
      setState({ track, receivedAt: performance.now() });

    const fetchTrack = () =>
      invoke<TrackInfo | null>("current_track").then(update).catch(console.error);
    fetchTrack();

    const unlistenTrack = listen<TrackInfo | null>("media-changed", (event) =>
      update(event.payload),
    );

    // Caché par une application plein écran : animations en pause, puis recalage au retour.
    const unlistenVisibility = listen<boolean>("widget-visibility", (event) => {
      document.documentElement.classList.toggle("widget-hidden", event.payload);
      if (!event.payload) fetchTrack();
    });

    return () => {
      unlistenTrack.then((fn) => fn());
      unlistenVisibility.then((fn) => fn());
    };
  }, []);

  return <SmtcWidget track={state.track} receivedAt={state.receivedAt} />;
};

export default App;
