import { ChangeEvent, useEffect, useMemo, useRef, useState } from 'react';
import type { ReactNode } from 'react';
import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { Link, useSearchParams } from 'react-router-dom';

import { getExplore, type PostMedia, type PostResponse } from '../features/posts/api';
import { searchAll } from '../features/search/api';

const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';

export function ExplorePage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const activeTag = searchParams.get('tag') ?? searchParams.get('hashtag');
  const activePlace = searchParams.get('place');
  const [searchValue, setSearchValue] = useState(activeTag ? `#${activeTag}` : activePlace ?? '');
  const debouncedSearch = useDebouncedValue(searchValue, 250);
  const loadMoreRef = useRef<HTMLDivElement | null>(null);
  const exploreQuery = useInfiniteQuery({
    queryKey: ['explore', activeTag, activePlace],
    queryFn: ({ pageParam }) =>
      getExplore({
        cursor: pageParam,
        hashtag: activeTag,
        place: activePlace,
      }),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
  });
  const { fetchNextPage, hasNextPage, isFetchingNextPage } = exploreQuery;
  const posts = useMemo(() => exploreQuery.data?.pages.flatMap((page) => page.posts) ?? [], [exploreQuery.data]);
  const searchQuery = useQuery({
    queryKey: ['global-search', debouncedSearch],
    queryFn: () => searchAll(debouncedSearch),
    enabled: debouncedSearch.trim().length >= 2,
    staleTime: 15_000,
  });
  const placeResults = useMemo(() => {
    const names = new Set<string>();
    for (const post of searchQuery.data?.posts ?? []) {
      if (post.location) {
        names.add(post.location);
      }
    }
    return Array.from(names).slice(0, 6);
  }, [searchQuery.data]);

  useEffect(() => {
    setSearchValue(activeTag ? `#${activeTag}` : activePlace ?? '');
  }, [activePlace, activeTag]);

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

  function handleSearchChange(event: ChangeEvent<HTMLInputElement>) {
    setSearchValue(event.target.value);
  }

  function clearFilters() {
    setSearchParams({});
    setSearchValue('');
  }

  return (
    <div className="space-y-5">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-2xl font-semibold">Explore</h1>
          <p className="mt-1 text-sm text-slate-600">Trending posts, tags, and places.</p>
        </div>
        {(activeTag || activePlace) ? (
          <button
            type="button"
            onClick={clearFilters}
            className="rounded-md border border-slate-300 px-3 py-2 text-sm font-semibold text-slate-800 hover:border-slate-950"
          >
            Clear filter
          </button>
        ) : null}
      </div>

      <div className="relative">
        <input
          value={searchValue}
          onChange={handleSearchChange}
          placeholder="Search people, tags, places"
          className="w-full rounded-lg border border-slate-300 bg-white px-4 py-3 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
        />
        {debouncedSearch.trim().length >= 2 ? (
          <SearchDropdown
            users={searchQuery.data?.users ?? []}
            tags={searchQuery.data?.hashtags ?? []}
            places={placeResults}
            isLoading={searchQuery.isLoading}
          />
        ) : null}
      </div>

      {activeTag || activePlace ? (
        <div className="rounded-lg border border-slate-200 bg-white px-4 py-3 shadow-soft">
          <p className="text-sm font-semibold text-slate-700">
            {activeTag ? `#${activeTag}` : activePlace}
          </p>
        </div>
      ) : null}

      {exploreQuery.isLoading ? (
        <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
          <p className="text-sm font-medium text-slate-600">Loading explore</p>
        </div>
      ) : null}

      {exploreQuery.isError ? (
        <div className="rounded-lg border border-rose-200 bg-white p-5 shadow-soft">
          <p className="text-sm font-medium text-rose-700">
            {exploreQuery.error instanceof Error ? exploreQuery.error.message : 'Explore failed to load'}
          </p>
        </div>
      ) : null}

      {posts.length > 0 ? (
        <div className="columns-2 gap-3 sm:columns-3">
          {posts.map((post, index) => (
            <ExploreTile key={post.id} post={post} tall={index % 5 === 1 || index % 5 === 4} />
          ))}
        </div>
      ) : !exploreQuery.isLoading ? (
        <div className="rounded-lg border border-slate-200 bg-white p-8 text-center shadow-soft">
          <p className="text-sm font-medium text-slate-600">No explore posts yet.</p>
        </div>
      ) : null}

      {hasNextPage ? <div ref={loadMoreRef} className="h-8" /> : null}
      {isFetchingNextPage ? (
        <p className="text-center text-sm font-medium text-slate-500">Loading more posts</p>
      ) : null}
    </div>
  );
}

function SearchDropdown({
  users,
  tags,
  places,
  isLoading,
}: {
  users: Array<{ id: string; handle: string; display_name: string }>;
  tags: Array<{ name: string; post_count: number }>;
  places: string[];
  isLoading: boolean;
}) {
  return (
    <div className="absolute left-0 right-0 top-full z-30 mt-2 overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft">
      {isLoading ? <p className="p-4 text-sm font-medium text-slate-600">Searching</p> : null}
      {!isLoading && users.length === 0 && tags.length === 0 && places.length === 0 ? (
        <p className="p-4 text-sm text-slate-600">No matches.</p>
      ) : null}
      <ResultSection title="Users">
        {users.map((user) => (
          <Link key={user.id} to={`/u/${user.handle}`} className="block rounded-md px-3 py-2 hover:bg-slate-50">
            <span className="block text-sm font-semibold text-slate-950">{user.display_name}</span>
            <span className="text-xs font-medium text-slate-500">@{user.handle}</span>
          </Link>
        ))}
      </ResultSection>
      <ResultSection title="Tags">
        {tags.map((tag) => (
          <Link
            key={tag.name}
            to={`/explore?tag=${encodeURIComponent(tag.name)}`}
            className="flex items-center justify-between rounded-md px-3 py-2 hover:bg-slate-50"
          >
            <span className="text-sm font-semibold text-slate-950">#{tag.name}</span>
            <span className="text-xs font-medium text-slate-500">{tag.post_count}</span>
          </Link>
        ))}
      </ResultSection>
      <ResultSection title="Places">
        {places.map((place) => (
          <Link
            key={place}
            to={`/explore?place=${encodeURIComponent(place)}`}
            className="block rounded-md px-3 py-2 text-sm font-semibold text-slate-950 hover:bg-slate-50"
          >
            {place}
          </Link>
        ))}
      </ResultSection>
    </div>
  );
}

function ResultSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="border-t border-slate-100 p-2 first:border-t-0">
      <h2 className="px-3 py-1 text-xs font-semibold uppercase tracking-normal text-slate-500">{title}</h2>
      <div>{children}</div>
    </section>
  );
}

function ExploreTile({ post, tall }: { post: PostResponse; tall: boolean }) {
  return (
    <Link
      to={`/p/${post.id}`}
      className="mb-3 block break-inside-avoid overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft transition hover:scale-[1.01]"
    >
      <TileMedia media={post.media[0]} tall={tall} />
      <div className="space-y-2 p-3">
        <div className="flex items-center justify-between gap-3">
          <span className="truncate text-sm font-semibold text-slate-950">@{post.author.handle}</span>
          <span className="shrink-0 text-xs font-medium text-slate-500">{post.media.length}</span>
        </div>
        {post.caption ? <p className="line-clamp-2 text-xs leading-5 text-slate-600">{post.caption}</p> : null}
      </div>
    </Link>
  );
}

function TileMedia({ media, tall }: { media?: PostMedia; tall: boolean }) {
  const imageUrl = media ? mediaAssetUrl(preferredImageKey(media)) : null;
  const posterUrl = media ? mediaAssetUrl(videoPosterKey(media)) : null;
  const url = media?.kind === 'video' ? posterUrl : imageUrl;

  return (
    <div
      className={[
        'grid place-items-center bg-gradient-to-br from-slate-100 via-cyan-50 to-emerald-100',
        tall ? 'aspect-[4/5]' : 'aspect-square',
      ].join(' ')}
    >
      {url ? (
        <img src={url} alt="" className="h-full w-full object-cover" />
      ) : (
        <span className="line-clamp-3 break-all p-4 text-center text-xs font-semibold text-slate-500">
          {media ? displayMediaKey(media) : 'Post'}
        </span>
      )}
    </div>
  );
}

function useDebouncedValue(value: string, delayMs: number) {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const timeout = window.setTimeout(() => setDebounced(value), delayMs);
    return () => window.clearTimeout(timeout);
  }, [delayMs, value]);

  return debounced;
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
