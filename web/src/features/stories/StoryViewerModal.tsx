import { FormEvent, useCallback, useEffect, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import {
  getStoryViewers,
  markStoryViewed,
  type StoryAuthorGroup,
  type StoryResponse,
  type StoryViewer,
} from './api';

const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';
const STORY_DURATION_MS = 5_000;
const PROGRESS_TICK_MS = 100;

export type StoryItem = {
  group: StoryAuthorGroup;
  story: StoryResponse;
  authorIndex: number;
  storyIndex: number;
};

type StoryViewerModalProps = {
  items: StoryItem[];
  activeIndex: number;
  currentUserId: string | null;
  onActiveIndexChange: (index: number) => void;
  onClose: () => void;
};

export function StoryViewerModal({
  items,
  activeIndex,
  currentUserId,
  onActiveIndexChange,
  onClose,
}: StoryViewerModalProps) {
  const activeItem = items[activeIndex];
  const [progress, setProgress] = useState(0);
  const [reply, setReply] = useState('');
  const [showViewers, setShowViewers] = useState(false);
  const queryClient = useQueryClient();
  const markViewedMutation = useMutation({
    mutationFn: markStoryViewed,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['stories-feed'] });
    },
  });
  const isOwnStory = Boolean(currentUserId && activeItem?.story.author.id === currentUserId);
  const viewersQuery = useQuery({
    queryKey: ['story-viewers', activeItem?.story.id],
    queryFn: () => getStoryViewers(activeItem.story.id),
    enabled: Boolean(activeItem && isOwnStory && showViewers),
  });

  const goNext = useCallback(() => {
    if (activeIndex < items.length - 1) {
      onActiveIndexChange(activeIndex + 1);
    } else {
      onClose();
    }
  }, [activeIndex, items.length, onActiveIndexChange, onClose]);

  const goPrevious = useCallback(() => {
    if (activeIndex > 0) {
      onActiveIndexChange(activeIndex - 1);
    }
  }, [activeIndex, onActiveIndexChange]);

  useEffect(() => {
    if (!activeItem) {
      onClose();
    }
  }, [activeItem, onClose]);

  useEffect(() => {
    if (!activeItem || isOwnStory) {
      return;
    }

    markViewedMutation.mutate(activeItem.story.id);
  }, [activeItem?.story.id, isOwnStory]);

  useEffect(() => {
    if (!activeItem) {
      return;
    }

    setProgress(0);
    setReply('');
    setShowViewers(false);
    const startedAt = Date.now();
    const progressTimer = window.setInterval(() => {
      const elapsed = Date.now() - startedAt;
      setProgress(Math.min(100, (elapsed / STORY_DURATION_MS) * 100));
    }, PROGRESS_TICK_MS);
    const advanceTimer = window.setTimeout(goNext, STORY_DURATION_MS);

    return () => {
      window.clearInterval(progressTimer);
      window.clearTimeout(advanceTimer);
    };
  }, [activeItem?.story.id, goNext]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose();
      }
      if (event.key === 'ArrowRight') {
        goNext();
      }
      if (event.key === 'ArrowLeft') {
        goPrevious();
      }
    }

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [goNext, goPrevious, onClose]);

  const groupStories = activeItem?.group.stories ?? [];
  const story = activeItem?.story;
  const mediaUrl = story ? mediaAssetUrl(preferredStoryMediaKey(story)) : null;
  const viewers = viewersQuery.data?.viewers ?? [];

  function handleReplySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setReply('');
  }

  if (!activeItem || !story) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 bg-slate-950 text-white">
      <div className="mx-auto flex h-full max-w-5xl flex-col">
        <header className="space-y-3 px-4 pb-3 pt-4">
          <div className="flex gap-1.5">
            {groupStories.map((groupStory, index) => (
              <div key={groupStory.id} className="h-1 flex-1 overflow-hidden rounded-full bg-white/25">
                <div
                  className="h-full rounded-full bg-white transition-[width]"
                  style={{ width: `${progressForSegment(index, activeItem.storyIndex, progress)}%` }}
                />
              </div>
            ))}
          </div>

          <div className="flex items-center justify-between gap-3">
            <div className="min-w-0">
              <p className="truncate text-sm font-semibold">@{story.author.handle}</p>
              <p className="text-xs font-medium text-white/60">{formatRelativeTime(story.created_at)}</p>
            </div>
            <button
              type="button"
              onClick={onClose}
              className="rounded-md border border-white/30 px-3 py-2 text-sm font-semibold text-white hover:bg-white/10"
            >
              Close
            </button>
          </div>
        </header>

        <main className="relative min-h-0 flex-1">
          <button
            type="button"
            onClick={goPrevious}
            disabled={activeIndex === 0}
            className="absolute inset-y-0 left-0 z-10 w-1/3 cursor-w-resize disabled:cursor-default"
            aria-label="Previous story"
          />
          <button
            type="button"
            onClick={goNext}
            className="absolute inset-y-0 right-0 z-10 w-1/3 cursor-e-resize"
            aria-label="Next story"
          />

          <div className="flex h-full items-center justify-center px-4 pb-4">
            {story.media.kind === 'video' && mediaUrl ? (
              <video
                src={mediaUrl}
                className="max-h-full max-w-full rounded-md object-contain"
                autoPlay
                muted
                playsInline
                controls
              />
            ) : mediaUrl ? (
              <img src={mediaUrl} alt="" className="max-h-full max-w-full rounded-md object-contain" />
            ) : (
              <div className="max-w-lg rounded-md border border-white/20 bg-white/10 p-6 text-center">
                <p className="text-sm font-semibold uppercase tracking-normal text-white/60">{story.media.kind}</p>
                <p className="mt-2 break-all text-sm font-medium text-white/90">{displayStoryMediaKey(story)}</p>
              </div>
            )}
          </div>
        </main>

        <footer className="grid gap-3 border-t border-white/10 px-4 py-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-start">
          <form className="flex gap-2" onSubmit={handleReplySubmit}>
            <input
              value={reply}
              onChange={(event) => setReply(event.target.value)}
              placeholder={`Reply to @${story.author.handle}`}
              className="min-w-0 flex-1 rounded-md border border-white/20 bg-white/10 px-3 py-2 text-sm text-white outline-none placeholder:text-white/50 focus:border-white/60"
            />
            <button
              type="submit"
              disabled={!reply.trim()}
              className="rounded-md border border-white/30 px-4 py-2 text-sm font-semibold text-white hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Reply
            </button>
          </form>

          {isOwnStory ? (
            <div className="md:w-72">
              <button
                type="button"
                onClick={() => setShowViewers((value) => !value)}
                className="w-full rounded-md border border-white/30 px-4 py-2 text-sm font-semibold text-white hover:bg-white/10"
              >
                {story.viewer_count} {story.viewer_count === 1 ? 'viewer' : 'viewers'}
              </button>
              {showViewers ? (
                <StoryViewersList
                  viewers={viewers}
                  isLoading={viewersQuery.isLoading}
                  isError={viewersQuery.isError}
                />
              ) : null}
            </div>
          ) : null}
        </footer>
      </div>
    </div>
  );
}

function StoryViewersList({
  viewers,
  isLoading,
  isError,
}: {
  viewers: StoryViewer[];
  isLoading: boolean;
  isError: boolean;
}) {
  return (
    <div className="mt-2 max-h-48 overflow-y-auto rounded-md border border-white/10 bg-white/10 p-2">
      {isLoading ? (
        <p className="px-2 py-1 text-sm text-white/70">Loading viewers</p>
      ) : isError ? (
        <p className="px-2 py-1 text-sm text-rose-200">Viewer list failed to load</p>
      ) : viewers.length > 0 ? (
        <div className="space-y-1">
          {viewers.map((viewer) => (
            <div key={viewer.id} className="flex items-center justify-between gap-3 rounded px-2 py-1.5 text-sm">
              <span className="truncate font-medium">@{viewer.handle}</span>
              <span className="shrink-0 text-xs text-white/50">{formatRelativeTime(viewer.viewed_at)}</span>
            </div>
          ))}
        </div>
      ) : (
        <p className="px-2 py-1 text-sm text-white/70">No viewers yet.</p>
      )}
    </div>
  );
}

function progressForSegment(index: number, activeIndex: number, progress: number) {
  if (index < activeIndex) {
    return 100;
  }
  if (index === activeIndex) {
    return progress;
  }
  return 0;
}

function preferredStoryMediaKey(story: StoryResponse) {
  if (story.media.kind === 'video') {
    return variantKey(story.media.variants.poster) ?? story.media.original_key;
  }

  const variants = story.media.variants;
  return (
    variantKey(variants.large) ??
    variantKey(variants.medium) ??
    variantKey(variants.thumb) ??
    story.media.original_key
  );
}

function displayStoryMediaKey(story: StoryResponse) {
  return preferredStoryMediaKey(story);
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

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  const diffSeconds = Math.max(0, Math.floor((Date.now() - timestamp) / 1000));

  if (diffSeconds < 60) {
    return 'Just now';
  }

  const diffMinutes = Math.floor(diffSeconds / 60);
  if (diffMinutes < 60) {
    return `${diffMinutes}m ago`;
  }

  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) {
    return `${diffHours}h ago`;
  }

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
  }).format(new Date(value));
}
