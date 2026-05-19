import { ChangeEvent, useEffect, useState } from 'react';

type AvatarCropperProps = {
  onChange: (blob: Blob | null) => void;
};

const OUTPUT_SIZE = 512;

export function AvatarCropper({ onChange }: AvatarCropperProps) {
  const [sourceUrl, setSourceUrl] = useState<string | null>(null);
  const [scale, setScale] = useState(1);

  useEffect(() => {
    if (!sourceUrl) {
      onChange(null);
      return;
    }

    let active = true;

    cropImage(sourceUrl, scale)
      .then((blob) => {
        if (active) {
          onChange(blob);
        }
      })
      .catch(() => {
        if (active) {
          onChange(null);
        }
      });

    return () => {
      active = false;
    };
  }, [onChange, scale, sourceUrl]);

  useEffect(() => {
    return () => {
      if (sourceUrl) {
        URL.revokeObjectURL(sourceUrl);
      }
    };
  }, [sourceUrl]);

  function handleFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      setSourceUrl(null);
      return;
    }

    setScale(1);
    setSourceUrl((current) => {
      if (current) {
        URL.revokeObjectURL(current);
      }
      return URL.createObjectURL(file);
    });
  }

  return (
    <div className="space-y-4">
      <label className="block">
        <span className="text-sm font-medium text-slate-700">Avatar image</span>
        <input
          type="file"
          accept="image/png,image/jpeg,image/webp"
          onChange={handleFileChange}
          className="mt-2 block w-full text-sm text-slate-700 file:mr-3 file:rounded-md file:border-0 file:bg-slate-950 file:px-3 file:py-2 file:text-sm file:font-semibold file:text-white"
        />
      </label>

      {sourceUrl ? (
        <div className="grid gap-4 sm:grid-cols-[200px_minmax(0,1fr)]">
          <div className="size-48 overflow-hidden rounded-full border border-slate-200 bg-slate-100">
            <img
              src={sourceUrl}
              alt=""
              className="size-full object-cover"
              style={{ transform: `scale(${scale})` }}
            />
          </div>
          <label className="block self-center">
            <span className="text-sm font-medium text-slate-700">Crop zoom</span>
            <input
              type="range"
              min="1"
              max="2"
              step="0.05"
              value={scale}
              onChange={(event) => setScale(Number(event.target.value))}
              className="mt-3 w-full accent-slate-950"
            />
          </label>
        </div>
      ) : null}
    </div>
  );
}

async function cropImage(sourceUrl: string, scale: number): Promise<Blob> {
  const image = await loadImage(sourceUrl);
  const canvas = document.createElement('canvas');
  canvas.width = OUTPUT_SIZE;
  canvas.height = OUTPUT_SIZE;
  const context = canvas.getContext('2d');

  if (!context) {
    throw new Error('Canvas is unavailable');
  }

  const sourceSize = Math.min(image.naturalWidth, image.naturalHeight) / scale;
  const sourceX = (image.naturalWidth - sourceSize) / 2;
  const sourceY = (image.naturalHeight - sourceSize) / 2;

  context.drawImage(image, sourceX, sourceY, sourceSize, sourceSize, 0, 0, OUTPUT_SIZE, OUTPUT_SIZE);

  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob) {
          resolve(blob);
        } else {
          reject(new Error('Avatar crop failed'));
        }
      },
      'image/jpeg',
      0.9,
    );
  });
}

function loadImage(sourceUrl: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error('Image could not be loaded'));
    image.src = sourceUrl;
  });
}
