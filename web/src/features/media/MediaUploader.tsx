import { ChangeEvent, useRef, useState } from 'react';

import {
  completeMediaUpload,
  requestMediaUpload,
  uploadMediaBlob,
  type MediaKind,
  type MediaUploadCompleteResponse,
} from './api';

type UploadStatus = 'queued' | 'preparing' | 'uploading' | 'completing' | 'complete' | 'failed';

export type MediaUploadSurface = 'post' | 'story' | 'reel' | 'dm';

export type MediaUploadResult = MediaUploadCompleteResponse & {
  file_name: string;
  kind: MediaKind;
};

type UploadItem = {
  id: string;
  fileName: string;
  kind: MediaKind;
  status: UploadStatus;
  progress: number;
  assetId?: string;
  error?: string;
};

type MediaUploaderProps = {
  surface: MediaUploadSurface;
  onUploaded?: (result: MediaUploadResult) => void;
  accept?: string;
  maxImageDimension?: number;
};

const DEFAULT_ACCEPT = 'image/jpeg,image/png,image/webp,image/gif,video/mp4,video/webm,video/quicktime,video/mpeg';
const DEFAULT_MAX_IMAGE_DIMENSION = 2048;

const surfaceLabels: Record<MediaUploadSurface, string> = {
  post: 'Post media',
  story: 'Story media',
  reel: 'Reel video',
  dm: 'Message attachment',
};

export function MediaUploader({
  surface,
  onUploaded,
  accept = DEFAULT_ACCEPT,
  maxImageDimension = DEFAULT_MAX_IMAGE_DIMENSION,
}: MediaUploaderProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [items, setItems] = useState<UploadItem[]>([]);

  async function handleFiles(event: ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files ?? []);
    event.target.value = '';

    const queued = files.map(createUploadItem);
    setItems((current) => [...queued, ...current]);

    queued.forEach((item, index) => {
      void uploadFile(item, files[index]);
    });
  }

  async function uploadFile(item: UploadItem, file: File | undefined) {
    if (!file) {
      markFailed(item.id, 'File could not be read.');
      return;
    }

    try {
      updateItem(item.id, { status: 'preparing', progress: 0 });
      const prepared = await prepareFileForUpload(file, item.kind, maxImageDimension);
      const upload = await requestMediaUpload({
        kind: item.kind,
        content_type: prepared.type || file.type || undefined,
      });

      updateItem(item.id, { assetId: upload.asset_id, status: 'uploading', progress: 1 });
      await uploadMediaBlob(upload.upload_url, prepared, (progress) => {
        updateItem(item.id, { progress });
      });

      updateItem(item.id, { status: 'completing', progress: 100 });
      const completed = await completeMediaUpload(upload.asset_id);
      updateItem(item.id, { status: 'complete', progress: 100 });
      onUploaded?.({
        ...completed,
        file_name: file.name,
        kind: item.kind,
      });
    } catch (error) {
      markFailed(item.id, errorMessage(error));
    }
  }

  function updateItem(id: string, patch: Partial<UploadItem>) {
    setItems((current) => current.map((item) => (item.id === id ? { ...item, ...patch } : item)));
  }

  function markFailed(id: string, error: string) {
    updateItem(id, { status: 'failed', error });
  }

  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-soft">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-base font-semibold text-slate-950">{surfaceLabels[surface]}</h2>
          <p className="mt-1 text-sm text-slate-600">Images and videos upload directly to storage.</p>
        </div>
        <input ref={inputRef} type="file" accept={accept} multiple onChange={handleFiles} className="hidden" />
        <button
          type="button"
          onClick={() => inputRef.current?.click()}
          className="w-full rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 sm:w-auto"
        >
          Select files
        </button>
      </div>

      {items.length > 0 ? (
        <div className="mt-4 divide-y divide-slate-100 rounded-md border border-slate-200">
          {items.map((item) => (
            <div key={item.id} className="grid gap-2 p-3">
              <div className="flex items-center justify-between gap-3">
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium text-slate-950">{item.fileName}</p>
                  <p className="text-xs capitalize text-slate-500">{item.kind}</p>
                </div>
                <span className={statusClassName(item.status)}>{statusLabel(item.status)}</span>
              </div>
              <div className="h-2 overflow-hidden rounded-full bg-slate-100">
                <div
                  className="h-full rounded-full bg-emerald-500 transition-all"
                  style={{ width: `${item.progress}%` }}
                />
              </div>
              {item.error ? <p className="text-sm text-rose-700">{item.error}</p> : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function createUploadItem(file: File): UploadItem {
  return {
    id: `${file.name}-${file.lastModified}-${crypto.randomUUID()}`,
    fileName: file.name,
    kind: mediaKind(file),
    status: 'queued',
    progress: 0,
  };
}

function mediaKind(file: File): MediaKind {
  return file.type.startsWith('video/') ? 'video' : 'image';
}

async function prepareFileForUpload(file: File, kind: MediaKind, maxImageDimension: number): Promise<Blob> {
  if (kind !== 'image' || file.type === 'image/gif') {
    return file;
  }

  const bitmap = await createOrientedBitmap(file);
  const scale = Math.min(1, maxImageDimension / Math.max(bitmap.width, bitmap.height));
  const width = Math.max(1, Math.round(bitmap.width * scale));
  const height = Math.max(1, Math.round(bitmap.height * scale));
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;

  const context = canvas.getContext('2d');
  if (!context) {
    bitmap.close();
    return file;
  }

  context.drawImage(bitmap, 0, 0, width, height);
  bitmap.close();

  return canvasToBlob(canvas, normalizedImageType(file.type));
}

async function createOrientedBitmap(file: File) {
  try {
    return await createImageBitmap(file, { imageOrientation: 'from-image' } as ImageBitmapOptions);
  } catch {
    return createImageBitmap(file);
  }
}

function canvasToBlob(canvas: HTMLCanvasElement, type: string) {
  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob) {
          resolve(blob);
          return;
        }

        reject(new Error('Image resize failed.'));
      },
      type,
      0.9,
    );
  });
}

function normalizedImageType(type: string) {
  return type === 'image/png' || type === 'image/webp' ? type : 'image/jpeg';
}

function statusLabel(status: UploadStatus) {
  switch (status) {
    case 'queued':
      return 'Queued';
    case 'preparing':
      return 'Preparing';
    case 'uploading':
      return 'Uploading';
    case 'completing':
      return 'Completing';
    case 'complete':
      return 'Complete';
    case 'failed':
      return 'Failed';
  }
}

function statusClassName(status: UploadStatus) {
  const base = 'shrink-0 rounded-full px-2.5 py-1 text-xs font-semibold';
  if (status === 'complete') {
    return `${base} bg-emerald-50 text-emerald-700`;
  }
  if (status === 'failed') {
    return `${base} bg-rose-50 text-rose-700`;
  }
  return `${base} bg-slate-100 text-slate-700`;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Upload failed';
}
