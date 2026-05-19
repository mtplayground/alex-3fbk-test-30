import { Link } from 'react-router-dom';

export function NotFoundPage() {
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-6 shadow-soft">
      <h1 className="text-2xl font-semibold">Not found</h1>
      <Link to="/" className="mt-4 inline-block text-sm font-semibold text-emerald-700">
        Home
      </Link>
    </div>
  );
}
