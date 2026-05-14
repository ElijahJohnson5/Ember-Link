import {
  createClient,
  type CreateClientOptions,
  type DefaultCustomMessageData,
  type DefaultPresence,
  type EmberClient
} from '@ember-link/core';
import { createContext, useContext, useEffect, useMemo, type PropsWithChildren } from 'react';
import { useShallowMemo } from './utils';

const ClientContext = createContext<EmberClient | null>(null);

export const useClientOrNull = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>(): EmberClient<P, C> | null => {
  return useContext(ClientContext) as EmberClient<P, C> | null;
};

export const useClient = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>(): EmberClient<P, C> => {
  const client = useClientOrNull<P, C>();

  if (!client) {
    throw new Error('You must call useClient from inside of a EmberLinkProvider');
  }

  return client;
};

const useEnsureNoEmberLinkProvider = () => {
  const client = useClientOrNull();

  if (client) {
    throw new Error('You cannot nest multiple EmberLinkProviders in the same tree');
  }
};

export const EmberLinkProvider = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>({
  children,
  ...options
}: PropsWithChildren<CreateClientOptions>) => {
  useEnsureNoEmberLinkProvider();

  // Stabilize the options object across renders so a parent passing a
  // shallow-equal literal each render doesn't re-create the client.
  const stableOptions = useShallowMemo(options);

  const client = useMemo(() => createClient<P, C>(stableOptions), [stableOptions]);

  // Destroy the client on unmount or when options change and the
  // memoized client is replaced. Cleans up channels + WebSockets.
  useEffect(() => {
    return () => {
      client.destroy();
    };
  }, [client]);

  return (
    <ClientContext.Provider value={client as unknown as EmberClient}>
      {children}
    </ClientContext.Provider>
  );
};
