export interface TrackInfo {
  title: string;
  artist: string;
  album: string;
  positionMs: number;
  durationMs: number;
  isPlaying: boolean;
  source: string;
  sessionCount: number;
}

export interface TrackState {
  track: TrackInfo | null;
  /** Instant (performance.now) où la position a été reçue, pour l'interpolation. */
  receivedAt: number;
}
