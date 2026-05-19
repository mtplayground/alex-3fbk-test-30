import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';

import {
  getUnreadCount,
  listNotifications,
  markAllNotificationsRead,
  type NotificationsPage,
} from './api';
import { useNotificationsSubscription } from '../realtime/hooks';

const NOTIFICATION_PAGE_SIZE = 20;

export function useNotifications() {
  const queryClient = useQueryClient();

  const notificationsQuery = useInfiniteQuery({
    queryKey: ['notifications', 'list'],
    queryFn: ({ pageParam }) => listNotifications(pageParam, NOTIFICATION_PAGE_SIZE),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage: NotificationsPage) => lastPage.next_cursor,
  });

  const unreadCountQuery = useQuery({
    queryKey: ['notifications', 'unread-count'],
    queryFn: getUnreadCount,
  });

  const markAllReadMutation = useMutation({
    mutationFn: markAllNotificationsRead,
    onSuccess: () => {
      queryClient.setQueryData(['notifications', 'unread-count'], { unread_count: 0 });
      queryClient.invalidateQueries({ queryKey: ['notifications', 'list'] });
    },
  });

  useNotificationsSubscription(
    useCallback(() => {
      queryClient.invalidateQueries({ queryKey: ['notifications', 'list'] });
      queryClient.invalidateQueries({ queryKey: ['notifications', 'unread-count'] });
    }, [queryClient]),
  );

  return {
    notifications: notificationsQuery.data?.pages.flatMap((page) => page.notifications) ?? [],
    unreadCount: unreadCountQuery.data?.unread_count ?? 0,
    isLoading: notificationsQuery.isLoading || unreadCountQuery.isLoading,
    isFetchingNextPage: notificationsQuery.isFetchingNextPage,
    hasNextPage: notificationsQuery.hasNextPage,
    fetchNextPage: notificationsQuery.fetchNextPage,
    markAllRead: markAllReadMutation.mutateAsync,
    isMarkingAllRead: markAllReadMutation.isPending,
  };
}
