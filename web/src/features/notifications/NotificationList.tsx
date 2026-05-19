import { Link } from 'react-router-dom';

import type { NotificationEventPayload } from '../realtime/types';

type NotificationListProps = {
  notifications: NotificationEventPayload[];
  onNavigate?: () => void;
};

export function NotificationList({ notifications, onNavigate }: NotificationListProps) {
  if (notifications.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-slate-300 bg-white p-6 text-center text-sm text-slate-500">
        No notifications yet.
      </div>
    );
  }

  return (
    <div className="space-y-5">
      {groupNotifications(notifications).map((group) => (
        <section key={group.label}>
          <h2 className="mb-2 text-xs font-semibold uppercase tracking-normal text-slate-500">{group.label}</h2>
          <div className="overflow-hidden rounded-lg border border-slate-200 bg-white">
            {group.items.map((notification) => (
              <NotificationRow key={notification.id} notification={notification} onNavigate={onNavigate} />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function NotificationRow({
  notification,
  onNavigate,
}: {
  notification: NotificationEventPayload;
  onNavigate?: () => void;
}) {
  const content = (
    <>
      <span className="grid size-9 shrink-0 place-items-center rounded-full bg-cyan-100 text-sm font-semibold text-cyan-950">
        {notification.kind.slice(0, 1).toUpperCase()}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-sm font-medium text-slate-950">{notificationText(notification)}</span>
        <span className="mt-1 block text-xs text-slate-500">
          {shortId(notification.actor_id)} · {relativeTime(notification.created_at)}
        </span>
      </span>
      {!notification.read_at ? <span className="size-2 shrink-0 rounded-full bg-cyan-500" /> : null}
    </>
  );
  const href = notificationLink(notification);

  if (!href) {
    return <div className="flex items-center gap-3 border-b border-slate-100 px-3 py-3 last:border-b-0">{content}</div>;
  }

  return (
    <Link
      to={href}
      onClick={onNavigate}
      className="flex items-center gap-3 border-b border-slate-100 px-3 py-3 last:border-b-0 hover:bg-slate-50"
    >
      {content}
    </Link>
  );
}

function groupNotifications(notifications: NotificationEventPayload[]) {
  const groups = new Map<string, NotificationEventPayload[]>();

  notifications.forEach((notification) => {
    const label = groupLabel(notification.created_at);
    groups.set(label, [...(groups.get(label) ?? []), notification]);
  });

  return Array.from(groups.entries()).map(([label, items]) => ({ label, items }));
}

function groupLabel(value: string) {
  const date = new Date(value);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);

  if (date.toDateString() === today.toDateString()) {
    return 'Today';
  }
  if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  }

  return new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    year: date.getFullYear() === today.getFullYear() ? undefined : 'numeric',
  }).format(date);
}

function notificationText(notification: NotificationEventPayload) {
  switch (notification.kind) {
    case 'like':
      return `Liked your ${targetLabel(notification.target_kind)}.`;
    case 'comment':
      return `Commented on your ${targetLabel(notification.target_kind)}.`;
    case 'follow':
      return 'Started following you.';
    case 'mention':
      return `Mentioned you in a ${targetLabel(notification.target_kind)}.`;
    case 'dm':
      return 'Sent you a message.';
    default:
      return 'New notification.';
  }
}

function notificationLink(notification: NotificationEventPayload) {
  if (notification.target_kind === 'post') {
    return `/p/${notification.target_id}`;
  }
  if (notification.target_kind === 'conversation') {
    return `/dm/${notification.target_id}`;
  }
  if (notification.kind === 'dm' || notification.target_kind === 'message') {
    return '/dm';
  }

  return null;
}

function targetLabel(kind: NotificationEventPayload['target_kind']) {
  return kind;
}

function shortId(value: string) {
  return value.slice(0, 8);
}

function relativeTime(value: string) {
  const elapsedSeconds = Math.max(1, Math.round((Date.now() - new Date(value).getTime()) / 1000));
  if (elapsedSeconds < 60) {
    return 'just now';
  }
  const elapsedMinutes = Math.round(elapsedSeconds / 60);
  if (elapsedMinutes < 60) {
    return `${elapsedMinutes}m ago`;
  }
  const elapsedHours = Math.round(elapsedMinutes / 60);
  if (elapsedHours < 24) {
    return `${elapsedHours}h ago`;
  }
  return `${Math.round(elapsedHours / 24)}d ago`;
}
