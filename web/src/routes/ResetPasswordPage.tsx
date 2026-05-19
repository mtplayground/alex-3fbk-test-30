import { FormEvent, useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';

export function ResetPasswordPage() {
  const auth = useAuth();
  const [searchParams] = useSearchParams();
  const token = useMemo(() => searchParams.get('token') ?? '', [searchParams]);
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [complete, setComplete] = useState(false);
  const [isSubmitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setSubmitting(true);

    try {
      await auth.resetPassword(token, password);
      setComplete(true);
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="mx-auto max-w-md rounded-lg border border-slate-200 bg-white p-6 shadow-soft">
      <div className="space-y-1">
        <h1 className="text-2xl font-semibold">New password</h1>
        <p className="text-sm text-slate-600">Choose a replacement password.</p>
      </div>

      {complete ? (
        <div className="mt-6 space-y-4">
          <p className="rounded-md bg-emerald-50 px-3 py-2 text-sm text-emerald-800">Password updated.</p>
          <Link to="/login" className="inline-flex rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white">
            Sign in
          </Link>
        </div>
      ) : (
        <form className="mt-6 space-y-4" onSubmit={handleSubmit}>
          <label className="block">
            <span className="text-sm font-medium text-slate-700">Password</span>
            <input
              type="password"
              value={password}
              minLength={8}
              onChange={(event) => setPassword(event.target.value)}
              className="mt-2 w-full rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
              autoComplete="new-password"
              required
              disabled={!token}
            />
          </label>

          {!token ? <p className="rounded-md bg-rose-50 px-3 py-2 text-sm text-rose-700">Reset token missing.</p> : null}
          {error ? <p className="rounded-md bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

          <button
            type="submit"
            disabled={isSubmitting || !token}
            className="w-full rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
          >
            {isSubmitting ? 'Updating password' : 'Update password'}
          </button>
        </form>
      )}
    </div>
  );
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Request failed';
}
