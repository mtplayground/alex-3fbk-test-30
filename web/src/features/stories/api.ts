import { apiRequest } from '../auth/api';

type JsonObject = Record<string, unknown>;

export type StoryAuthor = {
  id: string;
  handle: string;
  display_name: string;
  avatar_key?: string | null;
};

export type StoryMedia = {
  media_id: string;
  kind: 'image' | 'video';
  status: string;
  original_key: string;
  variants: JsonObject;
  width?: number | null;
  height?: number | null;
  duration_ms?: number | null;
};

export type StoryResponse = {
  id: string;
  author: StoryAuthor;
  media: StoryMedia;
  created_at: string;
  expires_at: string;
  viewer_count: number;
  viewed_at?: string | null;
};

export type StoryAuthorGroup = {
  author: StoryAuthor;
  stories: StoryResponse[];
};

export type StoriesFeedResponse = {
  authors: StoryAuthorGroup[];
};

export type StoryViewer = {
  id: string;
  handle: string;
  display_name: string;
  avatar_key?: string | null;
  viewed_at: string;
};

export type StoryViewersResponse = {
  viewers: StoryViewer[];
};

export function getStoriesFeed() {
  return apiRequest<StoriesFeedResponse>('/stories/feed');
}

export async function markStoryViewed(storyId: string) {
  await apiRequest<void>(`/stories/${encodeURIComponent(storyId)}/view`, {
    method: 'POST',
  });
}

export function getStoryViewers(storyId: string) {
  return apiRequest<StoryViewersResponse>(`/stories/${encodeURIComponent(storyId)}/viewers`);
}
