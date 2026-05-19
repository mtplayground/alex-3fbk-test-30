import { useEffect, useRef, useState } from 'react';
import { Link } from 'react-router-dom';

import { NotificationList } from './NotificationList';
import { useNotifications } from './hooks';

export function NotificationsDrawer() {
  const [open, setOpen] = useState(false);
  const panelRef = useRef<HTMLDivElement | null>(null);
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

  useEffect(() => {
    if (!open) {
      return undefined;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setOpen(false);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [open]);

  useEffect(() => {
    if (open) {
      panelRef.current?.focus();
    }
  }, [open]);

  return (
    <>
      <button
        type="button"
        aria-label="Open notifications"
        onClick={() => setOpen(true)}
        className="relative grid size-10 place-items-center rounded-md border border-slate-300 text-slate-700 hover:border-slate-950 hover:text-slate-950"
      >
        <BellIcon />
        {unreadCount > 0 ? (
          <span className="absolute -right-1 -top-1 min-w-5 rounded-full bg-cyan-500 px-1.5 text-center text-xs font-semibold leading-5 text-white">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        ) : null}
      </button>

      {open ? (
        <div className="fixed inset-0 z-40">
          <button
            type="button"
            aria-label="Close notifications"
            onClick={() => setOpen(false)}
            className="absolute inset-0 bg-slate-950/30"
          />
          <aside
            ref={panelRef}
            tabIndex={-1}
            className="absolute right-0 top-0 grid h-full w-full max-w-md grid-rows-[auto_minmax(0,1fr)_auto] bg-stone-50 shadow-2xl outline-none"
          >
            <header className="flex items-center justify-between gap-3 border-b border-slate-200 bg-white px-4 py-4">
              <div>
                <h2 className="text-lg font-semibold text-slate-950">Notifications</h2>
                <p className="text-sm text-slate-500">{unreadCount} unread</p>
              </div>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  disabled={unreadCount === 0 || isMarkingAllRead}
                  onClick={() => {
                    void markAllRead();
                  }}
                  className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:border-slate-950 disabled:cursor-not-allowed disabled:text-slate-400"
                >
                  Mark read
                </button>
                <button
                  type="button"
                  aria-label="Close notifications"
                  onClick={() => setOpen(false)}
                  className="grid size-9 place-items-center rounded-md text-slate-600 hover:bg-slate-100 hover:text-slate-950"
                >
                  <CloseIcon />
                </button>
              </div>
            </header>

            <div className="overflow-y-auto p-4">
              {isLoading ? (
                <p className="text-sm text-slate-500">Loading notifications...</p>
              ) : (
                <NotificationList notifications={notifications} onNavigate={() => setOpen(false)} />
              )}
              {hasNextPage ? (
                <button
                  type="button"
                  onClick={() => {
                    void fetchNextPage();
                  }}
                  disabled={isFetchingNextPage}
                  className="mt-4 w-full rounded-md border border-slate-300 px-3 py-2 text-sm font-semibold text-slate-800 hover:border-slate-950 disabled:cursor-wait disabled:text-slate-400"
                >
                  {isFetchingNextPage ? 'Loading...' : 'Load older'}
                </button>
              ) : null}
            </div>

            <footer className="border-t border-slate-200 bg-white p-4">
              <Link
                to="/notifications"
                onClick={() => setOpen(false)}
                className="block rounded-md bg-slate-950 px-4 py-2 text-center text-sm font-semibold text-white hover:bg-slate-800"
              >
                View all notifications
              </Link>
            </footer>
          </aside>
        </div>
      ) : null}
    </>
  );
}

function BellIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" className="size-5" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M15 17h5l-1.4-1.4A2 2 0 0 1 18 14.2V11a6 6 0 1 0-12 0v3.2c0 .5-.2 1-.6 1.4L4 17h5" />
      <path d="M9 17a3 3 0 0 0 6 0" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" className="size-5" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </svg>
  );
}
