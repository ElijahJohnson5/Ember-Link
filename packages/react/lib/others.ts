import type { DefaultCustomMessageData, DefaultPresence, User } from '@ember-link/core';
import { useChannel } from './channel-provider';
import { useCallback, useSyncExternalStore } from 'react';

export const useOthers = <P extends DefaultPresence, C extends DefaultCustomMessageData>() => {
  const channel = useChannel<P, C>();

  const subscribeFunction = useCallback(
    (callback: (others: User<P>[]) => void) => {
      return channel.events.subscribe('others', callback);
    },
    [channel]
  );

  const others = useSyncExternalStore(subscribeFunction, channel.getOthers, () => []);

  return others;
};
