import type { ClientEvent, ConnectionStatus, RealtimeEvent } from './types';

const DEFAULT_RECONNECT_MS = 1_000;
const MAX_RECONNECT_MS = 15_000;
const CLIENT_PING_MS = 25_000;

type Listener = (event: RealtimeEvent) => void;
type StatusListener = (status: ConnectionStatus) => void;

export type RealtimeClientOptions = {
  url?: string;
};

export class RealtimeClient {
  private socket: WebSocket | null = null;
  private token: string | null = null;
  private conversations = new Set<string>();
  private listeners = new Set<Listener>();
  private statusListeners = new Set<StatusListener>();
  private reconnectTimer: number | null = null;
  private pingTimer: number | null = null;
  private reconnectAttempt = 0;
  private shouldReconnect = false;
  private status: ConnectionStatus = 'idle';
  private readonly baseUrl: string;

  constructor(options: RealtimeClientOptions = {}) {
    this.baseUrl = options.url ?? defaultWebSocketUrl();
  }

  getStatus(): ConnectionStatus {
    return this.status;
  }

  setAccessToken(token: string | null) {
    if (this.token === token) {
      return;
    }

    this.token = token;
    this.reconnectAttempt = 0;

    if (!token) {
      this.disconnect();
      return;
    }

    this.connect();
  }

  setConversations(conversationIds: string[]) {
    const next = new Set(conversationIds.filter(Boolean));
    if (setsEqual(this.conversations, next)) {
      return;
    }

    this.conversations = next;

    if (this.token && this.shouldReconnect) {
      this.connect();
    }
  }

  addConversation(conversationId: string): () => void {
    if (!conversationId) {
      return () => undefined;
    }

    const next = new Set(this.conversations);
    next.add(conversationId);
    this.setConversations(Array.from(next));

    return () => {
      const remaining = new Set(this.conversations);
      remaining.delete(conversationId);
      this.setConversations(Array.from(remaining));
    };
  }

  connect() {
    this.shouldReconnect = Boolean(this.token);
    this.clearReconnectTimer();

    if (!this.token) {
      this.closeSocket();
      this.setStatus('idle');
      return;
    }

    this.closeSocket();
    this.setStatus('connecting');

    const socket = new WebSocket(this.connectionUrl());
    this.socket = socket;

    socket.addEventListener('open', () => {
      this.reconnectAttempt = 0;
      this.setStatus('open');
      this.startClientPing();
    });

    socket.addEventListener('message', (event) => {
      if (typeof event.data !== 'string') {
        return;
      }

      const parsed = parseRealtimeEvent(event.data);
      if (!parsed) {
        return;
      }

      this.listeners.forEach((listener) => listener(parsed));
    });

    socket.addEventListener('close', () => {
      if (this.socket !== socket) {
        return;
      }

      this.socket = null;
      this.stopClientPing();
      this.setStatus(this.shouldReconnect ? 'closed' : 'idle');
      this.scheduleReconnect();
    });

    socket.addEventListener('error', () => {
      this.setStatus('error');
      socket.close();
    });
  }

  disconnect() {
    this.shouldReconnect = false;
    this.clearReconnectTimer();
    this.closeSocket();
    this.setStatus('idle');
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  subscribeStatus(listener: StatusListener): () => void {
    this.statusListeners.add(listener);
    listener(this.status);
    return () => {
      this.statusListeners.delete(listener);
    };
  }

  send(event: ClientEvent): boolean {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) {
      return false;
    }

    this.socket.send(JSON.stringify(event));
    return true;
  }

  sendTyping(conversationId: string, isTyping = true): boolean {
    return this.send({ type: 'typing', conversation_id: conversationId, is_typing: isTyping });
  }

  sendRead(conversationId: string, messageId: string): boolean {
    return this.send({ type: 'read', conversation_id: conversationId, message_id: messageId });
  }

  private connectionUrl(): string {
    const url = new URL(this.baseUrl);
    if (this.token) {
      url.searchParams.set('token', this.token);
    }
    if (this.conversations.size > 0) {
      url.searchParams.set('conversations', Array.from(this.conversations).join(','));
    }
    return url.toString();
  }

  private scheduleReconnect() {
    if (!this.shouldReconnect || !this.token) {
      return;
    }

    const delay = Math.min(
      MAX_RECONNECT_MS,
      DEFAULT_RECONNECT_MS * 2 ** Math.min(this.reconnectAttempt, 4),
    );
    this.reconnectAttempt += 1;
    this.reconnectTimer = window.setTimeout(() => this.connect(), delay);
  }

  private startClientPing() {
    this.stopClientPing();
    this.pingTimer = window.setInterval(() => {
      this.send({ type: 'ping' });
    }, CLIENT_PING_MS);
  }

  private stopClientPing() {
    if (this.pingTimer !== null) {
      window.clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  private clearReconnectTimer() {
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private closeSocket() {
    this.stopClientPing();
    if (this.socket) {
      const socket = this.socket;
      this.socket = null;
      socket.close();
    }
  }

  private setStatus(status: ConnectionStatus) {
    if (this.status === status) {
      return;
    }

    this.status = status;
    this.statusListeners.forEach((listener) => listener(status));
  }
}

function parseRealtimeEvent(raw: string): RealtimeEvent | null {
  try {
    const event = JSON.parse(raw) as Partial<RealtimeEvent>;
    if (!event || typeof event.type !== 'string') {
      return null;
    }

    return event as RealtimeEvent;
  } catch {
    return null;
  }
}

function defaultWebSocketUrl(): string {
  const configured = import.meta.env.VITE_WS_BASE_URL as string | undefined;
  if (configured) {
    return withWsPath(configured);
  }

  const apiBase = import.meta.env.VITE_API_BASE_URL as string | undefined;
  const origin = apiBase || window.location.origin;
  return withWsPath(origin);
}

function withWsPath(value: string): string {
  const url = new URL(value, window.location.origin);
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  url.pathname = '/ws';
  url.search = '';
  return url.toString();
}

function setsEqual(left: Set<string>, right: Set<string>): boolean {
  if (left.size !== right.size) {
    return false;
  }

  for (const value of left) {
    if (!right.has(value)) {
      return false;
    }
  }

  return true;
}
