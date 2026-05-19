import { useQuery } from '@tanstack/react-query';
import { Link, useParams } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';
import { getProfile } from '../features/profile/api';

const placeholders = Array.from({ length: 9 }, (_, index) => index + 1);

export function ProfilePage() {
  const { handle = '' } = useParams();
  const auth = useAuth();
  const profileQuery = useQuery({
    queryKey: ['profile', handle],
    queryFn: () => getProfile(handle),
    enabled: Boolean(handle),
  });

  if (profileQuery.isLoading) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <p className="text-sm font-medium text-slate-600">Loading profile</p>
      </div>
    );
  }

  if (profileQuery.isError || !profileQuery.data) {
    return (
      <div className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <h1 className="text-2xl font-semibold">Profile unavailable</h1>
        <Link to="/explore" className="mt-4 inline-block text-sm font-semibold text-emerald-700">
          Explore
        </Link>
      </div>
    );
  }

  const profile = profileQuery.data;

  return (
    <div className="space-y-6">
      <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-4">
            <div className="grid size-20 shrink-0 place-items-center rounded-full bg-cyan-100 text-2xl font-semibold text-cyan-950">
              {profile.display_name.slice(0, 1).toUpperCase()}
            </div>
            <div className="min-w-0">
              <h1 className="break-words text-2xl font-semibold">{profile.display_name}</h1>
              <p className="mt-1 text-sm font-medium text-slate-600">@{profile.handle}</p>
              {profile.bio ? <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-700">{profile.bio}</p> : null}
              {profile.link ? (
                <a href={profile.link} className="mt-2 inline-block text-sm font-semibold text-emerald-700">
                  {profile.link}
                </a>
              ) : null}
            </div>
          </div>
          {auth.user?.handle === profile.handle ? (
            <Link
              to="/settings/profile"
              className="rounded-md border border-slate-300 px-3 py-2 text-center text-sm font-semibold text-slate-800 hover:border-slate-950"
            >
              Edit profile
            </Link>
          ) : null}
        </div>

        <div className="mt-6 grid grid-cols-3 gap-3 border-t border-slate-200 pt-5 text-center">
          <Stat label="Posts" value="0" />
          <Stat label="Followers" value="0" />
          <Stat label="Following" value="0" />
        </div>
      </section>

      <div className="grid grid-cols-3 gap-3">
        {placeholders.map((item) => (
          <a
            key={item}
            href={`/p/${profile.handle}-post-${item}`}
            className="aspect-square rounded-lg bg-gradient-to-br from-slate-100 via-cyan-100 to-rose-100 transition hover:scale-[1.01]"
            aria-label={`Profile post ${item}`}
          />
        ))}
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-lg font-semibold">{value}</p>
      <p className="text-xs font-medium uppercase tracking-normal text-slate-500">{label}</p>
    </div>
  );
}
