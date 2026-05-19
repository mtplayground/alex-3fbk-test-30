import { useParams } from 'react-router-dom';

export function PostPage() {
  const { id } = useParams();

  return (
    <article className="overflow-hidden rounded-lg border border-slate-200 bg-white shadow-soft">
      <div className="aspect-[4/3] bg-gradient-to-br from-slate-200 via-cyan-200 to-emerald-300" />
      <div className="space-y-3 p-5">
        <p className="text-sm font-semibold text-slate-500">Post</p>
        <h1 className="text-2xl font-semibold">{id}</h1>
        <p className="text-sm leading-6 text-slate-700">
          Comments, likes, saves, and media metadata will attach to this route.
        </p>
      </div>
    </article>
  );
}
