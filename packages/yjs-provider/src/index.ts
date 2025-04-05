import type { Channel } from '@ember-link/core';
import { Doc } from 'yjs';
import { EmberLinkYjsProvider } from '~/provider';

const providersMap = new WeakMap<Channel, EmberLinkYjsProvider>();

export const getYjsProviderForChannel = (channel: Channel) => {
  let provider = providersMap.get(channel);
  if (provider) {
    return provider;
  }

  const doc = new Doc();

  provider = new EmberLinkYjsProvider({
    channel,
    doc
  });

  channel.events.subscribeOnce('destroy', () => {
    providersMap.delete(channel);
    provider.destroy();
  });

  providersMap.set(channel, provider);
  return provider;
};

export { EmberLinkYjsProvider };
