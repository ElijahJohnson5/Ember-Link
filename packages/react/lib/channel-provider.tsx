import {
  Channel,
  type ChannelConfig,
  type DefaultCustomMessageData,
  type DefaultPresence,
  type IStorageProvider
} from '@ember-link/core';
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  useSyncExternalStore,
  type PropsWithChildren
} from 'react';
import { useClient } from './ember-link-provider';
import { useShallowMemo } from './utils';

const ChannelContext = createContext<Channel | null>(null);

export const useChannelOrNull = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>(): Channel<P, C> | null => {
  return useContext(ChannelContext) as Channel<P, C> | null;
};

export const useChannel = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>(): Channel<P, C> => {
  const channel = useChannelOrNull<P, C>();

  if (!channel) {
    throw new Error('You must call useChannel from inside of a ChannelProvider');
  }

  return channel;
};

interface ChannelProviderProps<
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence
> {
  channelName: string;
  options?: ChannelConfig<S, P>['options'];
}

export const ChannelProvider = <
  S extends IStorageProvider,
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>({
  channelName,
  options,
  children
}: PropsWithChildren<ChannelProviderProps<S, P>>) => {
  const client = useClient<P, C>();
  const stableOptions = useShallowMemo(options);

  // Lazy-initialize a borrow of the channel with autoConnect:false so
  // the first render has a Channel to put on context (no null window).
  // The underlying channel is ref-counted inside the client — multiple
  // borrows of the same name share one connection.
  const [pair, setPair] = useState(() =>
    client.joinChannel<S>(channelName, { ...(stableOptions ?? {}), autoConnect: false })
  );

  // Re-borrow when deps change. Each useEffect run adds a borrow with
  // the latest options and releases it on cleanup, so it's freed when
  // deps next change or on unmount.
  useEffect(() => {
    const newPair = client.joinChannel<S>(channelName, stableOptions);
    setPair(newPair);
    return () => {
      newPair.leave();
    };
  }, [client, channelName, stableOptions]);

  // Release the initial useState-held borrow on unmount. The captured
  // closure here is the FIRST-render pair (useEffect with empty deps
  // runs once on mount), so we always free the original even if `pair`
  // has been swapped via setPair in the effect above.
  useEffect(() => {
    const initial = pair;
    return () => {
      initial.leave();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <ChannelContext.Provider value={pair.channel as Channel}>{children}</ChannelContext.Provider>
  );
};

export const useMyPresence = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>() => {
  const channel = useChannel<P, C>();

  const subscribeFunction = useCallback(
    (callback: (presence: P) => void) => {
      return channel.events.subscribe('presence', callback);
    },
    [channel]
  );

  const myPresence = useSyncExternalStore(subscribeFunction, channel.getPresence, () => null);

  const setMyPresence = useCallback(
    (newPresence: P) => {
      channel.updatePresence(newPresence);
    },
    [channel]
  );

  // Stable tuple so consumers `const [p, setP] = useMyPresence()` don't
  // see fresh array identity on every render of the parent.
  return useMemo(() => [myPresence, setMyPresence] as const, [myPresence, setMyPresence]);
};

export const useCustomMessage = <
  P extends Record<string, unknown> = DefaultPresence,
  C extends Record<string, unknown> = DefaultCustomMessageData
>(
  callback: (message: C) => void
) => {
  const channel = useChannel<P, C>();

  const sendCustomMessage = useCallback(
    (data: C) => {
      channel.sendCustomMessage(data);
    },
    [channel]
  );

  useEffect(() => {
    const unsub = channel.events.subscribe('customMessage', callback);

    return () => {
      unsub();
    };
  }, [callback, channel.events]);

  return sendCustomMessage;
};
