import { apiRequest } from '../auth/api';

export type ConversationMember = {
  user_id: string;
  joined_at: string;
  last_read_message_id: string | null;
};

export type Conversation = {
  id: string;
  kind: 'dm' | 'group';
  title: string | null;
  created_at: string;
  updated_at: string;
  members: ConversationMember[];
};

export type Message = {
  id: string;
  conversation_id: string;
  author_id: string;
  body: string;
  media_id: string | null;
  created_at: string;
};

export type ConversationsResponse = {
  conversations: Conversation[];
};

export type MessagesPage = {
  messages: Message[];
  next_cursor: string | null;
};

export type CreateMessagePayload = {
  body?: string;
  media_id?: string;
};

export function listConversations(): Promise<ConversationsResponse> {
  return apiRequest<ConversationsResponse>('/conversations');
}

export function listMessages(conversationId: string, cursor?: string | null, limit = 30): Promise<MessagesPage> {
  const search = new URLSearchParams({ limit: String(limit) });
  if (cursor) {
    search.set('cursor', cursor);
  }

  return apiRequest<MessagesPage>(`/conversations/${encodeURIComponent(conversationId)}/messages?${search}`);
}

export function createMessage(conversationId: string, payload: CreateMessagePayload): Promise<Message> {
  return apiRequest<Message>(`/conversations/${encodeURIComponent(conversationId)}/messages`, {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export function markConversationRead(conversationId: string, messageId: string): Promise<ConversationMember> {
  return apiRequest<ConversationMember>(`/conversations/${encodeURIComponent(conversationId)}/read`, {
    method: 'POST',
    body: JSON.stringify({ message_id: messageId }),
  });
}
