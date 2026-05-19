import { createContext, ReactNode, useContext, useEffect, useMemo, useState } from 'react';

import { useAuth } from '../auth/AuthProvider';
import { RealtimeClient } from './client';
import type { ConnectionStatus } from './types';

type RealtimeContextValue = {
  client: RealtimeClient;
  status: ConnectionStatus;
};

const RealtimeContext = createContext<RealtimeContextValue | null>(null);

type RealtimeProviderProps = {
  children: ReactNode;
};

export function RealtimeProvider({ children }: RealtimeProviderProps) {
  const { accessToken } = useAuth();
  const client = useMemo(() => new RealtimeClient(), []);
  const [status, setStatus] = useState<ConnectionStatus>(client.getStatus());

  useEffect(() => client.subscribeStatus(setStatus), [client]);

  useEffect(() => {
    client.setAccessToken(accessToken);
  }, [accessToken, client]);

  useEffect(
    () => () => {
      client.disconnect();
    },
    [client],
  );

  const value = useMemo(() => ({ client, status }), [client, status]);

  return <RealtimeContext.Provider value={value}>{children}</RealtimeContext.Provider>;
}

export function useRealtime() {
  const context = useContext(RealtimeContext);
  if (!context) {
    throw new Error('useRealtime must be used inside RealtimeProvider');
  }

  return context;
}
