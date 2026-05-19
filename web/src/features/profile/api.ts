import { apiRequest } from '../auth/api';

export type Profile = {
  id: string;
  email?: string | null;
  handle: string;
  display_name: string;
  bio: string;
  link?: string | null;
  avatar_key?: string | null;
  is_private: boolean;
  email_verified: boolean;
};

export type UpdateProfilePayload = {
  display_name?: string;
  bio?: string;
  link?: string | null;
  is_private?: boolean;
};

export type AvatarUploadResponse = {
  key: string;
  upload_url: string;
  method: 'PUT';
  expires_in: number;
  user: Profile;
};

export type FollowState = 'accepted' | 'pending' | 'none';

export type FollowResponse = {
  follower_id: string;
  followee_id: string;
  state: FollowState;
};

export type FollowUser = {
  id: string;
  handle: string;
  display_name: string;
  avatar_key?: string | null;
  is_private: boolean;
};

export type FollowUsersResponse = {
  users: FollowUser[];
};

export function getProfile(handle: string) {
  return apiRequest<Profile>(`/users/${encodeURIComponent(handle)}`);
}

export function updateProfile(payload: UpdateProfilePayload) {
  return apiRequest<Profile>('/me', {
    method: 'PATCH',
    body: JSON.stringify(payload),
  });
}

export function requestAvatarUpload(contentType: string) {
  return apiRequest<AvatarUploadResponse>('/me/avatar', {
    method: 'POST',
    body: JSON.stringify({ content_type: contentType }),
  });
}

export function followUser(handle: string) {
  return apiRequest<FollowResponse>(`/users/${encodeURIComponent(handle)}/follow`, {
    method: 'POST',
  });
}

export function unfollowUser(handle: string) {
  return apiRequest<FollowResponse>(`/users/${encodeURIComponent(handle)}/follow`, {
    method: 'DELETE',
  });
}

export function getFollowers(handle: string) {
  return apiRequest<FollowUsersResponse>(`/users/${encodeURIComponent(handle)}/followers`);
}

export function getFollowing(handle: string) {
  return apiRequest<FollowUsersResponse>(`/users/${encodeURIComponent(handle)}/following`);
}

export async function uploadAvatarBlob(uploadUrl: string, blob: Blob) {
  const response = await fetch(uploadUrl, {
    method: 'PUT',
    body: blob,
    headers: {
      'Content-Type': blob.type || 'image/jpeg',
    },
  });

  if (!response.ok) {
    throw new Error(`Avatar upload failed with status ${response.status}`);
  }
}
