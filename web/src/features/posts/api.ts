import { apiRequest } from '../auth/api';

type JsonObject = Record<string, unknown>;

export type CreatePostPayload = {
  caption?: string;
  location?: string;
  media_ids: string[];
};

export type PostAuthor = {
  id: string;
  handle: string;
};

export type PostMedia = {
  media_id: string;
  position: number;
  kind: 'image' | 'video';
  original_key: string;
  variants: JsonObject;
  width?: number | null;
  height?: number | null;
  duration_ms?: number | null;
};

export type PostMention = {
  user_id: string;
  handle: string;
  position: number;
};

export type PostResponse = {
  id: string;
  author: PostAuthor;
  caption: string;
  location?: string | null;
  created_at: string;
  media: PostMedia[];
  hashtags: string[];
  mentions: PostMention[];
};

export type CommentAuthor = {
  id: string;
  handle: string;
};

export type Comment = {
  id: string;
  post_id: string;
  parent_id?: string | null;
  author: CommentAuthor;
  body: string;
  created_at: string;
  replies: Comment[];
};

export type CommentsResponse = {
  comments: Comment[];
};

export type ToggleCountResponse = {
  active: boolean;
  count: number;
};

export function createPost(payload: CreatePostPayload) {
  return apiRequest<PostResponse>('/posts', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function getPost(id: string) {
  return apiRequest<PostResponse>(`/posts/${encodeURIComponent(id)}`);
}

export function getPostComments(postId: string) {
  return apiRequest<CommentsResponse>(`/posts/${encodeURIComponent(postId)}/comments`);
}

export function createComment(postId: string, body: string, parentId?: string | null) {
  return apiRequest<Comment>(`/posts/${encodeURIComponent(postId)}/comments`, {
    method: 'POST',
    body: JSON.stringify({
      body,
      parent_id: parentId ?? null,
    }),
  });
}

export function togglePostLike(postId: string) {
  return apiRequest<ToggleCountResponse>(`/posts/${encodeURIComponent(postId)}/like`, {
    method: 'POST',
  });
}

export function togglePostSave(postId: string) {
  return apiRequest<ToggleCountResponse>(`/posts/${encodeURIComponent(postId)}/save`, {
    method: 'POST',
  });
}
