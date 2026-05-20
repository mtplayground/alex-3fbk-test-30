import { FormEvent, useState } from 'react';
import { Link, useLocation, useNavigate } from 'react-router-dom';

import { useAuth } from '../features/auth/AuthProvider';

type LocationState = {
  from?: {
    pathname?: string;
  };
};

const DEMO_LOGIN = {
  email: 'alice@example.test',
  password: 'password123',
};

export function LoginPage() {
  const auth = useAuth();
  const navigate = useNavigate();
  const location = useLocation();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setSubmitting] = useState(false);

  const redirectTo = ((location.state as LocationState | null)?.from?.pathname ?? '/') || '/';

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await submitLogin({ email, password });
  }

  async function handleDemoLogin() {
    setEmail(DEMO_LOGIN.email);
    setPassword(DEMO_LOGIN.password);
    await submitLogin(DEMO_LOGIN);
  }

  async function submitLogin(credentials: typeof DEMO_LOGIN) {
    setError(null);
    setSubmitting(true);

    try {
      if (isCommonDefaultCredential(credentials.email, credentials.password)) {
        throw new Error('Default credentials are not accepted.');
      }

      await auth.login(credentials);
      navigate(redirectTo, { replace: true });
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="mx-auto max-w-md rounded-lg border border-slate-200 bg-white p-6 shadow-soft">
      <div className="space-y-1">
        <h1 className="text-2xl font-semibold">Sign in</h1>
        <p className="text-sm text-slate-600">Continue to your account.</p>
      </div>

      <form className="mt-6 space-y-4" onSubmit={handleSubmit}>
        <label className="block">
          <span className="text-sm font-medium text-slate-700">Email</span>
          <input
            type="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            className="mt-2 w-full rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
            autoComplete="email"
            required
          />
        </label>
        <label className="block">
          <span className="text-sm font-medium text-slate-700">Password</span>
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            className="mt-2 w-full rounded-md border border-slate-300 px-3 py-2 text-sm outline-none focus:border-emerald-600 focus:ring-2 focus:ring-emerald-100"
            autoComplete="current-password"
            required
          />
        </label>

        {error ? <p className="rounded-md bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

        <button
          type="submit"
          disabled={isSubmitting}
          className="w-full rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:cursor-not-allowed disabled:bg-slate-400"
        >
          {isSubmitting ? 'Signing in' : 'Sign in'}
        </button>

        <button
          type="button"
          onClick={handleDemoLogin}
          disabled={isSubmitting}
          className="w-full rounded-md border border-emerald-200 bg-emerald-50 px-4 py-2 text-sm font-semibold text-emerald-800 hover:bg-emerald-100 disabled:cursor-not-allowed disabled:border-slate-200 disabled:bg-slate-100 disabled:text-slate-400"
        >
          {isSubmitting ? 'Signing in' : 'Guest / Demo Login'}
        </button>
      </form>

      <div className="mt-5 flex items-center justify-between gap-3 text-sm">
        <Link to="/forgot-password" className="font-medium text-emerald-700 hover:text-emerald-900">
          Forgot password
        </Link>
        <Link to="/signup" className="font-medium text-slate-800 hover:text-slate-950">
          Create account
        </Link>
      </div>
    </div>
  );
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Request failed';
}

function isCommonDefaultCredential(email: string, password: string) {
  const normalizedEmail = email.trim().toLowerCase();
  const normalizedPassword = password.trim().toLowerCase();

  return (
    ['admin', 'admin@example.com'].includes(normalizedEmail) &&
    ['change-me', 'password'].includes(normalizedPassword)
  );
}
