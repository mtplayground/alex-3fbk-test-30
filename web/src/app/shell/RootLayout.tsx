import { NavLink, Outlet } from 'react-router-dom';

import { useAuth } from '../../features/auth/AuthProvider';
import { NotificationsDrawer } from '../../features/notifications/NotificationsDrawer';
import { useTheme, type ThemeMode } from '../../features/theme/ThemeProvider';

const themeOptions: Array<{ label: string; shortLabel: string; mode: ThemeMode }> = [
  { label: 'Light', shortLabel: 'L', mode: 'light' },
  { label: 'Dark', shortLabel: 'D', mode: 'dark' },
  { label: 'System', shortLabel: 'Sys', mode: 'system' },
];

export function RootLayout() {
  const auth = useAuth();
  const navItems = [
    { label: 'Home', to: '/' },
    { label: 'Explore', to: '/explore' },
    { label: 'Reels', to: '/reels' },
    { label: 'Messages', to: '/dm' },
    { label: 'Profile', to: auth.user ? `/u/${auth.user.handle}` : '/login' },
    { label: 'Settings', to: '/settings/profile' },
  ];

  return (
    <div className="min-h-screen bg-stone-50 text-slate-950 transition-colors dark:bg-slate-950 dark:text-slate-100">
      <header className="sticky top-0 z-20 border-b border-slate-200 bg-white/95 backdrop-blur transition-colors dark:border-slate-800 dark:bg-slate-950/95">
        <div className="mx-auto flex h-16 max-w-6xl items-center justify-between gap-3 px-4">
          <NavLink
            to="/"
            className="min-w-0 truncate text-base font-semibold tracking-normal text-slate-950 dark:text-white sm:text-xl"
          >
            Yet another Instagram
          </NavLink>
          <nav className="hidden items-center gap-1 md:flex">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  [
                    'rounded-md px-3 py-2 text-sm font-medium transition',
                    isActive
                      ? 'bg-slate-950 text-white dark:bg-slate-100 dark:text-slate-950'
                      : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950 dark:text-slate-300 dark:hover:bg-slate-800 dark:hover:text-white',
                  ].join(' ')
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
          <div className="flex shrink-0 items-center gap-2">
            <ThemeSwitcher />
            {auth.isAuthenticated ? (
              <>
                <NotificationsDrawer />
                <button
                  type="button"
                  onClick={() => {
                    void auth.logout();
                  }}
                  className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-800 transition hover:border-slate-950 hover:text-slate-950 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-300 dark:hover:text-white"
                >
                  Sign out
                </button>
              </>
            ) : (
              <NavLink
                to="/login"
                className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-800 transition hover:border-slate-950 hover:text-slate-950 dark:border-slate-700 dark:text-slate-200 dark:hover:border-slate-300 dark:hover:text-white"
              >
                Sign in
              </NavLink>
            )}
          </div>
        </div>
      </header>

      <main className="mx-auto grid max-w-6xl gap-6 px-4 py-6 lg:grid-cols-[180px_minmax(0,1fr)]">
        <aside className="hidden lg:block">
          <nav className="sticky top-24 space-y-1">
            {navItems.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  [
                    'block rounded-md px-3 py-2 text-sm font-medium',
                    isActive
                      ? 'bg-emerald-100 text-emerald-950 dark:bg-emerald-400/15 dark:text-emerald-100'
                      : 'text-slate-600 hover:bg-white hover:text-slate-950 dark:text-slate-300 dark:hover:bg-slate-900 dark:hover:text-white',
                  ].join(' ')
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </aside>
        <section className="min-w-0">
          <Outlet />
        </section>
      </main>
    </div>
  );
}

function ThemeSwitcher() {
  const { mode, setMode } = useTheme();

  return (
    <div
      className="grid grid-cols-3 rounded-md border border-slate-300 bg-slate-100 p-0.5 dark:border-slate-700 dark:bg-slate-900"
      aria-label="Theme mode"
    >
      {themeOptions.map((option) => {
        const active = option.mode === mode;
        return (
          <button
            key={option.mode}
            type="button"
            onClick={() => setMode(option.mode)}
            aria-pressed={active}
            className={[
              'min-w-8 rounded px-1.5 py-1 text-xs font-medium transition sm:min-w-14 sm:px-2',
              active
                ? 'bg-white text-slate-950 shadow-sm dark:bg-slate-700 dark:text-white'
                : 'text-slate-600 hover:text-slate-950 dark:text-slate-300 dark:hover:text-white',
            ].join(' ')}
          >
            <span className="sm:hidden">{option.shortLabel}</span>
            <span className="hidden sm:inline">{option.label}</span>
          </button>
        );
      })}
    </div>
  );
}
