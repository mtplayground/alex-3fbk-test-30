import { useEffect, useMemo, useRef, useState } from 'react';
import { useInfiniteQuery, useQueryClient } from '@tanstack/react-query';
import { Link } from 'react-router-dom';

import { getReel, getReelsFeed, type ReelResponse } from '../features/reels/api';

const MEDIA_BASE_URL = import.meta.env.VITE_MEDIA_BASE_URL ?? '';

export function ReelsPage() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [muted, setMuted] = useState(true);
  const [likedReels, setLikedReels] = useState<Set<string>>(() => new Set());
  const containerRef = useRef<HTMLDivElement | null>(null);
  const itemRefs = useRef<Array<HTMLElement | null>>([]);
  const queryClient = useQueryClient();
  const reelsQuery = useInfiniteQuery({
    queryKey: ['reels-feed'],
    queryFn: ({ pageParam }) => getReelsFeed(pageParam),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
  });
  const reels = useMemo(() => reelsQuery.data?.pages.flatMap((page) => page.reels) ?? [], [reelsQuery.data]);
  const activeReel = reels[activeIndex];

  useEffect(() => {
    const container = containerRef.current;
    if (!container || reels.length === 0) {
      return;
    }

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((left, right) => right.intersectionRatio - left.intersectionRatio)[0];
        const index = Number(visible?.target.getAttribute('data-reel-index'));
        if (Number.isInteger(index)) {
          setActiveIndex(index);
        }
      },
      { root: container, threshold: [0.6, 0.8] },
    );

    for (const node of itemRefs.current) {
      if (node) {
        observer.observe(node);
      }
    }

    return () => observer.disconnect();
  }, [reels.length]);

  useEffect(() => {
    const nextReel = reels[activeIndex + 1];
    if (nextReel) {
      void queryClient.prefetchQuery({
        queryKey: ['reel', nextReel.id],
        queryFn: () => getReel(nextReel.id),
        staleTime: 30_000,
      });
    }

    if (
      reelsQuery.hasNextPage &&
      !reelsQuery.isFetchingNextPage &&
      activeIndex >= Math.max(0, reels.length - 2)
    ) {
      void reelsQuery.fetchNextPage();
    }
  }, [activeIndex, queryClient, reels, reelsQuery]);

  function toggleLike(reelId: string) {
    setLikedReels((current) => {
      const next = new Set(current);
      if (next.has(reelId)) {
        next.delete(reelId);
      } else {
        next.add(reelId);
      }
      return next;
    });
  }

  return (
    <div className="fixed inset-0 bg-slate-950 text-white">
      <header className="pointer-events-none fixed inset-x-0 top-0 z-30 flex items-center justify-between gap-3 px-4 py-4">
        <Link
          to="/"
          className="pointer-events-auto rounded-md bg-black/40 px-3 py-2 text-sm font-semibold text-white backdrop-blur hover:bg-black/60"
        >
          Home
        </Link>
        <div className="rounded-full bg-black/40 px-3 py-1.5 text-sm font-semibold backdrop-blur">Reels</div>
        <button
          type="button"
          onClick={() => setMuted((value) => !value)}
          className="pointer-events-auto rounded-md bg-black/40 px-3 py-2 text-sm font-semibold text-white backdrop-blur hover:bg-black/60"
        >
          {muted ? 'Unmute' : 'Mute'}
        </button>
      </header>

      {reelsQuery.isLoading ? (
        <CenteredState label="Loading reels" />
      ) : reelsQuery.isError ? (
        <CenteredState
          label={reelsQuery.error instanceof Error ? reelsQuery.error.message : 'Reels failed to load'}
        />
      ) : reels.length === 0 ? (
        <CenteredState label="No reels yet" />
      ) : (
        <div ref={containerRef} className="h-screen snap-y snap-mandatory overflow-y-auto overscroll-contain">
          {reels.map((reel, index) => (
            <ReelSlide
              key={reel.id}
              reel={reel}
              active={index === activeIndex}
              prefetch={index === activeIndex + 1}
              muted={muted}
              liked={likedReels.has(reel.id)}
              register={(node) => {
                itemRefs.current[index] = node;
              }}
              index={index}
              onToggleMute={() => setMuted((value) => !value)}
              onToggleLike={() => toggleLike(reel.id)}
            />
          ))}
          {reelsQuery.isFetchingNextPage ? (
            <section className="grid h-screen snap-start place-items-center">
              <p className="rounded-md bg-white/10 px-4 py-3 text-sm font-semibold text-white/70">
                Loading more reels
              </p>
            </section>
          ) : null}
        </div>
      )}

      {activeReel ? (
        <div className="pointer-events-none fixed bottom-4 left-4 z-30 text-xs font-medium text-white/50">
          {activeIndex + 1} / {reels.length}
        </div>
      ) : null}
    </div>
  );
}

function ReelSlide({
  reel,
  active,
  prefetch,
  muted,
  liked,
  register,
  index,
  onToggleMute,
  onToggleLike,
}: {
  reel: ReelResponse;
  active: boolean;
  prefetch: boolean;
  muted: boolean;
  liked: boolean;
  register: (node: HTMLElement | null) => void;
  index: number;
  onToggleMute: () => void;
  onToggleLike: () => void;
}) {
  const [commentsOpen, setCommentsOpen] = useState(false);
  const [shareStatus, setShareStatus] = useState<string | null>(null);
  const videoSource = mediaAssetUrl(hlsMasterKey(reel) ?? reel.media.original_key);
  const posterSource = mediaAssetUrl(posterKey(reel));
  const audioLabel = reel.audio.title ?? (reel.audio.is_original ? `Original audio by @${reel.author.handle}` : 'Audio');

  async function handleShare() {
    const url = `${window.location.origin}/reels?reel=${encodeURIComponent(reel.id)}`;
    try {
      const navigatorWithShare = navigator as Navigator & {
        share?: (data: { title: string; url: string }) => Promise<void>;
      };
      if (navigatorWithShare.share) {
        await navigatorWithShare.share({ title: `@${reel.author.handle} on Yet another Instagram`, url });
      } else {
        await navigator.clipboard.writeText(url);
        setShareStatus('Copied');
      }
    } catch {
      setShareStatus('Share canceled');
    }
  }

  useEffect(() => {
    if (!shareStatus) {
      return;
    }

    const timeout = window.setTimeout(() => setShareStatus(null), 1800);
    return () => window.clearTimeout(timeout);
  }, [shareStatus]);

  return (
    <section
      ref={register}
      data-reel-index={index}
      className="relative grid h-screen snap-start place-items-center px-4 py-16"
    >
      <div className="relative h-full w-full max-w-[430px] overflow-hidden rounded-lg bg-black shadow-2xl">
        {videoSource ? (
          <HlsVideo
            src={videoSource}
            poster={posterSource ?? undefined}
            active={active}
            prefetch={prefetch}
            muted={muted}
          />
        ) : (
          <div className="grid h-full place-items-center bg-slate-900 p-6 text-center">
            <p className="break-all text-sm font-medium text-white/70">{displayMediaKey(reel)}</p>
          </div>
        )}

        <div className="pointer-events-none absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent p-4 pr-20">
          <Link to={`/u/${reel.author.handle}`} className="pointer-events-auto text-sm font-semibold text-white">
            @{reel.author.handle}
          </Link>
          {reel.caption ? <p className="mt-2 line-clamp-3 text-sm leading-5 text-white/90">{reel.caption}</p> : null}
          <p className="mt-3 truncate text-xs font-medium text-white/65">
            {audioLabel}
            {reel.audio.artist ? ` - ${reel.audio.artist}` : ''}
          </p>
        </div>

        <div className="absolute bottom-24 right-3 z-20 grid gap-3">
          <OverlayButton active={liked} label={liked ? 'Liked' : 'Like'} onClick={onToggleLike} />
          <OverlayButton label="Comment" onClick={() => setCommentsOpen((value) => !value)} />
          <OverlayButton label="Share" onClick={() => void handleShare()} />
          <OverlayButton label={muted ? 'Sound off' : 'Sound on'} onClick={onToggleMute} />
        </div>

        {commentsOpen ? (
          <div className="absolute inset-x-3 bottom-3 z-30 rounded-md border border-white/15 bg-black/80 p-3 backdrop-blur">
            <div className="flex items-center justify-between gap-3">
              <p className="text-sm font-semibold">Comments</p>
              <button type="button" className="text-xs font-semibold text-white/70" onClick={() => setCommentsOpen(false)}>
                Close
              </button>
            </div>
            <textarea
              rows={3}
              placeholder="Comment support lands with reel conversations later"
              disabled
              className="mt-3 w-full resize-none rounded-md border border-white/15 bg-white/10 px-3 py-2 text-sm text-white placeholder:text-white/40"
            />
          </div>
        ) : null}

        {shareStatus ? (
          <div className="absolute right-3 top-16 rounded-md bg-white px-3 py-2 text-xs font-semibold text-slate-950">
            {shareStatus}
          </div>
        ) : null}
      </div>
    </section>
  );
}

function HlsVideo({
  src,
  poster,
  active,
  prefetch,
  muted,
}: {
  src: string;
  poster?: string;
  active: boolean;
  prefetch: boolean;
  muted: boolean;
}) {
  const videoRef = useRef<HTMLVideoElement | null>(null);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) {
      return;
    }
    const mediaElement = video;
    if (!active && !prefetch) {
      mediaElement.removeAttribute('src');
      mediaElement.load();
      return;
    }

    let canceled = false;
    let cleanup = () => {
      mediaElement.removeAttribute('src');
      mediaElement.load();
    };

    async function attachSource() {
      if (src.endsWith('.m3u8')) {
        const { default: Hls } = await import('hls.js');
        if (canceled) {
          return;
        }

        if (Hls.isSupported()) {
          const hls = new Hls({ enableWorker: true });
          hls.loadSource(src);
          hls.attachMedia(mediaElement);
          cleanup = () => hls.destroy();
          return;
        }

        if (mediaElement.canPlayType('application/vnd.apple.mpegurl')) {
          mediaElement.src = src;
          return;
        }
      }

      mediaElement.src = src;
    }

    void attachSource();

    return () => {
      canceled = true;
      cleanup();
    };
  }, [active, prefetch, src]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video) {
      return;
    }

    video.muted = muted;
    if (active) {
      void video.play().catch(() => undefined);
    } else {
      video.pause();
    }
  }, [active, muted]);

  return (
    <video
      ref={videoRef}
      poster={poster}
      className="h-full w-full object-cover"
      playsInline
      loop
      muted={muted}
      controls={false}
      preload={active ? 'auto' : 'metadata'}
    />
  );
}

function OverlayButton({
  label,
  active,
  onClick,
}: {
  label: string;
  active?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        'grid size-14 place-items-center rounded-full border text-[11px] font-semibold leading-tight backdrop-blur',
        active
          ? 'border-emerald-300 bg-emerald-400 text-emerald-950'
          : 'border-white/25 bg-black/45 text-white hover:bg-black/65',
      ].join(' ')}
    >
      {label}
    </button>
  );
}

function CenteredState({ label }: { label: string }) {
  return (
    <div className="grid h-screen place-items-center px-4 text-center">
      <p className="rounded-md bg-white/10 px-4 py-3 text-sm font-semibold text-white/70">{label}</p>
    </div>
  );
}

function hlsMasterKey(reel: ReelResponse) {
  const hls = reel.media.variants.hls;
  if (!hls || typeof hls !== 'object' || !('master_key' in hls)) {
    return null;
  }

  const key = (hls as { master_key?: unknown }).master_key;
  return typeof key === 'string' ? key : null;
}

function posterKey(reel: ReelResponse) {
  return variantKey(reel.media.variants.poster);
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

function displayMediaKey(reel: ReelResponse) {
  return hlsMasterKey(reel) ?? reel.media.original_key;
}
