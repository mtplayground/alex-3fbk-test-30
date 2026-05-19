import { useQueryClient } from '@tanstack/react-query';
import { useCallback, useEffect, useMemo, useState } from 'react';

import { useRealtime } from './RealtimeProvider';
import type {
  NotificationEventPayload,
  PresencePayload,
  ReadEventPayload,
  RealtimeEvent,
  RealtimeMessage,
  TypingEventPayload,
} from './types';

const TYPING_TIMEOUT_MS = 4_000;

export function useConversation(conversationId: string | null | undefined) {
  const { client, status } = useRealtime();
  const [messages, setMessages] = useState<RealtimeMessage[]>([]);
  const [readReceipts, setReadReceipts] = useState<ReadEventPayload[]>([]);

  useEffect(() => {
    if (!conversationId) {
      return undefined;
    }

    return client.addConversation(conversationId);
  }, [client, conversationId]);

  useEffect(() => {
    if (!conversationId) {
      return undefined;
    }

    return client.subscribe((event) => {
      if (event.type === 'message' && event.conversation_id === conversationId) {
        setMessages((current) => appendUniqueMessage(current, event.message));
      }
      if (event.type === 'read' && event.conversation_id === conversationId) {
        setReadReceipts((current) => [...current, event]);
      }
    });
  }, [client, conversationId]);

  const sendTyping = useCallback(
    (isTyping = true) => {
      if (!conversationId) {
        return false;
      }
      return client.sendTyping(conversationId, isTyping);
    },
    [client, conversationId],
  );

  const sendRead = useCallback(
    (messageId: string) => {
      if (!conversationId) {
        return false;
      }
      return client.sendRead(conversationId, messageId);
    },
    [client, conversationId],
  );

  return {
    status,
    messages,
    readReceipts,
    sendTyping,
    sendRead,
  };
}

export function useTyping(conversationId: string | null | undefined) {
  const { client } = useRealtime();
  const [typingUsers, setTypingUsers] = useState<Record<string, boolean>>({});

  useEffect(() => {
    if (!conversationId) {
      return undefined;
    }

    const timers = new Map<string, number>();
    const clearTyping = (userId: string) => {
      const timer = timers.get(userId);
      if (timer !== undefined) {
        window.clearTimeout(timer);
      }
      timers.set(
        userId,
        window.setTimeout(() => {
          setTypingUsers((current) => ({ ...current, [userId]: false }));
          timers.delete(userId);
        }, TYPING_TIMEOUT_MS),
      );
    };

    const unsubscribe = client.subscribe((event) => {
      if (event.type !== 'typing' || event.conversation_id !== conversationId) {
        return;
      }

      setTypingUsers((current) => ({ ...current, [event.user_id]: event.is_typing }));
      if (event.is_typing) {
        clearTyping(event.user_id);
      }
    });

    return () => {
      unsubscribe();
      timers.forEach((timer) => window.clearTimeout(timer));
    };
  }, [client, conversationId]);

  const sendTyping = useCallback(
    (isTyping = true) => {
      if (!conversationId) {
        return false;
      }
      return client.sendTyping(conversationId, isTyping);
    },
    [client, conversationId],
  );

  const activeTypingUsers = useMemo(
    () => Object.entries(typingUsers).filter(([, isTyping]) => isTyping).map(([userId]) => userId),
    [typingUsers],
  );

  return {
    typingUsers,
    activeTypingUsers,
    sendTyping,
  };
}

export function usePresence(userIds: string[] = []) {
  const { client, status } = useRealtime();
  const [presences, setPresences] = useState<Record<string, PresencePayload>>({});
  const watchedUserKey = userIds.join('|');
  const watchedUsers = useMemo(() => new Set(userIds), [watchedUserKey]);

  useEffect(
    () =>
      client.subscribe((event) => {
        if (event.type !== 'presence') {
          return;
        }
        if (watchedUsers.size > 0 && !watchedUsers.has(event.presence.user_id)) {
          return;
        }

        setPresences((current) => ({
          ...current,
          [event.presence.user_id]: event.presence,
        }));
      }),
    [client, watchedUsers],
  );

  const isOnline = useCallback(
    (userId: string) => presences[userId]?.status === 'online',
    [presences],
  );

  return {
    status,
    presences,
    isOnline,
  };
}

export function useNotificationsSubscription(
  onNotification?: (notification: NotificationEventPayload) => void,
) {
  const { client } = useRealtime();
  const queryClient = useQueryClient();
  const [latestNotification, setLatestNotification] = useState<NotificationEventPayload | null>(
    null,
  );

  useEffect(
    () =>
      client.subscribe((event: RealtimeEvent) => {
        if (event.type !== 'notification') {
          return;
        }

        setLatestNotification(event.notification);
        onNotification?.(event.notification);
        queryClient.invalidateQueries({ queryKey: ['notifications'] });
        queryClient.invalidateQueries({ queryKey: ['notifications', 'unread-count'] });
      }),
    [client, onNotification, queryClient],
  );

  return latestNotification;
}

function appendUniqueMessage(messages: RealtimeMessage[], message: RealtimeMessage) {
  if (messages.some((current) => current.id === message.id)) {
    return messages;
  }

  return [...messages, message];
}

export type { NotificationEventPayload, TypingEventPayload };
