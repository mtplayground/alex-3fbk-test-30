import { apiRequest } from '../auth/api';

export type CreatePostPayload = {
  caption?: string;
  location?: string;
  media_ids: string[];
};

export type PostResponse = {
  id: string;
  caption: string;
  location?: string | null;
  created_at: string;
};

export function createPost(payload: CreatePostPayload) {
  return apiRequest<PostResponse>('/posts', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}
