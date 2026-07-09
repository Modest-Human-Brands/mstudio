export type MediaType = 'photo' | 'video';

export type MediaMetadata = {
  size: number;
  resolution: string;
  aspectRatio: string;
  bitDepth: string;
  fps?: number;
};

export type MediaItem = {
  slug: string;
  type: MediaType;
  title: string;
  thumbnailUrl?: string;
  metadata: MediaMetadata;
};

// Append to existing content in index.ts
export type CertItem = {
  index: number;
  subject: string;
  issuer: string;
  certificateDerHex: string;
  certificateChainDerHex: string[];
};