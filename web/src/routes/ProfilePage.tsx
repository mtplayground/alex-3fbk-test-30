import { useEffect, useMemo, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';
import { getUserPosts, type PostMedia, type PostResponse } from '../features/posts/api';
import {
  followUser,
  getFollowers,
  getFollowing,
  getProfile,
  unfollowUser,
  type FollowState,
  type FollowUser,
} from '../features/profile/api';

const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';

type FollowModal = 'followers' | 'following' | null;

export function ProfilePage() {
  const { handle = '' } = useParams();
  const auth = useAuth();
  const queryClient = useQueryClient();
  const [localFollowState, setLocalFollowState] = useState<FollowState | null>(null);
  const [modal, setModal] = useState<FollowModal>(null);

  useEffect(() => {
    setLocalFollowState(null);
    setModal(null);
  }, [handle]);

  const profileQuery = useQuery({
    queryKey: ['profile', handle],
    queryFn: () => getProfile(handle),
    enabled: Boolean(handle),
  });
  const postsQuery = useQuery({
    queryKey: ['profile-posts', handle],
    queryFn: () => getUserPosts(handle),
    enabled: Boolean(handle),
  });
  const followersQuery = useQuery({
    queryKey: ['followers', handle],
    queryFn: () => getFollowers(handle),
    enabled: Boolean(handle),
  });
  const followingQuery = useQuery({
    queryKey: ['following', handle],
    queryFn: () => getFollowing(handle),
    enabled: Boolean(handle),
  });

  const followMutation = useMutation({
    mutationFn: () => followUser(handle),
    onSuccess: async (response) => {
      setLocalFollowState(response.state);
      await queryClient.invalidateQueries({ queryKey: ['followers', handle] });
    },
  });
  const unfollowMutation = useMutation({
    mutationFn: () => unfollowUser(handle),
    onSuccess: async () => {
      setLocalFollowState('none');
      await queryClient.invalidateQueries({ queryKey: ['followers', handle] });
    },
  });

  const followers = followersQuery.data?.users ?? [];
  const following = followingQuery.data?.users ?? [];
  const posts = postsQuery.data?.posts ?? [];
  const isOwnProfile = auth.user?.handle === handle;
  const acceptedFollowState = auth.user && followers.some((user) => user.id === auth.user?.id) ? 'accepted' : 'none';
  const relationship = localFollowState ?? acceptedFollowState;

  const modalUsers = useMemo(() => {
    if (modal === 'followers') {
      return followers;
    }
    if (modal === 'following') {
      return following;
    }
    return [];
  }, [followers, following, modal]);

  if (profileQuery.isLoading) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <p className="text-sm font-medium text-slate-600">Loading profile</p>
      </div>
    );
  }

  if (profileQuery.isError || !profileQuery.data) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <h1 className="text-2xl font-semibold">Profile unavailable</h1>
        <Link to="/explore" className="mt-4 inline-block text-sm font-semibold text-emerald-700">
          Explore
        </Link>
      </div>
    );
  }

  const profile = profileQuery.data;
  const isBusy = followMutation.isPending || unfollowMutation.isPending;

  return (
    <div className="space-y-6">
      <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <div className="grid size-20 shrink-0 place-items-center overflow-hidden rounded-full bg-cyan-100 text-2xl font-semibold text-cyan-950">
              {avatarUrl(profile.avatar_key) ? (
                <img src={avatarUrl(profile.avatar_key) ?? ''} alt="" className="h-full w-full object-cover" />
              ) : (
                profile.display_name.slice(0, 1).toUpperCase()
              )}
            </div>
            <div className="min-w-0">
              <div className="flex flex-wrap items-center gap-2">
                <h1 className="break-words text-2xl font-semibold">{profile.display_name}</h1>
                {profile.is_private ? (
                  <span className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-semibold text-slate-600">
                    Private
                  </span>
                ) : null}
              </div>
              <p className="mt-1 text-sm font-medium text-slate-600">@{profile.handle}</p>
              {profile.bio ? <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-700">{profile.bio}</p> : null}
              {profile.link ? (
                <a href={profile.link} className="mt-2 inline-block text-sm font-semibold text-emerald-700">
                  {profile.link}
                </a>
              ) : null}
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            {isOwnProfile ? (
              <Link
                to="/settings/profile"
                className="rounded-md border border-slate-300 px-3 py-2 text-center text-sm font-semibold text-slate-800 hover:border-slate-950"
              >
                Edit profile
              </Link>
            ) : (
              <FollowButton
                isAuthenticated={auth.isAuthenticated}
                state={relationship}
                disabled={isBusy || !auth.isAuthenticated}
                onFollow={() => followMutation.mutate()}
                onUnfollow={() => unfollowMutation.mutate()}
              />
            )}
          </div>
        </div>

        <div className="mt-6 grid grid-cols-3 gap-3 border-t border-slate-200 pt-5 text-center">
          <Stat label="Posts" value={posts.length.toString()} />
          <button type="button" onClick={() => setModal('followers')} className="rounded-md p-2 hover:bg-slate-50">
            <Stat label="Followers" value={followers.length.toString()} />
          </button>
          <button type="button" onClick={() => setModal('following')} className="rounded-md p-2 hover:bg-slate-50">
            <Stat label="Following" value={following.length.toString()} />
          </button>
        </div>
        {followMutation.isError || unfollowMutation.isError ? (
          <p className="mt-4 rounded-md bg-rose-50 px-3 py-2 text-sm text-rose-700">
            {followMutation.error instanceof Error
              ? followMutation.error.message
              : unfollowMutation.error instanceof Error
                ? unfollowMutation.error.message
                : 'Follow update failed'}
          </p>
        ) : null}
      </section>

      {postsQuery.isLoading ? (
        <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
          <p className="text-sm font-medium text-slate-600">Loading posts</p>
        </div>
      ) : posts.length > 0 ? (
        <ProfilePostGrid posts={posts} />
      ) : (
        <div className="rounded-lg border border-slate-200 bg-white p-8 text-center shadow-soft">
          <p className="text-sm font-medium text-slate-600">No posts yet.</p>
        </div>
      )}

      {modal ? (
        <FollowListModal
          title={modal === 'followers' ? 'Followers' : 'Following'}
          users={modalUsers}
          isLoading={modal === 'followers' ? followersQuery.isLoading : followingQuery.isLoading}
          onClose={() => setModal(null)}
        />
      ) : null}
    </div>
  );
}

function FollowButton({
  isAuthenticated,
  state,
  disabled,
  onFollow,
  onUnfollow,
}: {
  isAuthenticated: boolean;
  state: FollowState;
  disabled: boolean;
  onFollow: () => void;
  onUnfollow: () => void;
}) {
  if (!isAuthenticated) {
    return (
      <Link
        to="/login"
        className="rounded-md border border-slate-300 px-3 py-2 text-center text-sm font-semibold text-slate-800 hover:border-slate-950"
      >
        Sign in to follow
      </Link>
    );
  }

  if (state === 'accepted') {
    return (
      <button
        type="button"
        onClick={onUnfollow}
        disabled={disabled}
        className="rounded-md border border-slate-300 px-3 py-2 text-sm font-semibold text-slate-800 hover:border-slate-950 disabled:cursor-not-allowed disabled:opacity-60"
      >
        Following
      </button>
    );
  }

  if (state === 'pending') {
    return (
      <button
        type="button"
        onClick={onUnfollow}
        disabled={disabled}
        className="rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-sm font-semibold text-amber-900 disabled:cursor-not-allowed disabled:opacity-60"
      >
        Requested
      </button>
    );
  }

  return (
    <button
      type="button"
      onClick={onFollow}
      disabled={disabled}
      className="rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
    >
      Follow
    </button>
  );
}

function ProfilePostGrid({ posts }: { posts: PostResponse[] }) {
  return (
    <div className="grid grid-cols-3 gap-3">
      {posts.map((post) => (
        <Link
          key={post.id}
          to={`/p/${post.id}`}
          className="group relative aspect-square overflow-hidden rounded-lg bg-gradient-to-br from-slate-100 via-cyan-100 to-emerald-100 transition hover:scale-[1.01]"
          aria-label={`Open post by ${post.author.handle}`}
        >
          <PostTileMedia media={post.media[0]} />
          <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-slate-950/70 to-transparent p-2 opacity-0 transition group-hover:opacity-100">
            <p className="truncate text-xs font-semibold text-white">{post.caption || formatDate(post.created_at)}</p>
          </div>
        </Link>
      ))}
    </div>
  );
}

function PostTileMedia({ media }: { media?: PostMedia }) {
  const imageUrl = media ? mediaUrl(preferredImageKey(media)) : null;
  const posterUrl = media ? mediaUrl(videoPosterKey(media)) : null;
  const url = media?.kind === 'video' ? posterUrl : imageUrl;

  if (url) {
    return <img src={url} alt="" className="h-full w-full object-cover" />;
  }

  return (
    <div className="grid h-full w-full place-items-center p-3 text-center">
      <span className="line-clamp-3 break-all text-xs font-semibold text-slate-500">
        {media ? displayMediaKey(media) : 'Post'}
      </span>
    </div>
  );
}

function FollowListModal({
  title,
  users,
  isLoading,
  onClose,
}: {
  title: string;
  users: FollowUser[];
  isLoading: boolean;
  onClose: () => void;
}) {
  return (
    <div className="fixed inset-0 z-40 grid place-items-center bg-slate-950/40 px-4" role="dialog" aria-modal="true">
      <div className="w-full max-w-md overflow-hidden rounded-lg bg-white shadow-soft">
        <div className="flex items-center justify-between border-b border-slate-200 p-4">
          <h2 className="text-lg font-semibold text-slate-950">{title}</h2>
          <button
            type="button"
            onClick={onClose}
            className="rounded-md border border-slate-300 px-3 py-1.5 text-sm font-semibold text-slate-700 hover:border-slate-950"
          >
            Close
          </button>
        </div>
        <div className="max-h-96 overflow-y-auto p-2">
          {isLoading ? (
            <p className="p-4 text-sm font-medium text-slate-600">Loading people</p>
          ) : users.length > 0 ? (
            users.map((user) => (
              <Link
                key={user.id}
                to={`/u/${user.handle}`}
                onClick={onClose}
                className="flex items-center gap-3 rounded-md p-3 hover:bg-slate-50"
              >
                <div className="grid size-10 shrink-0 place-items-center overflow-hidden rounded-full bg-cyan-100 text-sm font-semibold text-cyan-950">
                  {avatarUrl(user.avatar_key) ? (
                    <img src={avatarUrl(user.avatar_key) ?? ''} alt="" className="h-full w-full object-cover" />
                  ) : (
                    user.display_name.slice(0, 1).toUpperCase()
                  )}
                </div>
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold text-slate-950">{user.display_name}</p>
                  <p className="truncate text-xs font-medium text-slate-500">@{user.handle}</p>
                </div>
                {user.is_private ? (
                  <span className="ml-auto rounded-full bg-slate-100 px-2 py-1 text-xs font-semibold text-slate-500">
                    Private
                  </span>
                ) : null}
              </Link>
            ))
          ) : (
            <p className="p-4 text-sm text-slate-600">No people to show.</p>
          )}
        </div>
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-lg font-semibold">{value}</p>
      <p className="text-xs font-medium uppercase tracking-normal text-slate-500">{label}</p>
    </div>
  );
}

function avatarUrl(key?: string | null) {
  return mediaUrl(key ?? null);
}

function preferredImageKey(media: PostMedia) {
  const variants = media.variants;
  return variantKey(variants.large) ?? variantKey(variants.medium) ?? variantKey(variants.thumb) ?? media.original_key;
}

function videoPosterKey(media: PostMedia) {
  return variantKey(media.variants.poster);
}

function variantKey(value: unknown) {
  if (!value || typeof value !== 'object' || !('key' in value)) {
    return null;
  }

  const key = (value as { key?: unknown }).key;
  return typeof key === 'string' ? key : null;
}

function mediaUrl(key: string | null) {
  if (!key || !MEDIA_BASE_URL) {
    return null;
  }

  return `${MEDIA_BASE_URL.replace(/\/$/, '')}/${key.replace(/^\//, '')}`;
}

function displayMediaKey(media: PostMedia) {
  if (media.kind === 'video') {
    return videoPosterKey(media) ?? media.original_key;
  }

  return preferredImageKey(media);
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(new Date(value));
}
