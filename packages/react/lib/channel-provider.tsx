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
  useState,
  type PropsWithChildren
} from 'react';
import { useClient } from './ember-link-provider';

const ChannelContext = createContext<Channel | null>(null);

export const useChannelOrNull = <
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>(): Channel<P, C> | null => {
  return useContext(ChannelContext);
};

export const useChannel = <
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>(): Channel<P, C> => {
  const channel = useChannelOrNull();

  if (!channel) {
    throw new Error('You must call useChannel from inside of a ChannelProvider');
  }

  return channel;
};

interface ChannelProviderProps<S extends IStorageProvider, P extends DefaultPresence> {
  channelName: string;
  options: ChannelConfig<S, P>['options'];
}

interface ChannelAndLeave<P extends DefaultPresence, C extends DefaultCustomMessageData> {
  channel: Channel<P, C>;
  leave: () => void;
}

export const ChannelProvider = <
  S extends IStorageProvider,
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>(
  props: PropsWithChildren<ChannelProviderProps<S, P>>
) => {
  const client = useClient<P, C>();

  const [cache] = useState(() => new Map<string, ChannelAndLeave<P, C>>());

  const joinChannel = useCallback(
    (channelName: string, options: ChannelConfig<S, P>['options']) => {
      const cached = cache.get(channelName);

      if (cached) {
        return cached;
      }

      const channelAndLeave = client.joinChannel<S>(channelName, options);

      const oldLeave = channelAndLeave.leave;

      channelAndLeave.leave = () => {
        oldLeave();
        cache.delete(channelName);
      };

      cache.set(channelName, channelAndLeave);

      return channelAndLeave;
    },
    [client, cache]
  );

  return <ChannelProviderInner<S, P, C> {...props} joinChannel={joinChannel} />;
};

const ChannelProviderInner = <
  S extends IStorageProvider,
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>({
  channelName,
  options,
  joinChannel,
  children
}: PropsWithChildren<
  ChannelProviderProps<S, P> & {
    joinChannel: (
      channelName: string,
      options: ChannelConfig<S, P>['options']
    ) => ChannelAndLeave<P, C>;
  }
>) => {
  const [{ channel }, setChannelLeavePair] = useState(() => {
    return joinChannel(channelName, { ...options, autoConnect: false });
  });

  useEffect(() => {
    const channelLeavePair = joinChannel(channelName, options);

    setChannelLeavePair(channelLeavePair);

    return () => {
      channelLeavePair.leave();
    };
  }, [channelName, joinChannel, options]);

  return <ChannelContext.Provider value={channel as Channel}>{children}</ChannelContext.Provider>;
};

export const useMyPresence = <P extends DefaultPresence, C extends DefaultCustomMessageData>() => {
  const channel = useChannel<P, C>();

  const [myPresence, setMyPresence] = useState<P | null>(null);

  useEffect(() => {
    const unsub = channel.events.subscribe('presence', (presence) => {
      setMyPresence(presence);
    });

    return () => {
      unsub();
    };
  }, [channel.events]);

  const setMyPresenceWrapper = useCallback(
    (newPresence: P) => {
      channel.updatePresence(newPresence);
    },
    [channel]
  );

  return [myPresence, setMyPresenceWrapper] as const;
};
