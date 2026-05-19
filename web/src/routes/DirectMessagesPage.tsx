import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { FormEvent, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';

import {
  createMessage,
  listConversations,
  listMessages,
  markConversationRead,
  type Conversation,
  type Message,
  type MessagesPage,
} from '../features/dm/api';
import { MediaUploader, type MediaUploadResult } from '../features/media/MediaUploader';
import { useAuth } from '../features/auth/AuthProvider';
import { useConversation, useTyping } from '../features/realtime/hooks';

const MESSAGE_PAGE_SIZE = 30;

export function DirectMessagesPage() {
  const { conversationId } = useParams();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const auth = useAuth();
  const currentUserId = auth.user?.id ?? null;
  const conversationsQuery = useQuery({
    queryKey: ['dm', 'conversations'],
    queryFn: listConversations,
  });
  const conversations = conversationsQuery.data?.conversations ?? [];
  const activeConversation = conversations.find((conversation) => conversation.id === conversationId) ?? null;
  const { messages: realtimeMessages, readReceipts, sendTyping, sendRead, status } = useConversation(conversationId);
  const typing = useTyping(conversationId);
  const [body, setBody] = useState('');
  const [attachment, setAttachment] = useState<MediaUploadResult | null>(null);
  const typingTimeoutRef = useRef<number | null>(null);
  const markedReadMessageRef = useRef<string | null>(null);
  const loadMoreRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!conversationId && conversations.length > 0) {
      navigate(`/dm/${conversations[0].id}`, { replace: true });
    }
  }, [conversationId, conversations, navigate]);

  const messagesQuery = useInfiniteQuery({
    queryKey: ['dm', 'messages', conversationId],
    queryFn: ({ pageParam }) => listMessages(conversationId ?? '', pageParam, MESSAGE_PAGE_SIZE),
    initialPageParam: null as string | null,
    getNextPageParam: (lastPage: MessagesPage) => lastPage.next_cursor,
    enabled: Boolean(conversationId),
  });

  const readMutation = useMutation({
    mutationFn: ({ targetConversationId, messageId }: { targetConversationId: string; messageId: string }) =>
      markConversationRead(targetConversationId, messageId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dm', 'conversations'] });
    },
  });

  const sendMessageMutation = useMutation({
    mutationFn: (payload: { body?: string; media_id?: string }) => createMessage(conversationId ?? '', payload),
    onSuccess: () => {
      setBody('');
      setAttachment(null);
      queryClient.invalidateQueries({ queryKey: ['dm', 'messages', conversationId] });
      queryClient.invalidateQueries({ queryKey: ['dm', 'conversations'] });
    },
  });

  const messages = useMemo(() => {
    const restMessages = messagesQuery.data?.pages.flatMap((page) => page.messages) ?? [];
    return mergeMessages([...restMessages].reverse(), realtimeMessages);
  }, [messagesQuery.data, realtimeMessages]);

  useEffect(() => {
    const latestMessage = messages[messages.length - 1];
    if (!conversationId || !latestMessage || latestMessage.id === markedReadMessageRef.current) {
      return;
    }

    markedReadMessageRef.current = latestMessage.id;
    readMutation.mutate({ targetConversationId: conversationId, messageId: latestMessage.id });
    sendRead(latestMessage.id);
  }, [conversationId, messages, readMutation, sendRead]);

  useEffect(() => {
    const node = loadMoreRef.current;
    if (!node || !messagesQuery.hasNextPage) {
      return undefined;
    }

    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting) && !messagesQuery.isFetchingNextPage) {
        void messagesQuery.fetchNextPage();
      }
    });
    observer.observe(node);

    return () => observer.disconnect();
  }, [messagesQuery]);

  function handleBodyChange(value: string) {
    setBody(value);
    sendTyping(true);
    if (typingTimeoutRef.current !== null) {
      window.clearTimeout(typingTimeoutRef.current);
    }
    typingTimeoutRef.current = window.setTimeout(() => {
      sendTyping(false);
      typingTimeoutRef.current = null;
    }, 1_200);
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = body.trim();
    if (!conversationId || (!trimmed && !attachment) || sendMessageMutation.isPending) {
      return;
    }

    sendTyping(false);
    sendMessageMutation.mutate({
      body: trimmed || undefined,
      media_id: attachment?.asset_id,
    });
  }

  return (
    <div className="grid min-h-[72vh] overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft lg:grid-cols-[320px_minmax(0,1fr)]">
      <aside className="border-b border-slate-200 lg:border-b-0 lg:border-r">
        <div className="flex items-center justify-between gap-3 border-b border-slate-200 p-4">
          <div>
            <h1 className="text-xl font-semibold text-slate-950">Messages</h1>
            <p className="text-sm text-slate-500">{statusLabel(status)}</p>
          </div>
        </div>

        <div className="max-h-[72vh] overflow-y-auto">
          {conversationsQuery.isLoading ? (
            <p className="p-4 text-sm text-slate-500">Loading conversations...</p>
          ) : conversations.length === 0 ? (
            <p className="p-4 text-sm text-slate-500">No conversations yet.</p>
          ) : (
            <div className="divide-y divide-slate-100">
              {conversations.map((conversation) => (
                <InboxRow
                  key={conversation.id}
                  conversation={conversation}
                  currentUserId={currentUserId}
                  isActive={conversation.id === conversationId}
                />
              ))}
            </div>
          )}
        </div>
      </aside>

      <section className="grid min-h-[72vh] grid-rows-[auto_minmax(0,1fr)_auto]">
        {activeConversation ? (
          <>
            <ThreadHeader conversation={activeConversation} currentUserId={currentUserId} />

            <div className="overflow-y-auto bg-slate-50 px-4 py-5">
              <div ref={loadMoreRef} className="h-2" />
              {messagesQuery.isFetchingNextPage ? (
                <p className="pb-3 text-center text-xs text-slate-500">Loading earlier messages...</p>
              ) : null}

              <div className="mx-auto flex max-w-3xl flex-col gap-3">
                {messagesQuery.isLoading ? (
                  <p className="text-sm text-slate-500">Loading thread...</p>
                ) : messages.length === 0 ? (
                  <p className="rounded-lg border border-dashed border-slate-300 bg-white p-6 text-center text-sm text-slate-500">
                    No messages in this thread yet.
                  </p>
                ) : (
                  messages.map((message) => (
                    <MessageBubble
                      key={message.id}
                      message={message}
                      isOwn={message.author_id === currentUserId}
                      isRead={isMessageRead(message.id, activeConversation, readReceipts, currentUserId)}
                    />
                  ))
                )}
              </div>
            </div>

            <div className="border-t border-slate-200 bg-white p-4">
              <div className="mx-auto max-w-3xl">
                <TypingDots
                  userIds={typing.activeTypingUsers.filter((userId) => userId !== currentUserId)}
                  conversation={activeConversation}
                  currentUserId={currentUserId}
                />
                {attachment ? (
                  <div className="mb-3 flex items-center justify-between gap-3 rounded-md border border-cyan-200 bg-cyan-50 px-3 py-2 text-sm text-cyan-950">
                    <span className="truncate">{attachment.file_name}</span>
                    <button type="button" className="font-semibold" onClick={() => setAttachment(null)}>
                      Remove
                    </button>
                  </div>
                ) : null}
                <form onSubmit={handleSubmit} className="grid gap-3">
                  <textarea
                    value={body}
                    onChange={(event) => handleBodyChange(event.target.value)}
                    rows={3}
                    placeholder="Write a message"
                    className="min-h-24 resize-none rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-cyan-500 focus:ring-2 focus:ring-cyan-100"
                  />
                  <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                    <div className="sm:max-w-sm">
                      <MediaUploader
                        surface="dm"
                        onUploaded={setAttachment}
                        accept="image/jpeg,image/png,image/webp,image/gif,video/mp4,video/webm"
                        maxImageDimension={1280}
                      />
                    </div>
                    <button
                      type="submit"
                      disabled={sendMessageMutation.isPending || (!body.trim() && !attachment)}
                      className="rounded-md bg-slate-950 px-5 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-300"
                    >
                      {sendMessageMutation.isPending ? 'Sending...' : 'Send'}
                    </button>
                  </div>
                </form>
              </div>
            </div>
          </>
        ) : (
          <div className="grid place-items-center bg-slate-50 p-8 text-center">
            <div>
              <h2 className="text-lg font-semibold text-slate-950">Select a conversation</h2>
              <p className="mt-2 text-sm text-slate-500">Your inbox threads appear here.</p>
            </div>
          </div>
        )}
      </section>
    </div>
  );
}

function InboxRow({
  conversation,
  currentUserId,
  isActive,
}: {
  conversation: Conversation;
  currentUserId: string | null;
  isActive: boolean;
}) {
  const unread = isUnread(conversation, currentUserId);
  const label = conversationLabel(conversation, currentUserId);

  return (
    <Link
      to={`/dm/${conversation.id}`}
      className={`grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-3 px-4 py-3 hover:bg-slate-50 ${
        isActive ? 'bg-cyan-50' : ''
      }`}
    >
      <span className="grid size-10 place-items-center rounded-full bg-slate-900 text-sm font-semibold text-white">
        {label.slice(0, 1).toUpperCase()}
      </span>
      <span className="min-w-0">
        <span className="block truncate text-sm font-semibold text-slate-950">{label}</span>
        <span className="block truncate text-xs text-slate-500">
          {conversation.kind === 'group' ? `${conversation.members.length} members` : 'Direct message'}
        </span>
      </span>
      {unread ? <span className="size-2 rounded-full bg-cyan-500" aria-label="Unread conversation" /> : null}
    </Link>
  );
}

function ThreadHeader({ conversation, currentUserId }: { conversation: Conversation; currentUserId: string | null }) {
  return (
    <header className="flex items-center justify-between gap-3 border-b border-slate-200 p-4">
      <div>
        <h2 className="text-lg font-semibold text-slate-950">{conversationLabel(conversation, currentUserId)}</h2>
        <p className="text-sm text-slate-500">{conversation.members.length} members</p>
      </div>
    </header>
  );
}

function MessageBubble({ message, isOwn, isRead }: { message: Message; isOwn: boolean; isRead: boolean }) {
  return (
    <div className={`flex ${isOwn ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[78%] rounded-lg px-3 py-2 text-sm ${
          isOwn ? 'bg-slate-950 text-white' : 'border border-slate-200 bg-white text-slate-950'
        }`}
      >
        {message.body ? <p className="whitespace-pre-wrap break-words">{message.body}</p> : null}
        {message.media_id ? (
          <p className={`mt-2 text-xs ${isOwn ? 'text-slate-300' : 'text-slate-500'}`}>
            Attachment: {message.media_id}
          </p>
        ) : null}
        <p className={`mt-1 text-right text-[11px] ${isOwn ? 'text-slate-300' : 'text-slate-500'}`}>
          {formatTime(message.created_at)}
          {isOwn && isRead ? ' · Read' : ''}
        </p>
      </div>
    </div>
  );
}

function TypingDots({
  userIds,
  conversation,
  currentUserId,
}: {
  userIds: string[];
  conversation: Conversation;
  currentUserId: string | null;
}) {
  if (userIds.length === 0) {
    return <div className="mb-2 h-5" />;
  }

  const names = userIds.map((userId) => memberLabel(userId, conversation, currentUserId)).join(', ');

  return (
    <p className="mb-2 text-xs font-medium text-cyan-700">
      {names} {userIds.length === 1 ? 'is' : 'are'} typing...
    </p>
  );
}

function conversationLabel(conversation: Conversation, currentUserId: string | null) {
  if (conversation.title) {
    return conversation.title;
  }

  const others = conversation.members.filter((member) => member.user_id !== currentUserId);
  if (others.length === 0) {
    return 'Saved messages';
  }

  return others.map((member) => shortId(member.user_id)).join(', ');
}

function memberLabel(userId: string, conversation: Conversation, currentUserId: string | null) {
  if (userId === currentUserId) {
    return 'You';
  }

  const member = conversation.members.find((value) => value.user_id === userId);
  return member ? shortId(member.user_id) : shortId(userId);
}

function isUnread(conversation: Conversation, currentUserId: string | null) {
  const currentMember = conversation.members.find((member) => member.user_id === currentUserId);
  return Boolean(currentMember && currentMember.last_read_message_id === null);
}

function isMessageRead(
  messageId: string,
  conversation: Conversation,
  readReceipts: Array<{ user_id: string; message_id: string }>,
  currentUserId: string | null,
) {
  return (
    readReceipts.some((receipt) => receipt.user_id !== currentUserId && receipt.message_id === messageId) ||
    conversation.members.some(
      (member) => member.user_id !== currentUserId && member.last_read_message_id === messageId,
    )
  );
}

function mergeMessages(restMessages: Message[], realtimeMessages: Message[]) {
  const byId = new Map<string, Message>();
  [...restMessages, ...realtimeMessages].forEach((message) => byId.set(message.id, message));
  return Array.from(byId.values()).sort(
    (left, right) => new Date(left.created_at).getTime() - new Date(right.created_at).getTime(),
  );
}

function shortId(value: string) {
  return value.slice(0, 8);
}

function formatTime(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    hour: 'numeric',
    minute: '2-digit',
  }).format(new Date(value));
}

function statusLabel(status: string) {
  if (status === 'open') {
    return 'Realtime connected';
  }
  if (status === 'connecting') {
    return 'Connecting...';
  }
  return 'Realtime idle';
}
