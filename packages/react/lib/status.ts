import type { DefaultCustomMessageData, DefaultPresence, Status } from '@ember-link/core';
import { useChannel } from './channel-provider';
import { useCallback, useSyncExternalStore } from 'react';

export const useStatus = <P extends DefaultPresence, C extends DefaultCustomMessageData>() => {
  const channel = useChannel<P, C>();

  const subscribeFunction = useCallback(
    (callback: (presence: Status) => void) => {
      return channel.events.subscribe('status', callback);
    },
    [channel]
  );

  const status = useSyncExternalStore(
    subscribeFunction,
    channel.getStatus,
    () => 'initial' as const
  );

  return status;
};
