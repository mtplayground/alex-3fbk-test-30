import { useSessionStore, type SessionUser } from '../../store/sessionStore';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '';

type ApiErrorBody = {
  error?: {
    code?: string;
    message?: string;
  };
};

export class ApiError extends Error {
  status: number;
  code?: string;

  constructor(status: number, message: string, code?: string) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
  }
}

export type AuthResponse = {
  access_token: string;
  token_type: 'Bearer';
  expires_in: number;
  user: SessionUser;
};

export type AccessTokenResponse = {
  access_token: string;
  token_type: 'Bearer';
  expires_in: number;
};

export type SignupPayload = {
  email: string;
  handle: string;
  password: string;
  display_name: string;
};

export type LoginPayload = {
  email: string;
  password: string;
};

export async function signup(payload: SignupPayload): Promise<AuthResponse> {
  return apiRequest<AuthResponse>('/auth/signup', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function login(payload: LoginPayload): Promise<AuthResponse> {
  return apiRequest<AuthResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

export async function refreshAccessToken(): Promise<AccessTokenResponse> {
  return rawRequest<AccessTokenResponse>('/auth/refresh', {
    method: 'POST',
  });
}

export async function logout(): Promise<void> {
  await apiRequest<void>('/auth/logout', {
    method: 'POST',
  });
}

export async function forgotPassword(email: string): Promise<void> {
  await apiRequest<void>('/auth/forgot-password', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
}

export async function resetPassword(token: string, password: string): Promise<void> {
  await apiRequest<void>('/auth/reset-password', {
    method: 'POST',
    body: JSON.stringify({ token, password }),
  });
}

export async function apiRequest<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await rawFetch(path, init, true);
  return parseResponse<T>(response);
}

async function rawRequest<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await rawFetch(path, init, false);
  return parseResponse<T>(response);
}

async function rawFetch(path: string, init: RequestInit, allowRefresh: boolean): Promise<Response> {
  const accessToken = useSessionStore.getState().accessToken;
  const headers = new Headers(init.headers);

  if (!headers.has('Content-Type') && init.body) {
    headers.set('Content-Type', 'application/json');
  }

  if (accessToken && !headers.has('Authorization')) {
    headers.set('Authorization', `Bearer ${accessToken}`);
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    credentials: 'include',
    headers,
  });

  if (response.status !== 401 || !allowRefresh) {
    return response;
  }

  const refreshed = await tryRefresh();
  if (!refreshed) {
    return response;
  }

  const retryHeaders = new Headers(headers);
  retryHeaders.set('Authorization', `Bearer ${refreshed}`);

  return fetch(`${API_BASE_URL}${path}`, {
    ...init,
    credentials: 'include',
    headers: retryHeaders,
  });
}

async function tryRefresh(): Promise<string | null> {
  try {
    const response = await refreshAccessToken();
    useSessionStore.getState().setAccessToken(response.access_token);
    return response.access_token;
  } catch {
    useSessionStore.getState().clearSession();
    return null;
  }
}

async function parseResponse<T>(response: Response): Promise<T> {
  if (response.ok) {
    if (response.status === 204) {
      return undefined as T;
    }

    const contentType = response.headers.get('Content-Type') ?? '';
    if (!contentType.includes('application/json')) {
      return undefined as T;
    }

    return response.json() as Promise<T>;
  }

  const body = await parseErrorBody(response);
  throw new ApiError(
    response.status,
    body.error?.message ?? `Request failed with status ${response.status}`,
    body.error?.code,
  );
}

async function parseErrorBody(response: Response): Promise<ApiErrorBody> {
  try {
    return (await response.json()) as ApiErrorBody;
  } catch {
    return {};
  }
}
