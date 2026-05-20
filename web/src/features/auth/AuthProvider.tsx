import { createContext, ReactNode, useCallback, useContext, useMemo, useState } from 'react';

import {
  forgotPassword as forgotPasswordRequest,
  apiRequest,
  login as loginRequest,
  logout as logoutRequest,
  resetPassword as resetPasswordRequest,
  signup as signupRequest,
  type LoginPayload,
  type SignupPayload,
} from './api';
import { useSessionStore, type SessionUser } from '../../store/sessionStore';

type AuthStatus = 'checking' | 'anonymous' | 'authenticated';

type AuthContextValue = {
  status: AuthStatus;
  user: SessionUser | null;
  accessToken: string | null;
  isAuthenticated: boolean;
  signup: (payload: SignupPayload) => Promise<void>;
  login: (payload: LoginPayload) => Promise<void>;
  logout: () => Promise<void>;
  forgotPassword: (email: string) => Promise<void>;
  resetPassword: (token: string, password: string) => Promise<void>;
  authenticatedFetch: typeof apiRequest;
};

const AuthContext = createContext<AuthContextValue | null>(null);

type AuthProviderProps = {
  children: ReactNode;
};

export function AuthProvider({ children }: AuthProviderProps) {
  const user = useSessionStore((state) => state.user);
  const accessToken = useSessionStore((state) => state.accessToken);
  const setSession = useSessionStore((state) => state.setSession);
  const clearSession = useSessionStore((state) => state.clearSession);
  const [status, setStatus] = useState<AuthStatus>(accessToken ? 'authenticated' : 'anonymous');

  const signup = useCallback(
    async (payload: SignupPayload) => {
      const response = await signupRequest(payload);
      setSession(response.user, response.access_token);
      setStatus('authenticated');
    },
    [setSession],
  );

  const login = useCallback(
    async (payload: LoginPayload) => {
      const response = await loginRequest(payload);
      setSession(response.user, response.access_token);
      setStatus('authenticated');
    },
    [setSession],
  );

  const logout = useCallback(async () => {
    try {
      await logoutRequest();
    } finally {
      clearSession();
      setStatus('anonymous');
    }
  }, [clearSession]);

  const forgotPassword = useCallback((email: string) => forgotPasswordRequest(email), []);
  const resetPassword = useCallback((token: string, password: string) => resetPasswordRequest(token, password), []);

  const value = useMemo<AuthContextValue>(
    () => ({
      status,
      user,
      accessToken,
      isAuthenticated: Boolean(accessToken),
      signup,
      login,
      logout,
      forgotPassword,
      resetPassword,
      authenticatedFetch: apiRequest,
    }),
    [accessToken, forgotPassword, login, logout, resetPassword, signup, status, user],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used inside AuthProvider');
  }

  return context;
}
