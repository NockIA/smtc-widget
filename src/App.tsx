import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import SpotifyWidget from "./components/SpotifyWidget/SpotifyWidget";
import { TrackInfo, TrackState } from "./types/track.types";

const App = () => {
  const [state, setState] = useState<TrackState>({
    track: null,
    receivedAt: performance.now(),
  });

  useEffect(() => {
    const update = (track: TrackInfo | null) =>
      setState({ track, receivedAt: performance.now() });

    invoke<TrackInfo | null>("current_track").then(update).catch(console.error);

    const unlisten = listen<TrackInfo | null>("media-changed", (event) =>
      update(event.payload),
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return <SpotifyWidget track={state.track} receivedAt={state.receivedAt} />;
};

export default App;
