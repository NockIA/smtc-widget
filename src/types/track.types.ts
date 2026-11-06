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
  /** performance.now() à la réception, pour interpoler la progression. */
  receivedAt: number;
}
