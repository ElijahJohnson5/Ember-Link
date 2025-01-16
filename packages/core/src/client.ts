// What the api should look like

import { IStorageProvider } from '@ember-link/storage';
import { Channel, createChannel } from './channel.js';

/*

const client = createClient({
  baseUrl: url,
  apiKey: apiKey,
});

const channel = client.joinChannel(channelName);


let unsub = channel.events.subscribe("presence", (presenceData) => {
  // Do stuff with the presence
});

*/

interface CreateClientOptions {
  baseUrl: string;
  apiKey?: string;
}

export function createClient({ baseUrl }: CreateClientOptions) {
  const channels = new Map<string, Channel>();

  function joinChannel<S extends IStorageProvider>(channelName: string, storageProvider?: S) {
    if (channels.has(channelName)) {
      return channels.get(channelName);
    }

    const { channel, leave } = createChannel<S>({
      channelName,
      baseUrl,
      storageProvider
    });

    channels.set(channelName, channel);

    return channel;
  }

  return {
    joinChannel
  };
}
