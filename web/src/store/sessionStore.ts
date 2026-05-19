import { create } from 'zustand';

type SessionUser = {
  id: string;
  email?: string;
  handle: string;
  display_name: string;
  avatar_key?: string | null;
};

type SessionState = {
  user: SessionUser | null;
  accessToken: string | null;
  setSession: (user: SessionUser, accessToken: string) => void;
  setSessionUser: (user: SessionUser) => void;
  setAccessToken: (accessToken: string | null) => void;
  clearSession: () => void;
};

export const useSessionStore = create<SessionState>((set) => ({
  user: null,
  accessToken: null,
  setSession: (user, accessToken) => set({ user, accessToken }),
  setSessionUser: (user) => set({ user }),
  setAccessToken: (accessToken) => set({ accessToken }),
  clearSession: () => set({ user: null, accessToken: null }),
}));

export type { SessionUser };
