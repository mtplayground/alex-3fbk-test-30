export type ConnectionStatus = 'idle' | 'connecting' | 'open' | 'closed' | 'error';

export type RealtimeMessage = {
  id: string;
  conversation_id: string;
  author_id: string;
  body: string;
  media_id: string | null;
  created_at: string;
};

export type NotificationEventPayload = {
  id: string;
  user_id: string;
  kind: 'like' | 'comment' | 'follow' | 'mention' | 'dm';
  actor_id: string;
  target_kind: 'post' | 'comment' | 'user' | 'message' | 'conversation';
  target_id: string;
  read_at: string | null;
  created_at: string;
};

export type PresencePayload = {
  user_id: string;
  status: 'online' | 'offline';
  seen_at: string;
};

export type ReadyEvent = {
  type: 'ready';
  user_id: string;
  channels: string[];
  heartbeat_ms: number;
};

export type HeartbeatEvent = {
  type: 'heartbeat';
  interval_ms: number;
};

export type PongEvent = {
  type: 'pong';
};

export type ErrorEventPayload = {
  type: 'error';
  code: string;
};

export type MessageEventPayload = {
  type: 'message';
  conversation_id: string;
  message: RealtimeMessage;
};

export type TypingEventPayload = {
  type: 'typing';
  conversation_id: string;
  user_id: string;
  is_typing: boolean;
};

export type ReadEventPayload = {
  type: 'read';
  conversation_id: string;
  user_id: string;
  message_id: string;
};

export type NotificationRealtimeEvent = {
  type: 'notification';
  notification: NotificationEventPayload;
};

export type PresenceEventPayload = {
  type: 'presence';
  presence: PresencePayload;
};

export type RealtimeEvent =
  | ReadyEvent
  | HeartbeatEvent
  | PongEvent
  | ErrorEventPayload
  | MessageEventPayload
  | TypingEventPayload
  | ReadEventPayload
  | NotificationRealtimeEvent
  | PresenceEventPayload;

export type ClientEvent =
  | { type: 'ping' }
  | { type: 'typing'; conversation_id: string; is_typing?: boolean }
  | { type: 'read'; conversation_id: string; message_id: string };
