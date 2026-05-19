import { apiRequest } from '../auth/api';
import type { PostResponse } from '../posts/api';

export type SearchUser = {
  id: string;
  handle: string;
  display_name: string;
  avatar_key?: string | null;
  is_private: boolean;
};

export type SearchHashtag = {
  name: string;
  post_count: number;
};

export type SearchResponse = {
  users: SearchUser[];
  hashtags: SearchHashtag[];
  posts: PostResponse[];
};

export function searchAll(query: string) {
  const params = new URLSearchParams({ q: query, limit: '8' });
  return apiRequest<SearchResponse>(`/search?${params.toString()}`);
}
