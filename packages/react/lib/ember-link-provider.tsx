import {
  createClient,
  type CreateClientOptions,
  type DefaultCustomMessageData,
  type DefaultPresence,
  type EmberClient
} from '@ember-link/core';
import { createContext, useContext, useMemo, type PropsWithChildren } from 'react';

const ClientContext = createContext<EmberClient | null>(null);

export const useClientOrNull = <
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>(): EmberClient<P, C> | null => {
  return useContext(ClientContext);
};

export const useClient = <
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
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

  // eslint-disable-next-line react-hooks/exhaustive-deps
  const client = useMemo(() => createClient<P, C>(options), []);

  return <ClientContext.Provider value={client}>{children}</ClientContext.Provider>;
};
