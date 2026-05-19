import { NotificationList } from '../features/notifications/NotificationList';
import { useNotifications } from '../features/notifications/hooks';

export function NotificationsPage() {
  const {
    notifications,
    unreadCount,
    isLoading,
    hasNextPage,
    isFetchingNextPage,
    fetchNextPage,
    markAllRead,
    isMarkingAllRead,
  } = useNotifications();

  return (
    <div className="space-y-4">
      <header className="flex flex-col gap-3 rounded-lg border border-slate-200 bg-white p-5 shadow-soft sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-semibold text-slate-950">Notifications</h1>
          <p className="mt-1 text-sm text-slate-500">{unreadCount} unread</p>
        </div>
        <button
          type="button"
          disabled={unreadCount === 0 || isMarkingAllRead}
          onClick={() => {
            void markAllRead();
          }}
          className="rounded-md border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-800 hover:border-slate-950 disabled:cursor-not-allowed disabled:text-slate-400"
        >
          Mark all read
        </button>
      </header>

      {isLoading ? (
        <div className="rounded-lg border border-slate-200 bg-white p-6 text-sm text-slate-500 shadow-soft">
          Loading notifications...
        </div>
      ) : (
        <NotificationList notifications={notifications} />
      )}

      {hasNextPage ? (
        <button
          type="button"
          disabled={isFetchingNextPage}
          onClick={() => {
            void fetchNextPage();
          }}
          className="w-full rounded-md border border-slate-300 bg-white px-4 py-2 text-sm font-semibold text-slate-800 hover:border-slate-950 disabled:cursor-wait disabled:text-slate-400"
        >
          {isFetchingNextPage ? 'Loading...' : 'Load older'}
        </button>
      ) : null}
    </div>
  );
}
