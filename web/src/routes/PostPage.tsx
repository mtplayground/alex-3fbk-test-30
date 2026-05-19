import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';
import {
  createComment,
  getPost,
  getPostComments,
  togglePostLike,
  togglePostSave,
  type Comment,
  type PostMedia,
} from '../features/posts/api';

const COMMENT_PAGE_SIZE = 8;
const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';

type ReplyTarget = {
  id: string;
  handle: string;
};

export function PostPage() {
  const { id = '' } = useParams();
  const auth = useAuth();
  const queryClient = useQueryClient();
  const [activeMediaIndex, setActiveMediaIndex] = useState(0);
  const [visibleCommentCount, setVisibleCommentCount] = useState(COMMENT_PAGE_SIZE);
  const [replyTarget, setReplyTarget] = useState<ReplyTarget | null>(null);
  const [commentBody, setCommentBody] = useState('');
  const loadMoreRef = useRef<HTMLDivElement | null>(null);

  const postQuery = useQuery({
    queryKey: ['post', id],
    queryFn: () => getPost(id),
    enabled: Boolean(id),
  });
  const commentsQuery = useQuery({
    queryKey: ['post-comments', id],
    queryFn: () => getPostComments(id),
    enabled: Boolean(id),
  });

  const comments = commentsQuery.data?.comments ?? [];
  const visibleComments = useMemo(() => comments.slice(0, visibleCommentCount), [comments, visibleCommentCount]);
  const hasMoreComments = visibleCommentCount < comments.length;

  useEffect(() => {
    setVisibleCommentCount(COMMENT_PAGE_SIZE);
  }, [id]);

  useEffect(() => {
    const node = loadMoreRef.current;
    if (!node || !hasMoreComments) {
      return;
    }

    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        setVisibleCommentCount((current) => Math.min(current + COMMENT_PAGE_SIZE, comments.length));
      }
    });
    observer.observe(node);

    return () => observer.disconnect();
  }, [comments.length, hasMoreComments]);

  const likeMutation = useMutation({
    mutationFn: () => togglePostLike(id),
  });
  const saveMutation = useMutation({
    mutationFn: () => togglePostSave(id),
  });
  const commentMutation = useMutation({
    mutationFn: () => createComment(id, commentBody, replyTarget?.id),
    onSuccess: async () => {
      setCommentBody('');
      setReplyTarget(null);
      await queryClient.invalidateQueries({ queryKey: ['post-comments', id] });
    },
  });

  function handleCommentSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!commentBody.trim() || !auth.isAuthenticated) {
      return;
    }

    commentMutation.mutate();
  }

  if (postQuery.isLoading) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <p className="text-sm font-medium text-slate-600">Loading post</p>
      </div>
    );
  }

  if (postQuery.isError || !postQuery.data) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <h1 className="text-2xl font-semibold">Post unavailable</h1>
        <Link to="/explore" className="mt-4 inline-block text-sm font-semibold text-emerald-700">
          Explore
        </Link>
      </div>
    );
  }

  const post = postQuery.data;
  const activeMedia = post.media[activeMediaIndex] ?? post.media[0];
  const likeState = likeMutation.data;
  const saveState = saveMutation.data;

  return (
    <article className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_360px]">
      <section className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft">
        <MediaViewer
          media={activeMedia}
          index={activeMediaIndex}
          total={post.media.length}
          onPrevious={() => setActiveMediaIndex((current) => Math.max(0, current - 1))}
          onNext={() => setActiveMediaIndex((current) => Math.min(post.media.length - 1, current + 1))}
        />
        {post.media.length > 1 ? (
          <div className="flex gap-2 overflow-x-auto border-t border-slate-200 p-3">
            {post.media.map((media, index) => (
              <button
                key={media.media_id}
                type="button"
                onClick={() => setActiveMediaIndex(index)}
                className={[
                  'grid size-16 shrink-0 place-items-center rounded-md border text-xs font-semibold',
                  activeMediaIndex === index
                    ? 'border-slate-950 bg-slate-950 text-white'
                    : 'border-slate-200 bg-slate-50 text-slate-600 hover:border-slate-400',
                ].join(' ')}
              >
                {index + 1}
              </button>
            ))}
          </div>
        ) : null}
      </section>

      <aside className="space-y-5">
        <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
          <div className="flex items-center justify-between gap-3">
            <Link to={`/u/${post.author.handle}`} className="font-semibold text-slate-950">
              @{post.author.handle}
            </Link>
            <time className="text-xs font-medium text-slate-500" dateTime={post.created_at}>
              {formatDate(post.created_at)}
            </time>
          </div>

          {post.caption ? <p className="mt-4 whitespace-pre-wrap text-sm leading-6 text-slate-700">{post.caption}</p> : null}
          {post.location ? <p className="mt-3 text-sm font-medium text-slate-500">{post.location}</p> : null}
          {post.hashtags.length > 0 ? (
            <div className="mt-4 flex flex-wrap gap-2">
              {post.hashtags.map((hashtag) => (
                <span key={hashtag} className="rounded-full bg-emerald-50 px-3 py-1 text-xs font-semibold text-emerald-800">
                  #{hashtag}
                </span>
              ))}
            </div>
          ) : null}

          <div className="mt-5 grid grid-cols-2 gap-3 border-t border-slate-200 pt-5">
            <ActionButton
              label={likeState?.active ? 'Liked' : 'Like'}
              count={likeState?.count}
              disabled={!auth.isAuthenticated || likeMutation.isPending}
              active={likeState?.active}
              onClick={() => likeMutation.mutate()}
            />
            <ActionButton
              label={saveState?.active ? 'Saved' : 'Save'}
              count={saveState?.count}
              disabled={!auth.isAuthenticated || saveMutation.isPending}
              active={saveState?.active}
              onClick={() => saveMutation.mutate()}
            />
          </div>
          {!auth.isAuthenticated ? (
            <Link to="/login" className="mt-3 inline-block text-sm font-semibold text-emerald-700">
              Sign in to like, save, or comment
            </Link>
          ) : null}
        </section>

        <section className="rounded-lg border border-slate-200 bg-white shadow-soft">
          <div className="border-b border-slate-200 p-5">
            <h2 className="text-lg font-semibold text-slate-950">Comments</h2>
            <p className="mt-1 text-sm text-slate-600">{comments.length} total</p>
          </div>

          <div className="max-h-[560px] overflow-y-auto">
            {commentsQuery.isLoading ? (
              <p className="p-5 text-sm font-medium text-slate-600">Loading comments</p>
            ) : visibleComments.length > 0 ? (
              <div className="divide-y divide-slate-100">
                {visibleComments.map((comment) => (
                  <CommentThread key={comment.id} comment={comment} onReply={setReplyTarget} />
                ))}
                {hasMoreComments ? <div ref={loadMoreRef} className="h-8" /> : null}
              </div>
            ) : (
              <p className="p-5 text-sm text-slate-600">No comments yet.</p>
            )}
          </div>

          <form className="grid gap-3 border-t border-slate-200 p-5" onSubmit={handleCommentSubmit}>
            {replyTarget ? (
              <div className="flex items-center justify-between gap-3 rounded-md bg-slate-100 px-3 py-2 text-sm text-slate-700">
                <span>Replying to @{replyTarget.handle}</span>
                <button type="button" onClick={() => setReplyTarget(null)} className="font-semibold text-slate-950">
                  Cancel
                </button>
              </div>
            ) : null}
            <textarea
              value={commentBody}
              onChange={(event) => setCommentBody(event.target.value)}
              rows={3}
              disabled={!auth.isAuthenticated}
              placeholder={auth.isAuthenticated ? 'Add a comment' : 'Sign in to comment'}
              className="resize-none rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100 disabled:bg-slate-100"
            />
            <button
              type="submit"
              disabled={!auth.isAuthenticated || !commentBody.trim() || commentMutation.isPending}
              className="rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
            >
              {commentMutation.isPending ? 'Posting' : replyTarget ? 'Post reply' : 'Post comment'}
            </button>
            {commentMutation.isError ? (
              <p className="text-sm text-rose-700">
                {commentMutation.error instanceof Error ? commentMutation.error.message : 'Comment failed'}
              </p>
            ) : null}
          </form>
        </section>
      </aside>
    </article>
  );
}

function MediaViewer({
  media,
  index,
  total,
  onPrevious,
  onNext,
}: {
  media?: PostMedia;
  index: number;
  total: number;
  onPrevious: () => void;
  onNext: () => void;
}) {
  const imageUrl = media ? mediaAssetUrl(preferredImageKey(media)) : null;
  const posterUrl = media ? mediaAssetUrl(videoPosterKey(media)) : null;

  return (
    <div className="relative flex aspect-[4/3] items-center justify-center bg-gradient-to-br from-slate-100 via-cyan-50 to-emerald-100">
      {media?.kind === 'image' && imageUrl ? (
        <img src={imageUrl} alt="" className="h-full w-full object-contain" />
      ) : media?.kind === 'video' && posterUrl ? (
        <img src={posterUrl} alt="" className="h-full w-full object-contain" />
      ) : (
        <div className="max-w-md space-y-3 p-6 text-center">
          <p className="text-sm font-semibold uppercase tracking-normal text-slate-500">
            {media?.kind ?? 'Media'}
          </p>
          <p className="break-all text-sm font-medium text-slate-700">{media ? displayMediaKey(media) : 'No media'}</p>
          {media?.width && media.height ? (
            <p className="text-xs font-medium text-slate-500">
              {media.width} x {media.height}
            </p>
          ) : null}
        </div>
      )}

      {total > 1 ? (
        <div className="absolute inset-x-0 bottom-3 flex items-center justify-between px-3">
          <button
            type="button"
            onClick={onPrevious}
            disabled={index === 0}
            className="rounded-md bg-white/90 px-3 py-2 text-sm font-semibold text-slate-900 shadow-sm disabled:cursor-not-allowed disabled:text-slate-400"
          >
            Previous
          </button>
          <span className="rounded-full bg-white/90 px-3 py-1 text-xs font-semibold text-slate-700 shadow-sm">
            {index + 1} / {total}
          </span>
          <button
            type="button"
            onClick={onNext}
            disabled={index === total - 1}
            className="rounded-md bg-white/90 px-3 py-2 text-sm font-semibold text-slate-900 shadow-sm disabled:cursor-not-allowed disabled:text-slate-400"
          >
            Next
          </button>
        </div>
      ) : null}
    </div>
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

function CommentThread({ comment, onReply }: { comment: Comment; onReply: (target: ReplyTarget) => void }) {
  return (
    <div className="p-5">
      <CommentCard comment={comment} onReply={onReply} />
      {comment.replies.length > 0 ? (
        <div className="mt-4 space-y-4 border-l border-slate-200 pl-4">
          {comment.replies.map((reply) => (
            <CommentCard key={reply.id} comment={reply} onReply={onReply} compact />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function CommentCard({
  comment,
  onReply,
  compact = false,
}: {
  comment: Comment;
  onReply: (target: ReplyTarget) => void;
  compact?: boolean;
}) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <Link to={`/u/${comment.author.handle}`} className="text-sm font-semibold text-slate-950">
          @{comment.author.handle}
        </Link>
        <time className="text-xs font-medium text-slate-500" dateTime={comment.created_at}>
          {formatDate(comment.created_at)}
        </time>
      </div>
      <p className={['whitespace-pre-wrap text-sm leading-6 text-slate-700', compact ? 'text-slate-600' : ''].join(' ')}>
        {comment.body}
      </p>
      {!compact ? (
        <button
          type="button"
          onClick={() => onReply({ id: comment.id, handle: comment.author.handle })}
          className="text-xs font-semibold text-emerald-700"
        >
          Reply
        </button>
      ) : null}
    </div>
  );
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
