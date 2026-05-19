import { apiRequest } from '../auth/api';
import type { NotificationEventPayload } from '../realtime/types';

export type NotificationsPage = {
  notifications: NotificationEventPayload[];
  next_cursor: string | null;
};

export type UnreadCountResponse = {
  unread_count: number;
};

export type ReadAllResponse = {
  updated_count: number;
};

export function listNotifications(cursor?: string | null, limit = 20): Promise<NotificationsPage> {
  const search = new URLSearchParams({ limit: String(limit) });
  if (cursor) {
    search.set('cursor', cursor);
  }

  return apiRequest<NotificationsPage>(`/notifications?${search.toString()}`);
}

export function getUnreadCount(): Promise<UnreadCountResponse> {
  return apiRequest<UnreadCountResponse>('/notifications/unread-count');
}

export function markAllNotificationsRead(): Promise<ReadAllResponse> {
  return apiRequest<ReadAllResponse>('/notifications/read-all', {
    method: 'POST',
  });
}
