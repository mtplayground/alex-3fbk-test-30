import { useEffect, useMemo, useRef } from 'react';
import { useInfiniteQuery, useMutation, useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';
import { PostComposer } from '../features/posts/PostComposer';
import {
  getFeed,
  getPostComments,
  togglePostLike,
  togglePostSave,
  type Comment,
  type PostMedia,
  type PostResponse,
} from '../features/posts/api';
import { StoriesRail } from '../features/stories/StoriesRail';

const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';

export function HomePage() {
  const auth = useAuth();
  const loadMoreRef = useRef<HTMLDivElement | null>(null);
  const feedQuery = useInfiniteQuery({
    queryKey: ['home-feed'],
    queryFn: ({ pageParam }) => getFeed(pageParam),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    enabled: auth.isAuthenticated,
  });
  const { fetchNextPage, hasNextPage, isFetchingNextPage } = feedQuery;
  const posts = useMemo(() => feedQuery.data?.pages.flatMap((page) => page.posts) ?? [], [feedQuery.data]);

  useEffect(() => {
    const node = loadMoreRef.current;
    if (!node || !hasNextPage || isFetchingNextPage) {
      return;
    }

    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        void fetchNextPage();
      }
    });
    observer.observe(node);

    return () => observer.disconnect();
  }, [fetchNextPage, hasNextPage, isFetchingNextPage]);

  return (
    <div className="space-y-5">
      <div className="flex items-end justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Home</h1>
          <p className="mt-1 text-sm text-slate-600">Latest posts from followed accounts.</p>
        </div>
      </div>

      <StoriesRail />

      {auth.isAuthenticated ? <PostComposer /> : null}

      {!auth.isAuthenticated ? (
        <section className="rounded-lg border border-slate-200 bg-white p-6 shadow-soft">
          <h2 className="text-lg font-semibold text-slate-950">Sign in to see your feed</h2>
          <p className="mt-2 text-sm leading-6 text-slate-600">
            Follow accounts to fill this page with recent posts, comments, and updates.
          </p>
          <Link
            to="/login"
            className="mt-4 inline-flex rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800"
          >
            Sign in
          </Link>
        </section>
      ) : null}

      {auth.isAuthenticated && feedQuery.isLoading ? (
        <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
          <p className="text-sm font-medium text-slate-600">Loading feed</p>
        </div>
      ) : null}

      {auth.isAuthenticated && feedQuery.isError ? (
        <div className="rounded-lg border border-rose-200 bg-white p-5 shadow-soft">
          <p className="text-sm font-medium text-rose-700">
            {feedQuery.error instanceof Error ? feedQuery.error.message : 'Feed failed to load'}
          </p>
        </div>
      ) : null}

      {posts.length > 0 ? (
        <div className="grid gap-5">
          {posts.map((post) => (
            <PostCard key={post.id} post={post} isAuthenticated={auth.isAuthenticated} />
          ))}
          {feedQuery.hasNextPage ? <div ref={loadMoreRef} className="h-8" /> : null}
          {feedQuery.isFetchingNextPage ? (
            <p className="text-center text-sm font-medium text-slate-500">Loading more posts</p>
          ) : null}
        </div>
      ) : auth.isAuthenticated && !feedQuery.isLoading ? (
        <div className="rounded-lg border border-slate-200 bg-white p-8 text-center shadow-soft">
          <p className="text-sm font-medium text-slate-600">No feed posts yet.</p>
        </div>
      ) : null}
    </div>
  );
}

function PostCard({ post, isAuthenticated }: { post: PostResponse; isAuthenticated: boolean }) {
  const commentsQuery = useQuery({
    queryKey: ['post-comments-summary', post.id],
    queryFn: () => getPostComments(post.id),
    staleTime: 15_000,
  });
  const likeMutation = useMutation({
    mutationFn: () => togglePostLike(post.id),
  });
  const saveMutation = useMutation({
    mutationFn: () => togglePostSave(post.id),
  });
  const comments = commentsQuery.data?.comments ?? [];
  const summary = commentSummary(comments);

  return (
    <article className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft">
      <header className="flex items-center justify-between gap-3 p-4">
        <Link to={`/u/${post.author.handle}`} className="font-semibold text-slate-950">
          @{post.author.handle}
        </Link>
        <time className="text-xs font-medium text-slate-500" dateTime={post.created_at}>
          {formatDate(post.created_at)}
        </time>
      </header>

      <FeedMedia postId={post.id} media={post.media[0]} mediaCount={post.media.length} />

      <div className="space-y-4 p-4">
        <div className="flex flex-wrap items-center gap-2">
          <ActionButton
            label={likeMutation.data?.active ? 'Liked' : 'Like'}
            count={likeMutation.data?.count}
            active={likeMutation.data?.active}
            disabled={!isAuthenticated || likeMutation.isPending}
            onClick={() => likeMutation.mutate()}
          />
          <Link
            to={`/p/${post.id}`}
            className="rounded-md border border-slate-300 px-3 py-2 text-sm font-semibold text-slate-800 hover:border-slate-950"
          >
            Comment
          </Link>
          <ActionButton
            label={saveMutation.data?.active ? 'Saved' : 'Save'}
            count={saveMutation.data?.count}
            active={saveMutation.data?.active}
            disabled={!isAuthenticated || saveMutation.isPending}
            onClick={() => saveMutation.mutate()}
          />
        </div>

        {post.caption ? (
          <p className="line-clamp-3 whitespace-pre-wrap text-sm leading-6 text-slate-700">{post.caption}</p>
        ) : null}

        {post.hashtags.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {post.hashtags.slice(0, 5).map((hashtag) => (
              <span key={hashtag} className="rounded-full bg-emerald-50 px-2.5 py-1 text-xs font-semibold text-emerald-800">
                #{hashtag}
              </span>
            ))}
          </div>
        ) : null}

        <Link to={`/p/${post.id}`} className="block rounded-md bg-slate-50 px-3 py-2 text-sm text-slate-600">
          {summary}
        </Link>
      </div>
    </article>
  );
}

function FeedMedia({ postId, media, mediaCount }: { postId: string; media?: PostMedia; mediaCount: number }) {
  const imageUrl = media ? mediaAssetUrl(preferredImageKey(media)) : null;
  const posterUrl = media ? mediaAssetUrl(videoPosterKey(media)) : null;
  const url = media?.kind === 'video' ? posterUrl : imageUrl;

  return (
    <Link
      to={`/p/${postId}`}
      className="relative flex aspect-[4/3] items-center justify-center bg-gradient-to-br from-slate-100 via-cyan-50 to-emerald-100"
      aria-label="Open post"
    >
      {url ? (
        <img src={url} alt="" className="h-full w-full object-contain" />
      ) : (
        <div className="max-w-md space-y-2 p-6 text-center">
          <p className="text-sm font-semibold uppercase tracking-normal text-slate-500">{media?.kind ?? 'Media'}</p>
          <p className="break-all text-sm font-medium text-slate-700">{media ? displayMediaKey(media) : 'No media'}</p>
        </div>
      )}
      {mediaCount > 1 ? (
        <span className="absolute right-3 top-3 rounded-full bg-white/90 px-2.5 py-1 text-xs font-semibold text-slate-800">
          {mediaCount}
        </span>
      ) : null}
    </Link>
  );
}

function ActionButton({
  label,
  count,
  active,
  disabled,
  onClick,
}: {
  label: string;
  count?: number;
  active?: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={[
        'rounded-md border px-3 py-2 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-60',
        active ? 'border-emerald-700 bg-emerald-50 text-emerald-800' : 'border-slate-300 text-slate-800 hover:border-slate-950',
      ].join(' ')}
    >
      {label}
      {typeof count === 'number' ? <span className="ml-2 text-slate-500">{count}</span> : null}
    </button>
  );
}

function commentSummary(comments: Comment[]) {
  const total = comments.reduce((count, comment) => count + 1 + comment.replies.length, 0);
  if (total === 0) {
    return 'No comments yet';
  }

  const latest = comments[comments.length - 1];
  const preview = latest?.body ? `: ${latest.body}` : '';
  return `${total} ${total === 1 ? 'comment' : 'comments'}${preview}`;
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

function mediaAssetUrl(key: string | null) {
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
