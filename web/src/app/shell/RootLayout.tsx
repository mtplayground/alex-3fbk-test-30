import { NavLink, Outlet } from 'react-router-dom';

import { useAuth } from '../../features/auth/AuthProvider';

export function RootLayout() {
  const auth = useAuth();
  const navItems = [
    { label: 'Home', to: '/' },
    { label: 'Explore', to: '/explore' },
    { label: 'Messages', to: '/dm' },
    { label: 'Profile', to: auth.user ? `/u/${auth.user.handle}` : '/login' },
  ];

  return (
    <div className="min-h-screen bg-stone-50 text-slate-950">
      <header className="sticky top-0 z-20 border-b border-slate-200 bg-white/95 backdrop-blur">
        <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4">
          <NavLink to="/" className="text-xl font-semibold tracking-normal">
            ZeroClaw
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
                      ? 'bg-slate-950 text-white'
                      : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950',
                  ].join(' ')
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
          {auth.isAuthenticated ? (
            <button
              type="button"
              onClick={() => {
                void auth.logout();
              }}
              className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-800 hover:border-slate-950 hover:text-slate-950"
            >
              Sign out
            </button>
          ) : (
            <NavLink
              to="/login"
              className="rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-800 hover:border-slate-950 hover:text-slate-950"
            >
              Sign in
            </NavLink>
          )}
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
                      ? 'bg-emerald-100 text-emerald-950'
                      : 'text-slate-600 hover:bg-white hover:text-slate-950',
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
