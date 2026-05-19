import { useMemo, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';

import { useAuth } from '../auth/AuthProvider';
import { getStoriesFeed, type StoryAuthorGroup } from './api';
import { StoryViewerModal, type StoryItem } from './StoryViewerModal';

const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';

export function StoriesRail() {
  const auth = useAuth();
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const storiesQuery = useQuery({
    queryKey: ['stories-feed'],
    queryFn: getStoriesFeed,
    enabled: auth.isAuthenticated,
    staleTime: 20_000,
    refetchInterval: 60_000,
  });
  const groups = storiesQuery.data?.authors ?? [];
  const storyItems = useStoryItems(groups);

  return (
    <section className="overflow-hidden rounded-lg border border-slate-200 bg-white p-4 shadow-soft">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h2 className="text-sm font-semibold uppercase tracking-normal text-slate-500">Stories</h2>
        {storiesQuery.isFetching ? <span className="text-xs font-medium text-slate-400">Refreshing</span> : null}
      </div>

      {!auth.isAuthenticated ? (
        <Link to="/login" className="block rounded-md bg-slate-50 px-4 py-3 text-sm font-medium text-slate-600">
          Sign in to see stories from people you follow.
        </Link>
      ) : storiesQuery.isLoading ? (
        <div className="flex gap-3 overflow-x-auto">
          {Array.from({ length: 5 }).map((_, index) => (
            <div key={index} className="grid w-20 shrink-0 justify-items-center gap-2">
              <div className="size-16 animate-pulse rounded-full bg-slate-100" />
              <div className="h-3 w-14 animate-pulse rounded bg-slate-100" />
            </div>
          ))}
        </div>
      ) : storiesQuery.isError ? (
        <p className="rounded-md bg-rose-50 px-4 py-3 text-sm font-medium text-rose-700">
          {storiesQuery.error instanceof Error ? storiesQuery.error.message : 'Stories failed to load'}
        </p>
      ) : groups.length > 0 ? (
        <div className="flex gap-3 overflow-x-auto pb-1">
          {groups.map((group) => {
            const firstItemIndex = storyItems.findIndex((item) => item.group.author.id === group.author.id);
            const hasUnseen = group.stories.some((story) => !story.viewed_at);

            return (
              <button
                key={group.author.id}
                type="button"
                onClick={() => setActiveIndex(firstItemIndex)}
                className="grid w-20 shrink-0 justify-items-center gap-2"
              >
                <span
                  className={[
                    'grid size-16 place-items-center overflow-hidden rounded-full border-2 bg-slate-100 text-lg font-semibold',
                    hasUnseen ? 'border-emerald-500 text-emerald-950' : 'border-slate-200 text-slate-600',
                  ].join(' ')}
                >
                  {avatarUrl(group.author.avatar_key) ? (
                    <img src={avatarUrl(group.author.avatar_key) ?? ''} alt="" className="h-full w-full object-cover" />
                  ) : (
                    group.author.handle.slice(0, 1).toUpperCase()
                  )}
                </span>
                <span className="max-w-full truncate text-xs font-semibold text-slate-600">@{group.author.handle}</span>
              </button>
            );
          })}
        </div>
      ) : (
        <p className="rounded-md bg-slate-50 px-4 py-3 text-sm font-medium text-slate-600">No active stories yet.</p>
      )}

      {activeIndex !== null && storyItems[activeIndex] ? (
        <StoryViewerModal
          items={storyItems}
          activeIndex={activeIndex}
          currentUserId={auth.user?.id ?? null}
          onActiveIndexChange={setActiveIndex}
          onClose={() => setActiveIndex(null)}
        />
      ) : null}
    </section>
  );
}

function useStoryItems(groups: StoryAuthorGroup[]) {
  return useMemo<StoryItem[]>(
    () =>
      groups.flatMap((group, authorIndex) =>
        group.stories.map((story, storyIndex) => ({
          group,
          story,
          authorIndex,
          storyIndex,
        })),
      ),
    [groups],
  );
}

function avatarUrl(key?: string | null) {
  if (!key || !MEDIA_BASE_URL) {
    return null;
  }

  return `${MEDIA_BASE_URL.replace(/\/$/, '')}/${key.replace(/^\//, '')}`;
}
