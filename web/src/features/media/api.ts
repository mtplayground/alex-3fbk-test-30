import { apiRequest } from '../auth/api';

export type MediaKind = 'image' | 'video';

export type MediaUploadRequest = {
  kind: MediaKind;
  content_type?: string;
};

export type MediaUploadResponse = {
  asset_id: string;
  key: string;
  upload_url: string;
  method: 'PUT';
  expires_in: number;
};

export type MediaUploadCompleteResponse = {
  asset_id: string;
  status: 'uploaded';
  job_id: string;
  job_kind: 'image_processing' | 'video_processing';
};

export function requestMediaUpload(payload: MediaUploadRequest) {
  return apiRequest<MediaUploadResponse>('/media/uploads', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function completeMediaUpload(assetId: string) {
  return apiRequest<MediaUploadCompleteResponse>(`/media/uploads/${encodeURIComponent(assetId)}/complete`, {
    method: 'POST',
  });
}

export function uploadMediaBlob(
  uploadUrl: string,
  blob: Blob,
  onProgress: (progress: number) => void,
) {
  return new Promise<void>((resolve, reject) => {
    const request = new XMLHttpRequest();

    request.upload.onprogress = (event) => {
      if (!event.lengthComputable) {
        return;
      }

      onProgress(Math.round((event.loaded / event.total) * 100));
    };

    request.onload = () => {
      if (request.status >= 200 && request.status < 300) {
        onProgress(100);
        resolve();
        return;
      }

      reject(new Error(`Upload failed with status ${request.status}`));
    };

    request.onerror = () => {
      reject(new Error('Upload failed'));
    };

    request.onabort = () => {
      reject(new Error('Upload cancelled'));
    };

    request.open('PUT', uploadUrl);
    request.setRequestHeader('Content-Type', blob.type || 'application/octet-stream');
    request.send(blob);
  });
}
