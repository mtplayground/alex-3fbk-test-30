import { FormEvent, KeyboardEvent, useEffect, useMemo, useRef, useState } from 'react';
import { useMutation } from '@tanstack/react-query';

import { MediaUploader, type MediaUploadResult } from '../media/MediaUploader';
import { createPost } from './api';

type ComposerAsset = MediaUploadResult & {
  altText: string;
  preset: FilterPreset;
};

type FilterPreset = 'clean' | 'warm' | 'cool' | 'mono';

const hashtagSuggestions = ['studio', 'travel', 'portrait', 'food', 'design', 'daily'];
const mentionSuggestions = ['mira', 'atlas', 'noor', 'zero'];

const presetFilters: Record<FilterPreset, string> = {
  clean: '',
  warm: 'sepia(0.18) saturate(1.12) contrast(1.03)',
  cool: 'saturate(1.08) hue-rotate(8deg) brightness(1.02)',
  mono: 'grayscale(1) contrast(1.12)',
};

export function PostComposer() {
  const [assets, setAssets] = useState<ComposerAsset[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [caption, setCaption] = useState('');
  const [location, setLocation] = useState('');
  const [brightness, setBrightness] = useState(100);
  const [contrast, setContrast] = useState(100);
  const [saturation, setSaturation] = useState(100);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const captionRef = useRef<HTMLTextAreaElement | null>(null);
  const assetsRef = useRef<ComposerAsset[]>([]);

  const activeAsset = assets[activeIndex];
  assetsRef.current = assets;
  const suggestions = useMemo(() => captionSuggestions(caption, captionRef.current?.selectionStart ?? caption.length), [
    caption,
  ]);

  useEffect(() => () => revokePreviewUrls(assetsRef.current), []);

  const publish = useMutation({
    mutationFn: () =>
      createPost({
        media_ids: assets.map((asset) => asset.asset_id),
        caption,
        location: location.trim() || undefined,
      }),
    onSuccess: () => {
      setMessage('Post published.');
      setError(null);
      revokePreviewUrls(assets);
      setAssets([]);
      setActiveIndex(0);
      setCaption('');
      setLocation('');
    },
    onError: (requestError) => {
      setMessage(null);
      setError(requestError instanceof Error ? requestError.message : 'Post failed');
    },
  });

  function handleUploaded(result: MediaUploadResult) {
    setAssets((current) => {
      setActiveIndex(current.length);
      return [
        ...current,
        {
          ...result,
          altText: '',
          preset: 'clean',
        },
      ];
    });
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setMessage(null);
    setError(null);

    if (assets.length === 0) {
      setError('Upload at least one image or video.');
      return;
    }

    publish.mutate();
  }

  function updateActiveAsset(patch: Partial<ComposerAsset>) {
    setAssets((current) => current.map((asset, index) => (index === activeIndex ? { ...asset, ...patch } : asset)));
  }

  function insertSuggestion(value: string) {
    const textarea = captionRef.current;
    const cursor = textarea?.selectionStart ?? caption.length;
    const token = activeToken(caption, cursor);
    if (!token) {
      return;
    }

    const next = `${caption.slice(0, token.start)}${token.marker}${value} ${caption.slice(cursor)}`;
    setCaption(next);
    requestAnimationFrame(() => {
      textarea?.focus();
      const position = token.start + value.length + 2;
      textarea?.setSelectionRange(position, position);
    });
  }

  function handleCaptionKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key !== 'Tab' || suggestions.length === 0) {
      return;
    }

    event.preventDefault();
    insertSuggestion(suggestions[0]);
  }

  const filter = [
    presetFilters[activeAsset?.preset ?? 'clean'],
    `brightness(${brightness}%)`,
    `contrast(${contrast}%)`,
    `saturate(${saturation}%)`,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
      <form className="grid gap-5" onSubmit={handleSubmit}>
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-950">Create post</h2>
            <p className="mt-1 text-sm text-slate-600">Compose media, caption, location, and presentation.</p>
          </div>
          <button
            type="submit"
            disabled={publish.isPending || assets.length === 0}
            className="rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
          >
            {publish.isPending ? 'Publishing' : 'Publish'}
          </button>
        </div>

        <MediaUploader surface="post" onUploaded={handleUploaded} />

        {assets.length > 0 ? (
          <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_280px]">
            <div className="overflow-hidden rounded-lg border border-slate-200">
              <div
                className="flex aspect-[4/3] items-center justify-center bg-slate-100 transition"
                style={{ filter }}
              >
                {activeAsset.preview_url ? (
                  activeAsset.kind === 'image' ? (
                    <img
                      src={activeAsset.preview_url}
                      alt={activeAsset.altText || activeAsset.file_name}
                      className="h-full w-full object-contain"
                    />
                  ) : (
                    <video
                      src={activeAsset.preview_url}
                      controls
                      className="h-full w-full object-contain"
                      aria-label={activeAsset.file_name}
                    />
                  )
                ) : (
                  <div className="rounded-md bg-white/80 px-3 py-2 text-sm font-semibold text-slate-800">
                    {activeAsset.file_name}
                  </div>
                )}
              </div>
              <div className="flex items-center justify-between gap-3 border-t border-slate-200 p-3">
                <button
                  type="button"
                  onClick={() => setActiveIndex((current) => Math.max(0, current - 1))}
                  disabled={activeIndex === 0}
                  className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium disabled:cursor-not-allowed disabled:text-slate-400"
                >
                  Previous
                </button>
                <span className="text-sm font-medium text-slate-600">
                  {activeIndex + 1} / {assets.length}
                </span>
                <button
                  type="button"
                  onClick={() => setActiveIndex((current) => Math.min(assets.length - 1, current + 1))}
                  disabled={activeIndex === assets.length - 1}
                  className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium disabled:cursor-not-allowed disabled:text-slate-400"
                >
                  Next
                </button>
              </div>
            </div>

            <div className="grid gap-4 rounded-lg border border-slate-200 p-4">
              <label className="grid gap-2">
                <span className="text-sm font-medium text-slate-700">Alt text</span>
                <textarea
                  value={activeAsset.altText}
                  onChange={(event) => updateActiveAsset({ altText: event.target.value })}
                  rows={3}
                  className="resize-none rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
                />
              </label>

              <div className="grid gap-2">
                <span className="text-sm font-medium text-slate-700">Preset</span>
                <div className="grid grid-cols-2 gap-2">
                  {(['clean', 'warm', 'cool', 'mono'] as FilterPreset[]).map((preset) => (
                    <button
                      key={preset}
                      type="button"
                      onClick={() => updateActiveAsset({ preset })}
                      className={[
                        'rounded-md border px-3 py-2 text-sm font-medium capitalize',
                        activeAsset.preset === preset
                          ? 'border-slate-950 bg-slate-950 text-white'
                          : 'border-slate-300 text-slate-700 hover:border-slate-950',
                      ].join(' ')}
                    >
                      {preset}
                    </button>
                  ))}
                </div>
              </div>

              <FilterSlider label="Brightness" value={brightness} onChange={setBrightness} />
              <FilterSlider label="Contrast" value={contrast} onChange={setContrast} />
              <FilterSlider label="Saturation" value={saturation} onChange={setSaturation} />
            </div>
          </div>
        ) : null}

        <label className="grid gap-2">
          <span className="text-sm font-medium text-slate-700">Caption</span>
          <textarea
            ref={captionRef}
            value={caption}
            onChange={(event) => setCaption(event.target.value)}
            onKeyDown={handleCaptionKeyDown}
            rows={4}
            className="resize-none rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
          />
        </label>

        {suggestions.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {suggestions.map((suggestion) => (
              <button
                key={suggestion}
                type="button"
                onClick={() => insertSuggestion(suggestion)}
                className="rounded-full bg-emerald-50 px-3 py-1 text-sm font-semibold text-emerald-800 hover:bg-emerald-100"
              >
                {suggestion}
              </button>
            ))}
          </div>
        ) : null}

        <label className="grid gap-2">
          <span className="text-sm font-medium text-slate-700">Location</span>
          <input
            value={location}
            onChange={(event) => setLocation(event.target.value)}
            className="rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
          />
        </label>

        {message ? <p className="rounded-md bg-emerald-50 px-3 py-2 text-sm text-emerald-800">{message}</p> : null}
        {error ? <p className="rounded-md bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}
      </form>
    </section>
  );
}

function FilterSlider({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="grid gap-2">
      <span className="flex items-center justify-between text-sm font-medium text-slate-700">
        {label}
        <span className="text-slate-500">{value}%</span>
      </span>
      <input
        type="range"
        min={50}
        max={150}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="accent-slate-950"
      />
    </label>
  );
}

function captionSuggestions(caption: string, cursor: number) {
  const token = activeToken(caption, cursor);
  if (!token) {
    return [];
  }

  const source = token.marker === '#' ? hashtagSuggestions : mentionSuggestions;
  return source.filter((item) => item.startsWith(token.value.toLowerCase())).slice(0, 5);
}

function activeToken(caption: string, cursor: number) {
  const beforeCursor = caption.slice(0, cursor);
  const match = /(^|\s)([#@])([a-zA-Z0-9_]*)$/.exec(beforeCursor);
  if (!match) {
    return null;
  }

  return {
    marker: match[2],
    value: match[3],
    start: beforeCursor.length - match[2].length - match[3].length,
  };
}

function revokePreviewUrls(assets: ComposerAsset[]) {
  assets.forEach((asset) => {
    if (asset.preview_url) {
      URL.revokeObjectURL(asset.preview_url);
    }
  });
}
