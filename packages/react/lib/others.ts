import type { DefaultCustomMessageData, DefaultPresence, User } from '@ember-link/core';
import { useChannel } from './channel-provider';
import { useEffect, useState } from 'react';

export const useOthers = <P extends DefaultPresence, C extends DefaultCustomMessageData>() => {
  const channel = useChannel<P, C>();

  const [others, setOthers] = useState<Record<string, User<P>>>({});

  useEffect(() => {
    const unsub = channel.events.others.subscribe('join', (user) => {
      setOthers((prev) => ({ ...prev, [user.clientId]: user }));
    });

    return () => {
      unsub();
    };
  }, [channel.events.others]);

  useEffect(() => {
    const unsub = channel.events.others.subscribe('leave', (user) => {
      setOthers((prev) => {
        delete prev[user.clientId];

        return { ...prev };
      });
    });

    return () => {
      unsub();
    };
  }, [channel.events.others]);

  useEffect(() => {
    const unsub = channel.events.others.subscribe('update', (user) => {
      setOthers((prev) => ({ ...prev, [user.clientId]: user }));
    });

    return () => {
      unsub();
    };
  }, [channel.events.others]);

  useEffect(() => {
    const unsub = channel.events.others.subscribe('reset', () => {
      setOthers({});
    });

    return () => {
      unsub();
    };
  }, [channel.events.others]);

  return others;
};
