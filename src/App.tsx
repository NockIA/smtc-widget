import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import SpotifyWidget from "./components/SpotifyWidget/SpotifyWidget";
import { TrackInfo } from "./types/track.types";

const App = () => {
  const [track, setTrack] = useState<TrackInfo | null>(null);

  useEffect(() => {
    invoke<TrackInfo | null>("current_track")
      .then(setTrack)
      .catch(console.error);

    const unlisten = listen<TrackInfo | null>("media-changed", (event) => {
      console.log("media-changed", event.payload);
      setTrack(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return <SpotifyWidget track={track} />;
};

export default App;
