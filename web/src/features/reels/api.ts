import { apiRequest } from '../auth/api';

type JsonObject = Record<string, unknown>;

export type ReelAuthor = {
  id: string;
  handle: string;
};

export type ReelMedia = {
  media_id: string;
  kind: 'video';
  status: string;
  original_key: string;
  variants: JsonObject;
  width?: number | null;
  height?: number | null;
  duration_ms?: number | null;
};

export type ReelAudio = {
  title?: string | null;
  artist?: string | null;
  is_original: boolean;
};

export type ReelResponse = {
  id: string;
  author: ReelAuthor;
  caption: string;
  media: ReelMedia;
  duration_ms?: number | null;
  audio: ReelAudio;
  created_at: string;
};

export type ReelsPageResponse = {
  reels: ReelResponse[];
  next_cursor?: string | null;
};

export function getReelsFeed(cursor?: string | null) {
  const params = new URLSearchParams({ limit: '8' });
  if (cursor) {
    params.set('cursor', cursor);
  }

  return apiRequest<ReelsPageResponse>(`/reels/feed?${params.toString()}`);
}

export function getReel(id: string) {
  return apiRequest<ReelResponse>(`/reels/${encodeURIComponent(id)}`);
}
