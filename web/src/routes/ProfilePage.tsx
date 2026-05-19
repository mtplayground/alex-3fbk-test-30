import { useParams } from 'react-router-dom';

export function ProfilePage() {
  const { handle } = useParams();

  return (
    <div className="space-y-6">
      <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-soft">
        <div className="flex items-center gap-4">
          <div className="grid size-16 place-items-center rounded-full bg-emerald-200 text-xl font-semibold text-emerald-950">
            {handle?.slice(0, 1).toUpperCase() ?? 'Z'}
          </div>
          <div>
            <h1 className="text-2xl font-semibold">@{handle}</h1>
            <p className="mt-1 text-sm text-slate-600">Profile grid and social stats route.</p>
          </div>
        </div>
      </section>
      <div className="grid grid-cols-3 gap-3">
        {Array.from({ length: 6 }, (_, index) => (
          <a
            key={index}
            href={`/p/${handle}-post-${index + 1}`}
            className="aspect-square rounded-lg bg-gradient-to-br from-slate-200 to-rose-200"
            aria-label={`Profile post ${index + 1}`}
          />
        ))}
      </div>
    </div>
  );
}
